//! 利用者がサインインしたときに起動するプログラム
//!
//! Windows では、利用者ごとのレジストリの `Run` キーに、名前とコマンドを書く。ほかの OS には対応しない（読むと
//! `None`、書く・消すと `Unsupported` の誤り）。

use std::io;

#[cfg(not(windows))]
mod other;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
use other as platform;
#[cfg(windows)]
use windows as platform;

/// 名前 `name` で、サインインしたときに起動するように書いたコマンド（書いていない・読めなければ `None`）
pub fn registered_command(name: &str) -> Option<String> {
    platform::registered_command(name)
}

/// 名前 `name` で、サインインしたときにコマンド `command` を起動するように書く（書いてあれば書き替える）
///
/// # Errors
///
/// 書けない・対応しない OS の場合。
pub fn register(name: &str, command: &str) -> io::Result<()> {
    platform::register(name, command)
}

/// 名前 `name` の、サインインしたときに起動するコマンドを消す
///
/// # Errors
///
/// 消せない（書いていない場合を含む）・対応しない OS の場合。
pub fn unregister(name: &str) -> io::Result<()> {
    platform::unregister(name)
}
