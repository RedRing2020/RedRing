//! 利用者の表示の言語

#[cfg(not(windows))]
mod other;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
use other as platform;
#[cfg(windows)]
use windows as platform;

#[cfg(test)]
mod tests;

/// 利用者の表示の言語の、主な言語のコード（`ja`・`en` など。小文字）
///
/// Windows では表示の言語（`GetUserDefaultUILanguage`）、ほかの OS では環境の変数 `LC_ALL`・`LC_MESSAGES`・`LANG`
/// の最初に値のあるものから決める。分からない（`C`・`POSIX` を含む）ときは `None`。
pub fn user_language() -> Option<String> {
    platform::user_language()
}

/// 言語の名前 `name`（`ja-JP`・`ja_JP.UTF-8`・`en` など）の、主な言語のコード（小文字。`C`・`POSIX`・空は `None`）
fn primary_language(name: &str) -> Option<String> {
    let primary = name
        .split(['-', '_', '.', '@'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let known = !primary.is_empty()
        && primary != "c"
        && primary != "posix"
        && primary.bytes().all(|byte| byte.is_ascii_alphabetic());
    known.then_some(primary)
}
