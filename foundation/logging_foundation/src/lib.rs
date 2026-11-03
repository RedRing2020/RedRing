//! RedRing ワークスペース共通のロギング基盤。
//!
//! 初期化・レート制御・パス生成・構造化フィールド定数を再エクスポートする。

pub mod filter;
pub mod init;
pub mod path;
pub mod structured;

pub use filter::{frame_interval_from_env, should_log_after, should_log_every_n_frames};
pub use init::{init_logging, LoggingInitConfig};
pub use path::compose_log_file_path;
pub use structured::{
    ERROR_KIND_APP, ERROR_KIND_SIMULATION, ERROR_KIND_SYSTEM, ERROR_KIND_VALIDATION,
};
