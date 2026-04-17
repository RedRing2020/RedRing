//! 共通ジョブマネージャー基盤
//!
//! 実行制御（submit/status/cancel/retry）とイベント契約を提供

pub mod artifact_manifest;
pub mod events;
pub mod executor;
pub mod manager;
pub mod types;

pub use artifact_manifest::{
    ArtifactManifest, ArtifactManifestError, ArtifactType, ContractValidationDecision,
    FormatVersionCompatibility, decide_contract_validation_error,
    decide_contract_validation_result, evaluate_format_version_compatibility, validate_io_contract,
    validate_output_contract,
};
pub use events::JobEvent;
pub use executor::{JobExecutionResult, JobExecutor};
pub use manager::JobManager;
pub use types::{
    JobError, JobGroupSummary, JobId, JobOutputRecord, JobOutputValidity, JobRecord, JobRelation,
    JobSpec, JobStatus, JobType, RetryPolicy,
};
