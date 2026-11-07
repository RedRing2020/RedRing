//! 画面の言語（CAD/CAM のアプリの設定のファイルで選んだ言語に合わせる）

use std::io::Read;
use std::path::PathBuf;

use i18n_foundation::UiLocale;
use serde::Deserialize;

/// 設定のファイルの大きさの上限
const MAX_SETTINGS_FILE_BYTES: u64 = 64 * 1024;

#[derive(Deserialize)]
struct Settings {
    #[serde(default)]
    locale: Option<String>,
}

/// CAD/CAM のアプリで選んだ言語（設定のファイルがない・読めないときは日本語）
pub(crate) fn app_locale() -> UiLocale {
    settings_file_path()
        .and_then(|path| {
            let mut bytes = Vec::new();
            std::fs::File::open(path)
                .ok()?
                .take(MAX_SETTINGS_FILE_BYTES)
                .read_to_end(&mut bytes)
                .ok()?;
            serde_json::from_slice::<Settings>(&bytes).ok()
        })
        .and_then(|settings| settings.locale)
        .map_or(UiLocale::Ja, |locale| match locale.as_str() {
            "en" => UiLocale::En,
            _ => UiLocale::Ja,
        })
}

/// CAD/CAM のアプリの設定のファイル（Windows では `%APPDATA%\RedRing\settings.json`）
fn settings_file_path() -> Option<PathBuf> {
    let absolute = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    let base = if cfg!(windows) {
        absolute("APPDATA")
    } else {
        absolute("XDG_CONFIG_HOME").or_else(|| absolute("HOME").map(|home| home.join(".config")))
    }?;
    Some(base.join("RedRing").join("settings.json"))
}
