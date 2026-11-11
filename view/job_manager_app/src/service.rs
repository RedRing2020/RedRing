//! ジョブマネージャーのサービスを、画面とは別のスレッドで一定の間隔で動かす

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use job_manager::{JobManagerService, ManagerEvent, ProcessLauncher};
use job_runtime::store::{JobStore, StoreError};

/// 受付のフォルダとランナーを確かめる間隔
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// 画面からの、ジョブの操作
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JobCommand {
    Cancel(u64),
    Retry(u64),
    /// 選んだジョブを、その成果物を使う後のジョブとあわせて消す（計算待ち・計算中のものは残す）
    Delete(Vec<u64>),
    /// 取り込めなかった投入（投入の ID）を消す
    RemoveRejected(Vec<String>),
    ChangePriority {
        job_id: u64,
        delta: i32,
    },
}

/// サービスへの頼み
pub(crate) enum ServiceRequest {
    /// ジョブの操作（結果は `ServiceChannel::replies` に返す）
    Command(JobCommand),
    /// CAD/CAM のアプリが口で頼んだ、ジョブの取り消し（画面の操作ではないため、結果は返さずにログに書く）
    CancelFromApp(Vec<u64>),
    /// 実行中のランナーを止めて、サービスを終える（終えたら `done` に送る）
    Quit { done: Sender<()> },
}

/// 操作の結果（消さずに残した、選んだジョブ。保存先を読み書きできなかったときは理由）
pub(crate) type CommandReply = Result<Vec<u64>, StoreFailure>;

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

/// 画面とサービスのスレッドをつなぐ口（ウィンドウを閉じて開き直しても、同じ口を使う）
#[derive(Clone)]
pub(crate) struct ServiceChannel {
    pub(crate) requests: Sender<ServiceRequest>,
    replies: Arc<Mutex<Receiver<CommandReply>>>,
    /// 結果を待っている操作の数（閉じたウィンドウから頼んだ操作も数える）
    outstanding: Arc<AtomicUsize>,
}

impl ServiceChannel {
    /// ジョブの操作 `command` を頼む（結果は `try_reply` で受け取る）
    ///
    /// # Errors
    ///
    /// サービスが止まっている場合。
    pub(crate) fn send_command(&self, command: JobCommand) -> Result<(), StoreFailure> {
        self.requests
            .send(ServiceRequest::Command(command))
            .map_err(|_| StoreFailure::ServiceStopped)?;
        self.outstanding.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// 返った操作の結果（なければ `None`）
    pub(crate) fn try_reply(&self) -> Option<CommandReply> {
        let reply = self
            .replies
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .try_recv()
            .ok()?;
        // 頼むのも受け取るのも画面のスレッドだけのため、確かめてから減らせばよい
        if self.outstanding.load(Ordering::SeqCst) > 0 {
            self.outstanding.fetch_sub(1, Ordering::SeqCst);
        }
        Some(reply)
    }

    /// 結果を待っている操作の数
    pub(crate) fn outstanding(&self) -> usize {
        self.outstanding.load(Ordering::SeqCst)
    }
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
        let (mut service, events) =
            match JobManagerService::start(store.clone(), ProcessLauncher::new(runner), now_ms()) {
                Ok(started) => started,
                Err(error) => {
                    let _ = started_tx.send(Err(error));
                    return;
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
                Some(Ok(ServiceRequest::CancelFromApp(job_ids))) => {
                    for job_id in job_ids {
                        let result = service.cancel(job_id, now_ms());
                        write(format!("cancel from the app {job_id} -> {result:?}"));
                    }
                }
                Some(Ok(ServiceRequest::Command(command))) => {
                    let result = run_command(&mut service, &command, now_ms());
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
        replies: Arc::new(Mutex::new(reply_rx)),
        outstanding: Arc::new(AtomicUsize::new(0)),
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
    command: &JobCommand,
    now_ms: u64,
) -> Result<Vec<u64>, StoreError> {
    // 操作の間に状態が変わって操作できなくなったジョブは、何もしない（一覧を読み直せば分かる）
    match command {
        JobCommand::Cancel(job_id) => service.cancel(*job_id, now_ms).map(|_| Vec::new()),
        JobCommand::Retry(job_id) => service.retry(*job_id, now_ms).map(|_| Vec::new()),
        JobCommand::Delete(job_ids) => service
            .delete(job_ids)
            .map(|plan| plan.kept.into_iter().collect()),
        JobCommand::RemoveRejected(submission_ids) => {
            service.remove_rejected(submission_ids).map(|()| Vec::new())
        }
        JobCommand::ChangePriority { job_id, delta } => service
            .change_priority(*job_id, *delta, now_ms)
            .map(|_| Vec::new()),
    }
}

fn describe(event: &ManagerEvent) -> String {
    format!("{event:?}")
}
