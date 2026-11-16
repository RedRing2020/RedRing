//! サインインしたときに起動するプログラムの、Windows 以外の実装（対応しない）

use std::io;

pub(super) fn registered_command(_name: &str) -> Option<String> {
    None
}

pub(super) fn register(_name: &str, _command: &str) -> io::Result<()> {
    Err(unsupported())
}

pub(super) fn unregister(_name: &str) -> io::Result<()> {
    Err(unsupported())
}

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "starting at sign-in is not supported on this OS",
    )
}
