//! 同じ PC のジョブの保存先のフォルダを通して、ジョブマネージャーに計算させるクライアント
//! （設計: JOB_MANAGER_STORAGE_DESIGN.md 4.3〜4.5）

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cam_sim::{
    read_cam_solver_input_v1, read_cutting_simulation_input_v1, write_cam_solver_input_v1,
    write_cutting_simulation_input_v1,
};
use job_domain::{CamJobSource, CamJobStatus};
use job_runtime::store::{
    DependencyRef, JobStore, STORE_FORMAT_VERSION, StoredJob, StoredJobKind, StoredJobSource,
    StoredJobStatus, SubmissionRequest, SubmittedJob,
};

use super::{JobClient, JobSubmission, PendingSubmission, SimulationJob};
use crate::cam_simulation_workflow::{CamSimulationWorkflowError, CamSimulationWorkflowResult};

/// ジョブマネージャーの実行ファイルの名前（拡張子を除く）
const JOB_MANAGER_NAME: &str = "redring-job-manager";
/// 投入の入力のファイルの名前
const CAM_INPUT_FILE: &str = "cam.bin";
const SIMULATION_INPUT_FILE: &str = "sim.bin";
/// 読み込む入力・成果物のファイルの大きさの上限
const MAX_FILE_BYTES: u64 = 4 << 30;
/// 起動したジョブマネージャーがすぐに終わらないかを見る時間
const MANAGER_START_WATCH: Duration = Duration::from_millis(300);
/// ほかのジョブマネージャーが動いているときの、ジョブマネージャーの終了の番号
const MANAGER_ALREADY_RUNNING: i32 = 1;

/// ジョブの保存先のフォルダを通して計算させるクライアント
#[derive(Debug)]
pub(crate) struct LocalStoreJobClient {
    /// 保存先（利用者ごとのフォルダが分からないときは `None`）
    root: Option<PathBuf>,
    /// ジョブマネージャーの実行ファイル
    manager: PathBuf,
    /// 起動したジョブマネージャー（終わったら片付ける。アプリを閉じても止めない）
    started: Option<Child>,
}

impl LocalStoreJobClient {
    /// 既定の保存先と、このアプリの実行ファイルと同じフォルダのジョブマネージャーを使う
    pub(crate) fn from_environment() -> Self {
        let manager = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .unwrap_or_default()
            .join(format!(
                "{JOB_MANAGER_NAME}{}",
                std::env::consts::EXE_SUFFIX
            ));
        Self {
            root: JobStore::default_root(),
            manager,
            started: None,
        }
    }

    /// 保存先 `root` と、ジョブマネージャーの実行ファイル `manager` を使う
    #[cfg(test)]
    pub(crate) fn new(root: PathBuf, manager: PathBuf) -> Self {
        Self {
            root: Some(root),
            manager,
            started: None,
        }
    }

    fn store(&self) -> Result<JobStore, CamSimulationWorkflowError> {
        let root = self
            .root
            .as_ref()
            .ok_or_else(|| store_error("the per-user data folder is not found"))?;
        JobStore::open(root).map_err(|error| store_error(error.to_string()))
    }

    /// ジョブマネージャーが保存先を開いていなければ起動する（シェルを通さず、決まった場所の実行ファイルだけ）
    ///
    /// 起動したジョブマネージャーがすぐに終わった場合（ランナーがない等）は、理由つきの失敗にする。ほかの
    /// ジョブマネージャーが動いていて終わった場合は、起動できているとみなす。
    fn ensure_manager_running(
        &mut self,
        store: &JobStore,
    ) -> Result<(), CamSimulationWorkflowError> {
        if !self.manager.is_file() {
            return Err(CamSimulationWorkflowError::JobManagerNotFound);
        }
        // 前に起動したジョブマネージャーが終わっていれば片付ける
        if let Some(child) = self.started.as_mut()
            && !matches!(child.try_wait(), Ok(None))
        {
            self.started = None;
        }
        if self.started.is_some()
            || store
                .is_locked()
                .map_err(|error| store_error(error.to_string()))?
        {
            return Ok(());
        }
        let mut command = Command::new(&self.manager);
        command
            .arg("--store")
            .arg(store.root())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        hide_console_window(&mut command);
        let mut child = command
            .spawn()
            .map_err(|error| store_error(format!("cannot start the job manager: {error}")))?;
        let deadline = Instant::now() + MANAGER_START_WATCH;
        while Instant::now() < deadline {
            match child.try_wait() {
                Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                Ok(Some(status)) if status.code() == Some(MANAGER_ALREADY_RUNNING) => return Ok(()),
                Ok(Some(status)) => {
                    return Err(store_error(format!("the job manager stopped: {status}")));
                }
                Err(error) => return Err(store_error(error.to_string())),
            }
        }
        self.started = Some(child);
        Ok(())
    }

    fn load_jobs(store: &JobStore) -> Result<Vec<StoredJob>, CamSimulationWorkflowError> {
        store
            .load_jobs()
            .map(|(jobs, _)| jobs)
            .map_err(|error| store_error(error.to_string()))
    }
}

impl JobClient for LocalStoreJobClient {
    fn submit(&mut self, submission: JobSubmission) -> Result<String, CamSimulationWorkflowError> {
        let store = self.store()?;
        // 起動できないときに投入だけが残らないように、投入より先に起動を確かめる（起動したジョブマネージャーは、
        // 後から置いた投入も読む）
        self.ensure_manager_running(&store)?;
        let mut cam_input = Vec::new();
        write_cam_solver_input_v1(&mut cam_input, &submission.solver_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        let mut simulation_input = Vec::new();
        write_cutting_simulation_input_v1(&mut simulation_input, &submission.simulation_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        let source = submission.source.map(|source| StoredJobSource {
            source_id: source.source_id,
            revision: source.revision,
        });
        let job = |kind, input_file: &str, depends_on| SubmittedJob {
            kind,
            input_file: input_file.to_string(),
            depends_on,
            priority: 0,
            group_id: None,
            source: source.clone(),
        };
        let request = SubmissionRequest {
            format_version: STORE_FORMAT_VERSION,
            jobs: vec![
                job(StoredJobKind::Cam, CAM_INPUT_FILE, vec![]),
                job(
                    StoredJobKind::Sim,
                    SIMULATION_INPUT_FILE,
                    vec![DependencyRef::Submitted(0)],
                ),
            ],
        };
        let submission_id = new_submission_id();
        store
            .write_submission(
                &submission_id,
                &request,
                &[
                    (CAM_INPUT_FILE, &cam_input),
                    (SIMULATION_INPUT_FILE, &simulation_input),
                ],
            )
            .map_err(|error| store_error(error.to_string()))?;
        Ok(submission_id)
    }

    fn run_pending(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        let store = self.store()?;
        self.ensure_manager_running(&store)?;
        Ok(0)
    }

    fn simulation_jobs(&self) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError> {
        let jobs = Self::load_jobs(&self.store()?)?;
        let by_id: HashMap<u64, &StoredJob> = jobs.iter().map(|job| (job.job_id, job)).collect();
        Ok(jobs
            .iter()
            .filter(|job| job.kind == StoredJobKind::Sim)
            .map(|job| SimulationJob {
                job_id: job.job_id,
                submission_id: job.submission_id.clone(),
                source: job.source.as_ref().map(source),
                status: status(job.status),
                // 記録を読めないツールパス生成のジョブは、待ちとして扱う
                toolpath_status: toolpath_job_in(&by_id, job)
                    .map_or(CamJobStatus::Queued, |toolpath| status(toolpath.status)),
            })
            .collect())
    }

    fn pending_submissions(&self) -> Result<Vec<PendingSubmission>, CamSimulationWorkflowError> {
        let submissions = self
            .store()?
            .inbox_submissions()
            .map_err(|error| store_error(error.to_string()))?;
        Ok(submissions
            .into_iter()
            .map(|submission| PendingSubmission {
                source: submission
                    .request
                    .as_ref()
                    .and_then(|request| request.jobs.first())
                    .and_then(|job| job.source.as_ref())
                    .map(source),
                rejected: submission.rejected.is_some(),
            })
            .collect())
    }

    fn import(
        &self,
        simulation_job_id: u64,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError> {
        let store = self.store()?;
        let jobs = Self::load_jobs(&store)?;
        let not_succeeded =
            || CamSimulationWorkflowError::NotSucceededSimulation(simulation_job_id);
        let simulation = jobs
            .iter()
            .find(|job| job.job_id == simulation_job_id)
            .filter(|job| {
                job.kind == StoredJobKind::Sim && job.status == StoredJobStatus::Succeeded
            })
            .ok_or_else(not_succeeded)?;
        let toolpath = toolpath_job(&jobs, simulation)
            .filter(|job| job.status == StoredJobStatus::Succeeded)
            .ok_or_else(not_succeeded)?;

        let missing_input = || CamSimulationWorkflowError::MissingInput(simulation_job_id);
        let solver_input = read_limited(&store.input_file(toolpath.job_id))
            .ok()
            .and_then(|bytes| read_cam_solver_input_v1(&mut bytes.as_slice()).ok())
            .ok_or_else(missing_input)?;
        let simulation_input = read_limited(&store.input_file(simulation_job_id))
            .ok()
            .and_then(|bytes| read_cutting_simulation_input_v1(&mut bytes.as_slice()).ok())
            .ok_or_else(missing_input)?;
        // 成果物のファイルは、記録の参照ではなく、ジョブの番号と種類から決まる名前で読む（書き換えた参照で、ほかの
        // ファイルを読ませない）
        let artifact = |job: &StoredJob| {
            job.result_ref
                .as_ref()
                .and_then(|_| read_limited(&store.result_file(job.job_id, job.kind)).ok())
                .ok_or(CamSimulationWorkflowError::MissingArtifact(
                    simulation_job_id,
                ))
        };
        Ok(CamSimulationWorkflowResult {
            toolpath_artifact: artifact(toolpath)?,
            simulation_artifact: artifact(simulation)?,
            solver_input,
            simulation_input,
        })
    }
}

/// 切削シミュレーションが入力にするツールパス生成のジョブ
fn toolpath_job<'a>(jobs: &'a [StoredJob], simulation: &StoredJob) -> Option<&'a StoredJob> {
    simulation.depends_on.iter().find_map(|&id| {
        jobs.iter()
            .find(|job| job.job_id == id && job.kind == StoredJobKind::Cam)
    })
}

/// 切削シミュレーションが入力にするツールパス生成のジョブ（番号の索引から）
fn toolpath_job_in<'a>(
    by_id: &HashMap<u64, &'a StoredJob>,
    simulation: &StoredJob,
) -> Option<&'a StoredJob> {
    simulation
        .depends_on
        .iter()
        .filter_map(|id| by_id.get(id).copied())
        .find(|job| job.kind == StoredJobKind::Cam)
}

fn source(source: &StoredJobSource) -> CamJobSource {
    CamJobSource {
        source_id: source.source_id.clone(),
        revision: source.revision,
    }
}

fn status(status: StoredJobStatus) -> CamJobStatus {
    match status {
        StoredJobStatus::Queued => CamJobStatus::Queued,
        StoredJobStatus::Running => CamJobStatus::Running,
        StoredJobStatus::Succeeded => CamJobStatus::Succeeded,
        StoredJobStatus::Failed => CamJobStatus::Failed,
        StoredJobStatus::Canceled => CamJobStatus::Canceled,
    }
}

fn store_error(reason: impl Into<String>) -> CamSimulationWorkflowError {
    CamSimulationWorkflowError::JobStore(reason.into())
}

/// 重ならない投入の ID（時刻・プロセスの番号・通し番号の 16 進数）
fn new_submission_id() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    format!(
        "{nanos:x}-{:x}-{:x}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

/// 大きさの上限まで読む（読んだ量で確かめる）
fn read_limited(path: &Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "the file is too large",
        ));
    }
    Ok(bytes)
}

/// Windows では、ジョブマネージャーのコンソールのウィンドウを出さない
fn hide_console_window(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = command;
}
