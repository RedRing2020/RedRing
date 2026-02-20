use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct LoggingInitConfig<'a> {
    pub default_filter: &'a str,
    pub log_dir: &'a str,
    pub log_file_name: &'a str,
    pub app_name: &'a str,
}

pub fn init_logging(config: &LoggingInitConfig<'_>) {
    let log_file_path = prepare_log_file(config.log_dir, config.log_file_name);

    match log_file_path {
        Some(path) => init_dual_output_logging(config, &path),
        None => init_console_only_logging(config),
    }
}

pub fn frame_interval_from_env(var_name: &str, default: u64) -> u64 {
    std::env::var(var_name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}

pub fn should_log_every_n_frames(frame: u64, interval: u64) -> bool {
    interval > 0 && frame.is_multiple_of(interval)
}

pub fn should_log_after(last: &mut Instant, interval: Duration) -> bool {
    if last.elapsed() >= interval {
        *last = Instant::now();
        true
    } else {
        false
    }
}

pub fn compose_log_file_path(log_dir: &str, log_file_name: &str) -> PathBuf {
    Path::new(log_dir).join(log_file_name)
}

fn prepare_log_file(log_dir: &str, log_file_name: &str) -> Option<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_log_file_path() {
        let p = compose_log_file_path("logs", "redring.log");
        assert_eq!(p, PathBuf::from("logs/redring.log"));
    }

    #[test]
    fn test_should_log_every_n_frames() {
        assert!(!should_log_every_n_frames(1, 60));
        assert!(should_log_every_n_frames(60, 60));
    }

    #[test]
    fn test_frame_interval_from_env_default() {
        let value = frame_interval_from_env("REDRING_UNSET_TEST_VAR", 120);
        assert_eq!(value, 120);
    }
}
