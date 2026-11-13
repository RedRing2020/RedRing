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
    CamSolverError, CamSolverInput, InverseOffsetShape, ScanlineInputError, SolverGeometry,
    build_inverse_offset_shape, estimate_toolpath_points,
};
use cam_core::{CoordinateFrame, LengthUnit, StockMargins};
use cam_entity::{MachiningOperation, MachiningSequence};
use cam_sim::{
    CuttingSimulationInput, CuttingSimulationInputError, SnapshotInterval,
    TOOLPATH_MEMORY_LIMIT_BYTES, cutting_simulation_input_errors, stock_around_product,
    toolpath_memory_bytes,
};
/// 軸に平行な直方体の型（素材の形に使う）
pub use geo_algorithms::Aabb3D;
use geo_algorithms::TessellationLimits;
use job_domain::{CamJobSource, CamJobStatus};

use crate::imported_toolpath::ImportedToolpathStore;
use crate::job_client::{
    JobClient, JobStatusSnapshot, JobSubmission, PipeJobClient, SimulationJob,
    SimulationSubmission, SimulationToolpath,
};
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
    /// ツールパス生成の入力（空間モデルの CL を入力にした切削シミュレーションは `None`）
    pub solver_input: Option<CamSolverInput>,
    /// 切削シミュレーションの入力
    pub simulation_input: CuttingSimulationInput,
    /// 切削シミュレーションのジョブの投入元（工程から投入していなければ `None`）
    pub(crate) source: Option<CamJobSource>,
}

/// 取り込んだツールパス生成の成果物（CL）と、そのジョブの投入元
#[derive(Debug, Clone)]
pub struct CamToolpathResult {
    /// ツールパスの成果物のバイト列（形式は外に見せない）
    pub(crate) artifact: Vec<u8>,
    /// ツールパス生成のジョブの投入元（工程から投入していなければ `None`）
    pub(crate) source: Option<CamJobSource>,
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

impl OperationJobStatus {
    /// 計算待ち・計算中・正常に終わったか（同じ条件の計算を、重ねて投入しない）
    pub fn is_waiting_or_done(self) -> bool {
        matches!(self, Self::Queued | Self::Running | Self::Succeeded)
    }
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

/// 加工工程の今の改訂の計算の状態（読み終えるのを待たない読み取りの結果）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationJobState {
    /// 計算の状態を、まだ読めていない（初めて読む前・前に読んだ状態が古い）
    NotRead,
    /// 今の改訂で投入していない（ジョブマネージャーがないときも）
    NotSubmitted,
    Status(OperationJobStatus),
}

/// 投入・取り込みの失敗
#[derive(Debug, Clone, PartialEq)]
pub enum CamSimulationWorkflowError {
    /// 入力の登録の失敗（理由）
    InputRegistration(String),
    /// 正常終了した切削シミュレーションのジョブではない（ジョブの番号）
    NotSucceededSimulation(u64),
    /// 正常終了したツールパス生成のジョブではない（ジョブの番号）
    NotSucceededToolpath(u64),
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
    /// ジョブマネージャーが動いていない（計算の状態を読むときは、起動しない）
    JobManagerNotRunning,
    /// ジョブの保存先を読み書きできない・ジョブマネージャーを起動できない（理由）
    JobStore(String),
    /// 加工工程の今の条件の CL を、空間モデルに取り込んでいない
    NoCurrentImportedToolpath,
    /// 見積もったツールパスが、ツールパス生成のメモリの上限を超える
    ToolpathTooLarge(ToolpathEstimate),
}

impl fmt::Display for CamSimulationWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputRegistration(reason) => write!(f, "input registration failed: {reason}"),
            Self::NotSucceededSimulation(job_id) => {
                write!(f, "job {job_id} is not a succeeded cutting simulation")
            }
            Self::NotSucceededToolpath(job_id) => {
                write!(f, "job {job_id} is not a succeeded toolpath generation")
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
            Self::JobManagerNotRunning => write!(f, "the job manager is not running"),
            Self::JobStore(reason) => write!(f, "job store failed: {reason}"),
            Self::NoCurrentImportedToolpath => {
                write!(f, "no imported toolpath of the current conditions")
            }
            Self::ToolpathTooLarge(estimate) => write!(
                f,
                "the estimated toolpath ({} points, {} bytes) exceeds the memory limit",
                estimate.points, estimate.memory_bytes
            ),
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
    /// この利用者のジョブマネージャーに問い合わせる口を通して、アプリと同じフォルダのジョブマネージャーに計算させる
    fn default() -> Self {
        Self {
            jobs: Box::new(PipeJobClient::from_environment()),
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

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_ids` のそれぞれについて、投入の前に計算条件の
    /// 誤りをすべて確かめ、`operation_ids` の順に返す（[`Self::check_operation`] と同じ判定）
    ///
    /// 加工の対象は手順で共有するため、その形状は 1 回だけ作る（工程がなければ作らない）。加工の対象の面を形状に
    /// できない場合・形状が空の場合は、手順にある工程のすべてにその誤りを返す。
    pub fn check_operations(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_ids: &[MachiningOperationId],
    ) -> Vec<Result<(), CamSimulationWorkflowError>> {
        if operation_ids.is_empty() {
            return Vec::new();
        }
        let sequence = match sequences.sequence(sequence_id) {
            Ok(sequence) => sequence,
            Err(error) => {
                return vec![Err(CamSimulationWorkflowError::Sequence(error)); operation_ids.len()];
            }
        };
        let mut target = resolve_target_geometry(shapes, sequence);
        let mut results = Vec::with_capacity(operation_ids.len());
        for &operation_id in operation_ids {
            // 工程を探してから形状の誤りを返す（[`Self::check_operation`] と同じ順）
            let operation = match sequences.operation(sequence_id, operation_id) {
                Ok(operation) => operation,
                Err(error) => {
                    results.push(Err(CamSimulationWorkflowError::Sequence(error)));
                    continue;
                }
            };
            let (geometry, bounds) = match &mut target {
                Ok((geometry, bounds)) => (geometry, *bounds),
                Err(error) => {
                    results.push(Err(error.clone()));
                    continue;
                }
            };
            // 形状を入力に渡し、確かめた後すぐに戻して、次の工程の確かめに使い回す（この間で抜けない）
            let (mut request, errors) =
                operation_request(sequence, operation, geometry_take(geometry), bounds);
            *geometry = geometry_take(&mut request.solver_input.geometry);
            results.push(if errors.is_empty() {
                Ok(())
            } else {
                Err(CamSimulationWorkflowError::Conditions(errors))
            });
        }
        results
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` のツールパスの大きさを、計算を始める前に
    /// 見積もり、確かめずに計算してよいか・確かめてから計算するかを返す
    ///
    /// 見積もれないときは、確かめずに計算してよいとする（計算すれば、計算の失敗として理由が分かる）。
    ///
    /// # Errors
    ///
    /// 投入前の確認（[`Self::check_operation`]）の誤り、見積もったツールパスがメモリの上限を超える場合。
    pub fn check_toolpath_size(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<ToolpathSize, CamSimulationWorkflowError> {
        let (request, _) = resolve_operation_request(shapes, sequences, sequence_id, operation_id)?;
        let started = std::time::Instant::now();
        match estimate_toolpath_points(&request.solver_input) {
            Ok(points) => {
                tracing::info!(
                    points,
                    memory_bytes = toolpath_memory_bytes(points),
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    "Estimated the toolpath size before the calculation"
                );
                toolpath_size(points)
            }
            Err(error) => {
                tracing::warn!(
                    "Cannot estimate the toolpath size, calculating without asking: {error}"
                );
                Ok(ToolpathSize::Normal)
            }
        }
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
    /// 投入する（計算を待たない）。2 つのジョブは、投入元として工程の ID と今の改訂の番号・改訂の鍵を持つ
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
        self.enqueue(request, Some(operation_source(operation)))
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の切削シミュレーションのジョブだけを投入する
    /// （計算を待たない）。工程の今の改訂で正常に終わり、切削シミュレーションを置いていないツールパス生成のジョブがあれば、
    /// その下に置いて成果物を読み、なければ空間モデルに取り込んだ今の条件の CL（`toolpaths`）を入力にする（ツールパス生成の
    /// ジョブ 1 つに、切削シミュレーションは 1 つだけ置く）。ジョブは、投入元として工程の ID と今の改訂の番号・
    /// 改訂の鍵を持つ
    ///
    /// # Errors
    ///
    /// 今の条件の CL を取り込んでいない場合、投入前の確認（[`Self::check_operation`]）の誤り、入力の書き出し・投入の
    /// 失敗、ジョブマネージャーがない場合。
    pub fn submit_imported_toolpath(
        &mut self,
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        toolpaths: &ImportedToolpathStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        let toolpath_artifact = toolpaths
            .current_artifact(sequences, sequence_id, operation_id)
            .map_err(CamSimulationWorkflowError::Sequence)?
            .ok_or(CamSimulationWorkflowError::NoCurrentImportedToolpath)?;
        let (request, operation) =
            resolve_operation_request(shapes, sequences, sequence_id, operation_id)?;
        let current = current_revision(sequences, sequence_id, operation_id)?;
        let toolpath_job = self.jobs.toolpath_jobs()?.into_iter().rev().find(|job| {
            job.status == CamJobStatus::Succeeded && !job.simulated && current(&job.source)
        });
        let toolpath = match toolpath_job {
            Some(job) => SimulationToolpath::Job(job.job_id),
            None => SimulationToolpath::Artifact(toolpath_artifact.to_vec()),
        };
        let submission_id = self.jobs.submit_simulation(SimulationSubmission {
            simulation_input: simulation_input(&request),
            toolpath,
            source: Some(operation_source(operation)),
        })?;
        Ok(CamSimulationSubmission { submission_id })
    }

    /// 同じプロセスの中で実行するときに、待ちのジョブを実行し、実行したジョブの数を返す（外のジョブマネージャーは、
    /// 待ちのジョブを自分で実行するため、何もせず 0）
    ///
    /// # Errors
    ///
    /// 同じプロセスの中で実行するときの、ジョブの状態の更新の失敗。
    pub fn run_queued_jobs(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        self.jobs.run_pending()
    }

    /// ジョブマネージャーのウィンドウを開く（動いていなければ、ウィンドウを出して起動する）
    ///
    /// # Errors
    ///
    /// ジョブマネージャーがない・起動できない場合。
    pub fn open_job_manager(&mut self) -> Result<(), CamSimulationWorkflowError> {
        self.jobs.open_manager_window()
    }

    /// 加工工程 `operation_id` の今の改訂の、計算待ち・計算中のジョブ（ツールパス生成と切削シミュレーション）のうち、
    /// 投入 `kept_submission_id`（あれば）のもの以外を取り消すよう頼み、頼んだジョブの数を返す（取り消し終えるのを待たない。
    /// 前の改訂のジョブ・ほかの工程のジョブには触れない。ジョブマネージャーがまだ取り込んでいない投入は取り消せない）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順・加工手順にない加工工程の場合、ジョブマネージャーがない・起動できない・ジョブを
    /// 読めない・取り消しを頼めない場合。
    pub fn cancel_latest_revision_jobs(
        &mut self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
        kept_submission_id: Option<&str>,
    ) -> Result<usize, CamSimulationWorkflowError> {
        let current = current_revision(sequences, sequence_id, operation_id)?;
        let active = |status: CamJobStatus| {
            matches!(
                status,
                CamJobStatus::Queued | CamJobStatus::Running | CamJobStatus::NeedsRecompute
            )
        };
        let kept = |submission_id: &str| kept_submission_id == Some(submission_id);
        // 切削シミュレーションを先に取り消す（ツールパス生成を先に取り消すと、それを待つ切削シミュレーションが、
        // 依存するジョブの取り消しによる失敗になるため）
        let mut job_ids: Vec<u64> = self
            .jobs
            .simulation_jobs()?
            .into_iter()
            .filter(|job| current(&job.source) && active(job.status) && !kept(&job.submission_id))
            .map(|job| job.job_id)
            .collect();
        job_ids.extend(
            self.jobs
                .toolpath_jobs()?
                .into_iter()
                .filter(|job| {
                    current(&job.source) && active(job.status) && !kept(&job.submission_id)
                })
                .map(|job| job.job_id),
        );
        if !job_ids.is_empty() {
            self.jobs.cancel(&job_ids)?;
        }
        Ok(job_ids.len())
    }

    /// 加工工程 `operation_id` から投入した切削シミュレーションのジョブ（前の改訂を含む。番号の順）
    fn operation_simulations(
        &self,
        operation_id: MachiningOperationId,
    ) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError> {
        Ok(of_operation(self.jobs.simulation_jobs()?, operation_id))
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂（今の改訂の鍵）で投入し、
    /// 正常に終わった切削シミュレーションのうち、最も新しいジョブの番号（今の改訂の結果がない場合は `None`。工程は
    /// 再計算待ち）。改訂の鍵を持たない投入元（前の版の記録）は、今の改訂とみなさない
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
        let current = current_revision(sequences, sequence_id, operation_id)?;
        Ok(self
            .operation_simulations(operation_id)?
            .into_iter()
            .rev()
            .find(|job| job.status == CamJobStatus::Succeeded && current(&job.source))
            .map(|job| job.job_id))
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂（今の改訂の鍵）で投入し、
    /// 正常に終わったツールパス生成のうち、最も新しいジョブの番号（今の改訂の結果がない場合は `None`）。改訂の鍵を
    /// 持たない投入元（前の版の記録）は、今の改訂とみなさない
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合、ジョブの状態を読めない場合。
    pub fn latest_revision_toolpath(
        &self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<Option<u64>, CamSimulationWorkflowError> {
        let current = current_revision(sequences, sequence_id, operation_id)?;
        Ok(self
            .jobs
            .toolpath_jobs()?
            .into_iter()
            .rev()
            .find(|job| job.status == CamJobStatus::Succeeded && current(&job.source))
            .map(|job| job.job_id))
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_ids` のそれぞれの、今の改訂（今の改訂の鍵）で
    /// 最後に投入した計算の状態（`operation_ids` の順。同じ 1 回の読み取りの状態から決める。今の改訂で投入していなければ
    /// `OperationJobState::NotSubmitted`）。改訂の鍵を持たない投入元（前の版の記録）は、今の改訂とみなさない
    ///
    /// 状態は読み終えるのを待たずに、最後に読んだものから決める（画面を止めないように。まだ読めていなければ
    /// `OperationJobState::NotRead`）。ジョブマネージャーは起動しない。
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合、ジョブの状態を読めない場合（ジョブマネージャーが
    /// 動いていない・見つからない場合を含む）。
    pub fn latest_revision_job_states(
        &self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_ids: &[MachiningOperationId],
    ) -> Result<Vec<OperationJobState>, CamSimulationWorkflowError> {
        let currents = operation_ids
            .iter()
            .map(|&operation_id| current_revision(sequences, sequence_id, operation_id))
            .collect::<Result<Vec<_>, _>>()?;
        // ジョブマネージャーが動いていない・見つからないときは、状態が分からないため誤りにする（投入していないとは
        // みなさない。計算済みのジョブがあるかもしれないため）
        let Some(snapshot) = self.jobs.status_snapshot()? else {
            return Ok(vec![OperationJobState::NotRead; operation_ids.len()]);
        };
        Ok(operation_ids
            .iter()
            .zip(&currents)
            .map(|(&operation_id, current)| operation_state(&snapshot, operation_id, current))
            .collect())
    }

    /// [`Self::latest_revision_job_states`] と同じく、加工工程の今の改訂の計算の状態を、ジョブマネージャーに今問い
    /// 合わせて返す（動いていなければ起動して、口が開くのを待つ。画面を止める）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順・加工手順にない加工工程の場合、ジョブマネージャーが見つからない・起動できない・状態を
    /// 読めない場合。
    pub fn latest_revision_job_states_now(
        &mut self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_ids: &[MachiningOperationId],
    ) -> Result<Vec<OperationJobState>, CamSimulationWorkflowError> {
        let currents = operation_ids
            .iter()
            .map(|&operation_id| current_revision(sequences, sequence_id, operation_id))
            .collect::<Result<Vec<_>, _>>()?;
        let snapshot = self.jobs.read_status_now()?;
        Ok(operation_ids
            .iter()
            .zip(&currents)
            .map(|(&operation_id, current)| operation_state(&snapshot, operation_id, current))
            .collect())
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂で、ジョブマネージャーがまだ
    /// 受け取っていない投入があるか（ジョブマネージャーに今問い合わせる。動いていなければ起動して、口が開くのを待つ）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順・加工手順にない加工工程の場合、ジョブマネージャーが見つからない・起動できない・状態を
    /// 読めない場合。
    pub fn latest_revision_has_pending_now(
        &mut self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<bool, CamSimulationWorkflowError> {
        let current = current_revision(sequences, sequence_id, operation_id)?;
        let snapshot = self.jobs.read_status_now()?;
        Ok(snapshot
            .pending
            .iter()
            .any(|submission| !submission.rejected && current(&submission.source)))
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

    /// 正常終了したツールパス生成のジョブ `toolpath_job_id` の成果物（CL）と、その投入元
    ///
    /// # Errors
    ///
    /// ジョブが正常終了したツールパス生成でない場合、成果物がない場合、問い合わせの失敗。
    pub fn toolpath_result(
        &self,
        toolpath_job_id: u64,
    ) -> Result<CamToolpathResult, CamSimulationWorkflowError> {
        self.jobs.toolpath_result(toolpath_job_id)
    }

    /// 切削シミュレーションのジョブ `simulation_job_id` の結果（その CL）を、空間モデルに取り込んだと伝える（ジョブ
    /// マネージャーが、取り込み済みの CL の元をまとめて消せるように）
    ///
    /// # Errors
    ///
    /// 正常終了した切削シミュレーションのジョブでない場合、問い合わせの失敗。
    pub fn record_imported(
        &self,
        simulation_job_id: u64,
    ) -> Result<(), CamSimulationWorkflowError> {
        self.jobs.record_imported(simulation_job_id)
    }
}

/// 計算の状態 `snapshot` の中の、加工工程 `operation_id` の今の改訂（`current` で見分ける）の計算の状態
fn operation_state(
    snapshot: &JobStatusSnapshot,
    operation_id: MachiningOperationId,
    current: &impl Fn(&Option<CamJobSource>) -> bool,
) -> OperationJobState {
    // ジョブマネージャーの取り込みを待っている投入は、ジョブより新しい
    if snapshot
        .pending
        .iter()
        .any(|submission| !submission.rejected && current(&submission.source))
    {
        return OperationJobState::Status(OperationJobStatus::Queued);
    }
    let source_id = operation_id.to_string();
    let latest = snapshot
        .simulations
        .iter()
        .filter(|job| {
            job.source
                .as_ref()
                .is_some_and(|source| source.source_id == source_id)
        })
        .rev()
        .find(|job| current(&job.source));
    let Some(job) = latest else {
        // 取り込めなかった投入は、失敗として出す
        let rejected = snapshot
            .pending
            .iter()
            .any(|submission| submission.rejected && current(&submission.source));
        return if rejected {
            OperationJobState::Status(OperationJobStatus::Failed)
        } else {
            OperationJobState::NotSubmitted
        };
    };
    OperationJobState::Status(match job.status {
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
    })
}

/// 切削シミュレーションのジョブ `simulations` のうち、加工工程 `operation_id` から投入したもの（順はそのまま）
fn of_operation(
    simulations: Vec<SimulationJob>,
    operation_id: MachiningOperationId,
) -> Vec<SimulationJob> {
    let source_id = operation_id.to_string();
    simulations
        .into_iter()
        .filter(|job| {
            job.source
                .as_ref()
                .is_some_and(|source| source.source_id == source_id)
        })
        .collect()
}

/// 加工工程 `operation` から投入するジョブの投入元（工程の ID と今の改訂の番号・改訂の鍵）
fn operation_source(operation: &MachiningOperation) -> CamJobSource {
    CamJobSource {
        source_id: operation.id().to_string(),
        revision: operation.revision(),
        revision_key: Some(operation.revision_key().to_string()),
    }
}

/// 投入元が、保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂か（工程の ID と改訂の鍵が
/// 同じか）を判定する関数。改訂の鍵を持たない投入元は、今の改訂とみなさない
fn current_revision(
    sequences: &MachiningSequenceStore,
    sequence_id: MachiningSequenceId,
    operation_id: MachiningOperationId,
) -> Result<impl Fn(&Option<CamJobSource>) -> bool, CamSimulationWorkflowError> {
    let revision_key = sequences
        .operation(sequence_id, operation_id)
        .map_err(CamSimulationWorkflowError::Sequence)?
        .revision_key()
        .to_string();
    let source_id = operation_id.to_string();
    Ok(move |source: &Option<CamJobSource>| {
        source.as_ref().is_some_and(|source| {
            source.source_id == source_id
                && source.revision_key.as_deref() == Some(revision_key.as_str())
        })
    })
}

/// ツールパス生成のメモリの見積もりが、計算の前に確かめる目安（バイト。2 GB）
const TOOLPATH_CONFIRM_BYTES: u64 = 2_000_000_000;

/// 計算を始める前に見積もったツールパスの大きさ（桁が合う程度）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolpathEstimate {
    /// パスの点の数
    pub points: u64,
    /// 生成に使うメモリ（バイト）
    pub memory_bytes: u64,
}

impl ToolpathEstimate {
    /// 生成に使うメモリ（GB）
    pub fn memory_gb(&self) -> f64 {
        self.memory_bytes as f64 / 1e9
    }

    /// 生成に使うメモリが、メモリの上限に近いか（上限の 8 割以上。見積もりの幅のため、計算しても途中で上限を
    /// 超えて止まることがある）
    pub fn is_near_limit(&self) -> bool {
        self.memory_bytes >= TOOLPATH_MEMORY_LIMIT_BYTES / 10 * 8
    }
}

/// 見積もったツールパスの大きさで決める、計算の始め方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolpathSize {
    /// 確かめずに計算してよい
    Normal,
    /// 確かめる目安を超えるため、確かめてから計算する
    Large(ToolpathEstimate),
}

/// 点 `points` 個のツールパスの計算の始め方
///
/// # Errors
///
/// メモリの見積もりがツールパス生成のメモリの上限を超える場合。
fn toolpath_size(points: u64) -> Result<ToolpathSize, CamSimulationWorkflowError> {
    let estimate = ToolpathEstimate {
        points,
        memory_bytes: toolpath_memory_bytes(points),
    };
    if estimate.memory_bytes > TOOLPATH_MEMORY_LIMIT_BYTES {
        Err(CamSimulationWorkflowError::ToolpathTooLarge(estimate))
    } else if estimate.memory_bytes >= TOOLPATH_CONFIRM_BYTES {
        Ok(ToolpathSize::Large(estimate))
    } else {
        Ok(ToolpathSize::Normal)
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
    let (geometry, bounds) = resolve_target_geometry(shapes, sequence)?;
    let (workflow_request, errors) = operation_request(sequence, operation, geometry, bounds);
    if errors.is_empty() {
        Ok((workflow_request, operation))
    } else {
        Err(CamSimulationWorkflowError::Conditions(errors))
    }
}

/// 加工手順 `sequence` の加工の対象の面から、ツールパス生成の入力の形状と、それを囲む箱を作る
fn resolve_target_geometry(
    shapes: &ProductShapeStore,
    sequence: &MachiningSequence,
) -> Result<(SolverGeometry, Aabb3D<f64>), CamSimulationWorkflowError> {
    let geometry = shapes
        .resolve_solver_geometry(sequence.target_faces())
        .map_err(CamSimulationWorkflowError::ProductShape)?;
    let bounds = geometry
        .aabb()
        .ok_or(CamSimulationWorkflowError::EmptyProductShape)?;
    Ok((geometry, bounds))
}

/// 形状 `geometry` を取り出し、空の形状を残す（大きな形状を写さずに、入力の間で受け渡すため）
fn geometry_take(geometry: &mut SolverGeometry) -> SolverGeometry {
    std::mem::replace(geometry, SolverGeometry::NurbsSurfaceSet(Vec::new()))
}

/// 加工手順 `sequence` の加工工程 `operation` と、加工の対象の形状 `geometry`（囲む箱 `bounds`）から、ツールパス生成から
/// 切削シミュレーションまでの入力を組み立て、計算条件の誤りをすべて返す（誤りがあっても、入力を返す）
///
/// 余裕が不正な場合は素材を決められないため、製品形状を囲む箱を素材として残りの条件を確かめ、
/// 素材の大きさで決まる線の本数は判定しない。
fn operation_request(
    sequence: &MachiningSequence,
    operation: &MachiningOperation,
    geometry: SolverGeometry,
    bounds: Aabb3D<f64>,
) -> (CamSimulationWorkflowRequest, Vec<CamConditionError>) {
    let stock_margins = sequence.stock().margins;
    let stock = stock_around_product(bounds, stock_margins);
    // 計算の入力は、工程の計算の条件だけから作る（属性は使わない）
    let conditions = &operation.content().conditions;
    let solver_input = CamSolverInput {
        operation_id: operation.id().to_string(),
        tool: conditions.tool.clone(),
        geometry,
        operation: conditions.spec,
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: conditions.tolerance_profile.tolerance_mm(),
        tessellation_limits: TessellationLimits::default(),
        boundary: conditions.boundary,
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
    (workflow_request, errors)
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
        cutting_simulation_input_errors(
            &request.stock,
            &request.solver_input.tool,
            request.precision,
            &request.snapshot_interval,
        )
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
