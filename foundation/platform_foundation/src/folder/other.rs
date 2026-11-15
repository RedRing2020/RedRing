//! 利用者ごとのフォルダの場所の、Windows 以外の実装（XDG Base Directory の決まり）

use std::path::PathBuf;

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
