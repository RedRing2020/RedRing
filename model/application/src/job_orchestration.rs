//! Job向け orchestration 境界。

/// 汎用 workflow 送信の入力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobWorkflowSubmitRequest {
    pub job_type: String,
    pub input_ref: String,
}

/// workflow 送信の出力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobWorkflowSubmitResult {
    pub job_id: u64,
}

/// job 送信ユースケースの orchestration port。
pub trait JobWorkflowOrchestration {
    fn submit_workflow(
        &self,
        request: JobWorkflowSubmitRequest,
    ) -> Result<JobWorkflowSubmitResult, String>;
}
