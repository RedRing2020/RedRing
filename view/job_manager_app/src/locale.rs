//! 画面の言語（ジョブマネージャー自身の設定で選ぶ。まだ選んでいなければ OS の表示言語に合わせる）
//!
//! 選んだ言語は、ジョブマネージャーの設定のファイルの項目 `language` に保存する。今の言語はプロセスの中で 1 つだけ
//! 持つ（選び直すと、すぐに今の言語が替わる）。

use std::sync::atomic::{AtomicU8, Ordering};

use serde_json::Value;

use crate::settings;

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
    let selected = settings::get(LANGUAGE_KEY)
        .as_ref()
        .and_then(Value::as_str)
        .and_then(Language::from_code);
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
    settings::set(LANGUAGE_KEY, Value::String(language.code().to_string()))
}

fn set_current(language: Language) {
    CURRENT.store(language.index(), Ordering::Relaxed);
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
    fn language_codes_round_trip() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
            assert_eq!(Language::from_index(language.index()), language);
        }
        assert_eq!(Language::from_code("fr"), None);
    }
}
