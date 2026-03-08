use std::collections::HashMap;
use std::time::SystemTime;

use crate::events::JobEvent;
use crate::executor::JobExecutor;
use crate::types::{JobError, JobId, JobRecord, JobSpec, JobStatus};

#[derive(Debug, Default)]
pub struct JobManager {
    jobs: HashMap<JobId, JobRecord>,
    event_queue: Vec<JobEvent>,
    next_id: u64,
}

impl JobManager {
    /// 空のマネージャーを生成
    pub fn new() -> Self {
        Self::default()
    }

    /// ジョブを登録してIDを返す
    pub fn submit(&mut self, spec: JobSpec) -> JobId {
        self.next_id += 1;
        let id = JobId(self.next_id);
        let now = SystemTime::now();

        let record = JobRecord {
            id,
            spec,
            status: JobStatus::Queued,
            attempts: 0,
            created_at: now,
            updated_at: now,
            result_ref: None,
            log_ref: None,
            last_error: None,
        };

        self.jobs.insert(id, record);
        id
    }

    /// 現在状態を取得
    pub fn status(&self, id: JobId) -> Result<JobStatus, JobError> {
        let record = self.jobs.get(&id).ok_or(JobError::JobNotFound(id))?;
        Ok(record.status)
    }

    /// ジョブ情報を取得
    pub fn get(&self, id: JobId) -> Result<&JobRecord, JobError> {
        self.jobs.get(&id).ok_or(JobError::JobNotFound(id))
    }

    /// ジョブ一覧をID順で取得
    pub fn list(&self) -> Vec<&JobRecord> {
        let mut records: Vec<&JobRecord> = self.jobs.values().collect();
        records.sort_by_key(|r| r.id.0);
        records
    }

    /// 待機中ジョブを実行中へ遷移
    pub fn start(&mut self, id: JobId) -> Result<(), JobError> {
        self.transition(id, JobStatus::Running)
    }

    /// 待機中または実行中ジョブをキャンセル
    pub fn cancel(&mut self, id: JobId) -> Result<(), JobError> {
        self.transition(id, JobStatus::Canceled)
    }

    /// 正常終了へ遷移し、参照情報を保存
    pub fn mark_succeeded(
        &mut self,
        id: JobId,
        result_ref: Option<String>,
        log_ref: Option<String>,
    ) -> Result<(), JobError> {
        self.transition(id, JobStatus::Succeeded)?;
        let record = self.jobs.get_mut(&id).ok_or(JobError::JobNotFound(id))?;
        record.result_ref = result_ref.clone();
        record.log_ref = log_ref.clone();
        self.event_queue.push(JobEvent::Completed {
            job_id: id,
            status: JobStatus::Succeeded,
            result_ref,
            log_ref,
        });
        Ok(())
    }

    /// 失敗終了へ遷移し、エラーと参照情報を保存
    pub fn mark_failed(
        &mut self,
        id: JobId,
        error: String,
        log_ref: Option<String>,
    ) -> Result<(), JobError> {
        self.transition(id, JobStatus::Failed)?;
        let record = self.jobs.get_mut(&id).ok_or(JobError::JobNotFound(id))?;
        record.last_error = Some(error);
        record.log_ref = log_ref.clone();
        self.event_queue.push(JobEvent::Completed {
            job_id: id,
            status: JobStatus::Failed,
            result_ref: None,
            log_ref,
        });
        Ok(())
    }

    /// 失敗ジョブを再投入
    pub fn retry(&mut self, id: JobId) -> Result<(), JobError> {
        let record = self.jobs.get_mut(&id).ok_or(JobError::JobNotFound(id))?;

        if record.status != JobStatus::Failed {
            return Err(JobError::InvalidTransition {
                from: record.status,
                to: JobStatus::Queued,
            });
        }

        if record.attempts >= record.spec.retry_policy.max_retries {
            return Err(JobError::RetryLimitExceeded {
                attempts: record.attempts,
                max_retries: record.spec.retry_policy.max_retries,
            });
        }

        record.attempts += 1;
        let from = record.status;
        record.status = JobStatus::Queued;
        record.updated_at = SystemTime::now();
        record.last_error = None;
        self.event_queue.push(JobEvent::StatusChanged {
            job_id: id,
            from,
            to: JobStatus::Queued,
        });
        Ok(())
    }

    /// 進捗更新イベントを追加
    pub fn emit_progress(&mut self, id: JobId, progress: f32, message: Option<String>) {
        self.event_queue.push(JobEvent::ProgressUpdated {
            job_id: id,
            progress,
            message,
        });
    }

    /// 成果物準備完了イベントを追加
    pub fn emit_artifact_ready(&mut self, id: JobId, result_ref: String) {
        self.event_queue.push(JobEvent::ArtifactReady {
            job_id: id,
            result_ref,
        });
    }

    /// スタブ実行器でジョブを1件実行
    pub fn execute_with<E: JobExecutor>(
        &mut self,
        id: JobId,
        executor: &E,
    ) -> Result<(), JobError> {
        self.start(id)?;

        let snapshot = self.jobs.get(&id).ok_or(JobError::JobNotFound(id))?.clone();
        let execution = executor.execute(&snapshot);

        let timeout_millis = snapshot.spec.timeout_secs.saturating_mul(1000);
        if execution.elapsed_millis > timeout_millis {
            return self.mark_failed(
                id,
                format!(
                    "timeout exceeded: elapsed={}ms, timeout={}ms",
                    execution.elapsed_millis, timeout_millis
                ),
                execution.log_ref,
            );
        }

        match execution.status {
            JobStatus::Succeeded => {
                if let Some(result_ref) = execution.result_ref.clone() {
                    self.emit_artifact_ready(id, result_ref);
                }
                self.mark_succeeded(id, execution.result_ref, execution.log_ref)
            }
            JobStatus::Failed => self.mark_failed(
                id,
                execution
                    .error
                    .unwrap_or_else(|| "executor returned failed status".to_string()),
                execution.log_ref,
            ),
            JobStatus::Canceled => self.cancel(id),
            JobStatus::Queued | JobStatus::Running => Err(JobError::InvalidTransition {
                from: JobStatus::Running,
                to: execution.status,
            }),
        }
    }

    /// 保留イベントを取り出して空にする
    pub fn take_events(&mut self) -> Vec<JobEvent> {
        std::mem::take(&mut self.event_queue)
    }

    fn transition(&mut self, id: JobId, to: JobStatus) -> Result<(), JobError> {
        let record = self.jobs.get_mut(&id).ok_or(JobError::JobNotFound(id))?;

        if record.status.is_terminal() {
            return Err(JobError::AlreadyTerminal(record.status));
        }

        let valid = matches!(
            (record.status, to),
            (JobStatus::Queued, JobStatus::Running)
                | (JobStatus::Queued, JobStatus::Canceled)
                | (JobStatus::Running, JobStatus::Succeeded)
                | (JobStatus::Running, JobStatus::Failed)
                | (JobStatus::Running, JobStatus::Canceled)
        );

        if !valid {
            return Err(JobError::InvalidTransition {
                from: record.status,
                to,
            });
        }

        let from = record.status;
        record.status = to;
        record.updated_at = SystemTime::now();

        self.event_queue.push(JobEvent::StatusChanged {
            job_id: id,
            from,
            to,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::executor::{JobExecutionResult, JobExecutor};
    use crate::types::{JobSpec, JobType, RetryPolicy};

    use super::*;

    fn sample_spec() -> JobSpec {
        JobSpec {
            job_type: JobType::CamProcessBatch,
            input_ref: "input://sample".to_string(),
            timeout_secs: 60,
            retry_policy: RetryPolicy {
                max_retries: 1,
                backoff_millis: 100,
            },
        }
    }

    fn simulation_spec() -> JobSpec {
        JobSpec {
            job_type: JobType::CuttingSimulationBatch,
            input_ref: "input://sim".to_string(),
            timeout_secs: 60,
            retry_policy: RetryPolicy {
                max_retries: 1,
                backoff_millis: 100,
            },
        }
    }

    struct StubExecutor {
        elapsed_millis: u64,
        succeed: bool,
    }

    impl JobExecutor for StubExecutor {
        fn execute(&self, job: &JobRecord) -> JobExecutionResult {
            let result_ref = match job.spec.job_type {
                JobType::CamProcessBatch => Some("result://cam".to_string()),
                JobType::CuttingSimulationBatch => Some("result://sim".to_string()),
            };

            if self.succeed {
                JobExecutionResult {
                    status: JobStatus::Succeeded,
                    elapsed_millis: self.elapsed_millis,
                    result_ref,
                    log_ref: Some("log://ok".to_string()),
                    error: None,
                }
            } else {
                JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: self.elapsed_millis,
                    result_ref: None,
                    log_ref: Some("log://ng".to_string()),
                    error: Some("stub failure".to_string()),
                }
            }
        }
    }

    #[test]
    fn submit_start_success_flow_works() {
        let mut manager = JobManager::new();
        let id = manager.submit(sample_spec());

        assert_eq!(manager.status(id).unwrap(), JobStatus::Queued);
        manager.start(id).unwrap();
        manager
            .mark_succeeded(id, Some("result://ok".to_string()), None)
            .unwrap();

        let final_status = manager.status(id).unwrap();
        assert_eq!(final_status, JobStatus::Succeeded);
    }

    #[test]
    fn failed_job_can_retry_within_limit() {
        let mut manager = JobManager::new();
        let id = manager.submit(sample_spec());

        manager.start(id).unwrap();
        manager
            .mark_failed(id, "temporary error".to_string(), None)
            .unwrap();

        manager.retry(id).unwrap();
        assert_eq!(manager.status(id).unwrap(), JobStatus::Queued);
    }

    #[test]
    fn stub_executor_runs_two_job_types() {
        let mut manager = JobManager::new();
        let cam_id = manager.submit(sample_spec());
        let sim_id = manager.submit(simulation_spec());
        let executor = StubExecutor {
            elapsed_millis: 300,
            succeed: true,
        };

        manager.execute_with(cam_id, &executor).unwrap();
        manager.execute_with(sim_id, &executor).unwrap();

        let cam = manager.get(cam_id).unwrap();
        let sim = manager.get(sim_id).unwrap();
        assert_eq!(cam.status, JobStatus::Succeeded);
        assert_eq!(sim.status, JobStatus::Succeeded);
        assert_eq!(cam.result_ref.as_deref(), Some("result://cam"));
        assert_eq!(sim.result_ref.as_deref(), Some("result://sim"));
    }

    #[test]
    fn timeout_retry_cancel_flow_works_with_stub() {
        let mut manager = JobManager::new();
        let mut spec = sample_spec();
        spec.timeout_secs = 1;
        spec.retry_policy.max_retries = 2;
        let id = manager.submit(spec);

        let slow_executor = StubExecutor {
            elapsed_millis: 2_000,
            succeed: true,
        };

        manager.execute_with(id, &slow_executor).unwrap();
        assert_eq!(manager.status(id).unwrap(), JobStatus::Failed);

        manager.retry(id).unwrap();
        assert_eq!(manager.status(id).unwrap(), JobStatus::Queued);

        manager.cancel(id).unwrap();
        assert_eq!(manager.status(id).unwrap(), JobStatus::Canceled);
    }

    #[test]
    fn progress_artifact_completed_events_can_be_taken() {
        let mut manager = JobManager::new();
        let id = manager.submit(sample_spec());
        let executor = StubExecutor {
            elapsed_millis: 100,
            succeed: true,
        };

        manager.emit_progress(id, 0.5, Some("half".to_string()));
        manager.execute_with(id, &executor).unwrap();

        let events = manager.take_events();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, JobEvent::ProgressUpdated { job_id, .. } if *job_id == id))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, JobEvent::ArtifactReady { job_id, .. } if *job_id == id))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, JobEvent::Completed { job_id, .. } if *job_id == id))
        );
    }
}
