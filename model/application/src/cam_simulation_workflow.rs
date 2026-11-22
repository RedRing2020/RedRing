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

mod segment;
#[cfg(test)]
mod tests;

/// 逆オフセット形状の表示用の三角形と、その種類
pub use cam_algorithms::{InverseOffsetPart, InverseOffsetPartKind};
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

use cam_algorithms::{
    CamSolverError, CamSolverInput, InverseOffsetShape, ScanlineInputError, SolverGeometry,
    build_inverse_offset_shape, estimate_toolpath_points,
};
use cam_core::{CoordinateFrame, LengthUnit, StockMargins};
use cam_entity::{MachiningOperation, MachiningSequence};
use cam_sim::{
    CuttingSimulationInput, CuttingSimulationInputError, SnapshotInterval,
    cutting_simulation_input_errors, stock_around_product, toolpath_memory_bytes,
};
/// 軸に平行な直方体の型（素材の形に使う）
pub use geo_algorithms::Aabb3D;
use geo_algorithms::TessellationLimits;
use job_domain::{CamJobSource, CamJobStatus};
/// ツールパスの大きさの、計算の前に確かめる目安と上限
pub use job_ipc::ToolpathLimits;
/// 計算の失敗の理由と、その種類
pub use job_ipc::{JobFailure, JobFailureReason};

use crate::imported_toolpath::ImportedToolpathStore;
use crate::job_client::{
    JobClient, JobStatusSnapshot, PipeJobClient, SegmentSubmission, SegmentToolpath, SimulationJob,
    StartStock, ToolpathJob,
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
    /// 切削シミュレーションの格子を決める工具の半径（手順のすべての区間で同じ）
    pub grid_tool_radius: f64,
    /// スナップショットの記録の間隔
    pub snapshot_interval: SnapshotInterval,
}

/// 投入（ジョブの番号はジョブマネージャーが振るため、投入の ID で引く）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CamSimulationSubmission {
    /// 投入の ID
    pub submission_id: String,
}

/// 投入する区間の切削シミュレーションが入力にする CL の決め方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentToolpathSource {
    /// ツールパス生成から計算し直す
    Solve,
    /// 今の条件の CL（今の改訂で正常に終わったツールパス生成のジョブの成果物か、空間モデルに取り込んだ CL）があれば
    /// それを使い、なければツールパス生成から計算する
    Current,
    /// 今の条件の CL を使う（空間モデルに取り込んでいなければ誤り）
    Imported,
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
    /// 計算の途中で、ジョブマネージャーの終了・異常終了・OS の再起動で止まった（計算で続けられる）
    Interrupted,
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
            // 格子を決める工具の半径は、工程の工具の半径から決める
            CuttingSimulationInputError::Tool | CuttingSimulationInputError::GridToolRadius => {
                Self::InvalidTool
            }
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
    /// 動いているジョブマネージャーの口の版が、このアプリと違う（前の版のジョブマネージャーが動き続けている）
    JobManagerOutdated,
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
            Self::JobManagerOutdated => {
                write!(
                    f,
                    "the running job manager is a different version from this app"
                )
            }
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

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_ids` を確かめる（[`Self::check_operation`]・
    /// [`Self::check_operations`]）ときに要る加工の対象の形状を、工程の精度で作り終えているか
    ///
    /// 作り終えていない形状は、別のスレッドで作り始める（シェルの面を三角形に分ける。待たない）。保管していない加工
    /// 手順・加工手順にない加工工程は、作るものがないとする（確かめで、その誤りを返す）。
    pub fn prepare_check(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_ids: &[MachiningOperationId],
    ) -> bool {
        let Ok(sequence) = sequences.sequence(sequence_id) else {
            return true;
        };
        let mut tolerances: Vec<u64> = operation_ids
            .iter()
            .filter_map(|&operation_id| sequences.operation(sequence_id, operation_id).ok())
            .map(|operation| operation_tolerance(operation).to_bits())
            .collect();
        tolerances.sort_unstable();
        tolerances.dedup();
        // すべての精度の形状を作り始めるため、途中でやめない
        tolerances.into_iter().fold(true, |ready, tolerance| {
            let prepared =
                shapes.prepare_solver_geometry(sequence.target_faces(), f64::from_bits(tolerance));
            ready && prepared
        })
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_ids` のそれぞれについて、投入の前に計算条件の
    /// 誤りをすべて確かめ、`operation_ids` の順に返す（[`Self::check_operation`] と同じ判定）
    ///
    /// 加工の対象は手順で共有するため、その形状は工程の精度（シェルの面を三角形に分ける許容弦誤差）ごとに 1 回だけ
    /// 作る（工程がなければ作らない）。加工の対象の面を形状にできない場合・形状が空の場合は、その精度の工程の
    /// すべてにその誤りを返す。
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
        // 許容弦誤差（ビット列）ごとの加工の対象の形状
        let mut targets: Vec<(u64, TargetGeometry)> = Vec::new();
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
            let tolerance = operation_tolerance(operation);
            let key = tolerance.to_bits();
            let index = match targets.iter().position(|(cached, _)| *cached == key) {
                Some(index) => index,
                None => {
                    targets.push((key, resolve_target_geometry(shapes, sequence, tolerance)));
                    targets.len() - 1
                }
            };
            let (geometry, bounds) = match &mut targets[index].1 {
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

    /// ツールパスの大きさの、計算の前に確かめる目安と上限（ジョブマネージャーが、起動のときに設定から決めたもの。
    /// 動いていなければ起動して問い合わせる）
    ///
    /// # Errors
    ///
    /// ジョブマネージャーに問い合わせられない場合。
    pub fn toolpath_limits(&self) -> Result<ToolpathLimits, CamSimulationWorkflowError> {
        self.jobs.toolpath_limits()
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` のツールパスの大きさを、計算を始める前に
    /// 別のスレッドで見積もり始める（待たない。見積もりは [`ToolpathSizeEstimation::size`] で受け取る）
    ///
    /// 計算の入力は、このスレッドで組み立てて確かめる。
    ///
    /// # Errors
    ///
    /// 投入前の確認（[`Self::check_operation`]）の誤り。
    pub fn estimate_toolpath_size(
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<ToolpathSizeEstimation, CamSimulationWorkflowError> {
        let (request, _) = resolve_operation_request(shapes, sequences, sequence_id, operation_id)?;
        let points = Arc::new(Mutex::new(None));
        let estimated = Arc::clone(&points);
        let estimate = move || {
            let started = std::time::Instant::now();
            // 見積もりの途中で panic しても、見積もれなかったとして結果を残す（見積もり中のままにしない）
            let estimated_points = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                estimate_toolpath_points(&request.solver_input)
            }));
            let result = match estimated_points {
                Err(_) => {
                    tracing::warn!(
                        "The toolpath size estimate panicked, calculating without asking"
                    );
                    None
                }
                Ok(Ok(points)) => {
                    tracing::info!(
                        points,
                        memory_bytes = toolpath_memory_bytes(points),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        "Estimated the toolpath size before the calculation"
                    );
                    Some(points)
                }
                Ok(Err(error)) => {
                    tracing::warn!(
                        "Cannot estimate the toolpath size, calculating without asking: {error}"
                    );
                    None
                }
            };
            *estimated.lock().unwrap_or_else(PoisonError::into_inner) = Some(result);
        };
        if let Err(error) = std::thread::Builder::new()
            .name("toolpath-estimate".to_string())
            .spawn(estimate)
        {
            // スレッドを作れなければ、見積もらずに計算してよいとする（計算すれば、計算の失敗として理由が分かる）
            tracing::warn!("Cannot start a thread to estimate the toolpath size: {error}");
            *points.lock().unwrap_or_else(PoisonError::into_inner) = Some(None);
        }
        Ok(ToolpathSizeEstimation { points })
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

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operations`（工程と、その切削シミュレーションが入力にする CL
    /// の決め方）の区間のジョブを、1 つの投入で置く（計算を待たない）
    ///
    /// 区間の切削シミュレーションは、加工の順に前の区間の終わりの素材から削る。`operations` の最も後の工程までのうち、
    /// 今の結果（区間の前提が今の手順と同じ、計算待ち・計算中・中断・正常に終わったジョブ）がない区間も、前の区間から
    /// 同じ投入に置く（その CL は [`SegmentToolpathSource::Current`] で決める）。今の結果がある区間は置かず、後の区間は
    /// その終わりの素材から削る。今の条件の CL は、今の改訂で正常に終わったツールパス生成のジョブ（切削シミュレーション
    /// を置いていないものを先に）、空間モデルに取り込んだ今の条件の CL（`toolpaths`）の順に探す。ジョブは、投入元として
    /// 工程の ID と今の改訂の番号・改訂の鍵を持ち、切削シミュレーションのジョブは区間の前提の鍵も持つ。計算の状態は、
    /// ジョブマネージャーに今問い合わせる
    ///
    /// # Errors
    ///
    /// 置く区間の工程の投入前の確認（[`Self::check_operation`]）の誤り、CL を今の条件の CL にする工程が今の条件の CL を
    /// 取り込んでいない場合、入力の書き出し・投入の失敗、ジョブマネージャーがない・状態を読めない場合。
    pub fn submit_operations(
        &mut self,
        shapes: &ProductShapeStore,
        sequences: &MachiningSequenceStore,
        toolpaths: &ImportedToolpathStore,
        sequence_id: MachiningSequenceId,
        operations: &[(MachiningOperationId, SegmentToolpathSource)],
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        let sequence = sequences
            .sequence(sequence_id)
            .map_err(CamSimulationWorkflowError::Sequence)?;
        for &(operation_id, _) in operations {
            sequences
                .operation(sequence_id, operation_id)
                .map_err(CamSimulationWorkflowError::Sequence)?;
        }
        let keys = segment::segment_keys(sequence);
        let requested = |operation_id| {
            operations
                .iter()
                .find(|(requested, _)| *requested == operation_id)
                .map(|&(_, source)| source)
        };
        let Some(last) = keys.iter().rposition(|(id, _)| requested(*id).is_some()) else {
            return Err(CamSimulationWorkflowError::InputRegistration(
                "no operations to submit".to_string(),
            ));
        };
        let snapshot = self.jobs.read_status_now()?;
        let toolpath_jobs = self.jobs.toolpath_jobs()?;
        let mut segments: Vec<SegmentSubmission> = Vec::new();
        let mut start_stock = None;
        for (operation_id, segment_key) in &keys[..=last] {
            let operation_id = *operation_id;
            let source = requested(operation_id);
            let current =
                current_segment(sequences, sequence_id, operation_id, segment_key.clone())?;
            let reusable = snapshot
                .simulations
                .iter()
                .rev()
                .find(|job| current(&job.source) && can_finish(job));
            if source.is_none()
                && let Some(job) = reusable
            {
                start_stock = Some(StartStock::Job(job.job_id));
                continue;
            }
            let (request, operation) =
                resolve_operation_request(shapes, sequences, sequence_id, operation_id)?;
            let revision = current_revision(sequences, sequence_id, operation_id)?;
            let toolpath_job = reusable_toolpath_job(&toolpath_jobs, revision);
            let imported = toolpaths
                .current_artifact(sequences, sequence_id, operation_id)
                .map_err(CamSimulationWorkflowError::Sequence)?;
            let simulation_input = simulation_input(&request);
            let toolpath = match (source, toolpath_job, imported) {
                (Some(SegmentToolpathSource::Solve), _, _) => {
                    SegmentToolpath::Solve(Box::new(request.solver_input))
                }
                (Some(SegmentToolpathSource::Imported), _, None) => {
                    return Err(CamSimulationWorkflowError::NoCurrentImportedToolpath);
                }
                (_, Some(job_id), _) => SegmentToolpath::Job(job_id),
                (_, None, Some(artifact)) => SegmentToolpath::Artifact(artifact.to_vec()),
                (Some(SegmentToolpathSource::Current) | None, None, None) => {
                    SegmentToolpath::Solve(Box::new(request.solver_input))
                }
            };
            segments.push(SegmentSubmission {
                toolpath,
                simulation_input,
                start_stock,
                source: Some(CamJobSource {
                    segment_key: Some(segment_key.clone()),
                    ..operation_source(operation)
                }),
            });
            start_stock = Some(StartStock::Submitted(segments.len() - 1));
        }
        let submission_id = self.jobs.submit(segments)?;
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

    /// 加工工程 `operation_id` の今の改訂の、計算待ち・計算中・中断のジョブ（ツールパス生成と切削シミュレーション）のうち、
    /// 投入 `kept_submission_id`（あれば）のもの以外を取り消すよう頼み、頼んだジョブの数を返す（取り消し終えるのを待たない。
    /// 前の改訂のジョブ・ほかの工程のジョブには触れない。ジョブマネージャーがまだ取り込んでいない投入は取り消せない）
    ///
    /// 後の区間の切削シミュレーションが始まりの素材として待つ切削シミュレーションは、取り消さない（後の区間が、依存する
    /// ジョブの取り消しで失敗しないように）。そのときは、この工程のツールパス生成も取り消さない
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
        let cancels = |status, source: &Option<CamJobSource>, submission_id: &str| {
            is_active(status) && current(source) && kept_submission_id != Some(submission_id)
        };
        let simulations = self.jobs.simulation_jobs()?;
        let waited: std::collections::HashSet<u64> = simulations
            .iter()
            .filter(|job| is_active(job.status))
            .filter_map(|job| job.start_stock)
            .collect();
        let (kept, mut job_ids): (Vec<&SimulationJob>, Vec<&SimulationJob>) = simulations
            .iter()
            .filter(|job| cancels(job.status, &job.source, &job.submission_id))
            .partition(|job| waited.contains(&job.job_id));
        let mut job_ids: Vec<u64> = job_ids.drain(..).map(|job| job.job_id).collect();
        if kept.is_empty() {
            job_ids.extend(
                self.jobs
                    .toolpath_jobs()?
                    .into_iter()
                    .filter(|job| cancels(job.status, &job.source, &job.submission_id))
                    .map(|job| job.job_id),
            );
        }
        if !job_ids.is_empty() {
            self.jobs.cancel(&job_ids)?;
        }
        Ok(job_ids.len())
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂の計算を止めるよう頼み、頼んだ
    /// ジョブの数を返す（止め終えるのを待たない。前の改訂のジョブ・ほかの工程のジョブには触れない）
    ///
    /// 計算中のジョブは止めて中断にし（途中の結果を残し、後から計算を続けられる）、まだ始まっていない計算待ちのジョブは
    /// 取り消す（`stop_plan`）。計算中・中断のツールパス生成を待つ計算待ちの切削シミュレーションは、取り消さずに待た
    /// せる（ツールパス生成を続けたら、そのまま進むように。止めた直後にもう一度止めたときも同じ）。中断のジョブには
    /// 触れない。ジョブマネージャーがまだ取り込んでいない投入は止められない。
    ///
    /// # Errors
    ///
    /// 保管していない加工手順・加工手順にない加工工程の場合、ジョブマネージャーがない・起動できない・ジョブを
    /// 読めない・頼めない場合。
    pub fn stop_latest_revision_jobs(
        &mut self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<usize, CamSimulationWorkflowError> {
        let current = current_revision(sequences, sequence_id, operation_id)?;
        let plan = stop_plan(
            &self.jobs.simulation_jobs()?,
            &self.jobs.toolpath_jobs()?,
            current,
        );
        if !plan.stop.is_empty() {
            self.jobs.stop(&plan.stop)?;
        }
        if !plan.cancel.is_empty() {
            self.jobs.cancel(&plan.cancel)?;
        }
        Ok(plan.stop.len() + plan.cancel.len())
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂の、中断のジョブ（ツールパス
    /// 生成と切削シミュレーション）を計算待ちに戻すよう頼み、頼んだジョブの数を返す（戻し終えるのを待たない。前の改訂
    /// のジョブ・ほかの工程のジョブには触れない）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順・加工手順にない加工工程の場合、ジョブマネージャーがない・起動できない・ジョブを
    /// 読めない・頼めない場合。
    pub fn resume_latest_revision_jobs(
        &mut self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<usize, CamSimulationWorkflowError> {
        let current = current_revision(sequences, sequence_id, operation_id)?;
        let job_ids = self
            .job_ids(|status, source, _| status == CamJobStatus::Interrupted && current(source))?;
        if !job_ids.is_empty() {
            self.jobs.resume(&job_ids)?;
        }
        Ok(job_ids.len())
    }

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_ids` から、今の改訂より前に投入した、計算待ち・
    /// 計算中・中断のジョブ（ツールパス生成と切削シミュレーション）を取り消すよう頼み、頼んだジョブの数を返す（取り消し終える
    /// のを待たない。今の改訂のジョブ・ほかの工程のジョブ・終わったジョブには触れない。改訂の鍵を持たない投入元（前の版
    /// の記録）は、前の改訂とみなす）。ジョブマネージャーが動いていなければ、起動せずに何もしない
    ///
    /// # Errors
    ///
    /// 保管していない加工手順・加工手順にない加工工程の場合、ジョブマネージャーにつなげない（動いていない・見つからない
    /// ときを除く）・ジョブを読めない・取り消しを頼めない場合。
    pub fn cancel_superseded_jobs(
        &mut self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_ids: &[MachiningOperationId],
    ) -> Result<usize, CamSimulationWorkflowError> {
        let revisions = operation_ids
            .iter()
            .map(|&operation_id| current_revision(sequences, sequence_id, operation_id))
            .collect::<Result<Vec<_>, _>>()?;
        let segments = operation_ids
            .iter()
            .map(|&operation_id| current_segment_of(sequences, sequence_id, operation_id))
            .collect::<Result<Vec<_>, _>>()?;
        let source_ids: Vec<String> = operation_ids.iter().map(ToString::to_string).collect();
        if !self.jobs.manager_running()? {
            return Ok(0);
        }
        // 切削シミュレーション（区間の前提の鍵を持つ）は、区間の前提が変わったものも取り消す
        self.cancel_active_jobs(|source, _| {
            let Some(job_source) = source.as_ref() else {
                return false;
            };
            source_ids.contains(&job_source.source_id)
                && if job_source.segment_key.is_some() {
                    !segments.iter().any(|current| current(source))
                } else {
                    !revisions.iter().any(|current| current(source))
                }
        })
    }

    /// 計算待ち・計算中・中断のジョブのうち、投入元と投入の ID が `cancels` に当たるものを取り消すよう頼み、頼んだジョブの数を
    /// 返す（ジョブマネージャーがまだ取り込んでいない投入は取り消せない）
    fn cancel_active_jobs(
        &mut self,
        cancels: impl Fn(&Option<CamJobSource>, &str) -> bool,
    ) -> Result<usize, CamSimulationWorkflowError> {
        let job_ids = self.job_ids(|status, source, submission_id| {
            is_active(status) && cancels(source, submission_id)
        })?;
        if !job_ids.is_empty() {
            self.jobs.cancel(&job_ids)?;
        }
        Ok(job_ids.len())
    }

    /// 状態・投入元・投入の ID が `matches` に当たるジョブの番号（切削シミュレーションを先に、ツールパス生成を後に
    /// 並べる。ツールパス生成を先に取り消すと、それを待つ切削シミュレーションが、依存するジョブの取り消しによる失敗に
    /// なるため）
    fn job_ids(
        &self,
        matches: impl Fn(CamJobStatus, &Option<CamJobSource>, &str) -> bool,
    ) -> Result<Vec<u64>, CamSimulationWorkflowError> {
        let mut job_ids: Vec<u64> = self
            .jobs
            .simulation_jobs()?
            .into_iter()
            .filter(|job| matches(job.status, &job.source, &job.submission_id))
            .map(|job| job.job_id)
            .collect();
        job_ids.extend(
            self.jobs
                .toolpath_jobs()?
                .into_iter()
                .filter(|job| matches(job.status, &job.source, &job.submission_id))
                .map(|job| job.job_id),
        );
        Ok(job_ids)
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
        let current = current_segment_of(sequences, sequence_id, operation_id)?;
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
            .map(|&operation_id| current_segment_of(sequences, sequence_id, operation_id))
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

    /// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂の計算が失敗していれば、その
    /// 理由（失敗していない・理由を受け取れなかったときは `None`）
    ///
    /// [`Self::latest_revision_job_states`] と同じく、最後に読んだ状態から決め、読み終えるのを待たない（まだ読めて
    /// いなければ `None`）。ジョブマネージャーは起動しない。
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合、ジョブの状態を読めない場合（ジョブマネージャーが
    /// 動いていない・見つからない場合を含む）。
    pub fn latest_revision_failure(
        &self,
        sequences: &MachiningSequenceStore,
        sequence_id: MachiningSequenceId,
        operation_id: MachiningOperationId,
    ) -> Result<Option<JobFailure>, CamSimulationWorkflowError> {
        let current = current_segment_of(sequences, sequence_id, operation_id)?;
        Ok(self
            .jobs
            .status_snapshot()?
            .and_then(|snapshot| operation_failure(&snapshot, operation_id, &current)))
    }

    /// ジョブマネージャーに 1 度だけつなぎ、`operation` の中のジョブマネージャーへの問い合わせを、すべてその接続で行う
    /// （動いていなければ起動して、口が開くのを待つ）。状態を今読んでから投入するような 1 回の操作で、つなぐのと起動を
    /// 待つのを 1 回にする。`operation` の中で問い合わせに失敗したら、つなぎ直さずに、後の問い合わせも誤りにする
    ///
    /// # Errors
    ///
    /// ジョブマネージャーが見つからない・起動できない・口が開くのを待つ間に開かない・版が違う場合（`operation` を
    /// 行わない）と、`operation` の誤り。
    pub fn with_job_manager_connection<T, E: From<CamSimulationWorkflowError>>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E> {
        self.jobs.hold_connection()?;
        let result = operation(self);
        self.jobs.release_connection();
        result
    }

    /// [`Self::latest_revision_job_states`] と同じく、加工工程の今の改訂の計算の状態を、ジョブマネージャーに今問い
    /// 合わせて返す（動いていなければ起動して、口が開くのを待つ。画面を止める。
    /// [`Self::with_job_manager_connection`] の中では、その接続で読む）
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
            .map(|&operation_id| current_segment_of(sequences, sequence_id, operation_id))
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
            .any(|submission| !submission.rejected && submitted_from(submission, &current)))
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
        .any(|submission| !submission.rejected && submitted_from(submission, current))
    {
        return OperationJobState::Status(OperationJobStatus::Queued);
    }
    let Some(job) = latest_simulation(snapshot, operation_id, current) else {
        // 取り込めなかった投入は、失敗として出す
        let rejected = snapshot
            .pending
            .iter()
            .any(|submission| submission.rejected && submitted_from(submission, current));
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
        CamJobStatus::Interrupted => OperationJobStatus::Interrupted,
        CamJobStatus::Running => OperationJobStatus::Running,
        // 切削シミュレーションは、ツールパス生成の計算の間も待っている
        CamJobStatus::Queued | CamJobStatus::NeedsRecompute => match job.toolpath_status {
            CamJobStatus::Running => OperationJobStatus::Running,
            CamJobStatus::Failed => OperationJobStatus::Failed,
            CamJobStatus::Canceled => OperationJobStatus::Canceled,
            CamJobStatus::Interrupted => OperationJobStatus::Interrupted,
            _ => OperationJobStatus::Queued,
        },
    })
}

/// まだジョブになっていない投入 `submission` が、投入元が `current` に当たるジョブを持つか
fn submitted_from(
    submission: &crate::job_client::PendingSubmission,
    current: &impl Fn(&Option<CamJobSource>) -> bool,
) -> bool {
    submission
        .sources
        .iter()
        .any(|source| current(&Some(source.clone())))
}

/// 計算の状態 `snapshot` の中の、加工工程 `operation_id` の今の改訂（`current` で見分ける）の計算が失敗していれば、
/// その理由
fn operation_failure(
    snapshot: &JobStatusSnapshot,
    operation_id: MachiningOperationId,
    current: &impl Fn(&Option<CamJobSource>) -> bool,
) -> Option<JobFailure> {
    if operation_state(snapshot, operation_id, current)
        != OperationJobState::Status(OperationJobStatus::Failed)
    {
        return None;
    }
    latest_simulation(snapshot, operation_id, current).and_then(|job| job.failure.clone())
}

/// 計算の状態 `snapshot` の中の、加工工程 `operation_id` の今の改訂（`current` で見分ける）で最後に投入した切削
/// シミュレーションのジョブ
fn latest_simulation<'a>(
    snapshot: &'a JobStatusSnapshot,
    operation_id: MachiningOperationId,
    current: &impl Fn(&Option<CamJobSource>) -> bool,
) -> Option<&'a SimulationJob> {
    let source_id = operation_id.to_string();
    snapshot
        .simulations
        .iter()
        .filter(|job| {
            job.source
                .as_ref()
                .is_some_and(|source| source.source_id == source_id)
        })
        .rev()
        .find(|job| current(&job.source))
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
        segment_key: None,
    }
}

/// 投入元が、保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の改訂か（工程の ID と改訂の鍵が
/// 同じか）を判定する関数。改訂の鍵を持たない投入元は、今の改訂とみなさない
/// 計算の止め方（止めて中断にする計算中のジョブと、取り消す計算待ちのジョブ。どちらも切削シミュレーションを先に、
/// ツールパス生成を後に並べる。ツールパス生成を先に取り消すと、それを待つ切削シミュレーションが、依存するジョブの
/// 取り消しによる失敗になるため）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StopPlan {
    stop: Vec<u64>,
    cancel: Vec<u64>,
}

/// 投入元が `current` に当たるジョブの止め方を、切削シミュレーション `simulations` とツールパス生成 `toolpaths`
/// から決める。計算中のジョブは止め、計算待ちのジョブは取り消す。ただし、計算中・中断のツールパス生成を待つ計算待ちの
/// 切削シミュレーションは、取り消さない（止めたツールパス生成を続けたら、そのまま進むように）
fn stop_plan(
    simulations: &[SimulationJob],
    toolpaths: &[ToolpathJob],
    current: impl Fn(&Option<CamJobSource>) -> bool,
) -> StopPlan {
    let waiting = |status: CamJobStatus| {
        matches!(status, CamJobStatus::Queued | CamJobStatus::NeedsRecompute)
    };
    let mut plan = StopPlan::default();
    for job in simulations.iter().filter(|job| current(&job.source)) {
        if job.status == CamJobStatus::Running {
            plan.stop.push(job.job_id);
        } else if waiting(job.status)
            && !matches!(
                job.toolpath_status,
                CamJobStatus::Running | CamJobStatus::Interrupted
            )
        {
            plan.cancel.push(job.job_id);
        }
    }
    for job in toolpaths.iter().filter(|job| current(&job.source)) {
        if job.status == CamJobStatus::Running {
            plan.stop.push(job.job_id);
        } else if waiting(job.status) {
            plan.cancel.push(job.job_id);
        }
    }
    plan
}

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

/// 切削シミュレーションの投入元が、保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の区間
/// （今の改訂の鍵と、区間の前提の鍵 `segment_key` が同じ）かを判定する関数。区間の前提の鍵を持たない投入元（前の版の
/// 記録）は、今の区間とみなさない
fn current_segment(
    sequences: &MachiningSequenceStore,
    sequence_id: MachiningSequenceId,
    operation_id: MachiningOperationId,
    segment_key: String,
) -> Result<impl Fn(&Option<CamJobSource>) -> bool, CamSimulationWorkflowError> {
    let revision = current_revision(sequences, sequence_id, operation_id)?;
    Ok(move |source: &Option<CamJobSource>| {
        revision(source)
            && source
                .as_ref()
                .and_then(|source| source.segment_key.as_deref())
                == Some(segment_key.as_str())
    })
}

/// 保管先 `sequences` の加工手順 `sequence_id` の加工工程 `operation_id` の今の区間（[`current_segment`]）かを判定する
/// 関数（区間の前提の鍵は、今の手順から求める）
fn current_segment_of(
    sequences: &MachiningSequenceStore,
    sequence_id: MachiningSequenceId,
    operation_id: MachiningOperationId,
) -> Result<impl Fn(&Option<CamJobSource>) -> bool, CamSimulationWorkflowError> {
    let sequence = sequences
        .sequence(sequence_id)
        .map_err(CamSimulationWorkflowError::Sequence)?;
    let segment_key = segment::segment_keys(sequence)
        .into_iter()
        .find(|(id, _)| *id == operation_id)
        .map(|(_, key)| key)
        .unwrap_or_default();
    current_segment(sequences, sequence_id, operation_id, segment_key)
}

/// ツールパス生成のジョブ `toolpath_jobs` のうち、投入元が `current` に当たり、正常に終わったものの番号（最も新しいもの。
/// 切削シミュレーションを置いていないものを先に探す。区間の前提が変わると、同じ CL の下に、別の前提の切削
/// シミュレーションを置く）
fn reusable_toolpath_job(
    toolpath_jobs: &[ToolpathJob],
    current: impl Fn(&Option<CamJobSource>) -> bool,
) -> Option<u64> {
    let succeeded =
        |job: &&ToolpathJob| job.status == CamJobStatus::Succeeded && current(&job.source);
    toolpath_jobs
        .iter()
        .rev()
        .filter(succeeded)
        .find(|job| !job.simulated)
        .or_else(|| toolpath_jobs.iter().rev().find(succeeded))
        .map(|job| job.job_id)
}

/// 計算待ち・計算中・中断の状態か（取り消せる）
fn is_active(status: CamJobStatus) -> bool {
    matches!(
        status,
        CamJobStatus::Queued
            | CamJobStatus::Running
            | CamJobStatus::NeedsRecompute
            | CamJobStatus::Interrupted
    )
}

/// 切削シミュレーションのジョブ `job` が、まだ正常に終わりうるか（失敗・取り消しで終わっておらず、入力のツールパス
/// 生成も失敗・取り消しで終わっていない）
fn can_finish(job: &SimulationJob) -> bool {
    let ended = |status| matches!(status, CamJobStatus::Failed | CamJobStatus::Canceled);
    !ended(job.status) && !ended(job.toolpath_status)
}

/// 計算を始める前に見積もったツールパスの大きさ（桁が合う程度）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolpathEstimate {
    /// パスの点の数
    pub points: u64,
    /// 生成に使うメモリ（バイト）
    pub memory_bytes: u64,
    /// ツールパス生成に使ってよいメモリの上限（バイト）
    pub limit_bytes: u64,
}

impl ToolpathEstimate {
    /// 生成に使うメモリ（GB）
    pub fn memory_gb(&self) -> f64 {
        self.memory_bytes as f64 / 1e9
    }

    /// 生成に使うメモリが、メモリの上限に近いか（上限の 8 割以上。見積もりの幅のため、計算しても途中で上限を
    /// 超えて止まることがある）
    pub fn is_near_limit(&self) -> bool {
        self.memory_bytes >= self.limit_bytes / 10 * 8
    }
}

/// 別のスレッドで見積もっている、計算を始める前のツールパスの大きさ
/// （[`CamSimulationWorkflow::estimate_toolpath_size`]）
#[derive(Debug)]
pub struct ToolpathSizeEstimation {
    /// 見積もったパスの点の数（見積もっている間は `None`。見積もれなければ `Some(None)`）
    points: Arc<Mutex<Option<Option<u64>>>>,
}

impl ToolpathSizeEstimation {
    /// 見積もり終えたか
    pub fn is_finished(&self) -> bool {
        self.points
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// 見積もり終えていれば、確かめる目安と上限 `limits`（[`CamSimulationWorkflow::toolpath_limits`]）で、確かめずに
    /// 計算してよいか・確かめてから計算するか（見積もっている間は `None`）
    ///
    /// 見積もれなかったときは、確かめずに計算してよいとする（計算すれば、計算の失敗として理由が分かる）。
    ///
    /// # Errors
    ///
    /// 見積もったツールパスがメモリの上限を超える場合。
    pub fn size(
        &self,
        limits: ToolpathLimits,
    ) -> Option<Result<ToolpathSize, CamSimulationWorkflowError>> {
        let points = *self.points.lock().unwrap_or_else(PoisonError::into_inner);
        points.map(|points| {
            points.map_or(Ok(ToolpathSize::Normal), |points| {
                toolpath_size(points, limits)
            })
        })
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

/// 点 `points` 個のツールパスの、確かめる目安とメモリの上限 `limits` での計算の始め方
///
/// # Errors
///
/// メモリの見積もりがツールパス生成のメモリの上限を超える場合。
fn toolpath_size(
    points: u64,
    limits: ToolpathLimits,
) -> Result<ToolpathSize, CamSimulationWorkflowError> {
    let estimate = ToolpathEstimate {
        points,
        memory_bytes: toolpath_memory_bytes(points),
        limit_bytes: limits.limit_bytes,
    };
    if estimate.memory_bytes > limits.limit_bytes {
        Err(CamSimulationWorkflowError::ToolpathTooLarge(estimate))
    } else if estimate.memory_bytes >= limits.confirm_bytes {
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
    let (geometry, bounds) =
        resolve_target_geometry(shapes, sequence, operation_tolerance(operation))?;
    let (workflow_request, errors) = operation_request(sequence, operation, geometry, bounds);
    if errors.is_empty() {
        Ok((workflow_request, operation))
    } else {
        Err(CamSimulationWorkflowError::Conditions(errors))
    }
}

/// ツールパス生成の入力の形状と、それを囲む箱（作れなければ、その誤り）
type TargetGeometry = Result<(SolverGeometry, Aabb3D<f64>), CamSimulationWorkflowError>;

/// 加工工程 `operation` の、ツールパス生成の許容弦誤差（シェルの面も、この値で三角形に分ける）
fn operation_tolerance(operation: &MachiningOperation) -> f64 {
    operation
        .content()
        .conditions
        .tolerance_profile
        .tolerance_mm()
}

/// 加工手順 `sequence` の加工の対象の面から、ツールパス生成の入力の形状（シェルの面は許容弦誤差
/// `chord_tolerance` で三角形に分ける）と、それを囲む箱を作る
fn resolve_target_geometry(
    shapes: &ProductShapeStore,
    sequence: &MachiningSequence,
    chord_tolerance: f64,
) -> TargetGeometry {
    let geometry = shapes
        .resolve_solver_geometry(sequence.target_faces(), chord_tolerance)
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
        chord_tolerance: operation_tolerance(operation),
        tessellation_limits: TessellationLimits::default(),
        boundary: conditions.boundary,
    };
    let workflow_request = CamSimulationWorkflowRequest {
        solver_input,
        stock: stock.unwrap_or(bounds),
        precision: sequence.stock().precision,
        grid_tool_radius: segment::grid_tool_radius(sequence, operation),
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
            request.grid_tool_radius,
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
        grid_tool_radius: request.grid_tool_radius,
        snapshot_interval: request.snapshot_interval.clone(),
        product: request.solver_input.geometry.clone(),
        tolerance: request.solver_input.chord_tolerance,
    }
}
