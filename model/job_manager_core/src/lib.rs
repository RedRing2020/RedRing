//! 共通ジョブマネージャー基盤
//!
//! 実行制御（submit/status/cancel/retry）とイベント契約を提供

pub mod events;
pub mod executor;
pub mod manager;
pub mod types;

pub use events::JobEvent;
pub use executor::{JobExecutionResult, JobExecutor};
pub use manager::JobManager;
pub use types::{JobError, JobId, JobRecord, JobSpec, JobStatus, JobType, RetryPolicy};
