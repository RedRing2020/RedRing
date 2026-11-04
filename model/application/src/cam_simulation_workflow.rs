//! ツールパス生成から切削シミュレーションまでのジョブの投入と、結果の取り込みを行うユースケース。
//!
//! 投入は、ツールパス生成の入力と切削シミュレーションの入力を登録先に登録し、ツールパス生成（CamProcess）の
//! ジョブと、それを親とする切削シミュレーション（CuttingSimulation）のジョブをキューに入れた時点で完了する。
//! 投入の前に、計算条件の誤りをすべて確かめる（投入前の確認）。投入は、確認を通った条件だけを受ける。
//! 取り込みは、正常終了した切削シミュレーションのジョブだけを対象に、ツールパスと切削シミュレーションの成果物と、
//! 投入した入力を返す。
//! 加工工程からの投入は、加工手順の加工の対象（製品形状の面）と素材、加工工程の工具・加工の種類と条件・加工領域・
//! 精度プロファイルから入力を組み立てる。形状の解決と素材の計算はそれぞれの担当に任せ、つなぐだけとする。
//! 加工工程から投入したジョブは、投入元として工程の ID と改訂の番号を持つ。工程からの取り込みは、工程の最も新しい
//! 改訂の結果とし、前の改訂の結果も消さずに残す（設計: CAM_PROCESS_ENTITY_DESIGN.md）。

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

use cam_algorithms::{
    CamSolverError, CamSolverInput, InverseOffsetShape, build_inverse_offset_shape,
};
use cam_core::{CoordinateFrame, LengthUnit, StockMargins};
use cam_entity::MachiningOperation;
use cam_sim::{
    CamJobExecutorAdapter, CamSolverInputProvider, CuttingSimulationInput,
    CuttingSimulationInputError, CuttingSimulationInputProvider, InMemoryCamSolverInputStore,
    InMemoryCuttingSimulationInputStore, SnapshotInterval, cutting_simulation_input_errors,
    stock_around_product,
};
use geo_algorithms::{Aabb3D, TessellationLimits};
use job_domain::{CamJobRecord, CamJobSource, CamJobStatus, CamJobType};
use job_runtime::JobManager;

use crate::job_orchestration::{
    JobOrchestrationError, JobWorkflowOrchestration, JobWorkflowOrchestrator,
    JobWorkflowResultQuery, JobWorkflowSubmitRequest,
};
use crate::machining_sequence::{
    MachiningOperationId, MachiningSequenceError, MachiningSequenceId, MachiningSequenceStore,
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
    /// 保管していない加工手順、または加工手順にない加工工程
    Sequence(MachiningSequenceError),
    /// 加工の対象の面を形状にできない
    ProductShape(ProductShapeError),
    /// 加工の対象の形状が空
    EmptyProductShape,
    /// 計算条件の誤り（すべて）
    Conditions(Vec<CamConditionError>),
    /// ジョブの投入・問い合わせの失敗
    Orchestration(JobOrchestrationError),
    /// 逆オフセット形状を作れない
    InverseOffsetShape(CamSolverError),
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
            Self::Sequence(error) => write!(f, "machining operation cannot be found: {error}"),
            Self::ProductShape(error) => write!(f, "product shape cannot be machined: {error}"),
            Self::EmptyProductShape => write!(f, "product shape is empty"),
            Self::Conditions(errors) => {
                write!(f, "calculation conditions are invalid:")?;
                for error in errors {
                    write!(f, " {error};")?;
                }
                Ok(())
            }
            Self::Orchestration(error) => write!(f, "job orchestration failed: {error}"),
            Self::InverseOffsetShape(error) => {
                write!(f, "inverse offset shape cannot be built: {error}")
            }
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
        self.enqueue(request, None)
    }

    /// 確認を通った入力を登録し、投入元を `source` としてジョブをキューに入れる
    fn enqueue(
        &mut self,
        request: CamSimulationWorkflowRequest,
        source: Option<CamJobSource>,
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
                source: source.clone(),
            })?;
        let simulation = self
            .orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CuttingSimulation,
                input_ref: simulation_input_ref,
                parent_job_id: Some(toolpath.job_id),
                source,
            })?;
        Ok(CamSimulationSubmission {
            toolpath_job_id: toolpath.job_id,
            simulation_job_id: simulation.job_id,
        })
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` からの投入の前に、計算条件の誤りをすべて確かめる
    /// （投入前の確認。投入と同じ判定）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順・加工手順にない加工工程の場合、加工の対象の面を形状にできない場合、形状が空の場合、
    /// 計算条件に誤りがある場合（すべての誤り）。
    pub fn check_operation(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<(), CamSimulationWorkflowError> {
        resolve_operation_request(shapes, sequences, sequence_id, operation_id).map(|_| ())
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` から、投入したときのツールパス生成と同じ逆オフセット形状を
    /// 作る（デバッグ表示に使う）
    ///
    /// # Errors
    ///
    /// 投入前の確認（[`Self::check_operation`]）の誤りと、逆オフセット形状を作れない場合。
    pub fn inverse_offset_shape(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<InverseOffsetShape, CamSimulationWorkflowError> {
        let (request, _) = resolve_operation_request(shapes, sequences, sequence_id, operation_id)?;
        build_inverse_offset_shape(&request.solver_input)
            .map_err(CamSimulationWorkflowError::InverseOffsetShape)
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` から、ツールパス生成と切削シミュレーションのジョブをキューに
    /// 入れる（実行しない）。2 つのジョブは、投入元として工程の ID と今の改訂の番号を持つ
    ///
    /// # Errors
    ///
    /// 投入前の確認（[`Self::check_operation`]）の誤り、入力の登録・投入の失敗。
    pub fn submit_operation(
        &mut self,
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        let (request, operation) =
            resolve_operation_request(shapes, sequences, sequence_id, operation_id)?;
        let source = CamJobSource {
            source_id: operation.id().to_string(),
            revision: operation.revision(),
        };
        self.enqueue(request, Some(source))
    }

    /// 実行できる待機中のジョブがなくなるまで実行し、実行したジョブの数を返す（ジョブマネージャーの実行の入口）
    ///
    /// # Errors
    ///
    /// ジョブの状態の更新の失敗。
    pub fn run_queued_jobs(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        Ok(self.orchestrator.run_queued_jobs()?)
    }

    /// 加工工程 `operation_id` から投入した、取り込める（正常終了した）切削シミュレーションのジョブ
    /// （前の改訂の結果を含む。ID の昇順）
    fn operation_simulations(&self, operation_id: MachiningOperationId) -> Vec<CamJobRecord> {
        let source_id = operation_id.to_string();
        self.orchestrator
            .list_succeeded_jobs(CamJobType::CuttingSimulation)
            .into_iter()
            .filter(|job| {
                job.spec
                    .source
                    .as_ref()
                    .is_some_and(|source| source.source_id == source_id)
            })
            .collect()
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂（最も新しい改訂）で投入した、
    /// 取り込める切削シミュレーションのうち、最も新しいジョブ（今の改訂の結果がない場合は `None`。工程は再計算待ち）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn latest_revision_simulation(
        &self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<Option<CamJobRecord>, CamSimulationWorkflowError> {
        let revision = sequences
            .operation(sequence_id, operation_id)
            .map_err(CamSimulationWorkflowError::Sequence)?
            .revision();
        Ok(self
            .operation_simulations(operation_id)
            .into_iter()
            .rev()
            .find(|job| {
                job.spec
                    .source
                    .as_ref()
                    .is_some_and(|source| source.revision == revision)
            }))
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

/// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` から、ツールパス生成から切削シミュレーション
/// までの入力を組み立て、
/// 計算条件の誤りをすべて確かめる。組み立てた入力と加工工程を返す
///
/// 余裕が不正な場合は素材を決められないため、製品形状を囲む箱を素材として残りの条件を確かめ、
/// 素材の大きさで決まる線の本数は判定しない。
fn resolve_operation_request<'a>(
    shapes: &ProductShapeStore,
    sequences: &'a MachiningSequenceStore,
    sequence_id: MachiningSequenceId,
    operation_id: MachiningOperationId,
) -> Result<(CamSimulationWorkflowRequest, &'a MachiningOperation), CamSimulationWorkflowError> {
    let sequence = sequences
        .sequence(sequence_id)
        .map_err(CamSimulationWorkflowError::Sequence)?;
    let operation = sequences
        .operation(sequence_id, operation_id)
        .map_err(CamSimulationWorkflowError::Sequence)?;
    let geometry = shapes
        .resolve_solver_geometry(sequence.target_faces())
        .map_err(CamSimulationWorkflowError::ProductShape)?;
    let bounds = geometry
        .aabb()
        .ok_or(CamSimulationWorkflowError::EmptyProductShape)?;
    let stock_margins = sequence.stock().margins;
    let stock = stock_around_product(bounds, stock_margins);
    let content = operation.content();
    let solver_input = CamSolverInput {
        operation_id: operation.id().to_string(),
        tool: content.tool.clone(),
        geometry,
        operation: content.spec,
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: content.tolerance_profile.tolerance_mm(),
        tessellation_limits: TessellationLimits::default(),
        boundary: content.boundary,
    };
    let workflow_request = CamSimulationWorkflowRequest {
        solver_input,
        stock: stock.unwrap_or(bounds),
        precision: sequence.stock().precision,
        snapshot_interval: SnapshotInterval::default(),
    };
    let errors = match stock {
        Some(_) => condition_errors(&workflow_request),
        None => {
            let mut errors = vec![CamConditionError::InvalidStockMargins(stock_margins)];
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
        Ok((workflow_request, operation))
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

/// 切削シミュレーションの入力（工具・製品形状・トレランスはツールパス生成の入力と同じ）
fn simulation_input(request: &CamSimulationWorkflowRequest) -> CuttingSimulationInput {
    CuttingSimulationInput {
        stock: request.stock,
        tool: request.solver_input.tool.clone(),
        precision: request.precision,
        snapshot_interval: request.snapshot_interval.clone(),
        product: request.solver_input.geometry.clone(),
        tolerance: request.solver_input.chord_tolerance,
    }
}
