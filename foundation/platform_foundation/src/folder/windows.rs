//! 利用者ごとのフォルダの場所の、Windows の実装

use std::path::PathBuf;

/// アプリのデータのフォルダ（`%LOCALAPPDATA%`）。`variable` は、環境の変数を絶対のパスとして読む
pub(super) fn data_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("LOCALAPPDATA")
}

/// 設定のフォルダ（`%APPDATA%`）。`variable` は、環境の変数を絶対のパスとして読む
pub(super) fn config_folder(variable: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    variable("APPDATA")
}
