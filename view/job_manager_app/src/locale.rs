//! 画面の言語（ジョブマネージャー自身の設定で選ぶ。まだ選んでいなければ OS の表示言語に合わせる）
//!
//! 選んだ言語は、利用者ごとのジョブマネージャーの設定のファイル（Windows では `%APPDATA%\RedRing\job_manager.json`）に
//! 保存する。設定のファイルのほかの項目は、読んだまま書き戻す（読めない設定のファイルは上書きしない）。今の言語は
//! プロセスの中で 1 つだけ持つ（選び直すと、すぐに今の言語が替わる）。

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

use serde_json::{Map, Value};

/// 設定のファイルの大きさの上限
const MAX_SETTINGS_FILE_BYTES: u64 = 64 * 1024;

/// 設定のファイルの言語の項目
const LANGUAGE_KEY: &str = "language";

/// 画面の言語
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Language {
    Ja,
    En,
}

impl Language {
    /// 選べる言語（メニューに並べる順）
    pub(crate) const ALL: [Self; 2] = [Self::Ja, Self::En];

    /// その言語での言語の名前（どの言語の表示でも、その言語の名前で出す）
    pub(crate) fn native_name(self) -> &'static str {
        match self {
            Self::Ja => "日本語",
            Self::En => "English",
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Ja => "ja",
            Self::En => "en",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
    }

    fn index(self) -> u8 {
        match self {
            Self::Ja => 0,
            Self::En => 1,
        }
    }

    fn from_index(index: u8) -> Self {
        match index {
            1 => Self::En,
            _ => Self::Ja,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

/// 今の画面の言語
pub(crate) fn current() -> Language {
    Language::from_index(CURRENT.load(Ordering::Relaxed))
}

/// 設定のファイルで選んだ言語を今の言語にする（選んでいない・読めないときは OS の表示言語）
pub(crate) fn initialize() {
    let selected = settings_file_path()
        .and_then(|path| read_settings(&path).ok().flatten())
        .and_then(|settings| {
            settings
                .get(LANGUAGE_KEY)
                .and_then(Value::as_str)
                .and_then(Language::from_code)
        });
    set_current(selected.unwrap_or_else(os_language));
}

/// 言語 `language` を今の言語にし、設定のファイルに保存する
///
/// # Errors
///
/// 設定のファイルの場所が分からない・読めない（ほかの項目を失わないよう、上書きしない）・書けない場合（今の言語は
/// 替える）。
pub(crate) fn select(language: Language) -> std::io::Result<()> {
    set_current(language);
    let path = settings_file_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "the per-user settings folder is unknown",
        )
    })?;
    let mut settings = read_settings(&path)?.unwrap_or_default();
    settings.insert(
        LANGUAGE_KEY.to_string(),
        Value::String(language.code().to_string()),
    );
    write_settings(&path, &settings)
}

fn set_current(language: Language) {
    CURRENT.store(language.index(), Ordering::Relaxed);
}

/// 設定のファイルを読む（ないときは `None`）
///
/// # Errors
///
/// 読めない・大きさの上限を超える・JSON のオブジェクトでない場合。
fn read_settings(path: &Path) -> std::io::Result<Option<Map<String, Value>>> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let invalid =
        |reason: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, reason.to_string());
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SETTINGS_FILE_BYTES {
        return Err(invalid("the settings file is too large"));
    }
    match serde_json::from_slice(&bytes).map_err(|error| invalid(&error.to_string()))? {
        Value::Object(settings) => Ok(Some(settings)),
        _ => Err(invalid("the settings file is not a JSON object")),
    }
}

/// 設定のファイルを書く（一時の名前で書き終えてから名前を変える）
fn write_settings(path: &Path, settings: &Map<String, Value>) -> std::io::Result<()> {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder)?;
    }
    let bytes = serde_json::to_vec_pretty(settings).map_err(std::io::Error::other)?;
    // ほかのプロセスと一時のファイルが重ならないよう、プロセスの番号を名前に入れる
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written =
        std::fs::write(&temporary, bytes).and_then(|()| std::fs::rename(&temporary, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    written
}

/// ジョブマネージャーの設定のファイル（Windows では `%APPDATA%\RedRing\job_manager.json`）
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
    Some(base.join("RedRing").join("job_manager.json"))
}

/// OS の表示言語（日本語のときは日本語、それ以外は英語）
#[cfg(windows)]
fn os_language() -> Language {
    use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
    /// 言語の ID の主な言語の部分の、日本語
    const LANG_JAPANESE: u16 = 0x11;
    // SAFETY: 引数のない、状態を変えない関数
    let language_id = unsafe { GetUserDefaultUILanguage() };
    if language_id & 0x3ff == LANG_JAPANESE {
        Language::Ja
    } else {
        Language::En
    }
}

/// OS の表示言語（環境の変数 `LC_ALL`・`LC_MESSAGES`・`LANG` の最初に値のあるものが日本語のときは日本語、それ以外は
/// 英語）
#[cfg(not(windows))]
fn os_language() -> Language {
    let japanese = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .is_some_and(|value| value.starts_with("ja"));
    if japanese { Language::Ja } else { Language::En }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_are_written_and_read_back_keeping_the_other_settings() {
        let folder = std::env::temp_dir().join(format!(
            "redring-job-manager-locale-test-{}",
            std::process::id()
        ));
        let path = folder.join("job_manager.json");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(&path, br#"{"store":"D:\\jobs","language":"ja"}"#).unwrap();

        let mut settings = read_settings(&path).unwrap().unwrap();
        settings.insert(LANGUAGE_KEY.to_string(), Value::String("en".to_string()));
        write_settings(&path, &settings).unwrap();
        let read = read_settings(&path).unwrap().unwrap();
        assert_eq!(read.get("language").and_then(Value::as_str), Some("en"));
        assert_eq!(read.get("store").and_then(Value::as_str), Some("D:\\jobs"));
        // 一時のファイルを残さない
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
        // 形の違うファイル・大きすぎるファイルは、読めない（ないファイルとは分ける）
        std::fs::write(&path, b"[1, 2]").unwrap();
        assert!(read_settings(&path).is_err());
        std::fs::write(&path, vec![b' '; MAX_SETTINGS_FILE_BYTES as usize + 1]).unwrap();
        assert!(read_settings(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(read_settings(&path).unwrap().is_none());
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn language_codes_round_trip() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
            assert_eq!(Language::from_index(language.index()), language);
        }
        assert_eq!(Language::from_code("fr"), None);
    }
}
