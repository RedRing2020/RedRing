//! 利用者ごとのフォルダの場所（動かす環境ごとの決まり）
//!
//! 環境の変数が空・相対のパスのときは使わない（起動したフォルダで場所が変わるため）。

use std::ffi::OsString;
use std::path::PathBuf;

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

/// 利用者ごとのアプリのデータのフォルダ（Windows では `%LOCALAPPDATA%`、ほかの OS では `$XDG_DATA_HOME`、ない場合は
/// `~/.local/share`）
///
/// 分からないときは `None`。
pub fn user_data_folder() -> Option<PathBuf> {
    platform::data_folder(absolute_variable)
}

/// 利用者ごとの設定のフォルダ（Windows では `%APPDATA%`、ほかの OS では `$XDG_CONFIG_HOME`、ない場合は `~/.config`）
///
/// 分からないときは `None`。
pub fn user_config_folder() -> Option<PathBuf> {
    platform::config_folder(absolute_variable)
}

/// 環境の変数 `name` の値（絶対のパスのときだけ）
fn absolute_variable(name: &str) -> Option<PathBuf> {
    absolute(std::env::var_os(name))
}

/// 環境の変数の値 `value`（ないときは `None`）を、絶対のパスのときだけ返す
fn absolute(value: Option<OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|path| path.is_absolute())
}
