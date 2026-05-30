use std::collections::HashMap;
use std::time::SystemTime;

use crate::events::JobEvent;
use crate::executor::JobExecutor;
use crate::types::{
    JobError, JobGroupSummary, JobId, JobOutputRecord, JobOutputValidity, JobRecord, JobRelation,
    JobSpec, JobStatus,
};

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
        self.submit_with_relation(spec, JobRelation::default())
            .expect("default relation should be valid")
    }

    /// 関連情報付きでジョブを登録してIDを返す
    pub fn submit_with_relation(
        &mut self,
        spec: JobSpec,
        relation: JobRelation,
    ) -> Result<JobId, JobError> {
        if let Some(parent_id) = relation.parent_job_id
            && !self.jobs.contains_key(&parent_id)
        {
            return Err(JobError::ParentJobNotFound(parent_id));
        }

        self.next_id += 1;
        let id = JobId(self.next_id);
        let now = SystemTime::now();

        let record = JobRecord {
            id,
            spec,
            parent_job_id: relation.parent_job_id,
            group_id: relation.group_id,
            status: JobStatus::Queued,
            attempts: 0,
            created_at: now,
            updated_at: now,
            output_history: Vec::new(),
            last_error: None,
        };

        self.jobs.insert(id, record);
        self.emit_group_progress_if_needed(id);
        Ok(id)
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

    /// 現在有効な成果物参照を取得
    pub fn active_result_ref(&self, id: JobId) -> Result<Option<&str>, JobError> {
        let record = self.jobs.get(&id).ok_or(JobError::JobNotFound(id))?;
        Ok(record
            .output_history
            .iter()
            .find(|o| o.validity == JobOutputValidity::Active)
            .map(|o| o.result_ref.as_str()))
    }

    fn latest_input_result_bytes(&self, id: JobId) -> Result<Option<Vec<u8>>, JobError> {
        let record = self.jobs.get(&id).ok_or(JobError::JobNotFound(id))?;
        Ok(record
            .output_history
            .iter()
            .rev()
            .find(|o| o.validity != JobOutputValidity::InvalidatedByDependency)
            .and_then(|o| o.artifact_bytes.clone()))
    }

    /// 成果物履歴を新しい順で取得
    pub fn output_history(&self, id: JobId) -> Result<Vec<&JobOutputRecord>, JobError> {
        let record = self.jobs.get(&id).ok_or(JobError::JobNotFound(id))?;
        Ok(record.output_history.iter().rev().collect())
    }

    /// ジョブ一覧をID順で取得
    pub fn list(&self) -> Vec<&JobRecord> {
        let mut records: Vec<&JobRecord> = self.jobs.values().collect();
        records.sort_by_key(|r| r.id.0);
        records
    }

    /// 親ジョブ配下をID順で取得
    pub fn list_by_parent(&self, parent_job_id: JobId) -> Vec<&JobRecord> {
        let mut records: Vec<&JobRecord> = self
            .jobs
            .values()
            .filter(|r| r.parent_job_id == Some(parent_job_id))
            .collect();
        records.sort_by_key(|r| r.id.0);
        records
    }

    /// グループIDで一覧をID順で取得
    pub fn list_by_group(&self, group_id: &str) -> Vec<&JobRecord> {
        let mut records: Vec<&JobRecord> = self
            .jobs
            .values()
            .filter(|r| r.group_id.as_deref() == Some(group_id))
            .collect();
        records.sort_by_key(|r| r.id.0);
        records
    }

    /// グループ集約情報を取得
    pub fn group_summary(&self, group_id: &str) -> Option<JobGroupSummary> {
        let records = self.list_by_group(group_id);
        if records.is_empty() {
            return None;
        }

        let mut queued = 0usize;
        let mut running = 0usize;
        let mut needs_recompute = 0usize;
        let mut succeeded = 0usize;
        let mut failed = 0usize;
        let mut canceled = 0usize;

        for record in &records {
            match record.status {
                JobStatus::Queued => queued += 1,
                JobStatus::Running => running += 1,
                JobStatus::NeedsRecompute => needs_recompute += 1,
                JobStatus::Succeeded => succeeded += 1,
                JobStatus::Failed => failed += 1,
                JobStatus::Canceled => canceled += 1,
            }
        }

        let total = records.len();
        let status = if failed > 0 {
            JobStatus::Failed
        } else if running > 0 {
            JobStatus::Running
        } else if needs_recompute > 0 {
            JobStatus::NeedsRecompute
        } else if queued > 0 {
            JobStatus::Queued
        } else if canceled == total {
            JobStatus::Canceled
        } else {
            JobStatus::Succeeded
        };

        let terminal = succeeded + failed + canceled;
        let progress = terminal as f32 / total as f32;

        Some(JobGroupSummary {
            group_id: group_id.to_string(),
            total,
            queued,
            running,
            succeeded,
            failed,
            canceled,
            status,
            progress,
        })
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
        artifact_bytes: Option<Vec<u8>>,
        log_ref: Option<String>,
    ) -> Result<(), JobError> {
        self.transition(id, JobStatus::Succeeded)?;

        if let Some(ref result_ref_value) = result_ref {
            self.invalidate_active_output(id, JobOutputValidity::Superseded);

            if let Some(record) = self.jobs.get_mut(&id) {
                record.output_history.push(JobOutputRecord {
                    result_ref: result_ref_value.clone(),
                    artifact_bytes,
                    log_ref: log_ref.clone(),
                    produced_at: SystemTime::now(),
                    validity: JobOutputValidity::Active,
                });
            }
        }

        self.event_queue.push(JobEvent::Completed {
            job_id: id,
            status: JobStatus::Succeeded,
            result_ref: result_ref.clone(),
            log_ref: log_ref.clone(),
        });

        self.supersede_parent_output_on_child_success(id);

        let is_rerun = self.jobs.get(&id).map(|r| r.attempts > 0).unwrap_or(false);
        if is_rerun {
            self.mark_descendants_needs_recompute(id);
        }

        self.emit_group_progress_if_needed(id);
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
        self.event_queue.push(JobEvent::Completed {
            job_id: id,
            status: JobStatus::Failed,
            result_ref: None,
            log_ref,
        });
        self.emit_group_progress_if_needed(id);
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
        self.emit_group_progress_if_needed(id);
        Ok(())
    }

    /// 終端状態ジョブを再実行待ちへ戻す
    pub fn rerun(&mut self, id: JobId) -> Result<(), JobError> {
        let record = self.jobs.get_mut(&id).ok_or(JobError::JobNotFound(id))?;

        if !record.status.is_terminal() && record.status != JobStatus::NeedsRecompute {
            return Err(JobError::RerunNotAllowed(record.status));
        }

        let from = record.status;
        record.status = JobStatus::Queued;
        record.attempts += 1;
        record.updated_at = SystemTime::now();
        record.last_error = None;

        self.event_queue.push(JobEvent::StatusChanged {
            job_id: id,
            from,
            to: JobStatus::Queued,
        });

        self.emit_group_progress_if_needed(id);
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

    fn emit_group_progress_if_needed(&mut self, id: JobId) {
        let group_id = self.jobs.get(&id).and_then(|r| r.group_id.clone());
        if let Some(group_id) = group_id
            && let Some(summary) = self.group_summary(&group_id)
        {
            self.event_queue.push(JobEvent::GroupProgressUpdated {
                group_id,
                status: summary.status,
                progress: summary.progress,
                total: summary.total,
            });
        }
    }

    fn invalidate_active_output(&mut self, id: JobId, validity: JobOutputValidity) {
        let mut changed_results = Vec::new();

        if let Some(record) = self.jobs.get_mut(&id) {
            for output in &mut record.output_history {
                if output.validity == JobOutputValidity::Active {
                    output.validity = validity;
                    changed_results.push(output.result_ref.clone());
                }
            }
        }

        for result_ref in changed_results {
            self.event_queue.push(JobEvent::OutputValidityChanged {
                job_id: id,
                result_ref,
                validity,
            });
        }
    }

    fn supersede_parent_output_on_child_success(&mut self, id: JobId) {
        let parent_id = self.jobs.get(&id).and_then(|r| r.parent_job_id);
        if let Some(parent_id) = parent_id {
            self.invalidate_active_output(parent_id, JobOutputValidity::Superseded);
        }
    }

    fn collect_descendants(&self, root: JobId) -> Vec<JobId> {
        let mut descendants = Vec::new();
        let mut stack = vec![root];

        while let Some(current) = stack.pop() {
            for child in self
                .jobs
                .values()
                .filter(|r| r.parent_job_id == Some(current))
                .map(|r| r.id)
            {
                descendants.push(child);
                stack.push(child);
            }
        }

        descendants
    }

    fn mark_descendants_needs_recompute(&mut self, root: JobId) {
        let descendants = self.collect_descendants(root);

        for descendant_id in descendants {
            self.invalidate_active_output(
                descendant_id,
                JobOutputValidity::InvalidatedByDependency,
            );

            let mut should_emit_status = false;
            if let Some(record) = self.jobs.get_mut(&descendant_id)
                && record.status != JobStatus::NeedsRecompute
            {
                record.status = JobStatus::NeedsRecompute;
                record.updated_at = SystemTime::now();
                should_emit_status = true;
            }

            if should_emit_status {
                self.event_queue.push(JobEvent::MarkedNeedsRecompute {
                    job_id: descendant_id,
                    by_job_id: root,
                });
            }

            self.emit_group_progress_if_needed(descendant_id);
        }
    }

    /// スタブ実行器でジョブを1件実行
    pub fn execute_with<E: JobExecutor>(
        &mut self,
        id: JobId,
        executor: &E,
    ) -> Result<(), JobError> {
        self.start(id)?;

        let snapshot = self.jobs.get(&id).ok_or(JobError::JobNotFound(id))?.clone();
        let input_artifact_bytes = match snapshot.parent_job_id {
            Some(parent_job_id) => self.latest_input_result_bytes(parent_job_id)?,
            None => None,
        };
        let execution = executor.execute(&snapshot, input_artifact_bytes.as_deref());

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
                self.mark_succeeded(
                    id,
                    execution.result_ref,
                    execution.artifact_bytes,
                    execution.log_ref,
                )
            }
            JobStatus::Failed => self.mark_failed(
                id,
                execution
                    .error
                    .unwrap_or_else(|| "executor returned failed status".to_string()),
                execution.log_ref,
            ),
            JobStatus::Canceled => self.cancel(id),
            JobStatus::Queued | JobStatus::Running | JobStatus::NeedsRecompute => {
                Err(JobError::InvalidTransition {
                    from: JobStatus::Running,
                    to: execution.status,
                })
            }
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
                | (JobStatus::NeedsRecompute, JobStatus::Running)
                | (JobStatus::Queued, JobStatus::Canceled)
                | (JobStatus::NeedsRecompute, JobStatus::Canceled)
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

        self.emit_group_progress_if_needed(id);

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
            job_type: JobType::CamProcess,
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
            job_type: JobType::CuttingSimulation,
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
        fn execute(
            &self,
            job: &JobRecord,
            _input_artifact_bytes: Option<&[u8]>,
        ) -> JobExecutionResult {
            let result_ref = match job.spec.job_type {
                JobType::CamProcess => Some("result://cam".to_string()),
                JobType::CuttingSimulation => Some("result://sim".to_string()),
                JobType::NcPostFromCam => Some("result://nc-post".to_string()),
            };

            if self.succeed {
                JobExecutionResult {
                    status: JobStatus::Succeeded,
                    elapsed_millis: self.elapsed_millis,
                    result_ref,
                    artifact_bytes: None,
                    log_ref: Some("log://ok".to_string()),
                    error: None,
                }
            } else {
                JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: self.elapsed_millis,
                    result_ref: None,
                    artifact_bytes: None,
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
            .mark_succeeded(id, Some("result://ok".to_string()), None, None)
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
        assert_eq!(
            manager.active_result_ref(cam_id).unwrap(),
            Some("result://cam")
        );
        assert_eq!(
            manager.active_result_ref(sim_id).unwrap(),
            Some("result://sim")
        );
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

    #[test]
    fn related_jobs_can_be_registered_and_listed() {
        let mut manager = JobManager::new();
        let parent = manager.submit(sample_spec());

        let child = manager
            .submit_with_relation(
                simulation_spec(),
                JobRelation {
                    parent_job_id: Some(parent),
                    group_id: Some("g1".to_string()),
                },
            )
            .unwrap();

        let by_parent = manager.list_by_parent(parent);
        let by_group = manager.list_by_group("g1");

        assert_eq!(by_parent.len(), 1);
        assert_eq!(by_parent[0].id, child);
        assert_eq!(by_group.len(), 1);
        assert_eq!(by_group[0].id, child);
    }

    #[test]
    fn submit_with_unknown_parent_fails() {
        let mut manager = JobManager::new();
        let result = manager.submit_with_relation(
            sample_spec(),
            JobRelation {
                parent_job_id: Some(JobId(999)),
                group_id: None,
            },
        );

        assert!(matches!(
            result,
            Err(JobError::ParentJobNotFound(JobId(999)))
        ));
    }

    #[test]
    fn group_summary_returns_aggregate_status_and_progress() {
        let mut manager = JobManager::new();
        let id1 = manager
            .submit_with_relation(
                sample_spec(),
                JobRelation {
                    parent_job_id: None,
                    group_id: Some("g2".to_string()),
                },
            )
            .unwrap();
        let id2 = manager
            .submit_with_relation(
                simulation_spec(),
                JobRelation {
                    parent_job_id: None,
                    group_id: Some("g2".to_string()),
                },
            )
            .unwrap();

        manager.start(id1).unwrap();
        manager
            .mark_succeeded(id1, Some("result://ok".to_string()), None, None)
            .unwrap();

        let summary = manager.group_summary("g2").unwrap();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.succeeded, 1);
        assert_eq!(summary.queued, 1);
        assert_eq!(summary.status, JobStatus::Queued);
        assert!((summary.progress - 0.5).abs() < f32::EPSILON);

        manager.start(id2).unwrap();
        manager
            .mark_failed(id2, "ng".to_string(), Some("log://ng".to_string()))
            .unwrap();

        let summary2 = manager.group_summary("g2").unwrap();
        assert_eq!(summary2.failed, 1);
        assert_eq!(summary2.status, JobStatus::Failed);
        assert!((summary2.progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn group_progress_event_is_emitted() {
        let mut manager = JobManager::new();
        let id = manager
            .submit_with_relation(
                sample_spec(),
                JobRelation {
                    parent_job_id: None,
                    group_id: Some("g3".to_string()),
                },
            )
            .unwrap();

        manager.start(id).unwrap();
        manager
            .mark_succeeded(id, Some("result://ok".to_string()), None, None)
            .unwrap();

        let events = manager.take_events();
        assert!(events.iter().any(|e| {
            matches!(
                e,
                JobEvent::GroupProgressUpdated {
                    group_id,
                    status,
                    total,
                    ..
                } if group_id == "g3" && *status == JobStatus::Succeeded && *total == 1
            )
        }));
    }

    #[test]
    fn child_success_supersedes_parent_active_output() {
        let mut manager = JobManager::new();
        let parent = manager.submit(sample_spec());
        let child = manager
            .submit_with_relation(
                simulation_spec(),
                JobRelation {
                    parent_job_id: Some(parent),
                    group_id: None,
                },
            )
            .unwrap();

        manager.start(parent).unwrap();
        manager
            .mark_succeeded(parent, Some("result://parent/v1".to_string()), None, None)
            .unwrap();
        assert_eq!(
            manager.active_result_ref(parent).unwrap(),
            Some("result://parent/v1")
        );

        manager.start(child).unwrap();
        manager
            .mark_succeeded(child, Some("result://child/v1".to_string()), None, None)
            .unwrap();

        assert_eq!(manager.active_result_ref(parent).unwrap(), None);
        let parent_history = manager.output_history(parent).unwrap();
        assert_eq!(parent_history.len(), 1);
        assert_eq!(parent_history[0].validity, JobOutputValidity::Superseded);
    }

    #[test]
    fn rerun_success_marks_descendants_needs_recompute() {
        let mut manager = JobManager::new();
        let root = manager.submit(sample_spec());
        let child = manager
            .submit_with_relation(
                simulation_spec(),
                JobRelation {
                    parent_job_id: Some(root),
                    group_id: None,
                },
            )
            .unwrap();

        manager.start(root).unwrap();
        manager
            .mark_succeeded(root, Some("result://root/v1".to_string()), None, None)
            .unwrap();

        manager.start(child).unwrap();
        manager
            .mark_succeeded(child, Some("result://child/v1".to_string()), None, None)
            .unwrap();
        assert_eq!(
            manager.active_result_ref(child).unwrap(),
            Some("result://child/v1")
        );

        manager.rerun(root).unwrap();
        manager.start(root).unwrap();
        manager
            .mark_succeeded(root, Some("result://root/v2".to_string()), None, None)
            .unwrap();

        assert_eq!(manager.status(child).unwrap(), JobStatus::NeedsRecompute);
        assert_eq!(manager.active_result_ref(child).unwrap(), None);

        let child_history = manager.output_history(child).unwrap();
        assert_eq!(child_history.len(), 1);
        assert_eq!(
            child_history[0].validity,
            JobOutputValidity::InvalidatedByDependency
        );
    }
}
