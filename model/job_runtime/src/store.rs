//! ジョブの保存先（設計: JOB_MANAGER_STORAGE_DESIGN.md 4・5 章）
//!
//! ```text
//! <ジョブの保存先>/
//!   manager.lock              ジョブマネージャーが OS のファイルロックで排他に開く
//!   inbox/<投入の ID>/        投入（request.json と入力のファイル）
//!   inbox/<投入の ID>.rejected/  取り込めなかった投入（reason.txt に理由）
//!   jobs/<ジョブの番号>/
//!     job.json                ジョブの記録
//!     input/input.bin         入力
//!     output/                 成果物（toolpath.bin・cutting_simulation.bin）
//!     logs/runner.log         ログ
//! ```
//!
//! ファイルは一時の名前（`*.tmp`）で書き終えてから名前を変え、読む側は `*.tmp` を無視する。保存先は同じ利用者のほかの
//! プロセスからも書けるため、投入・記録は信用できない入力として検証し、壊れたファイルで落ちない。ファイルの名前は
//! ジョブの番号と決まった名前だけから作り、投入・参照の文字列をパスに使わない。

#[cfg(test)]
mod tests;

mod files;
mod record;

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::{self, File, TryLockError};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use record::{
    DependencyRef, STORE_FORMAT_VERSION, StoredJob, StoredJobKind, StoredJobSource,
    StoredJobStatus, SubmissionRequest, SubmittedJob,
};

use crate::reference::RefParser;
use files::{
    copy_limited, is_plain_name, is_temporary, read_limited, remove_temporary_entries,
    write_atomically,
};

const LOCK_FILE: &str = "manager.lock";
const INBOX_DIR: &str = "inbox";
const JOBS_DIR: &str = "jobs";
const REQUEST_FILE: &str = "request.json";
const REJECTED_SUFFIX: &str = ".rejected";
const REJECTED_REASON_FILE: &str = "reason.txt";
const JOB_FILE: &str = "job.json";
const INPUT_DIR: &str = "input";
const INPUT_FILE: &str = "input.bin";
const OUTPUT_DIR: &str = "output";
const LOGS_DIR: &str = "logs";
const LOG_FILE: &str = "runner.log";
const TOOLPATH_FILE: &str = "toolpath.bin";
const CUTTING_SIMULATION_FILE: &str = "cutting_simulation.bin";

/// 投入の要求・ジョブの記録のファイルの大きさの上限
const MAX_JSON_BYTES: u64 = 64 * 1024;
/// 1 つの投入のジョブの数の上限
const MAX_JOBS_PER_SUBMISSION: usize = 64;
/// 入力のファイルの大きさの上限
const MAX_INPUT_BYTES: u64 = 1 << 30;
/// 投入の ID・入力のファイルの名前の長さの上限
const MAX_NAME_LEN: usize = 64;
/// グループ・投入元の ID の長さの上限
const MAX_LABEL_LEN: usize = 256;
/// 受付のフォルダの書きかけの投入を、書く側が落ちて残ったものとみなすまでの時間
const STALE_SUBMISSION_AGE: Duration = Duration::from_secs(60 * 60);

/// 保存先の誤り
#[derive(Debug)]
pub enum StoreError {
    /// ほかのジョブマネージャーが保存先を開いている
    AlreadyLocked,
    /// 保存先のファイルを読み書きできない
    Io(std::io::Error),
    /// 参照がジョブの保存先のファイルを指さない
    InvalidRef(String),
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyLocked => write!(f, "another job manager is using the job store"),
            Self::Io(error) => write!(f, "job store I/O error: {error}"),
            Self::InvalidRef(value) => write!(f, "reference does not point to a job file: {value}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// 保存先を排他に開いている間だけ持つロック（落ちた場合は OS が外す）
#[derive(Debug)]
pub struct JobStoreLock {
    _file: File,
}

/// 投入の取り込みの結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntakeOutcome {
    /// ジョブの番号を振って取り込んだ（同じ投入の ID をすでに取り込んでいた場合は、そのジョブの番号）
    Accepted {
        submission_id: String,
        job_ids: Vec<u64>,
    },
    /// 取り込めなかった（`inbox/<投入の ID>.rejected/` に理由を置いた）
    Rejected {
        submission_id: String,
        reason: String,
    },
    /// 投入のファイルを読み書きできず、今回は見送った（次の取り込みでやり直す）
    Postponed {
        submission_id: String,
        reason: String,
    },
}

/// 取り込みの間に持つ、すでにあるジョブの情報
struct IntakeState {
    kinds: HashMap<u64, StoredJobKind>,
    /// 投入の ID ごとの、作ったジョブの番号
    taken: HashMap<String, Vec<u64>>,
    next_id: u64,
}

/// 読めなかったジョブの記録
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadableJob {
    pub path: PathBuf,
    pub reason: String,
}

/// ジョブの保存先
#[derive(Debug, Clone)]
pub struct JobStore {
    root: PathBuf,
}

impl JobStore {
    /// 保存先 `root` を開く（受付・ジョブのフォルダがなければ作る）
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let store = Self { root: root.into() };
        fs::create_dir_all(store.root.join(INBOX_DIR))?;
        fs::create_dir_all(store.root.join(JOBS_DIR))?;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `manager.lock` を排他に開く。ほかのジョブマネージャーが開いていれば `StoreError::AlreadyLocked`
    pub fn lock(&self) -> Result<JobStoreLock, StoreError> {
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.root.join(LOCK_FILE))?;
        match file.try_lock() {
            Ok(()) => Ok(JobStoreLock { _file: file }),
            Err(TryLockError::WouldBlock) => Err(StoreError::AlreadyLocked),
            Err(TryLockError::Error(error)) => Err(StoreError::Io(error)),
        }
    }

    /// 書きかけのもの（ジョブのフォルダの `*.tmp`、ジョブの記録・成果物・進み具合の `*.tmp`）を消す
    ///
    /// ロックを持っている間に、起動のときに呼ぶ。受付のフォルダの `*.tmp` は、CAD/CAM のアプリが書いている途中の
    /// ことがあるため、更新から `STALE_SUBMISSION_AGE` を過ぎたものだけを消す。ログのフォルダの `runner.log.tmp` は、
    /// 落ちたランナーの途中までのログのため残す。
    pub fn remove_temporary_files(&self, _lock: &JobStoreLock) -> Result<(), StoreError> {
        remove_temporary_entries(&self.root.join(INBOX_DIR), Some(STALE_SUBMISSION_AGE))?;
        remove_temporary_entries(&self.root.join(JOBS_DIR), None)?;
        for job_id in self.job_ids()? {
            remove_temporary_entries(&self.job_dir(job_id), None)?;
            remove_temporary_entries(&self.output_dir(job_id), None)?;
        }
        Ok(())
    }

    /// すべてのジョブの記録を、ジョブの番号の順に読む。読めない記録は理由とともに分けて返す
    pub fn load_jobs(&self) -> Result<(Vec<StoredJob>, Vec<UnreadableJob>), StoreError> {
        let mut jobs = Vec::new();
        let mut unreadable = Vec::new();
        for job_id in self.job_ids()? {
            let path = self.job_dir(job_id).join(JOB_FILE);
            match read_job(&path, job_id) {
                Ok(job) => jobs.push(job),
                Err(reason) => unreadable.push(UnreadableJob { path, reason }),
            }
        }
        Ok((jobs, unreadable))
    }

    /// ジョブの記録を書く
    pub fn write_job(&self, _lock: &JobStoreLock, job: &StoredJob) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec_pretty(job).map_err(std::io::Error::other)?;
        write_atomically(&self.job_dir(job.job_id).join(JOB_FILE), &bytes)?;
        Ok(())
    }

    /// 受付のフォルダの投入を取り込む
    ///
    /// 投入ごとに要求と入力を検証し、ジョブの番号を振り、入力を写してジョブの記録を書き、投入のフォルダを消す。
    /// ジョブは `jobs/<ジョブの番号>.tmp/` に作ってから名前を変える。検証できない投入は `.rejected` の名前に変え、
    /// 理由を置く。読み書きできない投入（ほかのプロセスが開いている等）は、今回は見送り、ほかの投入を続ける。
    /// `now_ms` は作成の時刻（UNIX 時刻のミリ秒）。
    ///
    /// 取り込みの途中で止まった投入（一部のジョブだけを作った）は、次の取り込みで作ったジョブを消して取り込み直し、
    /// すべてのジョブを作った後に止まった投入は、投入のフォルダを消すだけにする（投入の ID はジョブを引く鍵のため、
    /// 投入する側が重ならない値（UUID）にする）。
    pub fn intake(
        &self,
        lock: &JobStoreLock,
        now_ms: u64,
    ) -> Result<Vec<IntakeOutcome>, StoreError> {
        let submission_ids = self.submission_ids()?;
        if submission_ids.is_empty() {
            return Ok(Vec::new());
        }
        let job_ids = self.job_ids()?;
        let (jobs, _) = self.load_jobs()?;
        let mut state = IntakeState {
            kinds: jobs.iter().map(|job| (job.job_id, job.kind)).collect(),
            taken: HashMap::new(),
            next_id: job_ids.last().map_or(1, |last| last + 1),
        };
        for job in &jobs {
            state
                .taken
                .entry(job.submission_id.clone())
                .or_default()
                .push(job.job_id);
        }

        let mut outcomes = Vec::new();
        for submission_id in submission_ids {
            let outcome = self
                .intake_one(lock, &submission_id, &mut state, now_ms)
                .unwrap_or_else(|error| IntakeOutcome::Postponed {
                    submission_id: submission_id.clone(),
                    reason: error.to_string(),
                });
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    fn intake_one(
        &self,
        lock: &JobStoreLock,
        submission_id: &str,
        state: &mut IntakeState,
        now_ms: u64,
    ) -> Result<IntakeOutcome, StoreError> {
        let dir = self.root.join(INBOX_DIR).join(submission_id);
        if let Some(job_ids) = state.taken.remove(submission_id) {
            let expected = read_request(&dir).map(|request| request.jobs.len());
            if expected.is_err() || expected == Ok(job_ids.len()) {
                // すべてのジョブを作った後、投入のフォルダを消す前に止まった
                fs::remove_dir_all(&dir)?;
                state
                    .taken
                    .insert(submission_id.to_string(), job_ids.clone());
                return Ok(IntakeOutcome::Accepted {
                    submission_id: submission_id.to_string(),
                    job_ids,
                });
            }
            // 一部のジョブだけを作って止まった（作ったジョブはまだ実行していない）
            for job_id in job_ids {
                fs::remove_dir_all(self.job_dir(job_id))?;
                state.kinds.remove(&job_id);
            }
        }
        let request = match validate_submission(&dir, submission_id, &state.kinds) {
            Ok(request) => request,
            Err(reason) => {
                self.reject(submission_id, &reason)?;
                return Ok(IntakeOutcome::Rejected {
                    submission_id: submission_id.to_string(),
                    reason,
                });
            }
        };
        let job_ids: Vec<u64> = (state.next_id..).take(request.jobs.len()).collect();
        state.next_id += request.jobs.len() as u64;
        let mut created = Vec::new();
        for (index, submitted) in request.jobs.iter().enumerate() {
            let job = StoredJob {
                format_version: STORE_FORMAT_VERSION,
                job_id: job_ids[index],
                submission_id: submission_id.to_string(),
                kind: submitted.kind,
                depends_on: submitted
                    .depends_on
                    .iter()
                    .map(|dependency| match *dependency {
                        DependencyRef::Submitted(position) => job_ids[position],
                        DependencyRef::Existing(job_id) => job_id,
                    })
                    .collect(),
                priority: submitted.priority,
                group_id: submitted.group_id.clone(),
                source: submitted.source.clone(),
                status: StoredJobStatus::Queued,
                attempts: 0,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
                result_ref: None,
                log_ref: None,
                last_error: None,
            };
            let made = self.create_job(lock, &job, &dir.join(&submitted.input_file));
            if let Err(error) = made {
                // 作ったジョブを残し、次の取り込みで、一部だけを作った投入として取り込み直す
                state.taken.insert(submission_id.to_string(), created);
                return Err(error);
            }
            state.kinds.insert(job.job_id, job.kind);
            created.push(job.job_id);
        }
        state
            .taken
            .insert(submission_id.to_string(), job_ids.clone());
        fs::remove_dir_all(&dir)?;
        Ok(IntakeOutcome::Accepted {
            submission_id: submission_id.to_string(),
            job_ids,
        })
    }

    /// ジョブのフォルダを一時の名前で作り（入力を写し、記録を書き）、名前を変える
    fn create_job(
        &self,
        _lock: &JobStoreLock,
        job: &StoredJob,
        input: &Path,
    ) -> Result<(), StoreError> {
        let dir = self.job_dir(job.job_id);
        let temporary = files::temporary_path(&dir);
        if temporary.exists() {
            fs::remove_dir_all(&temporary)?;
        }
        fs::create_dir_all(temporary.join(INPUT_DIR))?;
        copy_limited(
            input,
            &temporary.join(INPUT_DIR).join(INPUT_FILE),
            MAX_INPUT_BYTES,
        )?;
        let bytes = serde_json::to_vec_pretty(job).map_err(std::io::Error::other)?;
        write_atomically(&temporary.join(JOB_FILE), &bytes)?;
        fs::rename(&temporary, &dir)?;
        Ok(())
    }

    fn reject(&self, submission_id: &str, reason: &str) -> Result<(), StoreError> {
        let inbox = self.root.join(INBOX_DIR);
        let rejected = inbox.join(format!("{submission_id}{REJECTED_SUFFIX}"));
        if rejected.exists() {
            fs::remove_dir_all(&rejected)?;
        }
        fs::rename(inbox.join(submission_id), &rejected)?;
        write_atomically(&rejected.join(REJECTED_REASON_FILE), reason.as_bytes())?;
        Ok(())
    }

    /// 受付のフォルダの投入の ID（名前の順。`.` を含む名前は書きかけ・取り込めなかった投入のため除く）
    fn submission_ids(&self) -> Result<Vec<String>, StoreError> {
        let mut ids = Vec::new();
        for entry in fs::read_dir(self.root.join(INBOX_DIR))? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str()
                && !name.contains('.')
            {
                ids.push(name.to_string());
            }
        }
        ids.sort();
        Ok(ids)
    }

    /// ジョブのフォルダの番号（小さい順。番号の名前でないものは除く）
    fn job_ids(&self) -> Result<Vec<u64>, StoreError> {
        let mut ids = Vec::new();
        for entry in fs::read_dir(self.root.join(JOBS_DIR))? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if is_temporary(name) || !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(id) = parse_job_id(name) {
                ids.push(id);
            }
        }
        ids.sort_unstable();
        Ok(ids)
    }

    /// ジョブのフォルダ（`jobs/<ジョブの番号>/`）
    pub fn job_dir(&self, job_id: u64) -> PathBuf {
        self.root.join(JOBS_DIR).join(job_id.to_string())
    }

    /// 入力のファイル
    pub fn input_file(&self, job_id: u64) -> PathBuf {
        self.job_dir(job_id).join(INPUT_DIR).join(INPUT_FILE)
    }

    /// 成果物のフォルダ
    pub fn output_dir(&self, job_id: u64) -> PathBuf {
        self.job_dir(job_id).join(OUTPUT_DIR)
    }

    /// ログのフォルダ
    pub fn logs_dir(&self, job_id: u64) -> PathBuf {
        self.job_dir(job_id).join(LOGS_DIR)
    }

    /// 種類 `kind` のジョブの成果物のファイル
    pub fn result_file(&self, job_id: u64, kind: StoredJobKind) -> PathBuf {
        self.output_dir(job_id).join(match kind {
            StoredJobKind::Cam => TOOLPATH_FILE,
            StoredJobKind::Sim => CUTTING_SIMULATION_FILE,
        })
    }

    /// ログのファイル
    pub fn log_file(&self, job_id: u64) -> PathBuf {
        self.logs_dir(job_id).join(LOG_FILE)
    }

    /// 成果物・ログの参照（`result://<cam|sim>/<ジョブの番号>/...`・`log://<cam|sim>/<ジョブの番号>/...`）が指す
    /// ファイル
    ///
    /// ファイルの名前は、ジョブの番号と、ドメインから決まる名前だけで作る（接尾辞はパスに使わない）。ドメインが
    /// ジョブの記録の種類と合うかは、記録を持つ呼び出し側で確かめる。
    pub fn file_for_ref(&self, value: &str) -> Result<PathBuf, StoreError> {
        let invalid = || StoreError::InvalidRef(value.to_string());
        let (parsed, is_result) = match RefParser::parse_result(value) {
            Ok(parsed) => (parsed, true),
            Err(_) => (RefParser::parse_log(value).map_err(|_| invalid())?, false),
        };
        let [domain, job_id, _suffix, ..] = parsed.segments() else {
            return Err(invalid());
        };
        let kind = match domain.as_str() {
            "cam" => StoredJobKind::Cam,
            "sim" => StoredJobKind::Sim,
            _ => return Err(invalid()),
        };
        let job_id = parse_job_id(job_id).ok_or_else(invalid)?;
        Ok(if is_result {
            self.result_file(job_id, kind)
        } else {
            self.log_file(job_id)
        })
    }
}

/// ジョブの番号の名前（先頭が 0 でない 10 進数）を読む
fn parse_job_id(name: &str) -> Option<u64> {
    if name.starts_with('0') || !name.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    name.parse().ok().filter(|&id| id > 0)
}

fn read_job(path: &Path, job_id: u64) -> Result<StoredJob, String> {
    let bytes = read_limited(path, MAX_JSON_BYTES).map_err(|error| error.to_string())?;
    let job: StoredJob = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if job.format_version != STORE_FORMAT_VERSION {
        return Err(format!("unsupported format version {}", job.format_version));
    }
    if job.job_id != job_id {
        return Err(format!("job id {} does not match the folder", job.job_id));
    }
    Ok(job)
}

/// 投入の要求を読む。誤りは理由（英語）を返す
fn read_request(dir: &Path) -> Result<SubmissionRequest, String> {
    let bytes = read_limited(&dir.join(REQUEST_FILE), MAX_JSON_BYTES)
        .map_err(|error| format!("cannot read the request: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid request: {error}"))
}

/// 投入の要求と入力を検証する。誤りは理由（英語）を返す
fn validate_submission(
    dir: &Path,
    submission_id: &str,
    existing: &HashMap<u64, StoredJobKind>,
) -> Result<SubmissionRequest, String> {
    if !is_plain_name(submission_id, MAX_NAME_LEN) {
        return Err("invalid submission id".to_string());
    }
    let bytes = read_limited(&dir.join(REQUEST_FILE), MAX_JSON_BYTES)
        .map_err(|error| format!("cannot read the request: {error}"))?;
    let request: SubmissionRequest =
        serde_json::from_slice(&bytes).map_err(|error| format!("invalid request: {error}"))?;
    if request.format_version != STORE_FORMAT_VERSION {
        return Err(format!(
            "unsupported format version {}",
            request.format_version
        ));
    }
    if request.jobs.is_empty() || request.jobs.len() > MAX_JOBS_PER_SUBMISSION {
        return Err(format!(
            "a submission must have 1 to {MAX_JOBS_PER_SUBMISSION} jobs"
        ));
    }
    let mut input_files = HashSet::new();
    for (index, job) in request.jobs.iter().enumerate() {
        let error = |reason: String| format!("job {index}: {reason}");
        if !is_plain_name(&job.input_file, MAX_NAME_LEN) || job.input_file == REQUEST_FILE {
            return Err(error("invalid input file name".to_string()));
        }
        if !input_files.insert(job.input_file.as_str()) {
            return Err(error("input file is shared with another job".to_string()));
        }
        let metadata = fs::symlink_metadata(dir.join(&job.input_file))
            .map_err(|error_value| error(format!("cannot read the input file: {error_value}")))?;
        if !metadata.is_file() {
            return Err(error("the input is not a regular file".to_string()));
        }
        if metadata.len() > MAX_INPUT_BYTES {
            return Err(error(format!(
                "the input file is larger than {MAX_INPUT_BYTES} bytes"
            )));
        }
        let labels = [
            job.group_id.as_deref(),
            job.source.as_ref().map(|source| source.source_id.as_str()),
        ];
        if labels
            .iter()
            .flatten()
            .any(|label| label.len() > MAX_LABEL_LEN)
        {
            return Err(error(format!(
                "labels must be at most {MAX_LABEL_LEN} bytes"
            )));
        }
        let mut cam_dependencies = 0;
        let mut seen = HashSet::new();
        for dependency in &job.depends_on {
            let kind = match *dependency {
                DependencyRef::Submitted(position) if position < index => {
                    request.jobs[position].kind
                }
                DependencyRef::Submitted(_) => {
                    return Err(error(
                        "a dependency in the submission must come earlier".to_string(),
                    ));
                }
                DependencyRef::Existing(job_id) => *existing
                    .get(&job_id)
                    .ok_or_else(|| error(format!("dependency job {job_id} does not exist")))?,
            };
            if !seen.insert(*dependency) {
                return Err(error("a dependency is given twice".to_string()));
            }
            if kind == StoredJobKind::Cam {
                cam_dependencies += 1;
            }
        }
        // 切削シミュレーションは、ツールパスを作るジョブを 1 つだけ入力にする
        if job.kind == StoredJobKind::Sim && cam_dependencies != 1 {
            return Err(error(
                "a sim job must depend on exactly one cam job".to_string(),
            ));
        }
    }
    Ok(request)
}
