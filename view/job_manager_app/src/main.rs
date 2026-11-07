//! ジョブマネージャーの実行ファイル（設計: JOB_MANAGER_STORAGE_DESIGN.md 3.2、CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! ```text
//! redring-job-manager [--store <ジョブの保存先>] [--background]
//! ```
//!
//! 保存先を指定しなければ既定の保存先を使う。ランナーは、この実行ファイルと同じフォルダの `redring-batch-runner`
//! だけを起動する（作業のフォルダや PATH から探さない）。ジョブの管理（投入の取り込み・ランナーの起動）は別の
//! スレッドで一定の間隔で行い、ウィンドウにジョブの一覧を出す。ウィンドウを閉じても、ジョブの管理は続ける。
//! `--background` のときはウィンドウを出さずに、ジョブの管理だけを行う（CAD/CAM のアプリから起動したとき）。
//! ウィンドウを出すときの起動の誤りは、知らせの画面に出す（リリースのビルドはコンソールを持たないため）。

// リリースのビルドでは、コンソールのウィンドウを開かない
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fonts;
mod locale;
mod messages;
mod rows;
mod service;
mod window;

use std::path::PathBuf;
use std::process::ExitCode;

use i18n_foundation::UiLocale;
use job_runtime::store::{JobStore, StoreError};

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
    // 起動の誤り（文言のキー・差し込む値・終了の番号）を知らせて終わる
    let fail = |key: &str, values: &[(&str, String)], code: u8| {
        let message = messages::text(locale, key, values);
        eprintln!("{message}");
        if !background {
            show_error(locale, &message);
        }
        ExitCode::from(code)
    };
    let Some(root) = args.store.or_else(JobStore::default_root) else {
        return fail("startup.no_data_folder", &[], 2);
    };
    let runner = match runner_path() {
        Ok(runner) => runner,
        Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
    };
    let store = match JobStore::open(&root) {
        Ok(store) => store,
        Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
    };
    let service = match service::start_service(store.clone(), runner) {
        Ok(service) => service,
        Err(StoreError::AlreadyLocked) => return fail("startup.already_running", &[], 1),
        Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
    };
    if !background {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title(messages::text(locale, "window.title", &[]))
                .with_inner_size([960.0, 600.0]),
            ..Default::default()
        };
        let shown = eframe::run_native(
            "redring-job-manager",
            options,
            Box::new(move |creation| {
                fonts::install_cjk_font(&creation.egui_ctx);
                Ok(Box::new(window::JobManagerWindow::new(store, locale)))
            }),
        );
        if let Err(error) = shown {
            eprintln!("cannot show the job manager window: {error}");
        }
    }
    // ウィンドウを閉じても、ジョブの管理は続ける（サービスのスレッドが終わるのはパニックのときだけ）
    match service.join() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(101),
    }
}

/// 起動の誤りを、知らせの画面に出す
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
