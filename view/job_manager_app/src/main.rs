//! ジョブマネージャーの実行ファイル（設計: JOB_MANAGER_STORAGE_DESIGN.md 3.2、CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! ```text
//! redring-job-manager [--store <ジョブの保存先>] [--background]
//! ```
//!
//! 保存先を指定しなければ既定の保存先を使う。ランナーは、この実行ファイルと同じフォルダの `redring-batch-runner`
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
mod rows;
mod service;
#[cfg(windows)]
mod tray;
mod window;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use eframe::egui;
use i18n_foundation::UiLocale;
use job_runtime::store::{JobStore, StoreError};

use crate::service::ServiceChannel;

const USAGE: &str = "usage: redring-job-manager [--store <dir>] [--background]";

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
    let locale = locale::app_locale();
    let background = args.background;
    // 起動の誤り（文言のキー・差し込む値・終了の番号）を出して終わる
    let fail = |key: &str, values: &[(&str, String)], code: u8| {
        let message = messages::text(locale, key, values);
        eprintln!("{message}");
        if !background {
            show_error(locale, &message);
        }
        ExitCode::from(code)
    };
    let given_store = args.store.clone();
    let Some(root) = args.store.or_else(JobStore::default_root) else {
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
    let (service, channel) = match service::start_service(store.clone(), runner) {
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
    #[cfg(windows)]
    {
        let text = |key: &str| messages::text(locale, key, &[]);
        let texts = tray::TrayTexts {
            title: text("window.title"),
            open: text("tray.open"),
            autostart: text("tray.autostart"),
            quit: text("tray.quit"),
            quit_confirm: text("tray.quit_confirm"),
            autostart_failed: text("tray.autostart_failed"),
        };
        let command = autostart_command(given_store.as_deref());
        if let Err(error) = tray::create(store.clone(), channel.requests.clone(), texts, command) {
            return fail("startup.failed", &[("reason", error.to_string())], 3);
        }
        if background {
            tray::dispatch_messages(true);
        }
        show_window(store, channel, locale, true);
        // ウィンドウを出せなかったときも、トレイ（終了）は使えるようにする
        tray::dispatch_messages(false);
    }
    #[cfg(not(windows))]
    {
        let _ = given_store;
        if background {
            // 画面がないため、操作を渡す口を閉じる
            drop(channel);
        } else {
            show_window(store, channel, locale, false);
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
fn show_window(store: JobStore, channel: ServiceChannel, locale: UiLocale, hide_on_close: bool) {
    let rgba = icon::ring_rgba();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(messages::text(locale, "window.title", &[]))
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
                locale,
                hide_on_close,
            )))
        }),
    );
    if let Err(error) = shown {
        eprintln!("cannot show the job manager window: {error}");
    }
}

/// Windows の起動のときに起動するコマンド（ウィンドウを出さない。保存先を指定して起動したときは、同じ保存先）
///
/// この実行ファイルの場所が分からなければ `None`。
#[cfg(windows)]
fn autostart_command(store: Option<&std::path::Path>) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let mut command = format!("\"{}\" --background", exe.display());
    if let Some(store) = store {
        command.push_str(&format!(" --store \"{}\"", store.display()));
    }
    Some(command)
}

/// 起動の誤りを、誤りの画面に出す
fn show_error(locale: UiLocale, message: &str) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(messages::text(locale, "window.title", &[]))
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
