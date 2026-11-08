//! ジョブマネージャーの実行ファイル（設計: JOB_MANAGER_STORAGE_DESIGN.md 3.2、CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! ```text
//! redring-job-manager [--store <ジョブの保存先>] [--background]
//! ```
//!
//! 保存先を指定しなければ、設定のファイルの保存先（項目 `store`）、それもなければ既定の保存先を使う。開いた保存先は、
//! 所在のファイルに書く（投入する側が、保存先の場所を設定で持たずに知るため）。ランナーは、この実行ファイルと同じフォルダの `redring-batch-runner`
//! だけを起動する（作業のフォルダや PATH から探さない）。ジョブの管理（投入の取り込み・ランナーの起動）は別の
//! スレッドで一定の間隔で行い、ウィンドウにジョブの一覧を出し、ジョブの操作（取り消し・やり直し・削除・優先順位）を
//! サービスに渡す。ウィンドウを閉じても、ジョブの管理は続ける。
//! `--background` のときはウィンドウを出さない（CAD/CAM のアプリから起動したとき・Windows の起動のとき）。
//! ウィンドウを出すときの起動の誤りは、誤りの画面に出す（リリースのビルドはコンソールを持たないため）。
//!
//! Windows では、タスクトレイにアイコンを置く。ウィンドウを閉じるとトレイに残り、トレイから開き直し・終了する。
//! `--background` のときは、トレイから開くように頼まれてからウィンドウを作る（作ると最初の描画で一度出るため）。
//! 同じ保存先のジョブマネージャーが動いていれば、2 つ目の起動はそのウィンドウを開くように頼んで終わる。

// リリースのビルドでは、コンソールのウィンドウを開かない
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fonts;
mod icon;
mod locale;
mod messages;
mod notification;
mod rows;
mod service;
mod settings;
#[cfg(windows)]
mod tray;
mod window;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use eframe::egui;
use job_runtime::store::{JobStore, StoreError};

use crate::service::ServiceChannel;

const USAGE: &str = "usage: redring-job-manager [--store <dir>] [--background]";

/// 設定のファイルの保存先の項目
const STORE_KEY: &str = "store";

/// 引数
struct Args {
    store: Option<PathBuf>,
    background: bool,
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args_os().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    locale::initialize();
    let background = args.background;
    // 起動の誤り（文言のキー・差し込む値・終了の番号）を出して終わる
    let fail = |key: &str, values: &[(&str, String)], code: u8| {
        let message = messages::text(locale::current(), key, values);
        eprintln!("{message}");
        if !background {
            show_error(&message);
        }
        ExitCode::from(code)
    };
    let given_store = args.store.clone();
    let configured = match configured_store() {
        Ok(configured) => configured,
        Err(()) => return fail("startup.invalid_store_setting", &[], 2),
    };
    let Some(root) = args.store.or_else(|| configured.clone()) else {
        return fail("startup.no_data_folder", &[], 2);
    };
    // 動いているジョブマネージャーがあれば、そのウィンドウを開く（保存先のロックを待たない）
    #[cfg(windows)]
    if !background && tray::open_running(&root) {
        return ExitCode::SUCCESS;
    }
    let runner = match runner_path() {
        Ok(runner) => runner,
        Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
    };
    let store = match JobStore::open(&root) {
        Ok(store) => store,
        Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
    };
    // 出来事は、受け取り始めるまで溜めておく（ジョブが終わったときの吹き出しに使う）
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let (service, channel) = match service::start_service(store.clone(), runner, event_tx) {
        Ok(started) => started,
        Err(StoreError::AlreadyLocked) => {
            #[cfg(windows)]
            if !background && tray::open_running(&root) {
                return ExitCode::SUCCESS;
            }
            return fail("startup.already_running", &[], 1);
        }
        Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
    };
    // 設定の保存先を開けたら、所在のファイルに書く（試しに別の保存先を指定して起動したときは書かない。書けなくても、
    // ジョブの管理は続ける）
    if configured
        .as_deref()
        .is_some_and(|configured| same_folder(configured, store.root()))
        && let Err(error) = store.record_location()
    {
        eprintln!("cannot record the job store location: {error}");
    }
    #[cfg(windows)]
    {
        let command = autostart_command(given_store.as_deref());
        let notifier = match tray::create(store.clone(), channel.requests.clone(), command) {
            Ok(notifier) => notifier,
            Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
        };
        // 吹き出しは別のスレッドで出す（記録を読む・エクスプローラーを待つ間も、ジョブの管理を止めない）
        let notified_store = store.clone();
        std::thread::spawn(move || {
            for event in event_rx {
                notify_finished(&notifier, &notified_store, &event);
            }
        });
        if background {
            tray::dispatch_messages(true);
        }
        show_window(store, channel, true);
        // ウィンドウを出せなかったときも、トレイ（終了）は使えるようにする
        tray::dispatch_messages(false);
    }
    #[cfg(not(windows))]
    {
        let _ = (given_store, event_rx);
        if background {
            // 画面がないため、操作を渡す口を閉じる
            drop(channel);
        } else {
            show_window(store, channel, false);
        }
    }
    // ウィンドウを閉じても、ジョブの管理は続ける（サービスのスレッドが終わるのはパニックのときだけ。終了は
    // トレイから行い、プロセスを終える）
    match service.join() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(101),
    }
}

/// ジョブの一覧のウィンドウを出す（`hide_on_close` のときは、閉じると隠す）
fn show_window(store: JobStore, channel: ServiceChannel, hide_on_close: bool) {
    let rgba = icon::ring_rgba();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(messages::text(locale::current(), "window.title", &[]))
            .with_inner_size([960.0, 600.0])
            .with_icon(Arc::new(egui::IconData {
                rgba,
                width: icon::ICON_SIZE,
                height: icon::ICON_SIZE,
            })),
        ..Default::default()
    };
    let shown = eframe::run_native(
        "redring-job-manager",
        options,
        Box::new(move |creation| {
            fonts::install_cjk_font(&creation.egui_ctx);
            #[cfg(windows)]
            tray::attach_window(creation.egui_ctx.clone());
            Ok(Box::new(window::JobManagerWindow::new(
                store,
                channel,
                hide_on_close,
            )))
        }),
    );
    if let Err(error) = shown {
        eprintln!("cannot show the job manager window: {error}");
    }
}

/// ジョブが終わったときに、トレイの吹き出しを出す（示すジョブの決め方は `notification::finished_job`）
#[cfg(windows)]
fn notify_finished(
    notifier: &tray::TrayNotifier,
    store: &JobStore,
    event: &job_manager::ManagerEvent,
) {
    use job_runtime::store::StoredJobKind;
    use notification::{Finish, finished_job};
    // 終わったジョブの出来事のときだけ、記録を読む
    if !matches!(event, job_manager::ManagerEvent::Finished { .. }) {
        return;
    }
    let Ok((jobs, _)) = store.load_jobs() else {
        return;
    };
    let Some((finish, job_id, kind)) = finished_job(event, &jobs) else {
        return;
    };
    let locale = locale::current();
    let kind = match kind {
        StoredJobKind::Cam => "kind.cam",
        StoredJobKind::Sim => "kind.sim",
    };
    let key = match finish {
        Finish::Succeeded => "notify.succeeded",
        Finish::Failed => "notify.failed",
    };
    let text = messages::text(
        locale,
        key,
        &[
            ("kind", messages::text(locale, kind, &[])),
            ("job", job_id.to_string()),
        ],
    );
    notifier.notify(
        &messages::text(locale, "window.title", &[]),
        &text,
        finish == Finish::Failed,
    );
}

/// Windows の起動のときに起動するコマンド（ウィンドウを出さない。設定の保存先と違う保存先を指定して起動したときは、
/// 同じ保存先。設定の保存先のときは指定しない（後で設定の保存先を替えても、替えた保存先で起動するように））
///
/// この実行ファイルの場所が分からなければ `None`。
#[cfg(windows)]
fn autostart_command(store: Option<&std::path::Path>) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let mut command = format!("\"{}\" --background", exe.display());
    let configured = configured_store().ok().flatten();
    if let Some(store) = store.filter(|store| {
        !configured
            .as_deref()
            .is_some_and(|configured| same_folder(configured, store))
    }) {
        command.push_str(&format!(" --store \"{}\"", store.display()));
    }
    Some(command)
}

/// 設定の保存先（設定のファイルの項目 `store`。なければ既定の保存先。利用者ごとのフォルダが分からないときは `None`）
///
/// # Errors
///
/// 項目 `store` が、絶対のパスの文字列でない場合（黙って既定の保存先にしない）。
fn configured_store() -> Result<Option<PathBuf>, ()> {
    match settings::get(STORE_KEY) {
        None => Ok(JobStore::default_root()),
        Some(serde_json::Value::String(path)) if PathBuf::from(&path).is_absolute() => {
            Ok(Some(PathBuf::from(path)))
        }
        Some(_) => Err(()),
    }
}

/// 2 つのパスが同じフォルダか（書き方の違い（大文字・小文字・末尾の区切り）は、どちらのフォルダもあれば実体で比べる）
fn same_folder(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// 言語 `language` を選び（設定のファイルに保存する）、トレイの文言を替える
///
/// # Errors
///
/// 選んだ言語を保存できない場合（今の言語は替える）。
pub(crate) fn select_language(language: locale::Language) -> std::io::Result<()> {
    let saved = locale::select(language);
    #[cfg(windows)]
    tray::refresh_language();
    saved
}

/// 起動の誤りを、誤りの画面に出す
fn show_error(message: &str) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(messages::text(locale::current(), "window.title", &[]))
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

fn parse_args(args: impl Iterator<Item = std::ffi::OsString>) -> Result<Args, String> {
    let mut args = args;
    let mut parsed = Args {
        store: None,
        background: false,
    };
    while let Some(name) = args.next() {
        match name.to_str() {
            Some("--background") => parsed.background = true,
            Some("--store") => {
                let value = PathBuf::from(args.next().ok_or("--store needs a value")?);
                // 相対のパスは、起動したフォルダで保存先が変わるため使わない
                if !value.is_absolute() {
                    return Err("--store must be an absolute path".to_string());
                }
                if parsed.store.replace(value).is_some() {
                    return Err("--store is given twice".to_string());
                }
            }
            _ => return Err(format!("unknown argument: {}", name.to_string_lossy())),
        }
    }
    Ok(parsed)
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
