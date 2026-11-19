//! 利用者ごとのフォルダの場所（動かす環境ごとの決まり）と、利用者だけが読み書きできるこの PC のフォルダ
//!
//! 利用者ごとのフォルダは、環境の変数が空・相対のパスのときは使わない（起動したフォルダで場所が変わるため）。

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

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

/// フォルダが、利用者だけが読み書きできるこの PC のフォルダでない理由
#[derive(Debug)]
pub enum PrivateFolderError {
    /// この PC のフォルダでない（ネットワークのフォルダ・ドライブ）
    NotOnThisPc,
    /// 権限を付けられないファイルシステム（FAT32・exFAT など）のフォルダ
    NoPermissions,
    /// 持ち主がほかの利用者、またはほかの利用者・グループも読み書きできる
    OpenToOthers,
    /// フォルダを確かめられない（ない・読めない）
    Io(io::Error),
}

impl std::fmt::Display for PrivateFolderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOnThisPc => f.write_str("the folder is not on this PC"),
            Self::NoPermissions => f.write_str("the folder cannot hold permissions"),
            Self::OpenToOthers => f.write_str("the folder is open to other users"),
            Self::Io(error) => write!(f, "cannot check the folder: {error}"),
        }
    }
}

/// フォルダ `path` が、利用者だけが読み書きできるこの PC のフォルダかを確かめる（リンクは、たどった先を確かめる）
///
/// - Windows: ドライブがこの PC のもの（UNC のパス・ネットワークのドライブでない）、ファイルシステムが権限を持てる
///   （FAT32・exFAT でない）、持ち主がこの利用者・`SYSTEM`・`Administrators` で、ほかの利用者・グループに、たどる・
///   属性を読む以外の権利がない（拒む権利は見ない）
/// - ほかの OS: ネットワークのファイルシステム（NFS・SMB など。Linux と macOS で確かめる）でなく、持ち主がこの利用者で、
///   グループ・ほかの利用者の権利がない（0700）
///
/// # Errors
///
/// 満たさない理由。
pub fn check_private_folder(path: &Path) -> Result<(), PrivateFolderError> {
    let real = std::fs::canonicalize(path).map_err(PrivateFolderError::Io)?;
    if !real.is_dir() {
        return Err(PrivateFolderError::Io(io::Error::new(
            io::ErrorKind::NotADirectory,
            "not a folder",
        )));
    }
    platform::check_private_folder(&real)
}

/// フォルダ `path` がなければ、利用者だけが読み書きできるフォルダとして作る（上のフォルダは、ふだんどおりに作る）
///
/// Windows では、この利用者・`SYSTEM`・`Administrators` だけに権利を渡し、上のフォルダの権利を引き継がない。ほかの
/// OS では 0700 で作る。すでにあれば、何もしない。
///
/// # Errors
///
/// 作れない場合。
pub fn create_private_folder(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match platform::create_private_folder(path) {
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists && path.is_dir() => Ok(()),
        result => result,
    }
}

/// フォルダ `path` の権利を、[`create_private_folder`] で作ったときと同じにする（持ち主は変えない）
///
/// Windows では、中のファイル・フォルダが引き継いだ権利も替わる。ほかの OS では、このフォルダだけを 0700 にする
/// （中には、ほかの利用者が入れなくなる）。
///
/// # Errors
///
/// 権利を変えられない場合。
pub fn make_private(path: &Path) -> io::Result<()> {
    platform::make_private(path)
}

/// 環境の変数 `name` の値（絶対のパスのときだけ）
fn absolute_variable(name: &str) -> Option<PathBuf> {
    absolute(std::env::var_os(name))
}

/// 環境の変数の値 `value`（ないときは `None`）を、絶対のパスのときだけ返す
fn absolute(value: Option<OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|path| path.is_absolute())
}
