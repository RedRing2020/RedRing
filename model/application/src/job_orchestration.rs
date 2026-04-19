//! Job向け orchestration 境界。

use cam_sim::{CamJobExecutorAdapter, CamWorkflowError, CamWorkflowSubmitter};
use job_domain::{CamJobEvent, CamJobRecord, CamJobStatus, CamJobType};
use job_runtime::{JobError, JobId, JobManager, JobSpec, JobType, RetryPolicy};

const DEFAULT_TIMEOUT_SECS: u64 = 60;

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

/// 汎用 workflow 送信の入力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobWorkflowSubmitRequest {
    pub job_type: CamJobType,
    pub input_ref: String,
    pub parent_job_id: Option<u64>,
}

/// workflow 送信の出力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobWorkflowSubmitResult {
    pub job_id: u64,
    pub status: CamJobStatus,
}

/// status query の入力境界。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobWorkflowStatusQuery {
    pub job_id: u64,
}

/// status query の出力境界。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobWorkflowStatusResult {
    pub job_id: u64,
    pub status: CamJobStatus,
}

/// result query の入力境界。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobWorkflowResultQuery {
    pub job_id: u64,
}

/// result query の出力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobWorkflowResult {
    pub job: CamJobRecord,
    pub active_result_ref: Option<String>,
}

/// ViewModel 向けイベント batch。
#[derive(Debug, Clone, PartialEq)]
pub struct JobWorkflowEventBatch {
    pub events: Vec<CamJobEvent>,
}

/// job 実行ユースケースの orchestration port。
pub trait JobWorkflowOrchestration {
    fn submit_workflow(
        &mut self,
        request: JobWorkflowSubmitRequest,
    ) -> Result<JobWorkflowSubmitResult, JobOrchestrationError>;

    fn query_status(
        &self,
        query: JobWorkflowStatusQuery,
    ) -> Result<JobWorkflowStatusResult, JobOrchestrationError>;

    fn query_result(
        &self,
        query: JobWorkflowResultQuery,
    ) -> Result<JobWorkflowResult, JobOrchestrationError>;

    fn take_event_batch(&mut self) -> JobWorkflowEventBatch;
}

/// `job_runtime` と `cam_sim` adapter を Application 境界へ束ねる既定実装。
#[derive(Debug, Default)]
pub struct JobWorkflowOrchestrator {
    manager: JobManager,
    executor: CamJobExecutorAdapter,
}

impl JobWorkflowOrchestrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_runtime(manager: JobManager, executor: CamJobExecutorAdapter) -> Self {
        Self { manager, executor }
    }
}

impl JobWorkflowOrchestration for JobWorkflowOrchestrator {
    fn submit_workflow(
        &mut self,
        request: JobWorkflowSubmitRequest,
    ) -> Result<JobWorkflowSubmitResult, JobOrchestrationError> {
        let spec = build_job_spec(&request);
        let job_id = submit_to_runtime(&mut self.manager, &request, spec)?;
        self.manager.execute_with(job_id, &self.executor)?;

        Ok(JobWorkflowSubmitResult {
            job_id: job_id.0,
            status: self.manager.status(job_id)?.into(),
        })
    }

    fn query_status(
        &self,
        query: JobWorkflowStatusQuery,
    ) -> Result<JobWorkflowStatusResult, JobOrchestrationError> {
        let job_id = JobId(query.job_id);
        Ok(JobWorkflowStatusResult {
            job_id: query.job_id,
            status: self.manager.status(job_id)?.into(),
        })
    }

    fn query_result(
        &self,
        query: JobWorkflowResultQuery,
    ) -> Result<JobWorkflowResult, JobOrchestrationError> {
        let job_id = JobId(query.job_id);
        let active_result_ref = self.manager.active_result_ref(job_id)?.map(str::to_owned);
        let job = CamJobRecord::from(self.manager.get(job_id)?);

        Ok(JobWorkflowResult {
            job,
            active_result_ref,
        })
    }

    fn take_event_batch(&mut self) -> JobWorkflowEventBatch {
        let events = self.manager.take_events();
        JobWorkflowEventBatch {
            events: events.iter().map(CamJobEvent::from).collect(),
        }
    }
}

fn build_job_spec(request: &JobWorkflowSubmitRequest) -> JobSpec {
    JobSpec {
        job_type: map_job_type(request.job_type.clone()),
        input_ref: request.input_ref.clone(),
        timeout_secs: DEFAULT_TIMEOUT_SECS,
        retry_policy: RetryPolicy::default(),
    }
}

fn submit_to_runtime(
    manager: &mut JobManager,
    request: &JobWorkflowSubmitRequest,
    spec: JobSpec,
) -> Result<JobId, JobOrchestrationError> {
    match (request.job_type.clone(), request.parent_job_id) {
        (CamJobType::CamProcess, None) => {
            let mut submitter = CamWorkflowSubmitter::new(manager);
            Ok(submitter.submit_cam_process(spec)?)
        }
        (CamJobType::CuttingSimulation, None) => Err(JobOrchestrationError::InvalidRequest(
            "cutting simulation requires parent_job_id".to_string(),
        )),
        (CamJobType::CuttingSimulation, Some(parent_job_id)) => {
            let mut submitter = CamWorkflowSubmitter::new(manager);
            Ok(submitter.submit_cutting_simulation(JobId(parent_job_id), spec)?)
        }
        (CamJobType::NcPostFromCam, None) => Err(JobOrchestrationError::InvalidRequest(
            "nc post from cam requires parent_job_id".to_string(),
        )),
        (CamJobType::NcPostFromCam, Some(parent_job_id)) => {
            let mut submitter = CamWorkflowSubmitter::new(manager);
            Ok(submitter.submit_nc_post_from_cam(JobId(parent_job_id), spec)?)
        }
        (CamJobType::CamProcess, Some(parent_job_id)) => {
            let mut submitter = CamWorkflowSubmitter::new(manager);
            Ok(submitter.submit_child_under(JobId(parent_job_id), spec)?)
        }
    }
}

fn map_job_type(job_type: CamJobType) -> JobType {
    match job_type {
        CamJobType::CamProcess => JobType::CamProcess,
        CamJobType::CuttingSimulation => JobType::CuttingSimulation,
        CamJobType::NcPostFromCam => JobType::NcPostFromCam,
    }
}

#[cfg(test)]
mod tests {
    use job_domain::{CamJobEvent, CamJobStatus, CamJobType};

    use super::{
        JobWorkflowOrchestration, JobWorkflowOrchestrator, JobWorkflowResultQuery,
        JobWorkflowStatusQuery, JobWorkflowSubmitRequest,
    };

    #[test]
    fn submit_status_and_result_queries_work_for_cam_process() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/sample".to_string(),
                parent_job_id: None,
            })
            .expect("cam process should succeed");

        assert_eq!(submit.job_id, 1);
        assert_eq!(submit.status, CamJobStatus::Succeeded);

        let status = orchestrator
            .query_status(JobWorkflowStatusQuery {
                job_id: submit.job_id,
            })
            .expect("status query should succeed");
        assert_eq!(status.status, CamJobStatus::Succeeded);

        let result = orchestrator
            .query_result(JobWorkflowResultQuery {
                job_id: submit.job_id,
            })
            .expect("result query should succeed");
        assert_eq!(result.job.status, CamJobStatus::Succeeded);
        assert_eq!(
            result.active_result_ref.as_deref(),
            Some("result://cam/1/ok")
        );
        assert_eq!(result.job.output_history.len(), 1);
    }

    #[test]
    fn submit_failure_is_visible_through_status_and_result_queries() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://invalid".to_string(),
                parent_job_id: None,
            })
            .expect("submit should still complete with failed runtime status");

        assert_eq!(submit.status, CamJobStatus::Failed);

        let result = orchestrator
            .query_result(JobWorkflowResultQuery {
                job_id: submit.job_id,
            })
            .expect("failed job should still be queryable");
        assert_eq!(result.job.status, CamJobStatus::Failed);
        assert_eq!(result.active_result_ref, None);
        assert!(
            result
                .job
                .last_error
                .as_deref()
                .is_some_and(|message| message.contains("invalid input_ref"))
        );
    }

    #[test]
    fn event_batch_is_normalized_for_viewmodel_consumption() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/sample".to_string(),
                parent_job_id: None,
            })
            .expect("cam process should succeed");

        let batch = orchestrator.take_event_batch();

        assert!(batch.events.iter().any(|event| matches!(
            event,
            CamJobEvent::ArtifactReady { job_id, result_ref }
                if *job_id == submit.job_id && result_ref == "result://cam/1/ok"
        )));
        assert!(batch.events.iter().any(|event| matches!(
            event,
            CamJobEvent::Completed {
                job_id,
                status: CamJobStatus::Succeeded,
                ..
            } if *job_id == submit.job_id
        )));
    }

    #[test]
    fn cutting_simulation_requires_parent_job_id() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let error = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CuttingSimulation,
                input_ref: "input://sim/sample".to_string(),
                parent_job_id: None,
            })
            .expect_err("cutting simulation without parent should be rejected");

        assert_eq!(
            error.to_string(),
            "cutting simulation requires parent_job_id".to_string()
        );
    }

    #[test]
    fn cutting_simulation_submit_and_result_query_work_with_cam_parent() {
        let mut orchestrator = JobWorkflowOrchestrator::new();

        let cam_submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/sample".to_string(),
                parent_job_id: None,
            })
            .expect("cam process should succeed");

        let sim_submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CuttingSimulation,
                input_ref: "input://sim/sample".to_string(),
                parent_job_id: Some(cam_submit.job_id),
            })
            .expect("cutting simulation should succeed");

        assert_eq!(sim_submit.status, CamJobStatus::Succeeded);

        let result = orchestrator
            .query_result(JobWorkflowResultQuery {
                job_id: sim_submit.job_id,
            })
            .expect("result query should succeed");

        assert_eq!(result.job.status, CamJobStatus::Succeeded);
        assert_eq!(
            result.active_result_ref.as_deref(),
            Some("result://sim/2/ok")
        );
    }

    #[test]
    fn nc_post_from_cam_requires_parent_job_id() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let error = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::NcPostFromCam,
                input_ref: "result://cam/1/ok".to_string(),
                parent_job_id: None,
            })
            .expect_err("nc post from cam without parent should be rejected");

        assert_eq!(
            error.to_string(),
            "nc post from cam requires parent_job_id".to_string()
        );
    }
}
