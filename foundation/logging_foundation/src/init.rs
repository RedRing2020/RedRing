use std::fs;
use std::path::Path;

use crate::path::prepare_log_file;

/// ロギング初期化時の設定値。
pub struct LoggingInitConfig<'a> {
    pub default_filter: &'a str,
    pub log_dir: &'a str,
    pub log_file_name: &'a str,
    pub app_name: &'a str,
}

/// ログ出力先を準備して subscriber を初期化する。
pub fn init_logging(config: &LoggingInitConfig<'_>) {
    let log_file_path = prepare_log_file(config.log_dir, config.log_file_name);

    match log_file_path {
        Some(path) => init_dual_output_logging(config, &path),
        None => init_console_only_logging(config),
    }
}

fn init_dual_output_logging(config: &LoggingInitConfig<'_>, log_file_path: &Path) {
    let file = match fs::File::create(log_file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠ ログファイル作成失敗: {}", e);
            eprintln!("  コンソールログのみで続行します");
            init_console_only_logging(config);
            return;
        }
    };

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| config.default_filter.to_string());

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // コンソールとファイルへ同時出力する。
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file)
                .with_ansi(false),
        )
        .init();

    tracing::info!("{} 起動", config.app_name);
    tracing::info!("ログファイル: {}", log_file_path.display());
}

fn init_console_only_logging(config: &LoggingInitConfig<'_>) {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| config.default_filter.to_string());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .init();

    tracing::info!("{} 起動（コンソールログのみ）", config.app_name);
}
