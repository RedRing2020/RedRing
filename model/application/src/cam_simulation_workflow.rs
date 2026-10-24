//! ツールパス生成から切削シミュレーションまでのジョブの投入と、結果の取り込みを行うユースケース。
//!
//! 投入は、ツールパス生成の入力と切削シミュレーションの入力を登録先に登録し、ツールパス生成（CamProcess）の
//! ジョブと、それを親とする切削シミュレーション（CuttingSimulation）のジョブをキューに入れた時点で完了する。
//! 取り込みは、正常終了した切削シミュレーションのジョブだけを対象に、ツールパスと切削シミュレーションの成果物を返す。

use std::fmt;
use std::sync::Arc;

use cam_algorithms::CamSolverInput;
use cam_sim::{
    CamJobExecutorAdapter, CuttingSimulationInput, InMemoryCamSolverInputStore,
    InMemoryCuttingSimulationInputStore, SnapshotInterval,
};
use geo_algorithms::Aabb3D;
use job_domain::{CamJobRecord, CamJobStatus, CamJobType};
use job_runtime::JobManager;

use crate::job_orchestration::{
    JobOrchestrationError, JobWorkflowOrchestration, JobWorkflowOrchestrator,
    JobWorkflowResultQuery, JobWorkflowSubmitRequest,
};

/// ツールパス生成から切削シミュレーションまでの入力
#[derive(Debug, Clone)]
pub struct CamSimulationWorkflowRequest {
    /// ツールパス生成の入力（工具は切削シミュレーションでも使う）
    pub solver_input: CamSolverInput,
    /// 素材（直方体）
    pub stock: Aabb3D<f64>,
    /// 素材の形状の許容誤差
    pub precision: f64,
    /// スナップショットの記録の間隔
    pub snapshot_interval: SnapshotInterval,
}

/// 投入したジョブの番号
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CamSimulationSubmission {
    /// ツールパス生成のジョブ
    pub toolpath_job_id: u64,
    /// 切削シミュレーションのジョブ（ツールパス生成のジョブの子）
    pub simulation_job_id: u64,
}

/// 取り込んだツールパス生成と切削シミュレーションの成果物
#[derive(Debug, Clone, PartialEq)]
pub struct CamSimulationWorkflowResult {
    /// ツールパスの成果物のバイト列
    pub toolpath_artifact: Vec<u8>,
    /// 切削シミュレーションの成果物のバイト列
    pub simulation_artifact: Vec<u8>,
}

/// 投入・取り込みの失敗
#[derive(Debug, Clone, PartialEq)]
pub enum CamSimulationWorkflowError {
    /// 入力の登録の失敗（理由）
    InputRegistration(String),
    /// 正常終了した切削シミュレーションのジョブではない（ジョブの番号）
    NotSucceededSimulation(u64),
    /// 正常終了した切削シミュレーションのジョブに、成果物または入力のツールパスがない（ジョブの番号）
    MissingArtifact(u64),
    /// ジョブの投入・問い合わせの失敗
    Orchestration(JobOrchestrationError),
}

impl fmt::Display for CamSimulationWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputRegistration(reason) => write!(f, "input registration failed: {reason}"),
            Self::NotSucceededSimulation(job_id) => {
                write!(f, "job {job_id} is not a succeeded cutting simulation")
            }
            Self::MissingArtifact(job_id) => write!(f, "job {job_id} has no artifact"),
            Self::Orchestration(error) => write!(f, "job orchestration failed: {error}"),
        }
    }
}

impl std::error::Error for CamSimulationWorkflowError {}

impl From<JobOrchestrationError> for CamSimulationWorkflowError {
    fn from(value: JobOrchestrationError) -> Self {
        Self::Orchestration(value)
    }
}

/// ツールパス生成から切削シミュレーションまでのジョブの投入と、結果の取り込み
#[derive(Debug)]
pub struct CamSimulationWorkflow {
    orchestrator: JobWorkflowOrchestrator,
    solver_inputs: Arc<InMemoryCamSolverInputStore>,
    simulation_inputs: Arc<InMemoryCuttingSimulationInputStore>,
    /// 入力の参照の名前に使う通し番号
    next_run: u64,
}

impl Default for CamSimulationWorkflow {
    fn default() -> Self {
        let solver_inputs = Arc::new(InMemoryCamSolverInputStore::new());
        let simulation_inputs = Arc::new(InMemoryCuttingSimulationInputStore::new());
        let executor = CamJobExecutorAdapter::with_input_provider(solver_inputs.clone())
            .with_simulation_input_provider(simulation_inputs.clone());
        Self {
            orchestrator: JobWorkflowOrchestrator::with_runtime(JobManager::new(), executor),
            solver_inputs,
            simulation_inputs,
            next_run: 0,
        }
    }
}

impl CamSimulationWorkflow {
    pub fn new() -> Self {
        Self::default()
    }

    /// 入力を登録し、ツールパス生成と切削シミュレーションのジョブをキューに入れる（実行しない）
    ///
    /// # Errors
    ///
    /// 入力の登録の失敗、ジョブの投入の失敗。
    pub fn submit(
        &mut self,
        request: CamSimulationWorkflowRequest,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        self.next_run += 1;
        let cam_input_ref = format!("input://cam/run-{}", self.next_run);
        let simulation_input_ref = format!("input://sim/run-{}", self.next_run);
        let tool = request.solver_input.tool.clone();
        self.solver_inputs
            .register(&cam_input_ref, request.solver_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        self.simulation_inputs
            .register(
                &simulation_input_ref,
                CuttingSimulationInput {
                    stock: request.stock,
                    tool,
                    precision: request.precision,
                    snapshot_interval: request.snapshot_interval,
                },
            )
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;

        let toolpath = self
            .orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: cam_input_ref,
                parent_job_id: None,
            })?;
        let simulation = self
            .orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CuttingSimulation,
                input_ref: simulation_input_ref,
                parent_job_id: Some(toolpath.job_id),
            })?;
        Ok(CamSimulationSubmission {
            toolpath_job_id: toolpath.job_id,
            simulation_job_id: simulation.job_id,
        })
    }

    /// 実行できる待機中のジョブがなくなるまで実行し、実行したジョブの数を返す（ジョブマネージャーの実行の入口）
    ///
    /// # Errors
    ///
    /// ジョブの状態の更新の失敗。
    pub fn run_queued_jobs(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        Ok(self.orchestrator.run_queued_jobs()?)
    }

    /// 取り込める（正常終了した）切削シミュレーションのジョブ（ID の昇順）
    pub fn succeeded_simulations(&self) -> Vec<CamJobRecord> {
        self.orchestrator
            .list_succeeded_jobs(CamJobType::CuttingSimulation)
    }

    /// 正常終了した切削シミュレーションのジョブ `simulation_job_id` の、ツールパスと切削シミュレーションの成果物
    ///
    /// # Errors
    ///
    /// ジョブが正常終了した切削シミュレーションでない場合、成果物がない場合、問い合わせの失敗。
    pub fn import(
        &self,
        simulation_job_id: u64,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError> {
        let result = self.orchestrator.query_result(JobWorkflowResultQuery {
            job_id: simulation_job_id,
        })?;
        if result.job.status != CamJobStatus::Succeeded
            || result.job.spec.job_type != CamJobType::CuttingSimulation
        {
            return Err(CamSimulationWorkflowError::NotSucceededSimulation(
                simulation_job_id,
            ));
        }
        let missing = || CamSimulationWorkflowError::MissingArtifact(simulation_job_id);
        Ok(CamSimulationWorkflowResult {
            // 切削シミュレーションの入力に使ったツールパス（親のジョブの成果物）
            toolpath_artifact: result.input_artifact.ok_or_else(missing)?,
            simulation_artifact: result.active_artifact.ok_or_else(missing)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use cam_algorithms::{OperationSpec, ScanlineParams, SolverGeometry};
    use cam_core::{CoordinateFrame, LengthUnit, Tool, read_toolpath_artifact_v1};
    use geo_algorithms::TessellationLimits;
    use geo_algorithms::{Point3D, TriangleMesh3D, TriangleMesh3DConstructor};

    use super::*;

    /// 底面 z = 80・頂点 z = 90 の四角錐を、直径 6 のボールエンドミルで粗い走査線で加工する入力
    fn request(precision: f64) -> CamSimulationWorkflowRequest {
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
        CamSimulationWorkflowRequest {
            solver_input: CamSolverInput {
                operation_id: "op-scan-1".to_string(),
                tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
                geometry: SolverGeometry::TriangleMesh(pyramid),
                operation: OperationSpec::Scanline(ScanlineParams {
                    stepover: 6.0,
                    sample_pitch: 10.0,
                    feed_rate: 1200.0,
                    clearance_height: 5.0,
                }),
                units: LengthUnit::Millimeter,
                coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
                chord_tolerance: 0.01,
                tessellation_limits: TessellationLimits::default(),
                boundary: None,
            },
            stock: Aabb3D::new(
                Point3D::new(10.0, 10.0, 75.0),
                Point3D::new(70.0, 70.0, 92.0),
            ),
            precision,
            snapshot_interval: SnapshotInterval::default(),
        }
    }

    #[test]
    fn submitting_only_enqueues_the_jobs_without_waiting_for_the_computation() {
        let mut workflow = CamSimulationWorkflow::new();
        let started = Instant::now();
        let submission = workflow.submit(request(0.01)).unwrap();
        let submitting = started.elapsed();
        // 投入しただけでは実行されず、取り込めない
        assert!(workflow.succeeded_simulations().is_empty());
        assert_eq!(
            workflow.import(submission.simulation_job_id),
            Err(CamSimulationWorkflowError::NotSucceededSimulation(
                submission.simulation_job_id
            ))
        );

        let started = Instant::now();
        assert_eq!(workflow.run_queued_jobs().unwrap(), 2);
        let running = started.elapsed();
        assert!(
            submitting * 10 < running,
            "submitting {submitting:?}, running {running:?}"
        );
    }

    #[test]
    fn succeeded_simulations_are_imported_with_their_toolpaths() {
        let mut workflow = CamSimulationWorkflow::new();
        let submissions: Vec<CamSimulationSubmission> = (0..2)
            .map(|_| workflow.submit(request(0.01)).unwrap())
            .collect();
        assert_eq!(workflow.run_queued_jobs().unwrap(), 4);
        let succeeded: Vec<u64> = workflow
            .succeeded_simulations()
            .iter()
            .map(|job| job.id)
            .collect();
        assert_eq!(
            succeeded,
            submissions
                .iter()
                .map(|submission| submission.simulation_job_id)
                .collect::<Vec<_>>()
        );
        for submission in submissions {
            let result = workflow.import(submission.simulation_job_id).unwrap();
            let (_, toolpath) =
                read_toolpath_artifact_v1(&mut result.toolpath_artifact.as_slice()).unwrap();
            assert_eq!(toolpath.tool_id, "BEM6");
            let (_, simulation) = cam_sim::read_cutting_simulation_artifact_v1(
                &mut result.simulation_artifact.as_slice(),
            )
            .unwrap();
            assert!(simulation.snapshots.len() > 1);
            let first = simulation.snapshots[0].remaining_volume_mm3;
            let last = simulation.snapshots.last().unwrap().remaining_volume_mm3;
            assert!(last < first, "first = {first}, last = {last}");
        }
    }

    #[test]
    fn failed_jobs_are_not_imported() {
        let mut workflow = CamSimulationWorkflow::new();
        // ツールパス生成の失敗（子の切削シミュレーションは実行されずに失敗する）と、Preflight の拒否
        let mut invalid = request(0.01);
        invalid.solver_input.operation_id = String::new();
        let failed_toolpath = workflow.submit(invalid).unwrap();
        let rejected = workflow.submit(request(0.0)).unwrap();
        workflow.run_queued_jobs().unwrap();

        assert!(workflow.succeeded_simulations().is_empty());
        for submission in [failed_toolpath, rejected] {
            assert_eq!(
                workflow.import(submission.simulation_job_id),
                Err(CamSimulationWorkflowError::NotSucceededSimulation(
                    submission.simulation_job_id
                ))
            );
        }
        // ツールパス生成のジョブは切削シミュレーションとしては取り込めない
        assert_eq!(
            workflow.import(rejected.toolpath_job_id),
            Err(CamSimulationWorkflowError::NotSucceededSimulation(
                rejected.toolpath_job_id
            ))
        );
    }
}
