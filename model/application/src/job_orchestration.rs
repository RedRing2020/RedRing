//! ジョブの投入・実行の誤り。

use cam_sim::CamWorkflowError;
use job_runtime::JobError;

/// Job orchestration で返却する統一エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOrchestrationError {
    InvalidRequest(String),
    Workflow(String),
    Runtime(String),
}

impl std::fmt::Display for JobOrchestrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) | Self::Workflow(message) | Self::Runtime(message) => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for JobOrchestrationError {}

impl From<JobError> for JobOrchestrationError {
    fn from(value: JobError) -> Self {
        Self::Runtime(value.to_string())
    }
}

impl From<CamWorkflowError> for JobOrchestrationError {
    fn from(value: CamWorkflowError) -> Self {
        Self::Workflow(value.to_string())
    }
}
