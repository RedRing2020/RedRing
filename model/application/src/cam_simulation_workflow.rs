//! ツールパス生成から切削シミュレーションまでを実行するユースケース。
//!
//! ツールパス生成の入力と切削シミュレーションの入力を登録先に登録し、ツールパス生成（CamProcess）の
//! ジョブと、それを親とする切削シミュレーション（CuttingSimulation）のジョブを投入して、両方の成果物を返す。
//! ジョブは投入と同時に実行して終わる。

use std::fmt;
use std::sync::Arc;

use cam_algorithms::CamSolverInput;
use cam_sim::{
    CamJobExecutorAdapter, CuttingSimulationInput, InMemoryCamSolverInputStore,
    InMemoryCuttingSimulationInputStore, SnapshotInterval,
};
use geo_algorithms::Aabb3D;
use job_domain::{CamJobStatus, CamJobType};
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

/// ツールパス生成と切削シミュレーションの成果物
#[derive(Debug, Clone, PartialEq)]
pub struct CamSimulationWorkflowResult {
    /// ツールパスの成果物のバイト列
    pub toolpath_artifact: Vec<u8>,
    /// 切削シミュレーションの成果物のバイト列
    pub simulation_artifact: Vec<u8>,
}

/// ツールパス生成から切削シミュレーションまでの失敗
#[derive(Debug, Clone, PartialEq)]
pub enum CamSimulationWorkflowError {
    /// ツールパス生成のジョブが失敗した（理由）
    ToolpathGeneration(String),
    /// 切削シミュレーションのジョブが失敗した（Preflight の拒否を含む。理由）
    Simulation(String),
    /// ジョブの投入・問い合わせの失敗
    Orchestration(JobOrchestrationError),
}

impl fmt::Display for CamSimulationWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolpathGeneration(reason) => write!(f, "toolpath generation failed: {reason}"),
            Self::Simulation(reason) => write!(f, "cutting simulation failed: {reason}"),
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

/// ツールパス生成から切削シミュレーションまでを実行する
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

    /// 入力を登録し、ツールパス生成と切削シミュレーションを実行して、両方の成果物を返す
    ///
    /// # Errors
    ///
    /// ツールパス生成・切削シミュレーションのジョブの失敗（理由つき）、ジョブの投入・問い合わせの失敗。
    pub fn run(
        &mut self,
        request: CamSimulationWorkflowRequest,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError> {
        self.next_run += 1;
        let cam_input_ref = format!("input://cam/run-{}", self.next_run);
        let simulation_input_ref = format!("input://sim/run-{}", self.next_run);
        let tool = request.solver_input.tool.clone();
        self.solver_inputs
            .register(&cam_input_ref, request.solver_input)
            .map_err(|error| CamSimulationWorkflowError::ToolpathGeneration(error.to_string()))?;
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
            .map_err(|error| CamSimulationWorkflowError::Simulation(error.to_string()))?;

        let (toolpath_job_id, toolpath) =
            self.submit(CamJobType::CamProcess, cam_input_ref, None)?;
        let toolpath_artifact = toolpath.map_err(CamSimulationWorkflowError::ToolpathGeneration)?;
        let (_, simulation) = self.submit(
            CamJobType::CuttingSimulation,
            simulation_input_ref,
            Some(toolpath_job_id),
        )?;
        let simulation_artifact = simulation.map_err(CamSimulationWorkflowError::Simulation)?;

        Ok(CamSimulationWorkflowResult {
            toolpath_artifact,
            simulation_artifact,
        })
    }

    /// ジョブを投入し、ジョブの番号と、成功した場合は成果物・失敗した場合は理由を返す
    fn submit(
        &mut self,
        job_type: CamJobType,
        input_ref: String,
        parent_job_id: Option<u64>,
    ) -> Result<(u64, Result<Vec<u8>, String>), JobOrchestrationError> {
        let submitted = self
            .orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type,
                input_ref,
                parent_job_id,
            })?;
        let result = self.orchestrator.query_result(JobWorkflowResultQuery {
            job_id: submitted.job_id,
        })?;
        let outcome = if submitted.status == CamJobStatus::Succeeded {
            result
                .active_artifact
                .ok_or_else(|| "job succeeded without an artifact".to_string())
        } else {
            Err(result
                .job
                .last_error
                .unwrap_or_else(|| format!("job ended with status {:?}", submitted.status)))
        };
        Ok((submitted.job_id, outcome))
    }
}

#[cfg(test)]
mod tests {
    use cam_algorithms::{OperationSpec, ScanlineParams, SolverGeometry, TessellationLimits};
    use cam_core::{CoordinateFrame, LengthUnit, Tool, read_toolpath_artifact_v1};
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
    fn run_returns_the_toolpath_and_simulation_artifacts() {
        let mut workflow = CamSimulationWorkflow::new();
        for _ in 0..2 {
            let result = workflow.run(request(0.01)).unwrap();
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
    fn toolpath_generation_failure_is_distinguished() {
        let mut invalid = request(0.01);
        invalid.solver_input.operation_id = String::new();
        let error = CamSimulationWorkflow::new().run(invalid).unwrap_err();
        assert!(
            matches!(error, CamSimulationWorkflowError::ToolpathGeneration(_)),
            "{error}"
        );
    }

    #[test]
    fn simulation_rejection_is_distinguished() {
        let error = CamSimulationWorkflow::new().run(request(0.0)).unwrap_err();
        let CamSimulationWorkflowError::Simulation(reason) = error else {
            panic!("unexpected error: {error}");
        };
        assert!(reason.contains("precision"), "{reason}");
    }
}
