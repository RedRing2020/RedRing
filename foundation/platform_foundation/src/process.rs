//! 子プロセスの扱い（コンソールのウィンドウを出さない指定と、このプロセスが終わると一緒に止まる組）

use std::io;
use std::process::{Child, Command};

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

/// 起動する子プロセス `command` の、コンソールのウィンドウを出さない
///
/// Windows だけで効く。ほかの OS では、何もしない（コンソールのウィンドウを作らないため）。
pub fn hide_console_window(command: &mut Command) {
    platform::hide_console_window(command);
}

/// このプロセスが終わると一緒に止まる、子プロセスの組
///
/// このプロセスが終わる（異常終了を含む）か、組を捨てると、OS が組の子プロセスを止める。Windows では、ジョブ
/// オブジェクト（`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`）を使う。ほかの OS では、組に入れない（このプロセスが
/// 終わっても、子プロセスは止まらない）。
#[derive(Debug)]
pub struct ChildGroup(platform::Group);

impl ChildGroup {
    /// 空の組を作る
    pub fn new() -> io::Result<Self> {
        platform::Group::new()
            .map(Self)
            .map_err(|error| context(error, "cannot create the child process group"))
    }

    /// `command` を起動して、組に入れる。起動した子プロセスと、組に入れられなかった理由（入れたときは `None`）を返す
    ///
    /// 組に入れられなくても、子プロセスは止めない（組を使えない環境でも、子プロセスを動かせるように）。組に入れる前に
    /// 子プロセスが終わっていれば、入れられなかった理由は返さない。起動してから組に入れるため、その間に子プロセスが
    /// 起動したプロセスは、組に入らない。
    pub fn spawn(&self, command: &mut Command) -> io::Result<(Child, Option<io::Error>)> {
        let mut child = command.spawn()?;
        let ungrouped = match self.0.add(&child) {
            Ok(()) => None,
            Err(_) if matches!(child.try_wait(), Ok(Some(_))) => None,
            Err(error) => Some(context(error, "cannot add the child process to the group")),
        };
        Ok((child, ungrouped))
    }
}

/// OS の誤り `error` に、何をしていたか `doing` を付ける
fn context(error: io::Error, doing: &str) -> io::Error {
    io::Error::new(error.kind(), format!("{doing}: {error}"))
}
