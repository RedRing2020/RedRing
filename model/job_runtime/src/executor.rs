use crate::types::{JobRecord, JobStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobExecutionResult {
    pub status: JobStatus,
    pub elapsed_millis: u64,
    pub result_ref: Option<String>,
    pub artifact_bytes: Option<Vec<u8>>,
    pub log_ref: Option<String>,
    pub error: Option<String>,
}

pub trait JobExecutor {
    fn execute(&self, job: &JobRecord, input_artifact_bytes: Option<&[u8]>) -> JobExecutionResult;
}
