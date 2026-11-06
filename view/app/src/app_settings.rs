//! アプリの設定（表示の言語）を、利用者ごとのフォルダの JSON のファイルに保存する。
//!
//! 置き場所は、Windows では `%APPDATA%\RedRing\settings.json`、ほかの OS では `$XDG_CONFIG_HOME/RedRing/settings.json`
//! （ない場合は `~/.config/RedRing/settings.json`）。知らない項目は、書くときにそのまま残す（後の版の設定を消さない）。
//! 知らない値は既定の値として扱う。

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use viewmodel::message_catalog::UiLocale;

/// 設定のフォルダの名前
const SETTINGS_DIR_NAME: &str = "RedRing";
/// 設定のファイルの名前
const SETTINGS_FILE_NAME: &str = "settings.json";
/// 読む設定のファイルの大きさの上限（細工した大きなファイルでメモリを使い切らない）
const MAX_SETTINGS_FILE_BYTES: u64 = 64 * 1024;

/// 保存する設定
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppSettings {
    /// 表示の言語（`ja`・`en`）
    #[serde(skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    /// この版の知らない項目（書くときにそのまま残す）
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

impl AppSettings {
    /// 表示の言語（ない・知らない値のときは日本語）
    pub(crate) fn locale(&self) -> UiLocale {
        match self.locale.as_deref() {
            Some("en") => UiLocale::En,
            _ => UiLocale::Ja,
        }
    }

    pub(crate) fn set_locale(&mut self, locale: UiLocale) {
        let code = match locale {
            UiLocale::Ja => "ja",
            UiLocale::En => "en",
        };
        self.locale = Some(code.to_string());
    }
}

/// 設定のファイルの場所（利用者ごとのフォルダが分からないときは `None`）
pub(crate) fn settings_file_path() -> Option<PathBuf> {
    // 空・相対のパスは使わない（起動したフォルダに置かれ、起動の場所で設定が変わるため）
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
    Some(base.join(SETTINGS_DIR_NAME).join(SETTINGS_FILE_NAME))
}

/// 設定を読む（ファイルがないときは既定の設定）
pub(crate) fn load_settings(path: &Path) -> std::io::Result<AppSettings> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(AppSettings::default())
        }
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SETTINGS_FILE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("the settings file is larger than {MAX_SETTINGS_FILE_BYTES} bytes"),
        ));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

/// 設定を書く（書きかけのファイルに書き終えてから名前を替え、途中で止まっても前の設定を壊さない）
pub(crate) fn save_settings(path: &Path, settings: &AppSettings) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    // 書きかけのファイルの名前は、プロセスごとに分ける（2 つ起動していても、互いの書きかけを替えない）
    let temp = path.with_file_name(format!("{SETTINGS_FILE_NAME}.{}.tmp", std::process::id()));
    let result = std::fs::write(&temp, text).and_then(|()| std::fs::rename(&temp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_settings_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "redring_settings_test_{name}_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join(SETTINGS_FILE_NAME)
    }

    #[test]
    fn saved_locale_is_read_back() {
        let path = temp_settings_path("roundtrip");
        let mut settings = AppSettings::default();
        settings.set_locale(UiLocale::En);
        save_settings(&path, &settings).unwrap();
        assert_eq!(load_settings(&path).unwrap().locale(), UiLocale::En);
        settings.set_locale(UiLocale::Ja);
        save_settings(&path, &settings).unwrap();
        assert_eq!(load_settings(&path).unwrap().locale(), UiLocale::Ja);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn missing_file_gives_the_default_settings() {
        let path = temp_settings_path("missing");
        let settings = load_settings(&path).unwrap();
        assert_eq!(settings, AppSettings::default());
        assert_eq!(settings.locale(), UiLocale::Ja);
    }

    #[test]
    fn unknown_fields_are_kept_when_saving() {
        let path = temp_settings_path("unknown_fields");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, r#"{"locale":"ja","future":{"a":1}}"#).unwrap();
        let mut settings = load_settings(&path).unwrap();
        settings.set_locale(UiLocale::En);
        save_settings(&path, &settings).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["locale"], "en");
        assert_eq!(saved["future"]["a"], 1);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn unknown_fields_and_values_fall_back_to_japanese() {
        let settings: AppSettings = serde_json::from_str(r#"{"locale":"fr","future":1}"#).unwrap();
        assert_eq!(settings.locale(), UiLocale::Ja);
    }

    #[test]
    fn broken_or_large_files_are_errors() {
        let path = temp_settings_path("broken");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{ not json").unwrap();
        assert!(load_settings(&path).is_err());
        let large = " ".repeat(MAX_SETTINGS_FILE_BYTES as usize + 1);
        std::fs::write(&path, large).unwrap();
        assert!(load_settings(&path).is_err());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
