//! 子プロセスの組を持たない OS の子プロセスの扱い（コンソールのウィンドウがなく、子プロセスを組に入れない）

use std::io;
use std::process::{Child, Command};

pub(super) fn hide_console_window(_command: &mut Command) {}

#[derive(Debug)]
pub(super) struct Group;

impl Group {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self)
    }

    pub(super) fn add(&self, _child: &Child) -> io::Result<()> {
        Ok(())
    }
}
