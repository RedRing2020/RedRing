use crate::types::{JobRecord, JobStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobExecutionResult {
    pub status: JobStatus,
    pub elapsed_millis: u64,
    pub result_ref: Option<String>,
    pub log_ref: Option<String>,
    pub error: Option<String>,
}

pub trait JobExecutor {
    fn execute(&self, job: &JobRecord) -> JobExecutionResult;
}
