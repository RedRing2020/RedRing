//! 続けられない異常（パニックで終わる・描画の装置が失われた）を、OS の知らせの画面で伝える。
//!
//! リリースのビルドはコンソールを開かないため、ログだけでは利用者に異常が伝わらない。egui の描画もできないことが
//! あるため、OS の知らせの画面を使う。画面は、メインのスレッドで、イベントの処理の外（イベントループが終わった後
//! か、イベントループの終了を決めたとき）に出す。

use crate::help_messages::resolve_help_text;
use std::any::Any;
use viewmodel::message_catalog::UiLocale;

/// 知らせの画面の文言の言語（異常のときは、アプリの状態を参照できないため、既定の言語を使う）
const FATAL_ERROR_LOCALE: UiLocale = UiLocale::Ja;

fn show_error_dialog(title_key: &str, body_key: &str, detail: String) {
    let log_file = match crate::logging::log_file_path() {
        Some(path) => resolve_help_text(
            FATAL_ERROR_LOCALE,
            "fatal.log_file",
            &[("path", path.to_string())],
        ),
        None => resolve_help_text(FATAL_ERROR_LOCALE, "fatal.no_log_file", &[]),
    };
    let body = resolve_help_text(
        FATAL_ERROR_LOCALE,
        body_key,
        &[("detail", detail), ("log_file", log_file)],
    );
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(resolve_help_text(FATAL_ERROR_LOCALE, title_key, &[]))
        .set_description(body)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

/// パニックを、ログにも書く（どのスレッドでも、処理を続けるパニックでも書く。知らせの画面は出さない）
pub fn install_panic_logging() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!("panic: {info}");
        default_hook(info);
    }));
}

/// イベントループがパニックで終わったことを伝える（メインのスレッドで、イベントループの外から呼ぶ）
pub fn show_panic_dialog(payload: &(dyn Any + Send)) {
    let detail = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_default();
    show_error_dialog("fatal.panic.title", "fatal.panic.body", detail);
}

/// 描画の装置が失われ、続けられないことを伝える
pub(crate) fn show_device_lost_dialog() {
    show_error_dialog(
        "fatal.device_lost.title",
        "fatal.device_lost.body",
        String::new(),
    );
}
