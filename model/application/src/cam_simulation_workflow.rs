//! ツールパス生成から切削シミュレーションまでのジョブの投入と、結果の取り込みを行うユースケース。
//!
//! 投入は、ツールパス生成のジョブと、その成果物を入力にする切削シミュレーションのジョブを、ジョブの計算を行う
//! ジョブマネージャーへ渡した時点で完了する（計算を待たない。ジョブの境界は `job_client`）。
//! 投入の前に、計算条件の誤りをすべて確かめる（投入前の確認）。投入は、確認を通った条件だけを受ける。
//! 取り込みは、正常終了した切削シミュレーションのジョブだけを対象に、ツールパスと切削シミュレーションの成果物と、
//! 投入した入力を返す。
//! 加工工程からの投入は、加工手順の加工の対象（製品形状の面）と素材、加工工程の工具・加工の種類と条件・加工領域・
//! 精度プロファイルから入力を組み立てる。形状の解決と素材の計算はそれぞれの担当に任せ、つなぐだけとする。
//! 加工工程から投入したジョブは、投入元として工程の ID と改訂の番号を持つ。工程からの取り込みは、工程の最も新しい
//! 改訂の結果とし、前の改訂の結果も消さずに残す（設計: CAM_PROCESS_ENTITY_DESIGN.md）。

#[cfg(test)]
mod tests;

/// 逆オフセット形状の表示用の三角形と、その種類
pub use cam_algorithms::{InverseOffsetPart, InverseOffsetPartKind};
use std::fmt;

use cam_algorithms::{
    CamSolverError, CamSolverInput, InverseOffsetShape, ScanlineInputError,
    build_inverse_offset_shape,
};
use cam_core::{CoordinateFrame, LengthUnit, StockMargins};
use cam_entity::MachiningOperation;
use cam_sim::{
    CuttingSimulationInput, CuttingSimulationInputError, SnapshotInterval,
    cutting_simulation_input_errors, stock_around_product,
};
/// 軸に平行な直方体の型（素材の形に使う）
pub use geo_algorithms::Aabb3D;
use geo_algorithms::TessellationLimits;
use job_domain::{CamJobSource, CamJobStatus};

use crate::job_client::{JobClient, JobSubmission, LocalStoreJobClient, SimulationJob};
use crate::job_orchestration::JobOrchestrationError;
use crate::machining_sequence::{
    MachiningOperationId, MachiningSequenceError, MachiningSequenceId, MachiningSequenceStore,
};
use crate::product_shape::{ProductShapeError, ProductShapeStore};

/// ツールパス生成から切削シミュレーションまでの入力
#[derive(Debug, Clone)]
pub(crate) struct CamSimulationWorkflowRequest {
    /// ツールパス生成の入力（工具は切削シミュレーションでも使う）
    pub solver_input: CamSolverInput,
    /// 素材（直方体）
    pub stock: Aabb3D<f64>,
    /// 素材の形状の許容誤差
    pub precision: f64,
    /// スナップショットの記録の間隔
    pub snapshot_interval: SnapshotInterval,
}

/// 投入（ジョブの番号はジョブマネージャーが振るため、投入の ID で引く）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CamSimulationSubmission {
    /// 投入の ID
    pub submission_id: String,
}

/// 取り込んだツールパス生成と切削シミュレーションの成果物と、投入した入力
#[derive(Debug, Clone)]
pub struct CamSimulationWorkflowResult {
    /// ツールパスの成果物のバイト列（形式は外に見せない）
    pub(crate) toolpath_artifact: Vec<u8>,
    /// 切削シミュレーションの成果物のバイト列
    pub(crate) simulation_artifact: Vec<u8>,
    /// ツールパス生成の入力
    pub solver_input: CamSolverInput,
    /// 切削シミュレーションの入力
    pub simulation_input: CuttingSimulationInput,
}

/// 加工工程の今の改訂の計算（ツールパス生成と切削シミュレーション）の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationJobStatus {
    /// 計算を待っている
    Queued,
    /// 計算している
    Running,
    /// 正常に終わった（取り込める）
    Succeeded,
    /// 失敗した
    Failed,
    /// 取り消された
    Canceled,
}

/// 計算条件の誤りを出す入力の欄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamConditionField {
    /// 工具
    Tool,
    /// 加工（走査線の条件など、ツールパス生成の条件）
    Machining,
    /// 素材の余裕
    Stock,
    /// 素材の精度
    Precision,
    /// 入力の欄に当たらない条件（記録の間隔など）
    Other,
}

/// 計算条件の誤り（入力の欄と理由で分ける。計算の層の誤りの型は見せない）
#[derive(Debug, Clone, PartialEq)]
pub enum CamConditionError {
    /// 素材の余裕が負または有限でない
    InvalidStockMargins(StockMargins),
    /// 素材が体積を持つ直方体にならない
    InvalidStock,
    /// 工具の値が不正
    InvalidTool,
    /// ピックフィードが正の有限値でない
    StepoverNotPositive,
    /// ピックフィードが工具径を超える（投入した工具の径。mm）
    StepoverExceedsToolDiameter { tool_diameter: f64 },
    /// 送りが正の有限値でない
    FeedRateNotPositive,
    /// そのほかのツールパス生成の入力の誤り（理由）
    InvalidToolpathInput(String),
    /// 精度が正の有限値でない
    InvalidPrecision,
    /// 素材の精度が、ツールパス生成のトレランスより細かい（素材は CL より正確である必要がない）
    PrecisionFinerThanTolerance { precision: f64, tolerance: f64 },
    /// 精度が細かすぎて格子の線の本数の上限を超える（上限に収まる最も細かい精度。mm）
    TooManyDexelLines { finest_precision: f64 },
    /// 記録の間隔が不正
    InvalidSnapshotInterval,
}

impl CamConditionError {
    /// 誤りを出す入力の欄
    pub fn field(&self) -> CamConditionField {
        match self {
            Self::InvalidTool => CamConditionField::Tool,
            Self::StepoverNotPositive
            | Self::StepoverExceedsToolDiameter { .. }
            | Self::FeedRateNotPositive
            | Self::InvalidToolpathInput(_) => CamConditionField::Machining,
            Self::InvalidStockMargins(_) | Self::InvalidStock => CamConditionField::Stock,
            Self::InvalidPrecision
            | Self::PrecisionFinerThanTolerance { .. }
            | Self::TooManyDexelLines { .. } => CamConditionField::Precision,
            Self::InvalidSnapshotInterval => CamConditionField::Other,
        }
    }

    /// ツールパス生成の入力の誤り `error` を分ける（工具の径は、確かめた工具の径）
    fn from_toolpath_input(error: CamSolverError, tool_diameter: f64) -> Self {
        match error {
            CamSolverError::InvalidScanline(ScanlineInputError::StepoverNotPositive) => {
                Self::StepoverNotPositive
            }
            CamSolverError::InvalidScanline(ScanlineInputError::StepoverExceedsToolDiameter) => {
                Self::StepoverExceedsToolDiameter { tool_diameter }
            }
            CamSolverError::InvalidScanline(ScanlineInputError::FeedRateNotPositive) => {
                Self::FeedRateNotPositive
            }
            other => Self::InvalidToolpathInput(other.reason().to_string()),
        }
    }

    /// 切削シミュレーションの入力の誤り `error` を分ける
    fn from_simulation_input(error: CuttingSimulationInputError) -> Self {
        match error {
            CuttingSimulationInputError::Stock => Self::InvalidStock,
            CuttingSimulationInputError::Tool => Self::InvalidTool,
            CuttingSimulationInputError::Precision => Self::InvalidPrecision,
            CuttingSimulationInputError::SnapshotInterval => Self::InvalidSnapshotInterval,
            CuttingSimulationInputError::TooManyDexelLines {
                finest_precision, ..
            } => Self::TooManyDexelLines { finest_precision },
        }
    }
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
            Self::InvalidStock => write!(f, "stock must be a finite box with volume"),
            Self::InvalidTool => write!(f, "tool parameters are invalid"),
            Self::StepoverNotPositive => write!(f, "stepover must be positive and finite"),
            Self::StepoverExceedsToolDiameter { tool_diameter } => write!(
                f,
                "stepover must not exceed the tool diameter {tool_diameter}"
            ),
            Self::FeedRateNotPositive => write!(f, "feed rate must be positive and finite"),
            Self::InvalidToolpathInput(reason) => write!(f, "toolpath input is invalid: {reason}"),
            Self::InvalidPrecision => write!(f, "stock precision must be positive and finite"),
            Self::PrecisionFinerThanTolerance {
                precision,
                tolerance,
            } => write!(
                f,
                "stock precision {precision} must not be finer than the toolpath tolerance {tolerance}"
            ),
            Self::TooManyDexelLines { finest_precision } => write!(
                f,
                "stock precision exceeds the dexel line limit; the finest precision is {finest_precision}"
            ),
            Self::InvalidSnapshotInterval => write!(f, "snapshot interval is invalid"),
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
    /// 逆オフセット形状を表示用の三角形に分けられない（表示のきざみの誤り）
    InverseOffsetDisplay(CamSolverError),
    /// ジョブマネージャーがインストールされていない（アプリと同じフォルダにない）
    JobManagerNotFound,
    /// ジョブの保存先を読み書きできない・ジョブマネージャーを起動できない（理由）
    JobStore(String),
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
            Self::InverseOffsetDisplay(error) => {
                write!(f, "inverse offset shape cannot be displayed: {error}")
            }
            Self::JobManagerNotFound => write!(f, "the job manager is not installed"),
            Self::JobStore(reason) => write!(f, "job store failed: {reason}"),
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
    jobs: Box<dyn JobClient>,
}

impl Default for CamSimulationWorkflow {
    /// 同じ PC のジョブの保存先を通して、アプリと同じフォルダのジョブマネージャーに計算させる
    fn default() -> Self {
        Self {
            jobs: Box::new(LocalStoreJobClient::from_environment()),
        }
    }
}

impl CamSimulationWorkflow {
    pub fn new() -> Self {
        Self::default()
    }

    /// 同じプロセスの中でジョブを実行する（試験用）
    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn in_process() -> Self {
        Self {
            jobs: Box::new(crate::job_client::InProcessJobClient::default()),
        }
    }

    /// 確かめた入力を、投入元を `source` として投入する
    fn enqueue(
        &mut self,
        request: CamSimulationWorkflowRequest,
        source: Option<CamJobSource>,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        let simulation_input = simulation_input(&request);
        let submission_id = self.jobs.submit(JobSubmission {
            solver_input: request.solver_input,
            simulation_input,
            source,
        })?;
        Ok(CamSimulationSubmission { submission_id })
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
    /// 作り、角度 `angle_step`（ラジアン）のきざみで種類ごとの表示用の三角形にする
    ///
    /// # Errors
    ///
    /// 投入前の確認（[`Self::check_operation`]）の誤り、逆オフセット形状を作れない場合、`angle_step` が表示用のきざみの
    /// 範囲の外の場合。
    pub fn inverse_offset_shape_parts(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
        angle_step: f64,
    ) -> Result<Vec<InverseOffsetPart>, CamSimulationWorkflowError> {
        Self::inverse_offset_shape(shapes, sequences, sequence_id, operation_id)?
            .display_parts(angle_step)
            .map_err(CamSimulationWorkflowError::InverseOffsetDisplay)
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` から、投入したときのツールパス生成と同じ逆オフセット形状を
    /// 作る
    ///
    /// # Errors
    ///
    /// 投入前の確認（[`Self::check_operation`]）の誤りと、逆オフセット形状を作れない場合。
    fn inverse_offset_shape(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<InverseOffsetShape, CamSimulationWorkflowError> {
        let (request, _) = resolve_operation_request(shapes, sequences, sequence_id, operation_id)?;
        build_inverse_offset_shape(&request.solver_input)
            .map_err(CamSimulationWorkflowError::InverseOffsetShape)
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` から、ツールパス生成と切削シミュレーションのジョブを
    /// 投入する（計算を待たない）。2 つのジョブは、投入元として工程の ID と今の改訂の番号を持つ
    ///
    /// # Errors
    ///
    /// 投入前の確認（[`Self::check_operation`]）の誤り、入力の書き出し・投入の失敗、ジョブマネージャーがない場合。
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

    /// 待ちのジョブを実行させ、実行したジョブの数を返す（外のジョブマネージャーでは、止まっていれば起動し直すだけで 0）
    ///
    /// # Errors
    ///
    /// ジョブマネージャーがない・起動できない場合、ジョブの状態の更新の失敗。
    pub fn run_queued_jobs(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        self.jobs.run_pending()
    }

    /// 加工工程 `operation_id` から投入した切削シミュレーションのジョブ（前の改訂を含む。番号の順）
    fn operation_simulations(
        &self,
        operation_id: MachiningOperationId,
    ) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError> {
        let source_id = operation_id.to_string();
        Ok(self
            .jobs
            .simulation_jobs()?
            .into_iter()
            .filter(|job| {
                job.source
                    .as_ref()
                    .is_some_and(|source| source.source_id == source_id)
            })
            .collect())
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂（最も新しい改訂）で投入し、
    /// 正常に終わった切削シミュレーションのうち、最も新しいジョブの番号（今の改訂の結果がない場合は `None`。工程は
    /// 再計算待ち）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合、ジョブの状態を読めない場合。
    pub fn latest_revision_simulation(
        &self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<Option<u64>, CamSimulationWorkflowError> {
        let revision = sequences
            .operation(sequence_id, operation_id)
            .map_err(CamSimulationWorkflowError::Sequence)?
            .revision();
        Ok(self
            .operation_simulations(operation_id)?
            .into_iter()
            .rev()
            .find(|job| {
                job.status == CamJobStatus::Succeeded
                    && job
                        .source
                        .as_ref()
                        .is_some_and(|source| source.revision == revision)
            })
            .map(|job| job.job_id))
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂で、最後に投入した計算の状態
    /// （今の改訂で投入していなければ `None`）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合、ジョブの状態を読めない場合。
    pub fn latest_revision_job_status(
        &self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<Option<OperationJobStatus>, CamSimulationWorkflowError> {
        let revision = sequences
            .operation(sequence_id, operation_id)
            .map_err(CamSimulationWorkflowError::Sequence)?
            .revision();
        let source_id = operation_id.to_string();
        let current = |source: &Option<CamJobSource>| {
            source
                .as_ref()
                .is_some_and(|source| source.source_id == source_id && source.revision == revision)
        };
        let pending = self.jobs.pending_submissions()?;
        // ジョブマネージャーの取り込みを待っている投入は、ジョブより新しい
        if pending
            .iter()
            .any(|submission| !submission.rejected && current(&submission.source))
        {
            return Ok(Some(OperationJobStatus::Queued));
        }
        let latest = self
            .operation_simulations(operation_id)?
            .into_iter()
            .rev()
            .find(|job| current(&job.source));
        let Some(job) = latest else {
            // 取り込めなかった投入は、失敗として出す
            let rejected = pending
                .iter()
                .any(|submission| submission.rejected && current(&submission.source));
            return Ok(rejected.then_some(OperationJobStatus::Failed));
        };
        Ok(Some(match job.status {
            CamJobStatus::Succeeded => OperationJobStatus::Succeeded,
            CamJobStatus::Failed => OperationJobStatus::Failed,
            CamJobStatus::Canceled => OperationJobStatus::Canceled,
            CamJobStatus::Running => OperationJobStatus::Running,
            // 切削シミュレーションは、ツールパス生成の計算の間も待っている
            CamJobStatus::Queued | CamJobStatus::NeedsRecompute => match job.toolpath_status {
                CamJobStatus::Running => OperationJobStatus::Running,
                CamJobStatus::Failed => OperationJobStatus::Failed,
                CamJobStatus::Canceled => OperationJobStatus::Canceled,
                _ => OperationJobStatus::Queued,
            },
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
        self.jobs.import(simulation_job_id)
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
                    .filter(|error| !matches!(error, CamConditionError::TooManyDexelLines { .. })),
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
        errors.push(CamConditionError::from_toolpath_input(
            error,
            request.solver_input.tool.diameter(),
        ));
    }
    errors.extend(
        cutting_simulation_input_errors(&simulation_input(request))
            .into_iter()
            .map(CamConditionError::from_simulation_input),
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
