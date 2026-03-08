use job_manager_core::{JobExecutionResult, JobExecutor, JobRecord, JobStatus, JobType};

/// cam_sim から JobManager へ接続する初期アダプタ
#[derive(Debug, Default, Clone, Copy)]
pub struct CamJobExecutorAdapter;

impl CamJobExecutorAdapter {
    fn run_cam_process(&self, job: &JobRecord) -> JobExecutionResult {
        if !job.spec.input_ref.starts_with("input://cam/") {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                log_ref: Some(format!("log://cam/{}/invalid", job.id.0)),
                error: Some(format!(
                    "invalid input_ref for cam process: {}",
                    job.spec.input_ref
                )),
            };
        }

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 200,
            result_ref: Some(format!("result://cam/{}/ok", job.id.0)),
            log_ref: Some(format!("log://cam/{}/ok", job.id.0)),
            error: None,
        }
    }

    fn run_cutting_simulation(&self, job: &JobRecord) -> JobExecutionResult {
        if !job.spec.input_ref.starts_with("input://sim/") {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                log_ref: Some(format!("log://sim/{}/invalid", job.id.0)),
                error: Some(format!(
                    "invalid input_ref for cutting simulation: {}",
                    job.spec.input_ref
                )),
            };
        }

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 300,
            result_ref: Some(format!("result://sim/{}/ok", job.id.0)),
            log_ref: Some(format!("log://sim/{}/ok", job.id.0)),
            error: None,
        }
    }
}

impl JobExecutor for CamJobExecutorAdapter {
    fn execute(&self, job: &JobRecord) -> JobExecutionResult {
        match job.spec.job_type {
            JobType::CamProcessBatch => self.run_cam_process(job),
            JobType::CuttingSimulationBatch => self.run_cutting_simulation(job),
        }
    }
}
