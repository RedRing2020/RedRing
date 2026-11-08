//! ジョブマネージャー（設計: JOB_MANAGER_STORAGE_DESIGN.md 3・4・5 章）
//!
//! ジョブの保存先を排他に開き、受付のフォルダの投入を取り込み、依存関係と優先順位の順にジョブごとのランナーを
//! 起動し、終わり方でジョブの状態を更新する。ジョブの記録を書くのは、保存先のロックを持つ本サービスだけとする。

#[cfg(test)]
mod tests;

mod launcher;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use job_runtime::RefFactory;
use job_runtime::store::{
    IntakeOutcome, JobStore, JobStoreLock, StoreError, StoredJob, StoredJobError, StoredJobKind,
    StoredJobStatus, plan_schedule,
};

pub use launcher::{LaunchSpec, Launcher, ProcessLauncher, RunningJob};

/// 同時に実行するジョブの数
const MAX_RUNNING_JOBS: usize = 1;
/// 失敗の理由に添えるログの末尾を読む大きさ
const MAX_LOG_TAIL_BYTES: u64 = 64 * 1024;
/// 失敗の理由に添えるログの行の長さの上限（文字の数。記録が大きくなりすぎないように）
const MAX_LOG_LINE_CHARS: usize = 1024;

/// ジョブマネージャーの出来事（ログに書く）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerEvent {
    Intake(IntakeOutcome),
    /// 記録を読めないジョブ（読めないと分かったときに 1 回だけ）
    UnreadableJob {
        job_id: u64,
        reason: String,
    },
    /// ジョブマネージャーが止まったときに実行中だったため、失敗にした
    Interrupted {
        job_id: u64,
    },
    Started {
        job_id: u64,
    },
    /// ランナーが終わった（終わったときの状態と、失敗の理由）
    Finished {
        job_id: u64,
        status: StoredJobStatus,
        error: Option<String>,
    },
    /// 依存するジョブが実行できないため、失敗にした
    Blocked {
        job_id: u64,
        reason: String,
    },
    /// 保存先を読み書きできなかった（次の回にやり直す）
    Error(String),
}

/// 消すジョブの決め方の結果
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeletePlan {
    /// 消すジョブ（選んだジョブと、その成果物を使う後のジョブ）
    pub deleted: BTreeSet<u64>,
    /// 消さない、選んだジョブ（それか、その成果物を使う後のジョブが、計算待ち・計算中）
    pub kept: BTreeSet<u64>,
}

/// ジョブの記録 `jobs` のうち、選んだジョブ `selected` を消すときに消すジョブ
///
/// 選んだジョブは、その成果物を使う後のジョブ（依存関係をたどったすべて）とあわせて消す（後のジョブだけが残っても、
/// 計算も取り込みもできないため）。選んだジョブか後のジョブのどれかが計算待ち・計算中なら、どれも消さない。ない
/// ジョブは数えない。後のジョブだけを選んだときは、前のジョブを残す。
pub fn delete_plan(jobs: &[StoredJob], selected: &[u64]) -> DeletePlan {
    let by_id: HashMap<u64, &StoredJob> = jobs.iter().map(|job| (job.job_id, job)).collect();
    let mut plan = DeletePlan::default();
    for &job_id in selected {
        if !by_id.contains_key(&job_id) {
            continue;
        }
        let family = later_jobs(jobs, job_id, |_| true);
        if family.iter().all(|id| by_id[id].status.is_terminal()) {
            plan.deleted.extend(family);
        } else {
            plan.kept.insert(job_id);
        }
    }
    plan
}

/// ジョブの記録 `jobs` のうち、前の改訂の結果のジョブ（番号の順）
///
/// 投入元の ID ごとに、最も後に正常に終わったジョブ（番号の最も大きいジョブ）の改訂の鍵を今の改訂とみなし、それより
/// 前に投入した、ほかの改訂の鍵（鍵を持たない前の版の記録を含む）のジョブを前の改訂とする。後から投入して、まだ
/// 終わっていない・失敗したジョブでは決めない（前の結果を、新しい結果ができる前に消さないように）。改訂の番号は、
/// 複製したモデルファイル・保存せずに捨てた変更で重なるため使わない。投入元を持たないジョブは数えない。
pub fn superseded_jobs(jobs: &[StoredJob]) -> Vec<u64> {
    let mut latest: HashMap<&str, (u64, Option<&str>)> = HashMap::new();
    for job in jobs
        .iter()
        .filter(|job| job.status == StoredJobStatus::Succeeded)
    {
        if let Some(source) = &job.source {
            let candidate = (job.job_id, source.revision_key.as_deref());
            latest
                .entry(source.source_id.as_str())
                .and_modify(|current| {
                    if candidate.0 > current.0 {
                        *current = candidate;
                    }
                })
                .or_insert(candidate);
        }
    }
    jobs.iter()
        .filter(|job| {
            job.source.as_ref().is_some_and(|source| {
                latest
                    .get(source.source_id.as_str())
                    .is_some_and(|&(latest_id, latest_key)| {
                        let key = source.revision_key.as_deref();
                        job.job_id < latest_id && (key.is_none() || key != latest_key)
                    })
            })
        })
        .map(|job| job.job_id)
        .collect()
}

/// ジョブの記録 `jobs` のうち、時刻 `before_ms`（UNIX 時刻のミリ秒）より前に投入した、終わったジョブ（番号の順）
///
/// その成果物を使う後のジョブ（消すときに、あわせて消える）も、すべてその時刻より前に投入したものに限る（古い
/// ツールパス生成の下に、後から計算した切削シミュレーションを消さないように）。
pub fn old_jobs(jobs: &[StoredJob], before_ms: u64) -> Vec<u64> {
    let created: HashMap<u64, u64> = jobs
        .iter()
        .map(|job| (job.job_id, job.created_at_ms))
        .collect();
    jobs.iter()
        .filter(|job| job.status.is_terminal() && job.created_at_ms < before_ms)
        .filter(|job| {
            later_jobs(jobs, job.job_id, |_| true)
                .iter()
                .all(|id| created[id] < before_ms)
        })
        .map(|job| job.job_id)
        .collect()
}

/// ジョブマネージャーのサービス
pub struct JobManagerService<L: Launcher> {
    store: JobStore,
    lock: JobStoreLock,
    launcher: L,
    running: HashMap<u64, Box<dyn RunningJob>>,
    /// 読めないと知らせたジョブ・見送ったと知らせた投入（同じ出来事を毎回書かないため）
    reported_unreadable: HashSet<u64>,
    reported_postponed: HashSet<String>,
}

impl<L: Launcher> JobManagerService<L> {
    /// 保存先を排他に開き、書きかけのものを片付け、前に実行中だったジョブを失敗にして始める
    ///
    /// ほかのジョブマネージャーが保存先を開いていれば `StoreError::AlreadyLocked`。
    pub fn start(
        store: JobStore,
        launcher: L,
        now_ms: u64,
    ) -> Result<(Self, Vec<ManagerEvent>), StoreError> {
        let lock = store.lock()?;
        store.remove_temporary_files(&lock)?;
        let service = Self {
            store,
            lock,
            launcher,
            running: HashMap::new(),
            reported_unreadable: HashSet::new(),
            reported_postponed: HashSet::new(),
        };
        let mut events = Vec::new();
        let (jobs, _) = service.store.load_jobs()?;
        for mut job in jobs {
            if job.status == StoredJobStatus::Running {
                // ランナーは別のプロセスのため、止まったジョブマネージャーからは引き継げない
                fail(&mut job, StoredJobError::ManagerStopped, None, now_ms);
                // 止まっていた間を計算の時間に数えないように、始めた時刻を消す
                job.started_at_ms = None;
                service.store.write_job(&service.lock, &job)?;
                events.push(ManagerEvent::Interrupted { job_id: job.job_id });
            }
        }
        Ok((service, events))
    }

    /// 1 回分の仕事をする: 投入の取り込み、終わったランナーの記録、実行できないジョブの失敗、次のジョブの起動
    ///
    /// 保存先を読み書きできない誤りは `ManagerEvent::Error` にし、できるところまで続ける。
    pub fn tick(&mut self, now_ms: u64) -> Vec<ManagerEvent> {
        let mut events = Vec::new();
        match self.store.intake(&self.lock, now_ms) {
            Ok(outcomes) => {
                for outcome in outcomes {
                    if let IntakeOutcome::Postponed { submission_id, .. } = &outcome
                        && !self.reported_postponed.insert(submission_id.clone())
                    {
                        continue;
                    }
                    events.push(ManagerEvent::Intake(outcome));
                }
            }
            Err(error) => events.push(ManagerEvent::Error(error.to_string())),
        }
        let (mut jobs, unreadable) = match self.store.load_jobs() {
            Ok(loaded) => loaded,
            Err(error) => {
                events.push(ManagerEvent::Error(error.to_string()));
                return events;
            }
        };
        for job in &unreadable {
            if self.reported_unreadable.insert(job.job_id) {
                events.push(ManagerEvent::UnreadableJob {
                    job_id: job.job_id,
                    reason: job.reason.clone(),
                });
            }
        }
        let unreadable: Vec<u64> = unreadable.iter().map(|job| job.job_id).collect();
        if let Err(error) = self.collect_finished(&mut jobs, now_ms, &mut events) {
            events.push(ManagerEvent::Error(error.to_string()));
        }
        if let Err(error) = self.schedule(&mut jobs, &unreadable, now_ms, &mut events) {
            events.push(ManagerEvent::Error(error.to_string()));
        }
        events
    }

    /// ジョブを取り消す。実行中ならランナーを止める。取り消したら `true`（終わったジョブ・ないジョブは `false`）
    pub fn cancel(&mut self, job_id: u64, now_ms: u64) -> Result<bool, StoreError> {
        let (jobs, _) = self.store.load_jobs()?;
        let Some(mut job) = jobs.into_iter().find(|job| job.job_id == job_id) else {
            return Ok(false);
        };
        if job.status.is_terminal() {
            return Ok(false);
        }
        match self.running.get_mut(&job_id) {
            Some(running) => running.stop()?,
            // 計算を始めていないジョブ（前の計算の時刻が残るジョブを含む）は、計算の時間を持たない
            None => job.started_at_ms = None,
        }
        job.status = StoredJobStatus::Canceled;
        job.updated_at_ms = now_ms;
        // 書けなければ、止めたランナーを残し、次の回に「止められた」失敗として記録する
        self.store.write_job(&self.lock, &job)?;
        self.running.remove(&job_id);
        Ok(true)
    }

    /// 実行中のランナーをすべて止め、そのジョブを失敗にする（ジョブマネージャーを終了するときに）
    ///
    /// 記録を読み書きできなくても・止められないランナーがあっても、ほかのランナーは止め、最初の誤りを返す。書けな
    /// かった記録は実行中のまま残り、次に起動したときに失敗になる。
    pub fn stop_running(&mut self, now_ms: u64) -> Result<(), StoreError> {
        let mut first_error = None;
        let jobs = match self.store.load_jobs() {
            Ok((jobs, _)) => jobs,
            Err(error) => {
                first_error = Some(error);
                Vec::new()
            }
        };
        let mut stopped: Vec<u64> = self.running.keys().copied().collect();
        stopped.sort_unstable();
        for job_id in stopped {
            if let Some(mut running) = self.running.remove(&job_id)
                && let Err(error) = running.stop()
            {
                first_error.get_or_insert(error.into());
            }
            if let Some(mut job) = jobs.iter().find(|job| job.job_id == job_id).cloned() {
                fail(&mut job, StoredJobError::ManagerQuit, None, now_ms);
                job.started_at_ms = None;
                if let Err(error) = self.store.write_job(&self.lock, &job) {
                    first_error.get_or_insert(error);
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    /// 失敗・取り消しで終わったジョブを、計算待ちに戻す。戻したジョブの番号（小さい順。戻せなければ空）
    ///
    /// そのジョブと依存関係でつながる、失敗・取り消しで終わったジョブもあわせて戻す（前のジョブが終わったままでは
    /// 計算できず、後のジョブは前のジョブの失敗で失敗になっているため）。戻したジョブの前の計算の成果物・ログは
    /// 消す。計算した回数は数え続ける。
    pub fn retry(&mut self, job_id: u64, now_ms: u64) -> Result<Vec<u64>, StoreError> {
        let (jobs, _) = self.store.load_jobs()?;
        let by_id: HashMap<u64, &StoredJob> = jobs.iter().map(|job| (job.job_id, job)).collect();
        let retryable = |id: &u64| {
            by_id.get(id).is_some_and(|job| {
                matches!(
                    job.status,
                    StoredJobStatus::Failed | StoredJobStatus::Canceled
                )
            })
        };
        if !retryable(&job_id) {
            return Ok(Vec::new());
        }
        let targets = connected_jobs(&jobs, job_id, retryable);
        // 前の計算のファイルを先にすべて消す（消せないファイルがあれば、記録を変えずに誤りにする）
        for &id in &targets {
            self.store.remove_job_outputs(&self.lock, id)?;
        }
        // 前のジョブ（番号の小さいジョブ）から戻す
        for &id in &targets {
            let mut job = by_id[&id].clone();
            job.status = StoredJobStatus::Queued;
            job.started_at_ms = None;
            job.result_ref = None;
            job.log_ref = None;
            job.last_error = None;
            job.error = None;
            job.updated_at_ms = now_ms;
            self.store.write_job(&self.lock, &job)?;
        }
        Ok(targets.into_iter().collect())
    }

    /// 選んだジョブ `job_ids` を、その成果物を使う後のジョブとあわせて消す（決め方は [`delete_plan`]）。消した
    /// ジョブと、消さなかった選んだジョブを返す
    ///
    /// 後のジョブ（番号の大きいジョブ。依存するジョブは前のジョブに限る）から消す（途中で消せなくなっても、成果物を
    /// 使う後のジョブだけが残らないように）。
    pub fn delete(&mut self, job_ids: &[u64]) -> Result<DeletePlan, StoreError> {
        let (jobs, _) = self.store.load_jobs()?;
        let plan = delete_plan(&jobs, job_ids);
        for &job_id in plan.deleted.iter().rev() {
            self.store.delete_job(&self.lock, job_id)?;
        }
        Ok(plan)
    }

    /// 取り込めなかった投入 `submission_ids` を消す（もうないものは数えない）
    pub fn remove_rejected(&mut self, submission_ids: &[String]) -> Result<(), StoreError> {
        for submission_id in submission_ids {
            self.store
                .remove_rejected_submission(&self.lock, submission_id)?;
        }
        Ok(())
    }

    /// 計算待ちのジョブの優先順位を `delta` だけ変える。変えたら `true`（計算待ちでないジョブは `false`）
    ///
    /// そのジョブを待つ、計算待ちの後のジョブも同じだけ変える（待っているジョブの優先順位は、実行の順を決めるときに
    /// 前のジョブへ伝わるため、前のジョブだけを下げても順が変わらない。5.5）。
    pub fn change_priority(
        &mut self,
        job_id: u64,
        delta: i32,
        now_ms: u64,
    ) -> Result<bool, StoreError> {
        let (jobs, _) = self.store.load_jobs()?;
        let queued = |id: &u64| {
            jobs.iter()
                .any(|job| job.job_id == *id && job.status == StoredJobStatus::Queued)
        };
        if !queued(&job_id) {
            return Ok(false);
        }
        let targets = later_jobs(&jobs, job_id, queued);
        for mut job in jobs
            .iter()
            .filter(|job| targets.contains(&job.job_id))
            .cloned()
        {
            job.priority = job.priority.saturating_add(delta);
            job.updated_at_ms = now_ms;
            self.store.write_job(&self.lock, &job)?;
        }
        Ok(true)
    }

    fn collect_finished(
        &mut self,
        jobs: &mut [StoredJob],
        now_ms: u64,
        events: &mut Vec<ManagerEvent>,
    ) -> Result<(), StoreError> {
        let mut finished = Vec::new();
        for (&job_id, running) in &mut self.running {
            if let Some(code) = running.try_finish()? {
                finished.push((job_id, code));
            }
        }
        finished.sort_unstable();
        for (job_id, code) in finished {
            let Some(job) = jobs.iter_mut().find(|job| job.job_id == job_id) else {
                self.running.remove(&job_id);
                continue;
            };
            let domain = domain(job.kind);
            if code == Some(0) && self.store.result_file(job_id, job.kind).is_file() {
                job.status = StoredJobStatus::Succeeded;
                job.result_ref = RefFactory::result(domain, job_id, &["ok"]).ok();
                job.log_ref = RefFactory::log(domain, job_id, &["ok"]).ok();
                job.last_error = None;
                job.error = None;
                job.updated_at_ms = now_ms;
            } else {
                let (error, log_line) = runner_failure(code, &self.store, job_id);
                fail(job, error, log_line, now_ms);
                job.log_ref = RefFactory::log(domain, job_id, &["failed"]).ok();
            }
            // 書けなければ、終わったランナーを残し、次の回に記録し直す
            self.store.write_job(&self.lock, job)?;
            self.running.remove(&job_id);
            events.push(ManagerEvent::Finished {
                job_id,
                status: job.status,
                error: job.last_error.clone(),
            });
        }
        Ok(())
    }

    fn schedule(
        &mut self,
        jobs: &mut [StoredJob],
        unreadable: &[u64],
        now_ms: u64,
        events: &mut Vec<ManagerEvent>,
    ) -> Result<(), StoreError> {
        loop {
            let plan = plan_schedule(jobs, unreadable);
            for (job_id, reason) in plan.blocked {
                if let Some(job) = jobs.iter_mut().find(|job| job.job_id == job_id) {
                    fail(job, reason.clone(), None, now_ms);
                    self.store.write_job(&self.lock, job)?;
                    events.push(ManagerEvent::Blocked {
                        job_id,
                        reason: reason.to_string(),
                    });
                }
            }
            let Some(next) = plan.next.filter(|_| self.running.len() < MAX_RUNNING_JOBS) else {
                return Ok(());
            };
            let Some(index) = jobs.iter().position(|job| job.job_id == next) else {
                return Ok(());
            };
            let spec = self.launch_spec(jobs, &jobs[index]);
            let job = &mut jobs[index];
            job.attempts += 1;
            job.updated_at_ms = now_ms;
            job.started_at_ms = Some(now_ms);
            match self.launcher.launch(&spec) {
                Ok(mut running) => {
                    job.status = StoredJobStatus::Running;
                    if let Err(error) = self.store.write_job(&self.lock, job) {
                        // 記録できないランナーは、二重に起動しないように止める
                        let _ = running.stop();
                        job.status = StoredJobStatus::Queued;
                        return Err(error);
                    }
                    self.running.insert(next, running);
                    events.push(ManagerEvent::Started { job_id: next });
                }
                Err(error) => {
                    fail(
                        job,
                        StoredJobError::RunnerNotStarted,
                        Some(error.to_string()),
                        now_ms,
                    );
                    self.store.write_job(&self.lock, job)?;
                    events.push(ManagerEvent::Finished {
                        job_id: next,
                        status: job.status,
                        error: job.last_error.clone(),
                    });
                }
            }
        }
    }

    /// ジョブ `job` のランナーの引数（パスは保存先のジョブのフォルダの決まった名前だけ）
    fn launch_spec(&self, jobs: &[StoredJob], job: &StoredJob) -> LaunchSpec {
        // 切削シミュレーションは、依存するツールパス生成のジョブ（投入の検証で 1 つに限る）の成果物を読む。依存する
        // ジョブを持たないもの（投入の検証で、CL のファイルを持つものに限る）は、ジョブの入力のフォルダの CL を読む
        let toolpath = if job.kind != StoredJobKind::Sim {
            None
        } else if job.depends_on.is_empty() {
            Some(self.store.input_toolpath_file(job.job_id))
        } else {
            job.depends_on
                .iter()
                .find(|&&id| {
                    jobs.iter()
                        .any(|other| other.job_id == id && other.kind == StoredJobKind::Cam)
                })
                .map(|&id| self.store.result_file(id, StoredJobKind::Cam))
        };
        LaunchSpec {
            job_type: domain(job.kind),
            input: self.store.input_file(job.job_id),
            toolpath,
            output: self.store.output_dir(job.job_id),
            logs: self.store.logs_dir(job.job_id),
            progress: self.store.progress_file(job.job_id),
        }
    }
}

/// ジョブ `start` と、その成果物を使う後のジョブ（依存関係を後の向きにたどる。`member` を満たすジョブだけを
/// たどる。小さい順）
fn later_jobs(jobs: &[StoredJob], start: u64, member: impl Fn(&u64) -> bool) -> BTreeSet<u64> {
    let mut found = BTreeSet::from([start]);
    let mut pending = vec![start];
    while let Some(id) = pending.pop() {
        for job in jobs {
            if job.depends_on.contains(&id) && member(&job.job_id) && found.insert(job.job_id) {
                pending.push(job.job_id);
            }
        }
    }
    found
}

/// ジョブ `start` と、依存関係（前のジョブ・後のジョブのどちら向きも）でつながる、`member` を満たすジョブ（小さい順）
fn connected_jobs(jobs: &[StoredJob], start: u64, member: impl Fn(&u64) -> bool) -> BTreeSet<u64> {
    let mut found = BTreeSet::from([start]);
    let mut pending = vec![start];
    while let Some(id) = pending.pop() {
        let neighbors = jobs.iter().filter_map(|job| {
            if job.job_id == id {
                Some(job.depends_on.clone())
            } else if job.depends_on.contains(&id) {
                Some(vec![job.job_id])
            } else {
                None
            }
        });
        for neighbor in neighbors.flatten() {
            if member(&neighbor) && found.insert(neighbor) {
                pending.push(neighbor);
            }
        }
    }
    found
}

fn domain(kind: StoredJobKind) -> &'static str {
    match kind {
        StoredJobKind::Cam => "cam",
        StoredJobKind::Sim => "sim",
    }
}

/// ジョブ `job` を、理由の種類 `error`（`detail` は英語の文に添える詳しい理由）で失敗にする
fn fail(job: &mut StoredJob, error: StoredJobError, detail: Option<String>, now_ms: u64) {
    job.status = StoredJobStatus::Failed;
    job.last_error = Some(match detail {
        Some(detail) => format!("{error}: {detail}"),
        None => error.to_string(),
    });
    job.error = Some(error);
    job.updated_at_ms = now_ms;
}

/// ランナーの失敗の理由（終了の番号の意味と、ログの最後の行）
fn runner_failure(
    code: Option<i32>,
    store: &JobStore,
    job_id: u64,
) -> (StoredJobError, Option<String>) {
    let error = match code {
        Some(0) => StoredJobError::RunnerWroteNoResult,
        Some(2) => StoredJobError::RunnerInvalidArguments,
        Some(3) => StoredJobError::RunnerCannotReadInput,
        Some(4) => StoredJobError::InvalidInput,
        Some(5) => StoredJobError::CalculationFailed,
        Some(6) => StoredJobError::RunnerCannotWriteResult,
        Some(code) => StoredJobError::RunnerExitCode { code },
        None => StoredJobError::RunnerStopped,
    };
    let log = store.log_file(job_id);
    let mut partial = log.clone().into_os_string();
    partial.push(".tmp");
    (
        error,
        last_log_line(&log).or_else(|| last_log_line(Path::new(&partial))),
    )
}

/// ログの最後の空でない行（末尾の `MAX_LOG_TAIL_BYTES` バイトから。`MAX_LOG_LINE_CHARS` 文字まで）
fn last_log_line(path: &Path) -> Option<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(MAX_LOG_TAIL_BYTES)))
        .ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    String::from_utf8_lossy(&bytes)
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().chars().take(MAX_LOG_LINE_CHARS).collect())
}
