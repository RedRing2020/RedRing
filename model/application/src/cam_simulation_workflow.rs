//! ツールパス生成から切削シミュレーションまでのジョブの投入と、結果の取り込みを行うユースケース。
//!
//! 投入は、ツールパス生成の入力と切削シミュレーションの入力を登録先に登録し、ツールパス生成（CamProcess）の
//! ジョブと、それを親とする切削シミュレーション（CuttingSimulation）のジョブをキューに入れた時点で完了する。
//! 投入の前に、計算条件の誤りをすべて確かめる（投入前の確認）。投入は、確認を通った条件だけを受ける。
//! 取り込みは、正常終了した切削シミュレーションのジョブだけを対象に、ツールパスと切削シミュレーションの成果物と、
//! 投入した入力を返す。
//! 製品形状からの投入は、形状の解決・ツールパス生成の入力の組み立て・素材の計算をそれぞれの担当に任せ、つなぐだけとする。

use std::fmt;
use std::sync::Arc;

use cam_algorithms::{
    CamSolverError, CamSolverInput, OperationDefinition, ProcessTemplate, ScanlineParams,
    SolverDefaults, TemplateFailure, build_solver_input, resolve_operation,
};
use cam_core::Tool;
use cam_sim::{
    CamJobExecutorAdapter, CamSolverInputProvider, CuttingSimulationInput,
    CuttingSimulationInputError, CuttingSimulationInputProvider, InMemoryCamSolverInputStore,
    InMemoryCuttingSimulationInputStore, SnapshotInterval, StockMargins,
    cutting_simulation_input_errors, stock_around_product,
};
use geo_algorithms::{Aabb3D, TessellationLimits};
use geo_entity::FaceId;
use job_domain::{CamJobRecord, CamJobStatus, CamJobType};
use job_runtime::JobManager;

use crate::job_orchestration::{
    JobOrchestrationError, JobWorkflowOrchestration, JobWorkflowOrchestrator,
    JobWorkflowResultQuery, JobWorkflowSubmitRequest,
};
use crate::product_shape::{ProductShapeError, ProductShapeStore};

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

/// 製品形状から切削シミュレーションを投入する入力
#[derive(Debug, Clone)]
pub struct ProductShapeSimulationRequest {
    /// 加工の対象の面
    pub face_ids: Vec<FaceId>,
    /// 工程テンプレート（オペレーションの既定値）
    pub template: ProcessTemplate,
    /// オペレーションの定義
    pub operation: OperationDefinition,
    /// 工具
    pub tool: Tool<f64>,
    /// 走査線の条件
    pub scanline: ScanlineParams,
    /// 製品形状を囲む箱から素材の箱の各面までの余裕
    pub stock_margins: StockMargins,
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

/// 取り込んだツールパス生成と切削シミュレーションの成果物と、投入した入力
#[derive(Debug, Clone)]
pub struct CamSimulationWorkflowResult {
    /// ツールパスの成果物のバイト列
    pub toolpath_artifact: Vec<u8>,
    /// 切削シミュレーションの成果物のバイト列
    pub simulation_artifact: Vec<u8>,
    /// ツールパス生成の入力
    pub solver_input: CamSolverInput,
    /// 切削シミュレーションの入力
    pub simulation_input: CuttingSimulationInput,
}

/// 計算条件の誤り（入力の欄ごとに分けて知らせる）
#[derive(Debug, Clone, PartialEq)]
pub enum CamConditionError {
    /// 素材の余裕が負または有限でない
    InvalidStockMargins(StockMargins),
    /// 素材の精度が、ツールパス生成のトレランスより細かい（素材は CL より正確である必要がない）
    PrecisionFinerThanTolerance { precision: f64, tolerance: f64 },
    /// ツールパス生成の入力（工具・走査線の条件など）の誤り
    ToolpathInput(CamSolverError),
    /// 切削シミュレーションの入力の誤り
    SimulationInput(CuttingSimulationInputError),
}

impl fmt::Display for CamConditionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStockMargins(margins) => {
                write!(
                    f,
                    "stock margins must be finite and non-negative: {margins:?}"
                )
            }
            Self::PrecisionFinerThanTolerance {
                precision,
                tolerance,
            } => write!(
                f,
                "stock precision {precision} must not be finer than the toolpath tolerance {tolerance}"
            ),
            Self::ToolpathInput(error) => write!(f, "toolpath input is invalid: {error}"),
            Self::SimulationInput(error) => {
                write!(f, "cutting simulation input is invalid: {error}")
            }
        }
    }
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
    /// 正常終了した切削シミュレーションのジョブの入力がない（ジョブの番号）
    MissingInput(u64),
    /// 加工の対象の面を形状にできない
    ProductShape(ProductShapeError),
    /// 加工の対象の形状が空
    EmptyProductShape,
    /// 計算条件の誤り（すべて）
    Conditions(Vec<CamConditionError>),
    /// オペレーションの定義を解決できない
    Operation(TemplateFailure),
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
            Self::MissingInput(job_id) => write!(f, "job {job_id} has no registered input"),
            Self::ProductShape(error) => write!(f, "product shape cannot be machined: {error}"),
            Self::EmptyProductShape => write!(f, "product shape is empty"),
            Self::Conditions(errors) => {
                write!(f, "calculation conditions are invalid:")?;
                for error in errors {
                    write!(f, " {error};")?;
                }
                Ok(())
            }
            Self::Operation(error) => write!(f, "operation cannot be resolved: {error}"),
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
    /// 計算条件に誤りがある場合（すべての誤り）、入力の登録の失敗、ジョブの投入の失敗。
    pub fn submit(
        &mut self,
        request: CamSimulationWorkflowRequest,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        let errors = condition_errors(&request);
        if !errors.is_empty() {
            return Err(CamSimulationWorkflowError::Conditions(errors));
        }
        self.enqueue(request)
    }

    /// 確認を通った入力を登録し、ジョブをキューに入れる
    fn enqueue(
        &mut self,
        request: CamSimulationWorkflowRequest,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        self.next_run += 1;
        let cam_input_ref = format!("input://cam/run-{}", self.next_run);
        let simulation_input_ref = format!("input://sim/run-{}", self.next_run);
        let simulation_input = simulation_input(&request);
        self.solver_inputs
            .register(&cam_input_ref, request.solver_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        self.simulation_inputs
            .register(&simulation_input_ref, simulation_input)
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

    /// 製品形状の面からの投入の前に、計算条件の誤りをすべて確かめる（投入前の確認。投入と同じ判定）
    ///
    /// # Errors
    ///
    /// 面を形状にできない場合、形状が空の場合、オペレーションの定義を解決できない場合、
    /// 計算条件に誤りがある場合（すべての誤り）。
    pub fn check_product_shapes(
        shapes: &ProductShapeStore,
        request: &ProductShapeSimulationRequest,
    ) -> Result<(), CamSimulationWorkflowError> {
        resolve_product_shape_request(shapes, request.clone()).map(|_| ())
    }

    /// 製品形状の面からツールパス生成と切削シミュレーションのジョブをキューに入れる（実行しない）
    ///
    /// 面の形状の解決、ツールパス生成の入力の組み立て、製品形状を囲む素材の計算をつなぐ。
    ///
    /// # Errors
    ///
    /// 投入前の確認（`check_product_shapes`）の誤り、入力の登録・投入の失敗。
    pub fn submit_product_shapes(
        &mut self,
        shapes: &ProductShapeStore,
        request: ProductShapeSimulationRequest,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        let request = resolve_product_shape_request(shapes, request)?;
        self.enqueue(request)
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

    /// 正常終了した切削シミュレーションのジョブ `simulation_job_id` の、ツールパスと切削シミュレーションの成果物と、
    /// 投入した入力
    ///
    /// # Errors
    ///
    /// ジョブが正常終了した切削シミュレーションでない場合、成果物・入力がない場合、問い合わせの失敗。
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
        let missing_input = || CamSimulationWorkflowError::MissingInput(simulation_job_id);
        let simulation_input = self
            .simulation_inputs
            .resolve(&result.job.spec.input_ref)
            .ok_or_else(missing_input)?;
        let toolpath_job_id = result.job.parent_job_id.ok_or_else(missing_input)?;
        let toolpath_job = self.orchestrator.query_result(JobWorkflowResultQuery {
            job_id: toolpath_job_id,
        })?;
        let solver_input = self
            .solver_inputs
            .resolve(&toolpath_job.job.spec.input_ref)
            .ok_or_else(missing_input)?;
        Ok(CamSimulationWorkflowResult {
            // 切削シミュレーションの入力に使ったツールパス（親のジョブの成果物）
            toolpath_artifact: result.input_artifact.ok_or_else(missing)?,
            simulation_artifact: result.active_artifact.ok_or_else(missing)?,
            solver_input,
            simulation_input,
        })
    }
}

/// 製品形状の面から、ツールパス生成から切削シミュレーションまでの入力を組み立て、計算条件の誤りをすべて確かめる
///
/// 余裕が不正な場合は素材を決められないため、製品形状を囲む箱を素材として残りの条件を確かめ、
/// 素材の大きさで決まる線の本数は判定しない。
fn resolve_product_shape_request(
    shapes: &ProductShapeStore,
    request: ProductShapeSimulationRequest,
) -> Result<CamSimulationWorkflowRequest, CamSimulationWorkflowError> {
    let geometry = shapes
        .resolve_solver_geometry(&request.face_ids)
        .map_err(CamSimulationWorkflowError::ProductShape)?;
    let bounds = geometry
        .aabb()
        .ok_or(CamSimulationWorkflowError::EmptyProductShape)?;
    let stock = stock_around_product(bounds, request.stock_margins);
    let resolved = resolve_operation(
        &request.template,
        &request.operation,
        SolverDefaults::default(),
    )
    .map_err(CamSimulationWorkflowError::Operation)?;
    let solver_input = build_solver_input(
        &resolved,
        request.tool,
        geometry,
        request.scanline,
        TessellationLimits::default(),
    )
    .map_err(CamSimulationWorkflowError::Operation)?;
    let workflow_request = CamSimulationWorkflowRequest {
        solver_input,
        stock: stock.unwrap_or(bounds),
        precision: request.precision,
        snapshot_interval: request.snapshot_interval,
    };
    let errors = match stock {
        Some(_) => condition_errors(&workflow_request),
        None => {
            let mut errors = vec![CamConditionError::InvalidStockMargins(
                request.stock_margins,
            )];
            errors.extend(
                condition_errors(&workflow_request)
                    .into_iter()
                    .filter(|error| {
                        !matches!(
                            error,
                            CamConditionError::SimulationInput(
                                CuttingSimulationInputError::TooManyDexelLines { .. }
                            )
                        )
                    }),
            );
            errors
        }
    };
    if errors.is_empty() {
        Ok(workflow_request)
    } else {
        Err(CamSimulationWorkflowError::Conditions(errors))
    }
}

/// 計算条件の誤りをすべて返す（ツールパス生成と切削シミュレーションの実行のときと同じ判定）
fn condition_errors(request: &CamSimulationWorkflowRequest) -> Vec<CamConditionError> {
    let mut errors = Vec::new();
    let tolerance = request.solver_input.chord_tolerance;
    // 正の有限値でない精度は、切削シミュレーションの入力の誤りとして知らせる
    if request.precision > 0.0 && request.precision < tolerance {
        errors.push(CamConditionError::PrecisionFinerThanTolerance {
            precision: request.precision,
            tolerance,
        });
    }
    if let Err(error) = request.solver_input.validate() {
        errors.push(CamConditionError::ToolpathInput(error));
    }
    errors.extend(
        cutting_simulation_input_errors(&simulation_input(request))
            .into_iter()
            .map(CamConditionError::SimulationInput),
    );
    errors
}

/// 切削シミュレーションの入力（工具はツールパス生成の入力と同じ）
fn simulation_input(request: &CamSimulationWorkflowRequest) -> CuttingSimulationInput {
    CuttingSimulationInput {
        stock: request.stock,
        tool: request.solver_input.tool.clone(),
        precision: request.precision,
        snapshot_interval: request.snapshot_interval.clone(),
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
                    tool_range: cam_algorithms::ScanlineToolRange::default(),
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
            workflow.import(submission.simulation_job_id).unwrap_err(),
            CamSimulationWorkflowError::NotSucceededSimulation(submission.simulation_job_id)
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
        use cam_algorithms::{MachiningBoundary, RectangleBoundary};

        let mut workflow = CamSimulationWorkflow::new();
        // 工具が形状に接触しない加工範囲では、ツールパス生成が失敗し、子の切削シミュレーションは実行されずに失敗する
        let mut outside = request(0.01);
        outside.solver_input.boundary = Some(MachiningBoundary::Rectangle(RectangleBoundary {
            rect_min: [200.0, 200.0],
            rect_max: [210.0, 210.0],
        }));
        let failed = workflow.submit(outside).unwrap();
        let succeeded = workflow.submit(request(0.01)).unwrap();
        workflow.run_queued_jobs().unwrap();

        let simulations: Vec<u64> = workflow
            .succeeded_simulations()
            .iter()
            .map(|job| job.id)
            .collect();
        assert_eq!(simulations, vec![succeeded.simulation_job_id]);
        assert_eq!(
            workflow.import(failed.simulation_job_id).unwrap_err(),
            CamSimulationWorkflowError::NotSucceededSimulation(failed.simulation_job_id)
        );
        // ツールパス生成のジョブは切削シミュレーションとしては取り込めない
        assert_eq!(
            workflow.import(succeeded.toolpath_job_id).unwrap_err(),
            CamSimulationWorkflowError::NotSucceededSimulation(succeeded.toolpath_job_id)
        );
    }

    #[test]
    fn precision_finer_than_the_toolpath_tolerance_is_not_submitted() {
        let mut workflow = CamSimulationWorkflow::new();
        assert_eq!(
            workflow.submit(request(0.005)).unwrap_err(),
            CamSimulationWorkflowError::Conditions(vec![
                CamConditionError::PrecisionFinerThanTolerance {
                    precision: 0.005,
                    tolerance: 0.01,
                }
            ])
        );
        assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
    }

    fn product_shape_request(face_ids: Vec<geo_entity::FaceId>) -> ProductShapeSimulationRequest {
        use cam_algorithms::{OperationDefinition, ProcessTemplate};

        ProductShapeSimulationRequest {
            face_ids,
            template: ProcessTemplate {
                template_id: "template-1".to_string(),
                default_operation_type: Some("scanline".to_string()),
                default_tolerance_profile: Some("press_rough".to_string()),
            },
            operation: OperationDefinition {
                operation_id: "op-scan-1".to_string(),
                machining_stage: "finish".to_string(),
                operation_type: None,
                tolerance_profile: None,
                boundary: None,
            },
            tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
            scanline: ScanlineParams {
                stepover: 6.0,
                sample_pitch: 10.0,
                feed_rate: 1200.0,
                tool_range: cam_algorithms::ScanlineToolRange::default(),
            },
            stock_margins: StockMargins {
                minus_x: 5.0,
                plus_x: 5.0,
                minus_y: 5.0,
                plus_y: 5.0,
                bottom: 0.0,
                top: 5.0,
            },
            precision: 0.01,
            snapshot_interval: SnapshotInterval::default(),
        }
    }

    /// 四角錐の 1 面だけの製品形状
    fn pyramid_shape() -> (ProductShapeStore, geo_entity::FaceId) {
        use crate::product_shape::ProductFaceGeometry;

        let SolverGeometry::TriangleMesh(pyramid) = request(0.01).solver_input.geometry else {
            panic!("the test request uses a mesh");
        };
        let mut shapes = ProductShapeStore::new();
        let face = shapes.add_face(ProductFaceGeometry::Mesh(pyramid)).face_id;
        (shapes, face)
    }

    #[test]
    fn product_shapes_are_submitted_and_imported_with_the_submitted_inputs() {
        use crate::product_shape::{ProductFaceGeometry, ProductShapeStore};

        let SolverGeometry::TriangleMesh(pyramid) = request(0.01).solver_input.geometry else {
            panic!("the test request uses a mesh");
        };
        let mut shapes = ProductShapeStore::new();
        let face = shapes.add_face(ProductFaceGeometry::Mesh(pyramid)).face_id;
        let mut workflow = CamSimulationWorkflow::new();
        let submission = workflow
            .submit_product_shapes(&shapes, product_shape_request(vec![face]))
            .unwrap();
        assert_eq!(workflow.run_queued_jobs().unwrap(), 2);

        let result = workflow.import(submission.simulation_job_id).unwrap();
        let submitted = product_shape_request(vec![face]);
        assert_eq!(result.solver_input.tool, submitted.tool);
        assert_eq!(
            result.solver_input.operation,
            OperationSpec::Scanline(submitted.scanline)
        );
        // 工程テンプレートの press_rough のトレランス
        assert_eq!(result.solver_input.chord_tolerance, 0.001);
        assert_eq!(result.simulation_input.tool, submitted.tool);
        assert_eq!(result.simulation_input.precision, submitted.precision);
        assert_eq!(
            result.simulation_input.stock,
            Aabb3D::new(
                Point3D::new(15.0, 15.0, 80.0),
                Point3D::new(65.0, 65.0, 95.0)
            )
        );
        let (_, simulation) = cam_sim::read_cutting_simulation_artifact_v1(
            &mut result.simulation_artifact.as_slice(),
        )
        .unwrap();
        // 素材は四角錐（20〜60, 20〜60, 80〜90）を囲む箱に、XY の周りと上に 5 を足したもの（底の余裕は 0）
        let initial = simulation.snapshots[0].remaining_volume_mm3;
        let expected = 50.0 * 50.0 * 15.0;
        assert!((initial - expected).abs() < 1e-6 * expected, "{initial}");
        let last = simulation.snapshots.last().unwrap().remaining_volume_mm3;
        assert!(last < initial);
    }

    #[test]
    fn product_shapes_that_cannot_be_machined_are_not_submitted() {
        use crate::product_shape::{ProductShapeError, ProductShapeStore};

        let mut workflow = CamSimulationWorkflow::new();
        let shapes = ProductShapeStore::new();
        assert_eq!(
            workflow
                .submit_product_shapes(&shapes, product_shape_request(Vec::new()))
                .unwrap_err(),
            CamSimulationWorkflowError::ProductShape(ProductShapeError::NoFaces)
        );
        assert!(workflow.succeeded_simulations().is_empty());
        assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
    }

    #[test]
    fn stock_without_xy_margins_has_the_xy_size_of_the_product_shape() {
        use crate::product_shape::{ProductFaceGeometry, ProductShapeStore};

        let SolverGeometry::TriangleMesh(pyramid) = request(0.01).solver_input.geometry else {
            panic!("the test request uses a mesh");
        };
        let mut shapes = ProductShapeStore::new();
        let face = shapes.add_face(ProductFaceGeometry::Mesh(pyramid)).face_id;
        let mut request = product_shape_request(vec![face]);
        request.stock_margins = StockMargins {
            top: 5.0,
            ..StockMargins::default()
        };
        let mut workflow = CamSimulationWorkflow::new();
        let submission = workflow.submit_product_shapes(&shapes, request).unwrap();
        assert_eq!(workflow.run_queued_jobs().unwrap(), 2);

        let result = workflow.import(submission.simulation_job_id).unwrap();
        let (_, simulation) = cam_sim::read_cutting_simulation_artifact_v1(
            &mut result.simulation_artifact.as_slice(),
        )
        .unwrap();
        // 素材は四角錐（20〜60, 20〜60, 80〜90）を囲む箱の XY の範囲のまま、上に 5 を足したもの
        let initial = simulation.snapshots[0].remaining_volume_mm3;
        let expected = 40.0 * 40.0 * 15.0;
        assert!((initial - expected).abs() < 1e-6 * expected, "{initial}");
    }

    #[test]
    fn negative_stock_margins_are_not_submitted() {
        use crate::product_shape::{ProductFaceGeometry, ProductShapeStore};

        let SolverGeometry::TriangleMesh(pyramid) = request(0.01).solver_input.geometry else {
            panic!("the test request uses a mesh");
        };
        let mut shapes = ProductShapeStore::new();
        let face = shapes.add_face(ProductFaceGeometry::Mesh(pyramid)).face_id;
        let mut request = product_shape_request(vec![face]);
        request.stock_margins.minus_x = -1.0;
        let margins = request.stock_margins;
        let mut workflow = CamSimulationWorkflow::new();
        assert_eq!(
            workflow
                .submit_product_shapes(&shapes, request)
                .unwrap_err(),
            CamSimulationWorkflowError::Conditions(vec![CamConditionError::InvalidStockMargins(
                margins
            )])
        );
        assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
    }

    #[test]
    fn all_condition_errors_are_reported_before_submitting() {
        let (shapes, face) = pyramid_shape();
        let mut request = product_shape_request(vec![face]);
        request.stock_margins.top = f64::NAN;
        // press_rough のトレランス 0.001 より細かい精度
        request.precision = 0.0005;
        // ステップオーバーが工具径 6 を超える
        request.scanline.stepover = 7.0;
        // 工具長が 0
        request.tool = Tool::ball_end_mill("BEM6".to_string(), 6.0, 0.0);
        let margins = request.stock_margins;
        let Err(CamSimulationWorkflowError::Conditions(errors)) =
            CamSimulationWorkflow::check_product_shapes(&shapes, &request)
        else {
            panic!("the conditions must be rejected");
        };
        let [
            CamConditionError::InvalidStockMargins(rejected_margins),
            CamConditionError::PrecisionFinerThanTolerance {
                precision,
                tolerance,
            },
            CamConditionError::ToolpathInput(CamSolverError::InvalidInput(_)),
            CamConditionError::SimulationInput(CuttingSimulationInputError::Tool),
        ] = &errors[..]
        else {
            panic!("unexpected errors: {errors:?}");
        };
        assert!(rejected_margins.top.is_nan() && rejected_margins.minus_x == margins.minus_x);
        assert_eq!((*precision, *tolerance), (0.0005, 0.001));

        // 投入も同じ誤りで拒み、キューに入れない
        let mut workflow = CamSimulationWorkflow::new();
        let Err(CamSimulationWorkflowError::Conditions(rejected)) =
            workflow.submit_product_shapes(&shapes, request)
        else {
            panic!("the conditions must be rejected");
        };
        assert_eq!(rejected.len(), errors.len());
        assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
    }

    #[test]
    fn precision_beyond_the_line_limit_is_reported_with_the_finest_precision() {
        let (shapes, face) = pyramid_shape();
        let mut request = product_shape_request(vec![face]);
        request.stock_margins.minus_x = 1000.0;
        request.stock_margins.plus_x = 1000.0;
        request.stock_margins.minus_y = 1000.0;
        request.stock_margins.plus_y = 1000.0;
        request.precision = 0.001;
        let Err(CamSimulationWorkflowError::Conditions(errors)) =
            CamSimulationWorkflow::check_product_shapes(&shapes, &request)
        else {
            panic!("the precision must be rejected");
        };
        let [
            CamConditionError::SimulationInput(CuttingSimulationInputError::TooManyDexelLines {
                finest_precision,
                ..
            }),
        ] = errors[..]
        else {
            panic!("unexpected errors: {errors:?}");
        };
        assert!(finest_precision > request.precision);
        request.precision = finest_precision;
        assert_eq!(
            CamSimulationWorkflow::check_product_shapes(&shapes, &request),
            Ok(())
        );
    }
}
