//! 利用者ごとのフォルダの場所と、フォルダがこの PC にあるかの、Windows の実装

use std::path::{Component, Path, PathBuf, Prefix};

use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
use windows_sys::Win32::System::WindowsProgramming::{
    DRIVE_CDROM, DRIVE_FIXED, DRIVE_RAMDISK, DRIVE_REMOVABLE,
};

/// アプリのデータのフォルダ（`%LOCALAPPDATA%`）。`variable` は、環境の変数を絶対のパスとして読む
pub(super) fn data_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("LOCALAPPDATA")
}

/// 設定のフォルダ（`%APPDATA%`）。`variable` は、環境の変数を絶対のパスとして読む
pub(super) fn config_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("APPDATA")
}

/// パス `path` のドライブが、この PC のドライブか（UNC のパス・デバイスのパスは使わない）
pub(super) fn is_on_this_pc(path: &Path) -> bool {
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return false;
    };
    let letter = match prefix.kind() {
        Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => letter,
        // UNC のパス（\\サーバー\共有）・デバイスのパスは使わない
        _ => return false,
    };
    let root = crate::wide(&format!("{}:\\", char::from(letter)));
    // SAFETY: `root` は終わりが 0 の UTF-16 の文字列
    let kind = unsafe { GetDriveTypeW(root.as_ptr()) };
    matches!(
        kind,
        DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_CDROM | DRIVE_RAMDISK
    )
}
