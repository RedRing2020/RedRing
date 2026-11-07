//! ジョブマネージャーのサービスを、画面とは別のスレッドで一定の間隔で動かす

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
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

/// サービスのスレッドを始める。保存先を排他に開けたら、そのスレッドを返す
///
/// サービスはランナーのプロセスを持つため、スレッドの中で作り、ずっと動かす。出来事は保存先の `manager.log`
/// （英語。起動のたびに書き直す）に書く。
pub(crate) fn start_service(
    store: JobStore,
    runner: PathBuf,
) -> Result<JoinHandle<()>, StoreError> {
    let (started_tx, started_rx) = mpsc::channel();
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
        loop {
            service
                .tick(now_ms())
                .iter()
                .for_each(|event| write(describe(event)));
            std::thread::sleep(POLL_INTERVAL);
        }
    });
    match started_rx.recv() {
        Ok(Ok(())) => Ok(handle),
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

fn describe(event: &ManagerEvent) -> String {
    format!("{event:?}")
}
