use std::fs;
use std::path::{Path, PathBuf};

/// ログディレクトリとファイル名からログファイルパスを組み立てる。
pub fn compose_log_file_path(log_dir: &str, log_file_name: &str) -> PathBuf {
    Path::new(log_dir).join(log_file_name)
}

/// ログディレクトリ作成と既存ログ削除を行い、出力先パスを返す。
pub(crate) fn prepare_log_file(log_dir: &str, log_file_name: &str) -> Option<PathBuf> {
    let log_file_path = compose_log_file_path(log_dir, log_file_name);

    if let Err(e) = fs::create_dir_all(log_dir) {
        eprintln!("⚠ ログディレクトリ作成失敗: {}", e);
        eprintln!("  コンソールログのみで続行します");
        return None;
    }

    if log_file_path.exists() {
        if let Err(e) = fs::remove_file(&log_file_path) {
            eprintln!("⚠ 古いログファイル削除失敗: {}", e);
        }
    }

    Some(log_file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_log_file_path() {
        let p = compose_log_file_path("logs", "redring.log");
        assert_eq!(p, PathBuf::from("logs/redring.log"));
    }
}
