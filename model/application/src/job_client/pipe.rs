//! ジョブマネージャーに問い合わせる口を通して、計算させるクライアント（設計: JOB_IPC_DESIGN.md）
//!
//! 保存先のパスを知らずに、投入・状態の読み取り・成果物の取り込みを、口の要求で行う。口につなげなければ、このアプリと
//! 同じフォルダのジョブマネージャーを起動して、口が開くのを待つ。接続は操作ごとにつなぎ直し（ジョブマネージャーを
//! 起動し直した・保存先を替えた後も、そのまま使える）、読み書きには時間の上限を設ける。
//!
//! 計算の状態は、別のスレッドで 1 つの接続で読み、呼ぶ側には最後に読んだ状態を返す（画面を止めないように。ジョブ
//! マネージャーの起動を待つのも、そのスレッド）。投入できたら、取り込みを待つ投入として状態にすぐに出す。

use std::cell::OnceCell;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
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

use super::{JobClient, JobStatusSnapshot, JobSubmission, PendingSubmission, SimulationJob};
use super::{SimulationSubmission, SimulationToolpath, ToolpathJob};
use crate::cam_simulation_workflow::{
    CamSimulationWorkflowError, CamSimulationWorkflowResult, CamToolpathResult,
};

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
/// 口の読み書き 1 回ごとの時間の上限（ジョブマネージャーが応答しなくなっても、待ち続けないように）
const IO_TIMEOUT: Duration = Duration::from_secs(30);
/// 計算の状態の読み直しを頼む間隔（呼ぶ側は、読み終えるのを待たずに何度でも呼べる）
const STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
/// これより前に読んだ計算の状態は、古いとして返さない（ウィンドウを開き直したときに、前の状態を出さないように）
const STATUS_STALE_AFTER: Duration = Duration::from_secs(10);
/// ジョブマネージャーを起動し直さない間（起動してすぐに終わるときに、操作のたびに起動しないように）
const MANAGER_RESTART_INTERVAL: Duration = Duration::from_secs(10);

/// 口を通して計算させるクライアント
pub(crate) struct PipeJobClient {
    connector: Arc<Connector>,
    /// 別のスレッドで読む、計算の状態
    status: Arc<Mutex<StatusState>>,
    /// 状態を読むスレッドへの頼み（初めて頼むときに、スレッドを始める）
    status_requests: OnceCell<Sender<Work>>,
}

impl std::fmt::Debug for PipeJobClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipeJobClient")
            .field("endpoint", &self.connector.endpoint)
            .field("manager", &self.connector.manager)
            .finish_non_exhaustive()
    }
}

/// 状態を読むスレッドへの頼み
enum Work {
    /// 計算の状態を読み直す
    Refresh,
    /// 切削シミュレーションのジョブの結果を取り込んだと伝える
    RecordImported(u64),
}

/// 別のスレッドで読んだ計算の状態
#[derive(Default)]
struct StatusState {
    /// 最後に読んだ状態と、読んだ時刻（まだ読めていなければ `None`）
    latest: Option<(
        Result<JobStatusSnapshot, CamSimulationWorkflowError>,
        Instant,
    )>,
    /// 投入するたびに増やす数（読んでいる間に投入したら、読んだ状態を捨てて読み直す）
    generation: u64,
    /// 最後に読み直しを頼んだ時刻
    requested_at: Option<Instant>,
}

/// 口につなぐ役（ジョブマネージャーの起動を含む。画面のスレッドと、状態を読むスレッドで共有する）
struct Connector {
    /// ジョブマネージャーの口（利用者を確かめられなかったときは、その理由）
    endpoint: Result<Endpoint, String>,
    /// ジョブマネージャーの実行ファイル
    manager: PathBuf,
    starter: Mutex<Starter>,
}

/// 起動したジョブマネージャー
#[derive(Default)]
struct Starter {
    /// 起動したジョブマネージャー（終わったら片付ける。アプリを閉じても止めない）
    started: Option<Child>,
    /// 最後にジョブマネージャーを起動した時刻
    started_at: Option<Instant>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
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
            connector: Arc::new(Connector {
                endpoint,
                manager,
                starter: Mutex::new(Starter::default()),
            }),
            status: Arc::new(Mutex::new(StatusState::default())),
            status_requests: OnceCell::new(),
        }
    }

    fn jobs(&self) -> Result<Vec<JobSummary>, CamSimulationWorkflowError> {
        self.connector.connect()?.jobs().map_err(client_error)
    }

    /// ジョブの列を投入する（投入できたら、取り込みを待つ投入として、計算の状態にすぐに出す）
    fn submit_jobs(
        &self,
        jobs: Vec<SubmitJob>,
        inputs: &[(&str, &[u8])],
    ) -> Result<String, CamSimulationWorkflowError> {
        let submitted_source = jobs.first().and_then(|job| job.source.as_ref()).map(source);
        let submission_id = self
            .connector
            .connect()?
            .submit(jobs, inputs)
            .map_err(client_error)?;
        {
            let mut state = lock(&self.status);
            state.generation += 1;
            let pending = PendingSubmission {
                source: submitted_source,
                rejected: false,
            };
            match state.latest.as_mut() {
                Some((Ok(snapshot), read_at)) => {
                    snapshot.pending.push(pending);
                    *read_at = Instant::now();
                }
                _ => {
                    state.latest = Some((
                        Ok(JobStatusSnapshot {
                            pending: vec![pending],
                            simulations: Vec::new(),
                        }),
                        Instant::now(),
                    ));
                }
            }
        }
        self.request_status();
        Ok(submission_id)
    }

    /// 計算の状態を読み直すように頼む（読むスレッドがなければ始める。待たない）
    fn request_status(&self) {
        lock(&self.status).requested_at = Some(Instant::now());
        self.send(Work::Refresh);
    }

    /// 状態を読むスレッドに頼む（スレッドがなければ始める。待たない）
    fn send(&self, work: Work) {
        let requests = self.status_requests.get_or_init(|| {
            let (sender, receiver) = mpsc::channel();
            let connector = Arc::clone(&self.connector);
            let status = Arc::clone(&self.status);
            std::thread::spawn(move || read_status_loop(&connector, &status, &receiver));
            sender
        });
        let _ = requests.send(work);
    }
}

/// 頼まれるたびに計算の状態を読み、`status` に置く。取り込んだことを伝える頼みは、そのまま伝える（頼む側が
/// なくなったら終わる）
fn read_status_loop(connector: &Connector, status: &Mutex<StatusState>, requests: &Receiver<Work>) {
    while let Ok(work) = requests.recv() {
        let mut works = vec![work];
        works.extend(requests.try_iter());
        // 溜まった読み直しの頼みは、1 回の読み取りにまとめる
        let mut refresh = false;
        for work in works {
            match work {
                Work::Refresh => refresh = true,
                Work::RecordImported(simulation_job_id) => {
                    record_imported(connector, simulation_job_id);
                }
            }
        }
        if !refresh {
            continue;
        }
        loop {
            while let Ok(work) = requests.try_recv() {
                if let Work::RecordImported(simulation_job_id) = work {
                    record_imported(connector, simulation_job_id);
                }
            }
            let generation = lock(status).generation;
            // 状態を読むだけでは、ジョブマネージャーを起動しない（動いていなければ、そのことを返す）
            let read = read_status(connector, false);
            let mut state = lock(status);
            if state.generation == generation {
                state.latest = Some((read, Instant::now()));
                break;
            }
        }
    }
}

/// 切削シミュレーションのジョブ `simulation_job_id` の結果を取り込んだと伝える（ジョブマネージャーが動いていなければ
/// 起動せずにやめる。伝えられなくても、取り込みは終わっている）
fn record_imported(connector: &Connector, simulation_job_id: u64) {
    let recorded = connector
        .endpoint()
        .and_then(|endpoint| connect_now(endpoint).map_err(|error| store_error(error.to_string())))
        .and_then(client)
        .and_then(|mut client| {
            client
                .record_imported(simulation_job_id)
                .map_err(client_error)
        });
    if let Err(error) = recorded {
        tracing::warn!("Cannot record the imported toolpath in the job manager: {error}");
    }
}

/// 1 つの接続で、取り込みを待つ投入とジョブの一覧を読む（この順に読む。間に取り込まれた投入は、ジョブの一覧に出る）。
/// `start` のときは、ジョブマネージャーが動いていなければ起動して待つ。そうでなければ、動いていないことを返す
fn read_status(
    connector: &Connector,
    start: bool,
) -> Result<JobStatusSnapshot, CamSimulationWorkflowError> {
    let mut client = if start {
        connector.connect()?
    } else {
        connector.connect_running()?
    };
    let pending = client.pending_submissions().map_err(client_error)?;
    let jobs = client.jobs().map_err(client_error)?;
    Ok(JobStatusSnapshot {
        pending: pending.iter().map(pending_submission).collect(),
        simulations: simulation_jobs(&jobs),
    })
}

impl Connector {
    fn endpoint(&self) -> Result<&Endpoint, CamSimulationWorkflowError> {
        self.endpoint
            .as_ref()
            .map_err(|reason| store_error(reason.clone()))
    }

    /// 口につなぐ（ジョブマネージャーが動いていなければ、ウィンドウを出さずに起動して、口が開くのを待つ。起動し直さない
    /// 間は待たない）。読み書きには時間の上限を設ける
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
        client(connection)
    }

    /// 動いているジョブマネージャーの口につなぐ（起動しない。動いていなければ、実行ファイルがあれば動いていない誤り、
    /// なければ見つからない誤り）。読み書きには時間の上限を設ける
    fn connect_running(&self) -> Result<Client<Connection>, CamSimulationWorkflowError> {
        let endpoint = self.endpoint()?;
        match connect_now(endpoint) {
            Ok(connection) => client(connection),
            Err(error) if not_running(&error) => Err(if self.manager.is_file() {
                CamSimulationWorkflowError::JobManagerNotRunning
            } else {
                CamSimulationWorkflowError::JobManagerNotFound
            }),
            Err(error) => Err(store_error(error.to_string())),
        }
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
            let exited = lock(&self.starter)
                .started
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
        let mut starter = lock(&self.starter);
        // 前に起動したジョブマネージャーが終わっていれば片付ける
        if let Some(child) = starter.started.as_mut()
            && !matches!(child.try_wait(), Ok(None))
        {
            starter.started = None;
        }
        if starter.started.is_some()
            || starter
                .started_at
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
        starter.started = Some(child);
        starter.started_at = Some(Instant::now());
        Ok(true)
    }

    /// 起動したジョブマネージャーの口につなげなかった理由（すぐに終わっていれば、終わり方）
    fn start_error(&self, error: &io::Error) -> CamSimulationWorkflowError {
        let exited = lock(&self.starter)
            .started
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
}

/// 接続 `connection` の読み書きに時間の上限を設け、口の版を確かめる
fn client(mut connection: Connection) -> Result<Client<Connection>, CamSimulationWorkflowError> {
    connection
        .set_io_timeout(Some(IO_TIMEOUT))
        .map_err(|error| store_error(error.to_string()))?;
    Client::new(connection).map_err(client_error)
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
                submission_id: job.submission_id.clone(),
                source: job.source.as_ref().map(source),
                status: status(job.status),
                simulated: jobs.iter().any(|other| {
                    other.kind == JobKind::Sim && other.depends_on.contains(&job.job_id)
                }),
            })
            .collect())
    }

    fn run_pending(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        // ジョブマネージャーが、待ちのジョブを自分で実行する
        Ok(0)
    }

    fn open_manager_window(&mut self) -> Result<(), CamSimulationWorkflowError> {
        let connector = &self.connector;
        if !connector.manager.is_file() {
            return Err(CamSimulationWorkflowError::JobManagerNotFound);
        }
        let endpoint = connector.endpoint()?;
        match connect_now(endpoint) {
            Ok(connection) => {
                // ウィンドウを前に出す権利は、利用者が操作しているこのアプリにあるため、ジョブマネージャーに渡す
                connection.allow_server_foreground();
                client(connection)?.open_window().map_err(client_error)
            }
            // 動いていなければ、ウィンドウを出して起動する（起動し直さない間は、理由を返す）
            Err(error) if not_running(&error) => {
                if connector.start_manager(false)? {
                    Ok(())
                } else {
                    Err(connector.start_error(&error))
                }
            }
            Err(error) => Err(store_error(error.to_string())),
        }
    }

    fn simulation_jobs(&self) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError> {
        Ok(simulation_jobs(&self.jobs()?))
    }

    fn cancel(&mut self, job_ids: &[u64]) -> Result<(), CamSimulationWorkflowError> {
        self.connector
            .connect()?
            .cancel(job_ids.to_vec())
            .map_err(client_error)?;
        // 取り消しが一覧に出るように、読み直しを頼む（待たない）
        self.request_status();
        Ok(())
    }

    fn record_imported(&self, simulation_job_id: u64) -> Result<(), CamSimulationWorkflowError> {
        // 状態を読むスレッドで伝える（画面を止めない。伝えられなかったことは、ログに書く）
        self.send(Work::RecordImported(simulation_job_id));
        Ok(())
    }

    fn read_status_now(&mut self) -> Result<JobStatusSnapshot, CamSimulationWorkflowError> {
        let snapshot = read_status(&self.connector, true)?;
        // 読めた状態を、最後に読んだ状態にもする（読み終えるのを待たずに返す側も、同じ状態を返すように。読めなかった
        // ときは、前に読めた状態を残す）
        let mut state = lock(&self.status);
        state.generation += 1;
        state.latest = Some((Ok(snapshot.clone()), Instant::now()));
        Ok(snapshot)
    }

    fn status_snapshot(&self) -> Result<Option<JobStatusSnapshot>, CamSimulationWorkflowError> {
        let (due, latest) = {
            let state = lock(&self.status);
            let due = state
                .requested_at
                .is_none_or(|at| at.elapsed() >= STATUS_REFRESH_INTERVAL);
            // 古い状態は返さない（読み直すまで、まだ読めていないとする）
            let latest = state
                .latest
                .as_ref()
                .filter(|(_, read_at)| read_at.elapsed() < STATUS_STALE_AFTER)
                .map(|(read, _)| read.clone());
            (due, latest)
        };
        if due {
            self.request_status();
        }
        latest.transpose()
    }

    fn toolpath_result(
        &self,
        toolpath_job_id: u64,
    ) -> Result<CamToolpathResult, CamSimulationWorkflowError> {
        let mut client = self.connector.connect()?;
        let jobs = client.jobs().map_err(client_error)?;
        let toolpath = jobs
            .iter()
            .find(|job| {
                job.job_id == toolpath_job_id
                    && job.kind == JobKind::Cam
                    && job.status == JobStatus::Succeeded
            })
            .ok_or(CamSimulationWorkflowError::NotSucceededToolpath(
                toolpath_job_id,
            ))?;
        // ないジョブ・ないファイルは、成果物がない誤りにする（ほかの誤りは、そのまま返す）
        let artifact = client
            .read_job_file(toolpath_job_id, JobFile::Result)
            .map_err(|error| match error {
                ClientError::Remote {
                    kind: ErrorKind::JobNotFound | ErrorKind::FileNotFound,
                    ..
                } => CamSimulationWorkflowError::MissingArtifact(toolpath_job_id),
                error => client_error(error),
            })?;
        Ok(CamToolpathResult {
            artifact,
            source: toolpath.source.as_ref().map(source),
        })
    }

    fn import(
        &self,
        simulation_job_id: u64,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError> {
        let mut client = self.connector.connect()?;
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

/// ジョブの一覧 `jobs` の、切削シミュレーションのジョブ
fn simulation_jobs(jobs: &[JobSummary]) -> Vec<SimulationJob> {
    let by_id: HashMap<u64, &JobSummary> = jobs.iter().map(|job| (job.job_id, job)).collect();
    jobs.iter()
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
        .collect()
}

fn pending_submission(submission: &job_ipc::PendingSubmission) -> PendingSubmission {
    PendingSubmission {
        source: submission.source.as_ref().map(source),
        rejected: submission.rejected,
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
