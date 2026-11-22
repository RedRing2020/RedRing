//! 利用者の表示の言語の、Windows 以外の実装

/// 環境の変数 `LC_ALL`・`LC_MESSAGES`・`LANG` の最初に値のあるものの、主な言語のコード
pub(super) fn user_language() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .and_then(|value| super::primary_language(&value))
}
