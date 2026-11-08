//! ジョブマネージャーのサービスを、画面とは別のスレッドで一定の間隔で動かす

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use job_manager::{JobManagerService, ManagerEvent, ProcessLauncher};
use job_runtime::store::{JobStore, StoreError};

/// 受付のフォルダとランナーを確かめる間隔
const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// 保存先のロックを取れないときに、やり直す回数と間隔（CAD/CAM のアプリが、ジョブマネージャーが動いているかを
/// 確かめるために、ロックを一瞬だけ持つことがあるため）
const LOCK_ATTEMPTS: u32 = 10;
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(100);

/// 画面からの、ジョブの操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JobCommand {
    Cancel(u64),
    Retry(u64),
    Delete(u64),
    ChangePriority { job_id: u64, delta: i32 },
}

/// サービスへの頼み
pub(crate) enum ServiceRequest {
    /// ジョブの操作（結果は `ServiceChannel::replies` に返す）
    Command(JobCommand),
    /// 実行中のランナーを止めて、サービスを終える（終えたら `done` に送る）
    #[cfg_attr(not(windows), allow(dead_code))]
    Quit { done: Sender<()> },
}

/// 操作の結果（保存先を読み書きできなかったときは理由）
pub(crate) type CommandReply = Result<(), StoreFailure>;

/// ジョブの操作・保存先の読み込みの誤りの種類（画面の言語の文言にするため）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoreFailure {
    /// ほかのジョブマネージャーが保存先を開いている
    AlreadyLocked,
    /// 保存先のファイルが見つからない
    NotFound,
    /// 保存先のファイルを読み書きする権限がない
    PermissionDenied,
    /// 保存先のファイルを読み書きできない（そのほかの理由）
    Io,
    /// ジョブの記録の参照が、保存先のファイルを指さない
    InvalidRef,
    /// ジョブの操作を受けるサービスが止まった
    ServiceStopped,
}

impl StoreFailure {
    /// 保存先の誤り `error` の種類
    pub(crate) fn of(error: &StoreError) -> Self {
        match error {
            StoreError::AlreadyLocked => Self::AlreadyLocked,
            StoreError::Io(error) => match error.kind() {
                std::io::ErrorKind::NotFound => Self::NotFound,
                std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
                _ => Self::Io,
            },
            StoreError::InvalidRef(_) => Self::InvalidRef,
        }
    }
}

/// 画面とサービスのスレッドをつなぐ口
pub(crate) struct ServiceChannel {
    pub(crate) requests: Sender<ServiceRequest>,
    pub(crate) replies: Receiver<CommandReply>,
}

/// サービスのスレッドを始める。保存先を排他に開けたら、そのスレッドと、操作を渡す口を返す
///
/// サービスはランナーのプロセスを持つため、スレッドの中で作り、ずっと動かす。画面の操作は 1 回分の仕事の間に
/// 受け取り、受け取ったらすぐに次の回の仕事をする（操作した結果が早く一覧に出るように）。口を閉じても（ウィンドウを
/// 閉じても）、仕事は続ける。出来事と操作は保存先の `manager.log`（英語。起動のたびに書き直す）に書き、1 回分の
/// 仕事の出来事は `event_sender` にも送る（受け取る側がなくなっても、仕事は続ける）。
pub(crate) fn start_service(
    store: JobStore,
    runner: PathBuf,
    event_sender: Sender<ManagerEvent>,
) -> Result<(JoinHandle<()>, ServiceChannel), StoreError> {
    let (started_tx, started_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();
    let (reply_tx, reply_rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        let mut attempts = 0;
        let (mut service, events) = loop {
            match JobManagerService::start(
                store.clone(),
                ProcessLauncher::new(runner.clone()),
                now_ms(),
            ) {
                Ok(started) => break started,
                Err(StoreError::AlreadyLocked) if attempts < LOCK_ATTEMPTS => {
                    attempts += 1;
                    std::thread::sleep(LOCK_RETRY_INTERVAL);
                }
                Err(error) => {
                    let _ = started_tx.send(Err(error));
                    return;
                }
            }
        };
        let _ = started_tx.send(Ok(()));
        let mut log = File::create(store.root().join("manager.log")).ok();
        let mut write = |line: String| {
            if let Some(file) = log.as_mut() {
                let _ = writeln!(file, "{} {line}", now_ms());
            }
        };
        write(format!("started store={}", store.root().display()));
        events.iter().for_each(|event| write(describe(event)));
        let mut commands = Some(command_rx);
        loop {
            for event in service.tick(now_ms()) {
                write(describe(&event));
                let _ = event_sender.send(event);
            }
            match commands
                .as_ref()
                .map(|receiver| receiver.recv_timeout(POLL_INTERVAL))
            {
                Some(Ok(ServiceRequest::Quit { done })) => {
                    let stopped = service.stop_running(now_ms());
                    write(format!("quit {stopped:?}"));
                    let _ = done.send(());
                    return;
                }
                Some(Ok(ServiceRequest::Command(command))) => {
                    let result = run_command(&mut service, command, now_ms());
                    // 英語の詳しい理由はログに書き、画面には種類を返す
                    write(format!("command {command:?} -> {result:?}"));
                    let _ = reply_tx.send(result.map_err(|error| StoreFailure::of(&error)));
                }
                Some(Err(RecvTimeoutError::Timeout)) => {}
                Some(Err(RecvTimeoutError::Disconnected)) => commands = None,
                None => std::thread::sleep(POLL_INTERVAL),
            }
        }
    });
    let channel = ServiceChannel {
        requests: command_tx,
        replies: reply_rx,
    };
    match started_rx.recv() {
        Ok(Ok(())) => Ok((handle, channel)),
        Ok(Err(error)) => Err(error),
        Err(_) => Err(StoreError::Io(std::io::Error::other(
            "the job manager service stopped while starting",
        ))),
    }
}

/// 今の時刻（UNIX 時刻のミリ秒）
pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

fn run_command(
    service: &mut JobManagerService<ProcessLauncher>,
    command: JobCommand,
    now_ms: u64,
) -> Result<(), StoreError> {
    // 操作の間に状態が変わって操作できなくなったジョブは、何もしない（一覧を読み直せば分かる）
    match command {
        JobCommand::Cancel(job_id) => service.cancel(job_id, now_ms).map(drop),
        JobCommand::Retry(job_id) => service.retry(job_id, now_ms).map(drop),
        JobCommand::Delete(job_id) => service.delete(job_id).map(drop),
        JobCommand::ChangePriority { job_id, delta } => {
            service.change_priority(job_id, delta, now_ms).map(drop)
        }
    }
}

fn describe(event: &ManagerEvent) -> String {
    format!("{event:?}")
}
