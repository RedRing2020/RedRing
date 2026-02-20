//! ログシステムの初期化と管理
//!
//! アプリケーション起動時のログ設定を管理します。
//! - コンソールとファイルの両方に出力
//! - 起動ごとにログファイルを新規作成（古いログは自動削除）
//! - モジュール別のログレベルフィルタリング

use std::fs;
use std::path::PathBuf;

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
    let log_file_path = prepare_log_file();

    match log_file_path {
        Some(path) => init_dual_output_logging(&path),
        None => init_console_only_logging(),
    }
}

/// ログファイルを準備
///
/// ログディレクトリを作成し、古いログファイルを削除して新規ファイルを作成します。
///
/// # 戻り値
///
/// 成功した場合はログファイルのパス、失敗した場合は None
fn prepare_log_file() -> Option<PathBuf> {
    let log_dir = PathBuf::from(LOG_DIR);
    let log_file_path = log_dir.join(LOG_FILE_NAME);

    // ログディレクトリを作成
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("⚠ ログディレクトリ作成失敗: {}", e);
        eprintln!("  コンソールログのみで続行します");
        return None;
    }

    // 既存のログファイルを削除（起動ごとに新規作成）
    if log_file_path.exists() {
        if let Err(e) = fs::remove_file(&log_file_path) {
            eprintln!("⚠ 古いログファイル削除失敗: {}", e);
        }
    }

    Some(log_file_path)
}

/// コンソールとファイルの両方に出力するログシステムを初期化
fn init_dual_output_logging(log_file_path: &PathBuf) {
    // ログファイルを作成
    let file = match fs::File::create(log_file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠ ログファイル作成失敗: {}", e);
            eprintln!("  コンソールログのみで続行します");
            init_console_only_logging();
            return;
        }
    };

    // 環境変数からフィルタを取得（未設定時はデフォルト使用）
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_FILTER.to_string());

    // コンソールとファイルの両方に出力するレイヤーを設定
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true), // コンソールにはカラー出力
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file)
                .with_ansi(false), // ファイルには平文（カラーコードなし）
        )
        .init();

    tracing::info!("RedRing アプリケーション起動");
    tracing::info!("ログファイル: {}", log_file_path.display());
}

/// コンソールのみに出力するログシステムを初期化（フォールバック用）
fn init_console_only_logging() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_FILTER.to_string());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .init();

    tracing::info!("RedRing アプリケーション起動（コンソールログのみ）");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_file_path() {
        let log_dir = PathBuf::from(LOG_DIR);
        let log_file_path = log_dir.join(LOG_FILE_NAME);
        assert_eq!(log_file_path, PathBuf::from("logs/redring.log"));
    }
}
