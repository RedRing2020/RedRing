//! 利用者ごとのフォルダの場所（XDG Base Directory の決まり）と、利用者だけが読み書きできるこの PC のフォルダの、
//! Windows 以外の実装

use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::PrivateFolderError;

/// 利用者だけが読み書きできるフォルダの権利
const PRIVATE_MODE: u32 = 0o700;
/// グループ・ほかの利用者の権利
const OTHERS_MODE: u32 = 0o077;

/// アプリのデータのフォルダ（`$XDG_DATA_HOME`、ない場合は `~/.local/share`）。`variable` は、環境の変数を絶対の
/// パスとして読む
pub(super) fn data_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("XDG_DATA_HOME")
        .or_else(|| variable("HOME").map(|home| home.join(".local").join("share")))
}

/// 設定のフォルダ（`$XDG_CONFIG_HOME`、ない場合は `~/.config`）。`variable` は、環境の変数を絶対のパスとして読む
pub(super) fn config_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("XDG_CONFIG_HOME").or_else(|| variable("HOME").map(|home| home.join(".config")))
}

/// 実際のパス `path` のフォルダが、利用者だけが読み書きできるこの PC のフォルダか
pub(super) fn check_private_folder(path: &Path) -> Result<(), PrivateFolderError> {
    if !is_on_this_pc(path).map_err(PrivateFolderError::Io)? {
        return Err(PrivateFolderError::NotOnThisPc);
    }
    let metadata = std::fs::metadata(path).map_err(PrivateFolderError::Io)?;
    // SAFETY: 引数を持たず、失敗しない
    let user = unsafe { libc::getuid() };
    if metadata.uid() != user || metadata.mode() & OTHERS_MODE != 0 {
        return Err(PrivateFolderError::OpenToOthers);
    }
    Ok(())
}

/// フォルダ `path` を、利用者だけが読み書きできるフォルダ（0700）として作る
pub(super) fn create_private_folder(path: &Path) -> io::Result<()> {
    std::fs::DirBuilder::new().mode(PRIVATE_MODE).create(path)
}

/// フォルダ `path` を 0700 にする
pub(super) fn make_private(path: &Path) -> io::Result<()> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(PRIVATE_MODE))
}

/// パス `path` が、ネットワークのファイルシステムにないか（Linux はファイルシステムの種類、macOS はローカルの印で
/// 確かめる。ほかの OS では確かめず、`true`）
#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
fn is_on_this_pc(path: &Path) -> io::Result<bool> {
    use std::os::unix::ffi::OsStrExt;

    let name = std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: `name` は終わりが 0 の文字列。`stat` に書いてもらう
    if unsafe { libc::statfs(name.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: 成功したため、書かれている
    let stat = unsafe { stat.assume_init() };
    Ok(is_local_file_system(&stat))
}

/// ネットワークのファイルシステムの種類（`statfs` の `f_type`。NFS・SMB・CIFS・SMB2・AFS・Coda・NCP・Ceph・9P）
#[cfg(any(target_os = "linux", target_os = "android"))]
const NETWORK_FILE_SYSTEMS: [u32; 9] = [
    0x6969,
    0x517B,
    0xFF53_4D42,
    0xFE53_4D42,
    0x5346_414F,
    0x7375_7245,
    0x564C,
    0x00C3_6400,
    0x0102_1997,
];

#[cfg(any(target_os = "linux", target_os = "android"))]
fn is_local_file_system(stat: &libc::statfs) -> bool {
    // 種類の型は CPU で違う（符号の有無・長さ）ため、下の 32 ビットで比べる
    #[allow(clippy::unnecessary_cast)]
    let kind = stat.f_type as u32;
    !NETWORK_FILE_SYSTEMS.contains(&kind)
}

#[cfg(target_os = "macos")]
fn is_local_file_system(stat: &libc::statfs) -> bool {
    stat.f_flags & libc::MNT_LOCAL as u32 != 0
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
fn is_on_this_pc(_path: &Path) -> io::Result<bool> {
    Ok(true)
}
