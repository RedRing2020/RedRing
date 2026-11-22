//! ファイルの置き換えの、Windows 以外の実装

use std::io;
use std::path::Path;

/// 名前の変更は、ほかのプログラムが開いていても断られないため、一時的な拒否はない
pub(super) fn is_transient_refusal(_error: &io::Error) -> bool {
    false
}

/// `path` のあるフォルダを、ドライブまで書き込む（名前の変更を残すため。できなくても置き換えは済んでいる）
pub(super) fn sync_parent(path: &Path) {
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    if let Ok(folder) = std::fs::File::open(parent) {
        let _ = folder.sync_all();
    }
}
