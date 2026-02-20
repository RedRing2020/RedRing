//! ログシステムの初期化と管理
//!
//! アプリケーション起動時のログ設定を管理します。
//! - コンソールとファイルの両方に出力
//! - 起動ごとにログファイルを新規作成（古いログは自動削除）
//! - モジュール別のログレベルフィルタリング

use logging_foundation::{init_logging as init_with_foundation, LoggingInitConfig};

/// ログディレクトリのパス
const LOG_DIR: &str = "logs";

/// ログファイル名
const LOG_FILE_NAME: &str = "redring.log";

/// デフォルトのログフィルタ設定
///
/// wgpu/Vulkan関連は警告のみ、フレーム高頻度層はwarnで抑制。
/// 必要時は RUST_LOG で明示的に詳細化する。
const DEFAULT_LOG_FILTER: &str = "warn,\
    wgpu=warn,wgpu_hal=warn,wgpu_core=warn,naga=warn,\
    redring=info,stage=warn,viewmodel_graphics=warn,viewmodel_converter=warn,render=warn,cam_core=info";

/// ログシステムを初期化
///
/// コンソールとファイルの両方に出力するログシステムを設定します。
/// 既存のログファイルは自動的に削除され、新規ログファイルが作成されます。
///
/// # エラー処理
///
/// ログファイルの作成に失敗した場合は、コンソールのみのログに自動的にフォールバックします。
pub fn init_logging() {
    let config = LoggingInitConfig {
        default_filter: DEFAULT_LOG_FILTER,
        log_dir: LOG_DIR,
        log_file_name: LOG_FILE_NAME,
        app_name: "RedRing アプリケーション",
    };
    init_with_foundation(&config);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use logging_foundation::compose_log_file_path;

    #[test]
    fn test_log_file_path() {
        let log_file_path = compose_log_file_path(LOG_DIR, LOG_FILE_NAME);
        assert_eq!(log_file_path, PathBuf::from("logs/redring.log"));
    }
}
