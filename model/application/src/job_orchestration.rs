//! Job向け orchestration 境界。
//!
//! 投入はジョブをキューに入れた時点で完了し、実行しない。実行はジョブマネージャーの入口
//! （`run_queued_jobs`）が行い、結果は正常終了したジョブから取り込む。

use cam_sim::{CamJobExecutorAdapter, CamWorkflowError, CamWorkflowSubmitter};
use job_domain::{CamJobEvent, CamJobRecord, CamJobStatus, CamJobType};
use job_runtime::{
    JobError, JobId, JobManager, JobOutputValidity, JobSpec, JobStatus, JobType, RetryPolicy,
};

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
    /// 有効な成果物のバイト列（成果物を返さないジョブ・成果物がない場合は `None`）
    pub active_artifact: Option<Vec<u8>>,
    /// 入力に使う親のジョブの成果物のバイト列（親がない・親に成果物がない場合は `None`）
    pub input_artifact: Option<Vec<u8>>,
}

/// ViewModel 向けイベント batch。
#[derive(Debug, Clone, PartialEq)]
pub struct JobWorkflowEventBatch {
    pub events: Vec<CamJobEvent>,
}

/// job 実行ユースケースの orchestration port。
pub trait JobWorkflowOrchestration {
    /// ジョブをキューに入れる（実行しない。親のジョブの完了を待たずに子を投入できる）
    fn submit_workflow(
        &mut self,
        request: JobWorkflowSubmitRequest,
    ) -> Result<JobWorkflowSubmitResult, JobOrchestrationError>;

    /// 実行できる待機中のジョブがなくなるまで実行し、実行したジョブの数を返す（ジョブマネージャーの実行の入口）
    ///
    /// 親が失敗・キャンセルで終わった待機中のジョブは、実行せずに失敗にする。
    fn run_queued_jobs(&mut self) -> Result<usize, JobOrchestrationError>;

    /// 種類が `job_type` で正常終了したジョブ（ID の昇順）
    fn list_succeeded_jobs(&self, job_type: CamJobType) -> Vec<CamJobRecord>;

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

        Ok(JobWorkflowSubmitResult {
            job_id: job_id.0,
            status: self.manager.status(job_id)?.into(),
        })
    }

    fn run_queued_jobs(&mut self) -> Result<usize, JobOrchestrationError> {
        let mut executed = 0;
        while self.manager.execute_next_ready(&self.executor)?.is_some() {
            executed += 1;
        }
        Ok(executed)
    }

    fn list_succeeded_jobs(&self, job_type: CamJobType) -> Vec<CamJobRecord> {
        let job_type = map_job_type(job_type);
        self.manager
            .list()
            .into_iter()
            .filter(|record| {
                record.status == JobStatus::Succeeded && record.spec.job_type == job_type
            })
            .map(CamJobRecord::from)
            .collect()
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
        let active_artifact = self
            .manager
            .output_history(job_id)?
            .into_iter()
            .find(|output| output.validity == JobOutputValidity::Active)
            .and_then(|output| output.artifact_bytes.clone());
        let input_artifact = self
            .manager
            .input_artifact_bytes(job_id)?
            .map(<[u8]>::to_vec);
        let job = CamJobRecord::from(self.manager.get(job_id)?);

        Ok(JobWorkflowResult {
            job,
            active_result_ref,
            active_artifact,
            input_artifact,
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
    use geo_contracts::TriangleMesh3DConstructor;
    use job_domain::{CamJobEvent, CamJobStatus, CamJobType};

    use super::{
        JobWorkflowOrchestration, JobWorkflowOrchestrator, JobWorkflowResultQuery,
        JobWorkflowStatusQuery, JobWorkflowSubmitRequest,
    };

    /// 待機中のジョブを実行し、ジョブ `job_id` の状態を返す
    fn run_and_query_status(
        orchestrator: &mut JobWorkflowOrchestrator,
        job_id: u64,
    ) -> CamJobStatus {
        orchestrator.run_queued_jobs().unwrap();
        orchestrator
            .query_status(JobWorkflowStatusQuery { job_id })
            .unwrap()
            .status
    }

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
        assert_eq!(submit.status, CamJobStatus::Queued);
        assert_eq!(
            run_and_query_status(&mut orchestrator, submit.job_id),
            CamJobStatus::Succeeded
        );

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

        assert_eq!(submit.status, CamJobStatus::Queued);
        assert_eq!(
            run_and_query_status(&mut orchestrator, submit.job_id),
            CamJobStatus::Failed
        );

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
    fn submit_failure_no_solution_is_visible_through_result_query() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/no-solution".to_string(),
                parent_job_id: None,
            })
            .expect("submit should complete with failed runtime status");

        assert_eq!(submit.status, CamJobStatus::Queued);
        assert_eq!(
            run_and_query_status(&mut orchestrator, submit.job_id),
            CamJobStatus::Failed
        );

        let result = orchestrator
            .query_result(JobWorkflowResultQuery {
                job_id: submit.job_id,
            })
            .expect("failed job should still be queryable");
        assert_eq!(result.job.status, CamJobStatus::Failed);
        assert!(
            result
                .job
                .last_error
                .as_deref()
                .is_some_and(|message| message.contains("no_solution"))
        );
    }

    #[test]
    fn submit_failure_convergence_failure_is_visible_through_result_query() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/convergence-failure".to_string(),
                parent_job_id: None,
            })
            .expect("submit should complete with failed runtime status");

        assert_eq!(submit.status, CamJobStatus::Queued);
        assert_eq!(
            run_and_query_status(&mut orchestrator, submit.job_id),
            CamJobStatus::Failed
        );

        let result = orchestrator
            .query_result(JobWorkflowResultQuery {
                job_id: submit.job_id,
            })
            .expect("failed job should still be queryable");
        assert_eq!(result.job.status, CamJobStatus::Failed);
        assert!(
            result
                .job
                .last_error
                .as_deref()
                .is_some_and(|message| message.contains("convergence_failure"))
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
        orchestrator.run_queued_jobs().unwrap();

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
        use cam_core::Tool;
        use cam_sim::{
            CamJobExecutorAdapter, CuttingSimulationInput, InMemoryCuttingSimulationInputStore,
            SnapshotInterval,
        };
        use geo_algorithms::{Aabb3D, Point3D};
        use job_runtime::JobManager;
        use std::sync::Arc;

        let simulation_inputs = InMemoryCuttingSimulationInputStore::new();
        simulation_inputs
            .register(
                "input://sim/sample",
                CuttingSimulationInput {
                    stock: Aabb3D::new(Point3D::new(-5.0, -5.0, -5.0), Point3D::new(5.0, 5.0, 1.0)),
                    tool: Tool::flat_end_mill("nc-post-src".to_string(), 2.0, 10.0),
                    precision: 0.5,
                    snapshot_interval: SnapshotInterval::default(),
                },
            )
            .unwrap();
        let mut orchestrator = JobWorkflowOrchestrator::with_runtime(
            JobManager::new(),
            CamJobExecutorAdapter::default()
                .with_simulation_input_provider(Arc::new(simulation_inputs)),
        );

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

        assert_eq!(sim_submit.status, CamJobStatus::Queued);
        assert_eq!(
            run_and_query_status(&mut orchestrator, sim_submit.job_id),
            CamJobStatus::Succeeded
        );

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
        // 有効な成果物は切削シミュレーションの成果物として読める
        let artifact = result
            .active_artifact
            .expect("simulation returns an artifact");
        let (_, artifact) =
            cam_sim::read_cutting_simulation_artifact_v1(&mut artifact.as_slice()).unwrap();
        assert!(!artifact.snapshots.is_empty());

        let batch = orchestrator.take_event_batch();
        assert!(batch.events.iter().any(|event| {
            matches!(
                event,
                CamJobEvent::Completed {
                    job_id,
                    status: CamJobStatus::Succeeded,
                    log_ref: Some(log_ref),
                    ..
                } if *job_id == sim_submit.job_id && log_ref.contains("/ok/snapshots-")
            )
        }));
    }

    #[test]
    fn registered_nurbs_input_reports_real_convergence_failure() {
        use std::sync::Arc;

        use cam_algorithms::{CamSolverInput, OperationSpec, ScanlineParams, SolverGeometry};
        use cam_core::{CoordinateFrame, LengthUnit, Tool};
        use cam_sim::{CamJobExecutorAdapter, InMemoryCamSolverInputStore};
        use geo_algorithms::TessellationLimits;
        use geo_algorithms::{NurbsSurface3D, Point3D};
        use geo_contracts::NurbsSurface3DConstructor;
        use job_runtime::JobManager;

        // 曲面の弦誤差が収束しない設定（極小許容値・反復上限 1）で solver 実計算を失敗させる
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let dome = NurbsSurface3D::new(
            2,
            2,
            knots.clone(),
            knots,
            &[
                vec![
                    Point3D::new(0.0, 0.0, 80.0),
                    Point3D::new(0.0, 20.0, 80.0),
                    Point3D::new(0.0, 40.0, 80.0),
                ],
                vec![
                    Point3D::new(20.0, 0.0, 80.0),
                    Point3D::new(20.0, 20.0, 100.0),
                    Point3D::new(20.0, 40.0, 80.0),
                ],
                vec![
                    Point3D::new(40.0, 0.0, 80.0),
                    Point3D::new(40.0, 20.0, 80.0),
                    Point3D::new(40.0, 40.0, 80.0),
                ],
            ],
            None,
        )
        .unwrap();
        let store = InMemoryCamSolverInputStore::new();
        store
            .register(
                "input://cam/dome-unreachable-tolerance",
                CamSolverInput {
                    operation_id: "op-scan-1".to_string(),
                    tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
                    geometry: SolverGeometry::NurbsSurfaceSet(vec![dome]),
                    operation: OperationSpec::Scanline(ScanlineParams {
                        stepover: 2.0,
                        sample_pitch: 1.0,
                        feed_rate: 1200.0,
                        clearance_height: 5.0,
                        tool_range: cam_algorithms::ScanlineToolRange::default(),
                    }),
                    units: LengthUnit::Millimeter,
                    coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
                    chord_tolerance: 1e-9,
                    tessellation_limits: TessellationLimits {
                        max_subdivisions: 2,
                        max_refinement_iterations: 1,
                        max_vertices_per_surface: 1_000_000,
                    },
                    boundary: None,
                },
            )
            .unwrap();
        let mut orchestrator = JobWorkflowOrchestrator::with_runtime(
            JobManager::new(),
            CamJobExecutorAdapter::with_input_provider(Arc::new(store)),
        );

        let submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/dome-unreachable-tolerance".to_string(),
                parent_job_id: None,
            })
            .expect("submit should complete with failed runtime status");
        assert_eq!(submit.status, CamJobStatus::Queued);
        assert_eq!(
            run_and_query_status(&mut orchestrator, submit.job_id),
            CamJobStatus::Failed
        );

        let result = orchestrator
            .query_result(JobWorkflowResultQuery {
                job_id: submit.job_id,
            })
            .expect("failed job should still be queryable");
        assert!(
            result
                .job
                .last_error
                .as_deref()
                .is_some_and(|message| message.contains("convergence_failure"))
        );
    }

    #[test]
    fn registered_solver_input_flows_from_cam_process_to_cutting_simulation() {
        use std::sync::Arc;

        use cam_algorithms::{CamSolverInput, OperationSpec, ScanlineParams, SolverGeometry};
        use cam_core::{CoordinateFrame, LengthUnit, Tool};
        use cam_sim::{
            CamJobExecutorAdapter, CuttingSimulationInput, InMemoryCamSolverInputStore,
            InMemoryCuttingSimulationInputStore, SnapshotInterval,
        };
        use geo_algorithms::TessellationLimits;
        use geo_algorithms::{Aabb3D, Point3D, TriangleMesh3D};
        use job_runtime::JobManager;

        let pyramid = TriangleMesh3D::new(
            vec![
                Point3D::new(20.0, 20.0, 80.0),
                Point3D::new(60.0, 20.0, 80.0),
                Point3D::new(60.0, 60.0, 80.0),
                Point3D::new(20.0, 60.0, 80.0),
                Point3D::new(40.0, 40.0, 90.0),
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        )
        .unwrap();
        let tool = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
        let simulation_inputs = InMemoryCuttingSimulationInputStore::new();
        simulation_inputs
            .register(
                "input://sim/pyramid",
                CuttingSimulationInput {
                    stock: Aabb3D::new(
                        Point3D::new(0.0, 0.0, 0.0),
                        Point3D::new(100.0, 100.0, 100.0),
                    ),
                    tool: tool.clone(),
                    precision: 6.25,
                    snapshot_interval: SnapshotInterval::default(),
                },
            )
            .unwrap();
        let store = InMemoryCamSolverInputStore::new();
        store
            .register(
                "input://cam/pyramid",
                CamSolverInput {
                    operation_id: "op-scan-1".to_string(),
                    tool,
                    geometry: SolverGeometry::TriangleMesh(pyramid),
                    operation: OperationSpec::Scanline(ScanlineParams {
                        stepover: 4.0,
                        sample_pitch: 2.0,
                        feed_rate: 1200.0,
                        clearance_height: 5.0,
                        tool_range: cam_algorithms::ScanlineToolRange::default(),
                    }),
                    units: LengthUnit::Millimeter,
                    coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
                    chord_tolerance: 0.01,
                    tessellation_limits: TessellationLimits::default(),
                    boundary: None,
                },
            )
            .unwrap();
        let mut orchestrator = JobWorkflowOrchestrator::with_runtime(
            JobManager::new(),
            CamJobExecutorAdapter::with_input_provider(Arc::new(store))
                .with_simulation_input_provider(Arc::new(simulation_inputs)),
        );

        let cam_submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/pyramid".to_string(),
                parent_job_id: None,
            })
            .expect("cam submit should succeed");
        assert_eq!(cam_submit.status, CamJobStatus::Queued);

        let sim_submit = orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CuttingSimulation,
                input_ref: "input://sim/pyramid".to_string(),
                parent_job_id: Some(cam_submit.job_id),
            })
            .expect("cutting simulation submit should succeed");
        assert_eq!(sim_submit.status, CamJobStatus::Queued);
        assert_eq!(
            run_and_query_status(&mut orchestrator, sim_submit.job_id),
            CamJobStatus::Succeeded
        );

        let result = orchestrator
            .query_result(JobWorkflowResultQuery {
                job_id: sim_submit.job_id,
            })
            .expect("result query should succeed");
        assert_eq!(
            result.active_result_ref,
            Some(format!("result://sim/{}/ok", sim_submit.job_id))
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

    #[test]
    fn only_succeeded_jobs_of_the_type_are_listed_for_import() {
        let mut orchestrator = JobWorkflowOrchestrator::new();
        let submit = |orchestrator: &mut JobWorkflowOrchestrator, input_ref: &str| {
            orchestrator
                .submit_workflow(JobWorkflowSubmitRequest {
                    job_type: CamJobType::CamProcess,
                    input_ref: input_ref.to_string(),
                    parent_job_id: None,
                })
                .unwrap()
                .job_id
        };
        let succeeded = submit(&mut orchestrator, "input://cam/sample");
        let failed = submit(&mut orchestrator, "input://invalid");
        assert!(
            orchestrator
                .list_succeeded_jobs(CamJobType::CamProcess)
                .is_empty()
        );

        assert_eq!(orchestrator.run_queued_jobs().unwrap(), 2);
        let listed: Vec<u64> = orchestrator
            .list_succeeded_jobs(CamJobType::CamProcess)
            .iter()
            .map(|job| job.id)
            .collect();
        assert_eq!(listed, vec![succeeded]);
        assert!(!listed.contains(&failed));
        assert!(
            orchestrator
                .list_succeeded_jobs(CamJobType::CuttingSimulation)
                .is_empty()
        );
    }
}
