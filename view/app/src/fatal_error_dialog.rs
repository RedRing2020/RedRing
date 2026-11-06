//! 続けられない異常（パニックで終わる・描画の装置が失われた）を、OS の知らせの画面で伝える。
//!
//! リリースのビルドはコンソールを開かないため、ログだけでは利用者に異常が伝わらない。egui の描画もできないことが
//! あるため、OS の知らせの画面を使う。画面は、メインのスレッドで、イベントの処理の外（イベントループが終わった後
//! か、イベントループの終了を決めたとき）に出す。

use crate::help_messages::resolve_help_text;
use std::any::Any;
use std::sync::atomic::{AtomicU8, Ordering};
use viewmodel::message_catalog::UiLocale;

/// 知らせの画面の文言の言語（異常のときはアプリの状態を参照できないため、言語を替えるたびに覚えておく。
/// 初めは日本語）
static FATAL_ERROR_LOCALE: AtomicU8 = AtomicU8::new(LOCALE_JA);

const LOCALE_JA: u8 = 0;
const LOCALE_EN: u8 = 1;

/// 知らせの画面の文言の言語を、画面の文言の言語 `locale` に合わせる
pub(crate) fn set_fatal_error_locale(locale: UiLocale) {
    let code = match locale {
        UiLocale::Ja => LOCALE_JA,
        UiLocale::En => LOCALE_EN,
    };
    FATAL_ERROR_LOCALE.store(code, Ordering::Relaxed);
}

fn fatal_error_locale() -> UiLocale {
    match FATAL_ERROR_LOCALE.load(Ordering::Relaxed) {
        LOCALE_EN => UiLocale::En,
        _ => UiLocale::Ja,
    }
}

fn show_error_dialog(title_key: &str, body_key: &str, detail: String) {
    let locale = fatal_error_locale();
    let log_file = match crate::logging::log_file_path() {
        Some(path) => resolve_help_text(locale, "fatal.log_file", &[("path", path.to_string())]),
        None => resolve_help_text(locale, "fatal.no_log_file", &[]),
    };
    let body = resolve_help_text(
        locale,
        body_key,
        &[("detail", detail), ("log_file", log_file)],
    );
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(resolve_help_text(locale, title_key, &[]))
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
