//! ジョブマネージャーの実行ファイル（画面なし。設計: JOB_MANAGER_STORAGE_DESIGN.md 3 章）
//!
//! ```text
//! redring-job-manager [--store <ジョブの保存先>]
//! ```
//!
//! 保存先を指定しなければ既定の保存先を使う。ランナーは、この実行ファイルと同じフォルダの `redring-batch-runner`
//! だけを起動する（作業のフォルダや PATH から探さない）。止めるまで、一定の間隔で受付のフォルダを読み、ジョブを
//! 実行する。出来事は保存先の `manager.log`（英語。起動のたびに書き直す）に書く。

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use job_manager::{JobManagerService, ManagerEvent, ProcessLauncher};
use job_runtime::store::{JobStore, StoreError};

/// 受付のフォルダとランナーを確かめる間隔
const POLL_INTERVAL: Duration = Duration::from_millis(500);
const USAGE: &str = "usage: redring-job-manager [--store <dir>]";
/// 保存先のロックを取れないときに、やり直す回数と間隔
const LOCK_ATTEMPTS: u32 = 10;
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> ExitCode {
    let root = match parse_store_arg(std::env::args_os().skip(1)) {
        Ok(Some(root)) => root,
        Ok(None) => match JobStore::default_root() {
            Some(root) => root,
            None => {
                eprintln!("cannot find the per-user data folder; give --store");
                return ExitCode::from(2);
            }
        },
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let runner = match runner_path() {
        Ok(runner) => runner,
        Err(error) => {
            eprintln!("cannot find the runner: {error}");
            return ExitCode::from(3);
        }
    };
    let store = match JobStore::open(&root) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("cannot open the job store {}: {error}", root.display());
            return ExitCode::from(3);
        }
    };
    // CAD/CAM のアプリが、ジョブマネージャーが動いているかを確かめるために、ロックを一瞬だけ持つことがあるため、
    // 少しの間やり直す
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
            Err(StoreError::AlreadyLocked) => {
                eprintln!("another job manager is running for {}", root.display());
                return ExitCode::from(1);
            }
            Err(error) => {
                eprintln!("cannot start the job manager: {error}");
                return ExitCode::from(3);
            }
        }
    };
    let mut log = File::create(root.join("manager.log")).ok();
    let mut write = |line: String| {
        if let Some(file) = log.as_mut() {
            let _ = writeln!(file, "{} {line}", now_ms());
        }
    };
    write(format!("started store={}", root.display()));
    events.iter().for_each(|event| write(describe(event)));
    loop {
        service
            .tick(now_ms())
            .iter()
            .for_each(|event| write(describe(event)));
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// `--store <dir>` を読む（なければ `None`）
fn parse_store_arg(
    args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<Option<PathBuf>, String> {
    let mut args = args;
    let mut store = None;
    while let Some(name) = args.next() {
        if name != "--store" {
            return Err(format!("unknown argument: {}", name.to_string_lossy()));
        }
        let value = PathBuf::from(args.next().ok_or("--store needs a value")?);
        // 相対のパスは、起動したフォルダで保存先が変わるため使わない
        if !value.is_absolute() {
            return Err("--store must be an absolute path".to_string());
        }
        if store.replace(value).is_some() {
            return Err("--store is given twice".to_string());
        }
    }
    Ok(store)
}

/// この実行ファイルと同じフォルダのランナー
fn runner_path() -> std::io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or_else(|| std::io::Error::other("the executable has no folder"))?;
    Ok(dir.join(format!(
        "redring-batch-runner{}",
        std::env::consts::EXE_SUFFIX
    )))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

fn describe(event: &ManagerEvent) -> String {
    format!("{event:?}")
}
