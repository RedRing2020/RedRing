//! ジョブマネージャーに問い合わせる口を通して、計算させるクライアント（設計: JOB_IPC_DESIGN.md）
//!
//! 保存先のパスを知らずに、投入・状態の読み取り・成果物の取り込みを、口の要求で行う。口につなげなければ、このアプリと
//! 同じフォルダのジョブマネージャーを起動して、口が開くのを待つ。接続は操作ごとにつなぎ直す（ジョブマネージャーを
//! 起動し直した・保存先を替えた後も、そのまま使える）。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use cam_sim::{
    read_cam_solver_input_v1, read_cutting_simulation_input_v1, write_cam_solver_input_v1,
    write_cutting_simulation_input_v1,
};
use job_domain::{CamJobSource, CamJobStatus};
use job_ipc::{
    Client, ClientError, Connection, Dependency, Endpoint, ErrorKind, JobFile, JobKind, JobSource,
    JobStatus, JobSummary, SubmitJob,
};

use super::{JobClient, JobSubmission, PendingSubmission, SimulationJob};
use super::{SimulationSubmission, SimulationToolpath, ToolpathJob};
use crate::cam_simulation_workflow::{CamSimulationWorkflowError, CamSimulationWorkflowResult};

/// ジョブマネージャーの実行ファイルの名前（拡張子を除く）
const JOB_MANAGER_NAME: &str = "redring-job-manager";
/// 投入の入力の名前
const CAM_INPUT_FILE: &str = "cam.bin";
const SIMULATION_INPUT_FILE: &str = "sim.bin";
/// 切削シミュレーションが入力にする CL の名前
const TOOLPATH_INPUT_FILE: &str = "toolpath.bin";
/// 起動したジョブマネージャーの口が開くのを待つ時間
const MANAGER_START_WAIT: Duration = Duration::from_secs(5);
/// 口が空いていない（つないでいる数が上限・次の接続を待つ用意の間）ときに、空くのを待つ時間
const BUSY_WAIT: Duration = Duration::from_secs(2);
/// 起動したジョブマネージャーの口が開くのを待つ間に、確かめる間隔
const START_POLL_INTERVAL: Duration = Duration::from_millis(50);
/// ほかのジョブマネージャーが保存先を開いているときの、ジョブマネージャーの終了の番号
const MANAGER_ALREADY_RUNNING: i32 = 1;
/// ジョブマネージャーを起動し直さない間（起動してすぐに終わるときに、操作のたびに起動しないように）
const MANAGER_RESTART_INTERVAL: Duration = Duration::from_secs(10);

/// 口を通して計算させるクライアント
pub(crate) struct PipeJobClient {
    /// ジョブマネージャーの口（利用者を確かめられなかったときは、その理由）
    endpoint: Result<Endpoint, String>,
    /// ジョブマネージャーの実行ファイル
    manager: PathBuf,
    /// 起動したジョブマネージャー（終わったら片付ける。アプリを閉じても止めない）
    started: RefCell<Option<Child>>,
    /// 最後にジョブマネージャーを起動した時刻
    started_at: Cell<Option<Instant>>,
}

impl std::fmt::Debug for PipeJobClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipeJobClient")
            .field("endpoint", &self.endpoint)
            .field("manager", &self.manager)
            .finish_non_exhaustive()
    }
}

impl PipeJobClient {
    /// この利用者のジョブマネージャーの口と、このアプリの実行ファイルと同じフォルダのジョブマネージャーを使う
    pub(crate) fn from_environment() -> Self {
        let manager = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .unwrap_or_default()
            .join(format!(
                "{JOB_MANAGER_NAME}{}",
                std::env::consts::EXE_SUFFIX
            ));
        Self::new(
            Endpoint::for_current_user().map_err(|error| error.to_string()),
            manager,
        )
    }

    /// 口 `endpoint` と、ジョブマネージャーの実行ファイル `manager` を使う
    pub(crate) fn new(endpoint: Result<Endpoint, String>, manager: PathBuf) -> Self {
        Self {
            endpoint,
            manager,
            started: RefCell::new(None),
            started_at: Cell::new(None),
        }
    }

    fn endpoint(&self) -> Result<&Endpoint, CamSimulationWorkflowError> {
        self.endpoint
            .as_ref()
            .map_err(|reason| store_error(reason.clone()))
    }

    /// 口につなぐ（ジョブマネージャーが動いていなければ、ウィンドウを出さずに起動して、口が開くのを待つ。起動し直さない
    /// 間は待たない（画面を止めないように））
    fn connect(&self) -> Result<Client<Connection>, CamSimulationWorkflowError> {
        let endpoint = self.endpoint()?;
        let connection = match connect_now(endpoint) {
            Ok(connection) => connection,
            Err(error) if not_running(&error) => {
                if !self.start_manager(true)? {
                    return Err(self.start_error(&error));
                }
                self.wait_for_started(endpoint)?
            }
            Err(error) => return Err(store_error(error.to_string())),
        };
        Client::new(connection).map_err(client_error)
    }

    /// 起動したジョブマネージャーの口が開くのを待ってつなぐ（起動したジョブマネージャーが終われば、待たずに誤りにする）
    fn wait_for_started(
        &self,
        endpoint: &Endpoint,
    ) -> Result<Connection, CamSimulationWorkflowError> {
        let deadline = Instant::now() + MANAGER_START_WAIT;
        loop {
            let error = match connect_now(endpoint) {
                Ok(connection) => return Ok(connection),
                Err(error) => error,
            };
            let exited = self
                .started
                .borrow_mut()
                .as_mut()
                .is_some_and(|child| !matches!(child.try_wait(), Ok(None)));
            if !not_running(&error) || exited || Instant::now() >= deadline {
                return Err(self.start_error(&error));
            }
            std::thread::sleep(START_POLL_INTERVAL);
        }
    }

    /// ジョブマネージャーを起動する（シェルを通さず、決まった場所の実行ファイルだけ。`background` のときはウィンドウを
    /// 出さない）。起動したら `true`。前に起動したジョブマネージャーが動いている・起動してから
    /// `MANAGER_RESTART_INTERVAL` の間は、起動し直さずに `false`
    fn start_manager(&self, background: bool) -> Result<bool, CamSimulationWorkflowError> {
        if !self.manager.is_file() {
            return Err(CamSimulationWorkflowError::JobManagerNotFound);
        }
        // 前に起動したジョブマネージャーが終わっていれば片付ける
        let mut started = self.started.borrow_mut();
        if let Some(child) = started.as_mut()
            && !matches!(child.try_wait(), Ok(None))
        {
            *started = None;
        }
        if started.is_some()
            || self
                .started_at
                .get()
                .is_some_and(|at| at.elapsed() < MANAGER_RESTART_INTERVAL)
        {
            return Ok(false);
        }
        let mut command = Command::new(&self.manager);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if background {
            command.arg("--background");
        }
        hide_console_window(&mut command);
        let child = command
            .spawn()
            .map_err(|error| store_error(format!("cannot start the job manager: {error}")))?;
        *started = Some(child);
        self.started_at.set(Some(Instant::now()));
        Ok(true)
    }

    /// 起動したジョブマネージャーの口につなげなかった理由（すぐに終わっていれば、終わり方）
    fn start_error(&self, error: &io::Error) -> CamSimulationWorkflowError {
        let exited = self
            .started
            .borrow_mut()
            .as_mut()
            .and_then(|child| child.try_wait().ok().flatten());
        match exited {
            // 口を開かない（前の版の）ジョブマネージャーが、保存先を開いている
            Some(status) if status.code() == Some(MANAGER_ALREADY_RUNNING) => store_error(
                "another job manager that does not answer requests is running; quit it and try again",
            ),
            Some(status) => store_error(format!("the job manager stopped: {status}")),
            None => store_error(format!("cannot connect to the job manager: {error}")),
        }
    }

    fn jobs(&self) -> Result<Vec<JobSummary>, CamSimulationWorkflowError> {
        self.connect()?.jobs().map_err(client_error)
    }

    fn submit_jobs(
        &self,
        jobs: Vec<SubmitJob>,
        inputs: &[(&str, &[u8])],
    ) -> Result<String, CamSimulationWorkflowError> {
        self.connect()?.submit(jobs, inputs).map_err(client_error)
    }
}

/// 口につなぐ（口が空いていないときは、少しだけ待つ。口がないときは待たない）
fn connect_now(endpoint: &Endpoint) -> io::Result<Connection> {
    match endpoint.connect(Duration::ZERO) {
        Err(error) if error.kind() == io::ErrorKind::ResourceBusy => endpoint.connect(BUSY_WAIT),
        result => result,
    }
}

/// 口がない・動いていない（ジョブマネージャーを起動すればつなげる）
fn not_running(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
    )
}

impl JobClient for PipeJobClient {
    fn submit(&mut self, submission: JobSubmission) -> Result<String, CamSimulationWorkflowError> {
        let mut cam_input = Vec::new();
        write_cam_solver_input_v1(&mut cam_input, &submission.solver_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        let mut simulation_input = Vec::new();
        write_cutting_simulation_input_v1(&mut simulation_input, &submission.simulation_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        let source = submission.source.map(ipc_source);
        let job = |kind, input_file: &str, depends_on| SubmitJob {
            kind,
            input_file: input_file.to_string(),
            depends_on,
            priority: 0,
            source: source.clone(),
            toolpath_file: None,
        };
        self.submit_jobs(
            vec![
                job(JobKind::Cam, CAM_INPUT_FILE, vec![]),
                job(
                    JobKind::Sim,
                    SIMULATION_INPUT_FILE,
                    vec![Dependency::Submitted(0)],
                ),
            ],
            &[
                (CAM_INPUT_FILE, &cam_input),
                (SIMULATION_INPUT_FILE, &simulation_input),
            ],
        )
    }

    fn submit_simulation(
        &mut self,
        submission: SimulationSubmission,
    ) -> Result<String, CamSimulationWorkflowError> {
        let mut simulation_input = Vec::new();
        write_cutting_simulation_input_v1(&mut simulation_input, &submission.simulation_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        let job = |depends_on, toolpath_file: Option<&str>| SubmitJob {
            kind: JobKind::Sim,
            input_file: SIMULATION_INPUT_FILE.to_string(),
            depends_on,
            priority: 0,
            source: submission.source.clone().map(ipc_source),
            toolpath_file: toolpath_file.map(str::to_string),
        };
        match &submission.toolpath {
            // すでにあるツールパス生成のジョブの下に置く
            SimulationToolpath::Job(job_id) => self.submit_jobs(
                vec![job(vec![Dependency::Existing(*job_id)], None)],
                &[(SIMULATION_INPUT_FILE, &simulation_input)],
            ),
            SimulationToolpath::Artifact(artifact) => self.submit_jobs(
                vec![job(Vec::new(), Some(TOOLPATH_INPUT_FILE))],
                &[
                    (SIMULATION_INPUT_FILE, &simulation_input),
                    (TOOLPATH_INPUT_FILE, artifact),
                ],
            ),
        }
    }

    fn toolpath_jobs(&self) -> Result<Vec<ToolpathJob>, CamSimulationWorkflowError> {
        let jobs = self.jobs()?;
        Ok(jobs
            .iter()
            .filter(|job| job.kind == JobKind::Cam)
            .map(|job| ToolpathJob {
                job_id: job.job_id,
                source: job.source.as_ref().map(source),
                status: status(job.status),
                simulated: jobs.iter().any(|other| {
                    other.kind == JobKind::Sim && other.depends_on.contains(&job.job_id)
                }),
            })
            .collect())
    }

    fn run_pending(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        // つなげれば（止まっていれば起動して）、ジョブマネージャーが待ちのジョブを実行する
        self.connect()?;
        Ok(0)
    }

    fn open_manager_window(&mut self) -> Result<(), CamSimulationWorkflowError> {
        if !self.manager.is_file() {
            return Err(CamSimulationWorkflowError::JobManagerNotFound);
        }
        let endpoint = self.endpoint()?;
        match connect_now(endpoint) {
            Ok(connection) => {
                // ウィンドウを前に出す権利は、利用者が操作しているこのアプリにあるため、ジョブマネージャーに渡す
                connection.allow_server_foreground();
                Client::new(connection)
                    .and_then(|mut client| client.open_window())
                    .map_err(client_error)
            }
            // 動いていなければ、ウィンドウを出して起動する（起動し直さない間は、理由を返す）
            Err(error) if not_running(&error) => {
                if self.start_manager(false)? {
                    Ok(())
                } else {
                    Err(self.start_error(&error))
                }
            }
            Err(error) => Err(store_error(error.to_string())),
        }
    }

    fn simulation_jobs(&self) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError> {
        let jobs = self.jobs()?;
        let by_id: HashMap<u64, &JobSummary> = jobs.iter().map(|job| (job.job_id, job)).collect();
        Ok(jobs
            .iter()
            .filter(|job| job.kind == JobKind::Sim)
            .map(|job| SimulationJob {
                job_id: job.job_id,
                submission_id: job.submission_id.clone(),
                source: job.source.as_ref().map(source),
                status: status(job.status),
                // CL を入力にするジョブ（依存するジョブを持たない）は、入力の CL があればツールパスができている（なければ
                // 失敗として扱う）。一覧にないツールパス生成のジョブは、待ちとして扱う
                toolpath_status: if job.depends_on.is_empty() {
                    if job.has_input_toolpath {
                        CamJobStatus::Succeeded
                    } else {
                        CamJobStatus::Failed
                    }
                } else {
                    toolpath_job(&by_id, job)
                        .map_or(CamJobStatus::Queued, |toolpath| status(toolpath.status))
                },
            })
            .collect())
    }

    fn pending_submissions(&self) -> Result<Vec<PendingSubmission>, CamSimulationWorkflowError> {
        let submissions = self
            .connect()?
            .pending_submissions()
            .map_err(client_error)?;
        Ok(submissions
            .into_iter()
            .map(|submission| PendingSubmission {
                source: submission.source.as_ref().map(source),
                rejected: submission.rejected,
            })
            .collect())
    }

    fn import(
        &self,
        simulation_job_id: u64,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError> {
        let mut client = self.connect()?;
        let jobs = client.jobs().map_err(client_error)?;
        let by_id: HashMap<u64, &JobSummary> = jobs.iter().map(|job| (job.job_id, job)).collect();
        let not_succeeded =
            || CamSimulationWorkflowError::NotSucceededSimulation(simulation_job_id);
        let simulation = by_id
            .get(&simulation_job_id)
            .filter(|job| job.kind == JobKind::Sim && job.status == JobStatus::Succeeded)
            .ok_or_else(not_succeeded)?;
        // CL を入力にするジョブ（依存するジョブを持たない）は、ツールパス生成の入力を持たない
        let toolpath = if simulation.depends_on.is_empty() {
            None
        } else {
            Some(
                toolpath_job(&by_id, simulation)
                    .filter(|job| job.status == JobStatus::Succeeded)
                    .ok_or_else(not_succeeded)?,
            )
        };
        let missing_input = || CamSimulationWorkflowError::MissingInput(simulation_job_id);
        let missing_artifact = || CamSimulationWorkflowError::MissingArtifact(simulation_job_id);
        // ないジョブ・ないファイルは、入力・成果物がない誤りにする（ほかの誤りは、そのまま返す）
        let mut read = |job_id: u64,
                        file: JobFile,
                        missing: &dyn Fn() -> CamSimulationWorkflowError|
         -> Result<Vec<u8>, CamSimulationWorkflowError> {
            client
                .read_job_file(job_id, file)
                .map_err(|error| match error {
                    ClientError::Remote {
                        kind: ErrorKind::JobNotFound | ErrorKind::FileNotFound,
                        ..
                    } => missing(),
                    error => client_error(error),
                })
        };
        let solver_input = match toolpath {
            Some(toolpath) => Some(
                read_cam_solver_input_v1(
                    &mut read(toolpath.job_id, JobFile::Input, &missing_input)?.as_slice(),
                )
                .map_err(|_| missing_input())?,
            ),
            None => None,
        };
        let simulation_input = read_cutting_simulation_input_v1(
            &mut read(simulation_job_id, JobFile::Input, &missing_input)?.as_slice(),
        )
        .map_err(|_| missing_input())?;
        let toolpath_artifact = match toolpath {
            Some(toolpath) => read(toolpath.job_id, JobFile::Result, &missing_artifact)?,
            None => read(simulation_job_id, JobFile::InputToolpath, &missing_input)?,
        };
        let simulation_artifact = read(simulation_job_id, JobFile::Result, &missing_artifact)?;
        Ok(CamSimulationWorkflowResult {
            toolpath_artifact,
            simulation_artifact,
            solver_input,
            simulation_input,
            source: simulation.source.as_ref().map(source),
        })
    }
}

/// 切削シミュレーションが入力にするツールパス生成のジョブ（番号の索引から）
fn toolpath_job<'a>(
    by_id: &HashMap<u64, &'a JobSummary>,
    simulation: &JobSummary,
) -> Option<&'a JobSummary> {
    simulation
        .depends_on
        .iter()
        .filter_map(|id| by_id.get(id).copied())
        .find(|job| job.kind == JobKind::Cam)
}

fn ipc_source(source: CamJobSource) -> JobSource {
    JobSource {
        source_id: source.source_id,
        revision: source.revision,
        revision_key: source.revision_key,
    }
}

fn source(source: &JobSource) -> CamJobSource {
    CamJobSource {
        source_id: source.source_id.clone(),
        revision: source.revision,
        revision_key: source.revision_key.clone(),
    }
}

fn status(status: JobStatus) -> CamJobStatus {
    match status {
        JobStatus::Queued => CamJobStatus::Queued,
        JobStatus::Running => CamJobStatus::Running,
        JobStatus::Succeeded => CamJobStatus::Succeeded,
        JobStatus::Failed => CamJobStatus::Failed,
        JobStatus::Canceled => CamJobStatus::Canceled,
    }
}

fn store_error(reason: impl Into<String>) -> CamSimulationWorkflowError {
    CamSimulationWorkflowError::JobStore(reason.into())
}

fn client_error(error: ClientError) -> CamSimulationWorkflowError {
    store_error(error.to_string())
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
