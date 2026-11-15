//! ファイルの置き換えの Windows の実装

use std::io;
use std::path::Path;

/// アクセス拒否（`ERROR_ACCESS_DENIED`）
const ERROR_ACCESS_DENIED: i32 = 5;
/// 共有違反（`ERROR_SHARING_VIOLATION`）
const ERROR_SHARING_VIOLATION: i32 = 32;

/// ほかのプログラムがファイルを開いているための、一時的な拒否か
pub(super) fn is_transient_refusal(error: &io::Error) -> bool {
    matches!(
        error.raw_os_error(),
        Some(ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION)
    )
}

/// 名前の変更の記録は NTFS のジャーナルに任せ、何もしない
pub(super) fn sync_parent(_path: &Path) {}
