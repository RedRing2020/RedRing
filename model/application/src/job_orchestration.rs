//! Job orchestration boundaries.

/// Request boundary for a generic workflow submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobWorkflowSubmitRequest {
    pub job_type: String,
    pub input_ref: String,
}

/// Result boundary for a workflow submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobWorkflowSubmitResult {
    pub job_id: u64,
}

/// Orchestration port for job submission use cases.
pub trait JobWorkflowOrchestration {
    fn submit_workflow(
        &self,
        request: JobWorkflowSubmitRequest,
    ) -> Result<JobWorkflowSubmitResult, String>;
}
