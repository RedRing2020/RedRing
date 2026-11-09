//! ジョブマネージャーの実行ファイル（設計: JOB_MANAGER_STORAGE_DESIGN.md 3.2、CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! ```text
//! redring-job-manager [--store <ジョブの保存先>] [--background]
//! ```
//!
//! 保存先を指定しなければ、設定のファイルの保存先（項目 `store`）、それもなければ既定の保存先を使う。開いた保存先は、
//! 問い合わせる口を開く（投入する側は、保存先の場所を知らずに口で問い合わせる）。保存先をウィンドウで替えたときは、
//! 設定に書いてジョブマネージャーを起動し直す（`store_change`）。ランナーは、この実行ファイルと同じフォルダの `redring-batch-runner`
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
mod store_change;
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
/// 終了のときに、サービスが実行中のランナーを止め終えるのを待つ時間
const QUIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

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
    // 設定の保存先を開けたら、問い合わせる口を開く（試しに別の保存先を指定して起動したときは開かない。開けなくても、
    // ジョブの管理は続ける）
    let configured_store = configured
        .as_deref()
        .is_some_and(|configured| same_folder(configured, store.root()));
    #[cfg(windows)]
    {
        let command = autostart_command(given_store.as_deref());
        let notifier = match tray::create(store.clone(), channel.requests.clone(), command) {
            Ok(notifier) => notifier,
            Err(error) => return fail("startup.failed", &[("reason", error.to_string())], 3),
        };
        // ウィンドウを開く要求は、トレイの見えないウィンドウに頼むため、トレイを置いてから口を開く
        if configured_store {
            start_endpoint(&store);
        }
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
        show_window(store, channel, true, tray::attach_window);
        // ウィンドウを出せなかったときも、トレイ（終了）は使えるようにする
        tray::dispatch_messages(false);
    }
    #[cfg(not(windows))]
    {
        let _ = (given_store, event_rx);
        let opener = WindowOpener::new();
        if configured_store {
            start_endpoint(&store, &opener);
        }
        if !background {
            opener.show(&store, &channel);
        }
        // ウィンドウを閉じた後・ウィンドウを出さずに起動したときは、口から開くように頼まれるたびにウィンドウを出す
        // （トレイがないため。ジョブの管理は続ける。口を開かないとき（試しに別の保存先を指定したとき）は頼まれない）
        if configured_store {
            opener.serve_requests(&store, &channel, &service);
        }
    }
    // ウィンドウを閉じても、ジョブの管理は続ける（サービスのスレッドが終わるのはパニックのときだけ。終了は
    // トレイから行い、プロセスを終える）
    match service.join() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(101),
    }
}

/// トレイのない OS で、問い合わせる口からウィンドウを開く頼みを受ける役（出していなければ出し、出していれば前に出す）
#[cfg(not(windows))]
struct WindowOpener {
    /// 出しているウィンドウの画面
    context: Arc<std::sync::Mutex<Option<egui::Context>>>,
    requests: std::sync::mpsc::Sender<()>,
    received: std::sync::mpsc::Receiver<()>,
}

#[cfg(not(windows))]
impl WindowOpener {
    fn new() -> Self {
        let (requests, received) = std::sync::mpsc::channel();
        Self {
            context: Arc::default(),
            requests,
            received,
        }
    }

    /// ウィンドウを開く口（ほかのスレッドから呼べる。出しているかを確かめてから頼むまで、画面を持つ）
    fn open_window(&self) -> job_manager::OpenWindow {
        let context = Arc::clone(&self.context);
        let requests = self.requests.clone();
        Arc::new(move || {
            let context = context
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match context.as_ref() {
                Some(context) => {
                    context.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    context.send_viewport_cmd(egui::ViewportCommand::Focus);
                    context.request_repaint();
                    true
                }
                // ウィンドウを出すのは、このプロセスの最初のスレッドに頼む
                None => requests.send(()).is_ok(),
            }
        })
    }

    /// ウィンドウを出し、閉じるまで待つ
    fn show(&self, store: &JobStore, channel: &ServiceChannel) {
        let context = Arc::clone(&self.context);
        show_window(store.clone(), channel.clone(), false, move |created| {
            *context
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(created);
        });
        // ウィンドウを作る前に溜まった頼みは、このウィンドウで済んだものとする。画面を外すのと同じ間に捨てる（外した
        // 後の頼みは、口の側が溜めるため、次のウィンドウで受ける）
        let mut context = self
            .context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *context = None;
        while self.received.try_recv().is_ok() {}
    }

    /// 開くように頼まれるたびに、ウィンドウを出す（サービスのスレッドが終わったら戻る）
    fn serve_requests(
        &self,
        store: &JobStore,
        channel: &ServiceChannel,
        service: &std::thread::JoinHandle<()>,
    ) {
        while !service.is_finished() {
            match self
                .received
                .recv_timeout(std::time::Duration::from_secs(1))
            {
                Ok(()) => self.show(store, channel),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    }
}

/// この利用者の、ジョブマネージャーに問い合わせる口を開き、保存先 `store` の要求を受ける（開けなくても、ジョブの
/// 管理は続ける）
fn start_endpoint(store: &JobStore, #[cfg(not(windows))] opener: &WindowOpener) {
    let endpoint = match job_ipc::Endpoint::for_current_user() {
        Ok(endpoint) => endpoint,
        Err(error) => {
            eprintln!("cannot find the job manager endpoint: {error}");
            return;
        }
    };
    #[cfg(windows)]
    let open_window: job_manager::OpenWindow = {
        let root = store.root().to_path_buf();
        Arc::new(move || tray::open_running(&root))
    };
    #[cfg(not(windows))]
    let open_window = opener.open_window();
    job_manager::start_endpoint_server(
        endpoint,
        store.clone(),
        open_window,
        env!("CARGO_PKG_VERSION"),
    );
}

/// ジョブの一覧のウィンドウを出す（`hide_on_close` のときは、閉じると隠す）。ウィンドウを作ったら、その画面を
/// `attach` に渡す
fn show_window(
    store: JobStore,
    channel: ServiceChannel,
    hide_on_close: bool,
    attach: impl FnOnce(egui::Context) + 'static,
) {
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
        // 先に用意するフレームを 1 つにすると、大きさを変える間に描き直しが追いつかず、広げた所が白く出る
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration::default()
            .with_surface_config(eframe::egui_wgpu::SurfaceConfig::HIGH_THROUGHPUT),
        ..Default::default()
    };
    let shown = eframe::run_native(
        "redring-job-manager",
        options,
        Box::new(move |creation| {
            fonts::install_cjk_font(&creation.egui_ctx);
            attach(creation.egui_ctx.clone());
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

/// `requests` のサービスを終え（実行中のランナーを止める。`QUIT_TIMEOUT` まで待つ）、トレイのアイコンを消して、
/// プロセスを終える
pub(crate) fn exit_process(requests: &std::sync::mpsc::Sender<service::ServiceRequest>) -> ! {
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    if requests
        .send(service::ServiceRequest::Quit { done: done_tx })
        .is_ok()
    {
        let _ = done_rx.recv_timeout(QUIT_TIMEOUT);
    }
    #[cfg(windows)]
    tray::remove();
    std::process::exit(0);
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
