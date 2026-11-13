//! 製品形状からの切削シミュレーションの投入・実行・取り込み
//!
//! 空間モデルの文書（`SpaceModelDocument`）の、編集の対象の加工工程から投入・取り込みを行う。加工の対象は、確かめ・
//! 投入・取り込みのときに表示中の製品形状のすべての面にする。計算はアプリの外のジョブマネージャーが行い、セッションは
//! 投入・状態の読み取り・取り込みだけを行う。

use crate::space_model_document::MachiningOperationId;
use application::cam_simulation_workflow::{
    Aabb3D, CamSimulationWorkflow, CamSimulationWorkflowError, ToolpathSize,
};
pub use application::cam_simulation_workflow::{
    CamConditionError, CamConditionField, OperationJobState, OperationJobStatus,
};
pub use application::machining_sequence::{
    MachiningBoundary, MachiningOperationContent, MachiningStage, MachiningStock, OperationSpec,
    ScanlineCutDirection, ScanlineToolRange, StockMargins, Tool, flat_surface_cusp_height,
};
use application::machining_sequence::{
    MachiningRevisionKey, MachiningSequenceStore, ScanlineParams,
};
use std::collections::HashMap;

use application::cutting_simulation_playback::{
    CuttingSimulationPlayback, CuttingSimulationPlaybackError,
};

use crate::cutting_animation::CuttingAnimation;
use crate::failure_message_mapper::workflow_failure_message;
use crate::inverse_offset_shape_converter::{InverseOffsetShapeMesh, inverse_offset_shape_meshes};
use crate::space_model_document::{ImportedToolpathState, SpaceModelDocument};
use i18n_foundation::UiMessage;

/// 選べる工具の種類（ボールエンドミル・フラットエンドミル）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CuttingToolKind {
    #[default]
    BallEndMill,
    FlatEndMill,
}

impl CuttingToolKind {
    /// 工具 `tool` の種類（ボールエンドミル・フラットエンドミルでない場合は `None`）
    pub fn of(tool: &Tool<f64>) -> Option<Self> {
        if tool.is_ball_end_mill() {
            Some(Self::BallEndMill)
        } else if tool.is_flat_end_mill() {
            Some(Self::FlatEndMill)
        } else {
            None
        }
    }

    /// この種類の、直径 `diameter`（mm）・工具長 `cutting_length`（mm。`None` は長さに制限のない工具）の工具
    pub fn tool(self, diameter: f64, cutting_length: Option<f64>) -> Tool<f64> {
        // 工具長は、作った後に入力の値（指定しない場合は `None`）にする
        let mut tool = match self {
            Self::BallEndMill => {
                Tool::ball_end_mill(format!("ball_endmill_{diameter}mm"), diameter, 0.0)
            }
            Self::FlatEndMill => {
                Tool::flat_end_mill(format!("flat_endmill_{diameter}mm"), diameter, 0.0)
            }
        };
        tool.cutting_length = cutting_length;
        tool
    }
}

/// 取り込んだ切削シミュレーションの計算に使った条件
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedConditions {
    /// 工具
    pub tool: Tool<f64>,
    /// 走査線の条件
    pub scanline: ScanlineParams,
    /// ツールパス生成のトレランス（mm）
    pub toolpath_tolerance: f64,
    /// 素材（直方体）
    pub stock: Aabb3D<f64>,
    /// 素材の形状の許容誤差（mm）
    pub precision: f64,
}

/// 取り込んだ切削シミュレーション
#[derive(Debug)]
pub struct ImportedCuttingSimulation {
    /// 切削シミュレーションのジョブの番号
    pub job_id: u64,
    /// 切削アニメーション
    pub animation: CuttingAnimation,
    /// 計算に使った条件
    pub conditions: ImportedConditions,
}

/// 計算条件の確かめで見つかった問題（条件を設定していない・製品形状がない・条件の誤り・そのほか）
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionIssue {
    /// 工程の計算の条件を、まだ設定していない（条件を一度も適用していない）
    NotSet,
    /// 加工の対象の製品形状がない、または形状にできない
    NoProductShape,
    /// 計算条件の誤り（出す欄は `CamConditionError::field`）
    Condition(CamConditionError),
    /// そのほかの理由で投入できない（画面に出す理由）
    Other(UiMessage),
}

impl ConditionIssue {
    fn from_workflow_error(error: &CamSimulationWorkflowError) -> Vec<Self> {
        match error {
            CamSimulationWorkflowError::Conditions(errors) => {
                errors.iter().cloned().map(Self::Condition).collect()
            }
            CamSimulationWorkflowError::ProductShape(_)
            | CamSimulationWorkflowError::EmptyProductShape => {
                vec![Self::NoProductShape]
            }
            other => vec![Self::Other(workflow_failure_message(other))],
        }
    }
}

/// 投入・実行・取り込みの失敗
#[derive(Debug)]
pub enum CuttingSimulationSessionError {
    /// 投入・実行・取り込みの失敗
    Workflow(CamSimulationWorkflowError),
    /// 加工工程の今の改訂の、取り込める（正常終了した）切削シミュレーションがない
    NoSucceededSimulation,
    /// 加工工程の今の改訂の、取り込める（正常終了した）ツールパス生成がない
    NoSucceededToolpath,
    /// 取り込んだ結果を再生できない
    Playback(CuttingSimulationPlaybackError),
    /// 工程の計算の条件を、まだ設定していない（条件を一度も適用していない）
    ConditionsNotSet,
}

impl CuttingSimulationSessionError {
    /// ジョブマネージャーが動いていないため、できなかったか（計算の状態を読むときは、起動しない）
    pub fn is_job_manager_not_running(&self) -> bool {
        matches!(
            self,
            Self::Workflow(CamSimulationWorkflowError::JobManagerNotRunning)
        )
    }

    /// ジョブマネージャーが見つからない（インストールされていない）ため、できなかったか
    pub fn is_job_manager_not_found(&self) -> bool {
        matches!(
            self,
            Self::Workflow(CamSimulationWorkflowError::JobManagerNotFound)
        )
    }
}

impl std::fmt::Display for CuttingSimulationSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workflow(error) => write!(f, "{error}"),
            Self::NoSucceededSimulation => write!(
                f,
                "no succeeded cutting simulation of the current conditions to import"
            ),
            Self::NoSucceededToolpath => write!(
                f,
                "no succeeded toolpath generation of the current conditions to import"
            ),
            Self::Playback(error) => write!(f, "{error}"),
            Self::ConditionsNotSet => {
                write!(f, "the calculation conditions of the operation are not set")
            }
        }
    }
}

impl std::error::Error for CuttingSimulationSessionError {}

impl From<CamSimulationWorkflowError> for CuttingSimulationSessionError {
    fn from(value: CamSimulationWorkflowError) -> Self {
        Self::Workflow(value)
    }
}

/// 工程から投入した計算
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionKind {
    /// ツールパス生成と切削シミュレーション
    Full,
    /// 空間モデルに取り込んだ今の条件の CL を入力にする、切削シミュレーションだけ
    SimulationOnly,
}

/// 計算停止で、ジョブマネージャーが投入を受け取るのを待って取り消し直す回数（最初の 1 回を含む）
const STOP_ATTEMPTS: u32 = 8;

/// 計算停止で、取り消し直すまでの間
const STOP_RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

/// 計算停止の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopOutcome {
    /// 今の条件の計算待ち・計算中のジョブを取り消すよう頼んだ
    Stopped,
    /// 今の条件の計算は、待ってもジョブマネージャーが受け取らない投入があり、すべては取り消せなかった
    NotYetAccepted,
    /// 今の条件の計算待ち・計算中のジョブがない
    NothingToStop,
}

/// 再計算の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedoOutcome {
    /// 今の条件の計算が計算待ちのため、投入しなかった
    Waiting,
    /// 投入した（`canceled` は、今の条件の前の計算を取り消すよう頼んだか）
    Submitted { canceled: bool },
    /// ツールパスが大きくなることを確かめて、計算しないことにした
    Declined,
}

/// 計算の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitOutcome {
    /// 投入した
    Submitted(SubmissionKind),
    /// 今の改訂の計算が計算待ち・計算中・正常に終わっているため、投入しなかった
    AlreadyCalculated,
    /// ツールパスが大きくなることを確かめて、計算しないことにした
    Declined,
}

/// 全工程の計算の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceOutcome {
    /// 投入した
    Submitted(SequenceSubmission),
    /// ツールパスが大きくなることを確かめて、どの工程も計算しないことにした
    Declined,
}

/// 確かめる目安を超える大きさのツールパスになる工程
#[derive(Debug, Clone, PartialEq)]
pub struct LargeToolpath {
    /// 工程の加工の順の番号（1 から）
    pub position: usize,
    /// 工程の名前
    pub operation: String,
    /// ツールパスの生成に使うメモリの見積もり（GB）
    pub memory_gb: f64,
    /// 見積もりがメモリの上限に近い（計算しても、途中で上限を超えて止まることがある）
    pub near_limit: bool,
}

impl SubmissionKind {
    /// 工程に取り込んだ CL の状態 `toolpath` と、工程の今の改訂の計算の状態 `state` から決める投入（今の改訂の計算
    /// が計算待ち・計算中・正常に終わっていれば、投入しない `None`。今の条件の CL があれば、CL をもう一度計算しない
    /// ように、切削シミュレーションだけ）
    fn of(toolpath: ImportedToolpathState, state: OperationJobState) -> Option<Self> {
        let done_or_pending =
            matches!(state, OperationJobState::Status(status) if status.is_waiting_or_done());
        if done_or_pending {
            None
        } else if toolpath == ImportedToolpathState::Current {
            Some(Self::SimulationOnly)
        } else {
            Some(Self::Full)
        }
    }
}

/// 工程 `operation`（加工の順の番号 `position`（1 から）・名前 `name`）のツールパスの大きさを、計算を始める前に
/// 見積もる（確かめる目安を超えるなら、その工程と見積もり）
///
/// # Errors
///
/// 投入前の確認の誤り、見積もったツールパスがメモリの上限を超える場合。
fn large_toolpath(
    document: &SpaceModelDocument,
    operation: MachiningOperationId,
    (position, name): (usize, &str),
) -> Result<Option<LargeToolpath>, CamSimulationWorkflowError> {
    let (shapes, sequences, sequence, _) = document.machining();
    let ToolpathSize::Large(estimate) =
        CamSimulationWorkflow::check_toolpath_size(shapes, sequences, sequence, operation)?
    else {
        return Ok(None);
    };
    Ok(Some(LargeToolpath {
        position,
        operation: name.to_string(),
        memory_gb: estimate.memory_gb(),
        near_limit: estimate.is_near_limit(),
    }))
}

/// 選んだ加工工程 `operation` の、加工の順の番号（1 から）と名前
fn operation_label(
    document: &SpaceModelDocument,
    operation: MachiningOperationId,
) -> (usize, String) {
    document
        .operations()
        .into_iter()
        .enumerate()
        .find(|(_, item)| item.id == operation)
        .map(|(index, item)| (index + 1, item.name))
        // 選んだ工程は手順の中にあるはずだが、なくても計算の操作を落とさない（番号 0・名前なしで出す）
        .unwrap_or((0, String::new()))
}

/// 手順の全体の投入の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceSubmission {
    /// 投入した工程の数（切削シミュレーションだけを投入した工程を含む）
    pub submitted: usize,
    /// 投入した工程のうち、今の条件の CL を入力にして切削シミュレーションだけを投入した工程の数
    pub simulation_only: usize,
    /// 今の改訂の計算が計算待ち・計算中・正常に終わっていたため、投入しなかった工程の数
    pub skipped: usize,
}

/// 手順の全体の投入の失敗
#[derive(Debug)]
pub enum SequenceSubmissionError {
    /// 工程の条件の誤り（どの工程も投入していない。工程の加工の順の番号（1 から）・名前と理由）
    Conditions {
        position: usize,
        operation: String,
        error: CuttingSimulationSessionError,
    },
    /// 計算の状態を読めない（ジョブマネージャーが見つからない・起動できない等。投入し直さない工程を決められない
    /// ため、どの工程も投入していない）
    StatusUnreadable(CuttingSimulationSessionError),
    /// 工程の投入の失敗（その前の `submitted` 個の工程は投入した。工程の加工の順の番号（1 から）・名前と理由）
    Submit {
        position: usize,
        operation: String,
        submitted: usize,
        error: CuttingSimulationSessionError,
    },
}

impl std::fmt::Display for SequenceSubmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conditions {
                position,
                operation,
                error,
            } => write!(f, "operation {position} ({operation:?}): {error}"),
            Self::StatusUnreadable(error) => write!(f, "cannot read the job status: {error}"),
            Self::Submit {
                position,
                operation,
                submitted,
                error,
            } => write!(
                f,
                "operation {position} ({operation:?}): {error} ({submitted} operations were submitted before it)"
            ),
        }
    }
}

/// 製品形状からの切削シミュレーションの投入・実行・取り込み
#[derive(Debug, Default)]
pub struct CuttingSimulationSession {
    workflow: CamSimulationWorkflow,
    /// 工程ごとに、確かめた改訂の鍵と、その改訂の計算条件の問題（誤りがなければ空）
    condition_issues: HashMap<MachiningOperationId, (MachiningRevisionKey, Vec<ConditionIssue>)>,
}

impl CuttingSimulationSession {
    /// 同じプロセスの中でジョブを実行するセッション（試験用）
    #[cfg(test)]
    pub(crate) fn in_process() -> Self {
        Self {
            workflow: CamSimulationWorkflow::in_process(),
            condition_issues: HashMap::new(),
        }
    }

    /// 表示中の製品形状と加工工程・加工手順の条件を、投入と同じ判定で確かめ、問題をすべて返す（問題がなければ空。
    /// 条件を設定していない工程は、そのことだけを返す）
    pub fn check(&self, document: &mut SpaceModelDocument) -> Vec<ConditionIssue> {
        document.target_visible_faces();
        if !document.conditions_set_of(document.selected_operation()) {
            return vec![ConditionIssue::NotSet];
        }
        let (shapes, sequences, sequence, operation) = document.machining();
        match CamSimulationWorkflow::check_operation(shapes, sequences, sequence, operation) {
            Ok(()) => Vec::new(),
            Err(error) => ConditionIssue::from_workflow_error(&error),
        }
    }

    /// 工程ごとに覚えた、計算の条件の確かめの結果を捨てる（別の文書では、同じ工程の ID・改訂の鍵でも形状が違いうる）
    pub fn forget_condition_checks(&mut self) {
        self.condition_issues.clear();
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工手順の工程のうち、計算の条件に誤りがある工程と、その問題
    /// （加工の順。問題は、工程を選んで [`Self::check`] で確かめたときと同じ。条件を設定していない工程は含めない）
    ///
    /// 工程の改訂ごとに確かめた結果を覚え、改訂が進んだ工程（計算の条件・素材・加工の対象が変わった）だけを
    /// 確かめ直す。確かめ直す工程はまとめて確かめ、加工の対象の形状は 1 回だけ作る。
    pub fn operation_condition_issues(
        &mut self,
        document: &mut SpaceModelDocument,
    ) -> Vec<(MachiningOperationId, Vec<ConditionIssue>)> {
        document.target_visible_faces();
        let operations = document.operations();
        self.condition_issues
            .retain(|operation, _| operations.iter().any(|item| item.id == *operation));
        // 設定した工程と、今の改訂の鍵（加工の順）
        let keyed: Vec<(MachiningOperationId, MachiningRevisionKey)> = operations
            .iter()
            .filter(|item| item.conditions_set)
            .filter_map(|item| Some((item.id, document.revision_key_of(item.id)?)))
            .collect();
        let stale: Vec<(MachiningOperationId, MachiningRevisionKey)> = keyed
            .iter()
            .copied()
            .filter(|(id, key)| {
                !matches!(self.condition_issues.get(id), Some((checked, _)) if checked == key)
            })
            .collect();
        if !stale.is_empty() {
            let (shapes, sequences, sequence, _) = document.machining();
            let ids: Vec<MachiningOperationId> = stale.iter().map(|(id, _)| *id).collect();
            let results =
                CamSimulationWorkflow::check_operations(shapes, sequences, sequence, &ids);
            for ((id, key), result) in stale.into_iter().zip(results) {
                let issues = match result {
                    Ok(()) => Vec::new(),
                    Err(error) => ConditionIssue::from_workflow_error(&error),
                };
                self.condition_issues.insert(id, (key, issues));
            }
        }
        keyed
            .into_iter()
            .filter_map(|(id, _)| {
                let (_, issues) = self.condition_issues.get(&id)?;
                (!issues.is_empty()).then(|| (id, issues.clone()))
            })
            .collect()
    }

    /// 表示中の製品形状のすべての面を加工の対象として、選んだ加工工程の中身を `content` に、加工手順の素材を `stock` に
    /// 替えたときの計算条件の問題（文書の工程・手順は替えない。適用する前の条件を確かめる）
    pub fn check_draft(
        &self,
        document: &mut SpaceModelDocument,
        content: &MachiningOperationContent,
        stock: MachiningStock,
    ) -> Vec<ConditionIssue> {
        document.target_visible_faces();
        let (shapes, sequences, sequence, operation) = document.machining();
        // 保管先の写しに替えて確かめる（記録から戻すため、ID・改訂は同じ）
        let mut draft = MachiningSequenceStore::from_records(sequences.records())
            .expect("the records of a held store are restored as they are");
        draft
            .set_operation_content(sequence, operation, content.clone())
            .and_then(|()| draft.set_stock(sequence, stock))
            .expect("the document holds its machining sequence and operation");
        match CamSimulationWorkflow::check_operation(shapes, &draft, sequence, operation) {
            Ok(()) => Vec::new(),
            Err(error) => ConditionIssue::from_workflow_error(&error),
        }
    }

    /// 表示中の製品形状と加工工程・加工手順の条件で、投入したときのツールパス生成と同じ逆オフセット形状を作り、角度
    /// `angle_step`（ラジアン）のきざみで種類ごとのメッシュにする（デバッグ表示）
    ///
    /// # Errors
    ///
    /// 投入前の確認の誤り、逆オフセット形状を作れない場合、`angle_step` が表示用のきざみの範囲の外の場合。
    pub fn inverse_offset_shape_meshes(
        &self,
        document: &mut SpaceModelDocument,
        angle_step: f64,
    ) -> Result<Vec<InverseOffsetShapeMesh>, CuttingSimulationSessionError> {
        document.target_visible_faces();
        let (shapes, sequences, sequence, operation) = document.machining();
        let parts = CamSimulationWorkflow::inverse_offset_shape_parts(
            shapes, sequences, sequence, operation, angle_step,
        )?;
        Ok(inverse_offset_shape_meshes(parts))
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工工程から投入し（実行しない）、投入した計算を返す
    ///
    /// 今の改訂の計算が計算待ち・計算中・正常に終わっていれば、重ねて投入しない。そうでなく、空間モデルに今の条件の
    /// CL があれば、その CL を入力にして切削シミュレーションだけを投入する。どちらでもなければ、ツールパスの大きさを
    /// 見積もり、確かめる目安を超えるなら `confirm` で確かめてから、ツールパス生成から投入する。
    ///
    /// # Errors
    ///
    /// 表示中の製品形状がない場合・計算条件に誤りがある場合・見積もったツールパスがメモリの上限を超える場合を
    /// 含む、投入の失敗。
    pub fn submit(
        &mut self,
        document: &mut SpaceModelDocument,
        confirm: impl FnOnce(&[LargeToolpath]) -> bool,
    ) -> Result<SubmitOutcome, CuttingSimulationSessionError> {
        document.target_visible_faces();
        if !document.conditions_set_of(document.selected_operation()) {
            return Err(CuttingSimulationSessionError::ConditionsNotSet);
        }
        let (_, sequences, sequence, operation) = document.machining();
        // 切削シミュレーションだけを投入するかは、状態を今読んでから決める（ジョブマネージャーが動いていなければ起動
        // する。読まずに決めると、計算済みのジョブがあっても重ねて投入する）。見つからなければ投入できない。ほかの理由で
        // 読めなくても、投入はつなぎ直して行うため、まだ読めていないときと同じに決める
        let state =
            match self
                .workflow
                .latest_revision_job_states_now(sequences, sequence, &[operation])
            {
                Ok(mut states) => states.remove(0),
                Err(CamSimulationWorkflowError::JobManagerNotFound) => {
                    return Err(CamSimulationWorkflowError::JobManagerNotFound.into());
                }
                Err(error) => {
                    tracing::warn!("Cannot read the job status before submitting: {error}");
                    OperationJobState::NotRead
                }
            };
        let Some(kind) = SubmissionKind::of(document.imported_toolpath_state(), state) else {
            return Ok(SubmitOutcome::AlreadyCalculated);
        };
        if kind == SubmissionKind::Full {
            let (position, name) = operation_label(document, operation);
            if let Some(large) = large_toolpath(document, operation, (position, &name))?
                && !confirm(&[large])
            {
                return Ok(SubmitOutcome::Declined);
            }
        }
        self.submit_kind(document, operation, kind)?;
        Ok(SubmitOutcome::Submitted(kind))
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工工程の空間モデルの CL を除き、ツールパス生成から投入し直す
    /// （実行しない。CL は、投入できたときだけ除く）。投入できたら、今の条件の前の計算中のジョブを取り消すよう頼む
    /// （取り消し終えるのを待たない）
    ///
    /// 今の条件の計算が計算待ちなら、投入せずに `RedoOutcome::Waiting` を返す（計算待ちの計算は、まだ始まって
    /// いない。ジョブマネージャーがまだ受け取っていない投入も計算待ちのため、取り消せない投入に重ねない）。
    /// ツールパスの大きさを見積もり、確かめる目安を超えるなら `confirm` で確かめてから投入する。
    ///
    /// # Errors
    ///
    /// 表示中の製品形状がない場合・計算条件に誤りがある場合・見積もったツールパスがメモリの上限を超える場合を
    /// 含む、投入の失敗。
    pub fn redo(
        &mut self,
        document: &mut SpaceModelDocument,
        confirm: impl FnOnce(&[LargeToolpath]) -> bool,
    ) -> Result<RedoOutcome, CuttingSimulationSessionError> {
        document.target_visible_faces();
        if !document.conditions_set_of(document.selected_operation()) {
            return Err(CuttingSimulationSessionError::ConditionsNotSet);
        }
        // 大きさは、状態を読む前に確かめる（確かめている間に状態が変わっても、読んだ状態ですぐに投入するため）
        let operation = document.selected_operation();
        let (position, name) = operation_label(document, operation);
        if let Some(large) = large_toolpath(document, operation, (position, &name))?
            && !confirm(&[large])
        {
            return Ok(RedoOutcome::Declined);
        }
        let (shapes, sequences, sequence, operation) = document.machining();
        // 計算待ちかは、状態を今読んで決める（ボタンは最後に読んだ状態で決めるため）。見つからなければ投入できない。
        // ほかの理由で読めなくても、投入はつなぎ直して行う
        match self
            .workflow
            .latest_revision_job_states_now(sequences, sequence, &[operation])
        {
            Ok(states) if states[0] == OperationJobState::Status(OperationJobStatus::Queued) => {
                return Ok(RedoOutcome::Waiting);
            }
            Ok(_) => {}
            Err(CamSimulationWorkflowError::JobManagerNotFound) => {
                return Err(CamSimulationWorkflowError::JobManagerNotFound.into());
            }
            Err(error) => {
                tracing::warn!("Cannot read the job status before recalculating: {error}");
            }
        }
        let submission = self
            .workflow
            .submit_operation(shapes, sequences, sequence, operation)?;
        // 取り消すのは投入できてから（先に取り消すと、投入できなかったときに今の計算を失うため）。頼めなくても、
        // 投入は済んでいる（前の計算が残り、重ねて計算するだけ）
        let canceled = match self.workflow.cancel_latest_revision_jobs(
            sequences,
            sequence,
            operation,
            Some(&submission.submission_id),
        ) {
            Ok(count) => count > 0,
            Err(error) => {
                tracing::warn!("Cannot cancel the earlier calculation when recalculating: {error}");
                false
            }
        };
        document.remove_toolpath();
        Ok(RedoOutcome::Submitted { canceled })
    }

    /// 選んだ加工工程の今の条件の、計算待ち・計算中のジョブ（ツールパス生成と切削シミュレーション）を取り消すよう
    /// 頼む（取り消し終えるのを待たない。ジョブマネージャーが動いていなければ起動する）
    ///
    /// 取り消せるのは、ジョブマネージャーが受け取ったジョブだけのため、まだ受け取っていない投入があれば、受け取る
    /// まで少しの間待ってから、もう一度取り消す（受け取りは短い間隔で行われる。待つ間は画面を止める）。
    ///
    /// # Errors
    ///
    /// ジョブマネージャーが見つからない・状態を読めない・取り消しを頼めない場合。
    pub fn stop(
        &mut self,
        document: &SpaceModelDocument,
    ) -> Result<StopOutcome, CuttingSimulationSessionError> {
        let (_, sequences, sequence, operation) = document.machining();
        let mut canceled = 0;
        for attempt in 1..=STOP_ATTEMPTS {
            // 受け取っていない投入を先に読む（後で読むと、間に受け取られたジョブを取り消さずに終える）
            let pending = self
                .workflow
                .latest_revision_has_pending_now(sequences, sequence, operation)?;
            canceled += self
                .workflow
                .cancel_latest_revision_jobs(sequences, sequence, operation, None)?;
            if !pending {
                return Ok(if canceled > 0 {
                    StopOutcome::Stopped
                } else {
                    StopOutcome::NothingToStop
                });
            }
            if attempt < STOP_ATTEMPTS {
                std::thread::sleep(STOP_RETRY_INTERVAL);
            }
        }
        Ok(StopOutcome::NotYetAccepted)
    }

    /// 加工工程 `operation` から、計算 `kind` を投入する
    fn submit_kind(
        &mut self,
        document: &SpaceModelDocument,
        operation: MachiningOperationId,
        kind: SubmissionKind,
    ) -> Result<(), CamSimulationWorkflowError> {
        let (shapes, sequences, sequence, _) = document.machining();
        match kind {
            SubmissionKind::Full => {
                self.workflow
                    .submit_operation(shapes, sequences, sequence, operation)?;
            }
            SubmissionKind::SimulationOnly => {
                self.workflow.submit_imported_toolpath(
                    shapes,
                    sequences,
                    document.toolpaths(),
                    sequence,
                    operation,
                )?;
            }
        }
        Ok(())
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工手順のすべての加工工程を加工の順に投入する（実行しない）
    ///
    /// 先にすべての工程の条件を確かめ、誤りがあればどの工程も投入しない。今の改訂の計算が計算待ち・計算中・正常に
    /// 終わった工程は、投入し直さない（同じ計算を重ねないように）。計算の状態をまだ読めていない・読めないときは、
    /// 投入し直さない工程を決められないため、どの工程も投入しない。投入する工程の計算は、工程の投入（`Self::submit`）
    /// と同じく決める。ツールパス生成から投入する工程のツールパスの大きさを見積もり、確かめる目安を超える工程が
    /// あれば、まとめて `confirm` で 1 回だけ確かめてから投入する。
    ///
    /// # Errors
    ///
    /// 工程の条件の誤り・見積もったツールパスがメモリの上限を超える・計算の状態を読めていない（どの工程も投入
    /// しない）、投入の失敗（その工程の前までは投入した）の場合。
    pub fn submit_sequence(
        &mut self,
        document: &mut SpaceModelDocument,
        confirm: impl FnOnce(&[LargeToolpath]) -> bool,
    ) -> Result<SequenceOutcome, SequenceSubmissionError> {
        document.target_visible_faces();
        let operations = document.operations();
        let (shapes, sequences, sequence, _) = document.machining();
        let ids: Vec<MachiningOperationId> = operations.iter().map(|item| item.id).collect();
        // 加工の順に、最初の誤りを返す。最初の未設定の工程より前の工程だけを、まとめて確かめる（加工の対象の形状は
        // 1 回だけ作る）
        let not_set = operations.iter().position(|item| !item.conditions_set);
        let checked = &ids[..not_set.unwrap_or(ids.len())];
        let checks = CamSimulationWorkflow::check_operations(shapes, sequences, sequence, checked);
        let conditions = |index: usize, error| SequenceSubmissionError::Conditions {
            position: index + 1,
            operation: operations[index].name.clone(),
            error,
        };
        for (index, check) in checks.into_iter().enumerate() {
            check.map_err(|error| conditions(index, error.into()))?;
        }
        if let Some(index) = not_set {
            return Err(conditions(
                index,
                CuttingSimulationSessionError::ConditionsNotSet,
            ));
        }
        // 計算待ち・計算中・計算済みの工程を投入し直さないよう、状態を今読んでから決める（ジョブマネージャーが動いて
        // いなければ起動する）
        let states = self
            .workflow
            .latest_revision_job_states_now(sequences, sequence, &ids)
            .map_err(|error| SequenceSubmissionError::StatusUnreadable(error.into()))?;
        let kinds: Vec<Option<SubmissionKind>> = operations
            .iter()
            .zip(states)
            .map(|(item, state)| {
                SubmissionKind::of(document.imported_toolpath_state_of(item.id), state)
            })
            .collect();
        let mut large = Vec::new();
        for (index, (item, kind)) in operations.iter().zip(&kinds).enumerate() {
            if *kind != Some(SubmissionKind::Full) {
                continue;
            }
            let found =
                large_toolpath(document, item.id, (index + 1, &item.name)).map_err(|error| {
                    SequenceSubmissionError::Conditions {
                        position: index + 1,
                        operation: item.name.clone(),
                        error: error.into(),
                    }
                })?;
            large.extend(found);
        }
        if !large.is_empty() && !confirm(&large) {
            return Ok(SequenceOutcome::Declined);
        }
        let mut submission = SequenceSubmission {
            submitted: 0,
            simulation_only: 0,
            skipped: 0,
        };
        for (index, (item, kind)) in operations.iter().zip(kinds).enumerate() {
            let Some(kind) = kind else {
                submission.skipped += 1;
                continue;
            };
            self.submit_kind(document, item.id, kind).map_err(|error| {
                SequenceSubmissionError::Submit {
                    position: index + 1,
                    operation: item.name.clone(),
                    submitted: submission.submitted,
                    error: error.into(),
                }
            })?;
            submission.submitted += 1;
            if kind == SubmissionKind::SimulationOnly {
                submission.simulation_only += 1;
            }
        }
        Ok(SequenceOutcome::Submitted(submission))
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工手順のすべての加工工程の今の改訂の計算の状態（加工の順。
    /// 最後に読んだ状態から決め、読み終えるのを待たない）
    ///
    /// # Errors
    ///
    /// ジョブの状態を読めない場合。
    pub fn job_states(
        &mut self,
        document: &mut SpaceModelDocument,
    ) -> Result<Vec<(MachiningOperationId, OperationJobState)>, CuttingSimulationSessionError> {
        document.target_visible_faces();
        let operations: Vec<MachiningOperationId> = document
            .operations()
            .into_iter()
            .map(|item| item.id)
            .collect();
        let (_, sequences, sequence, _) = document.machining();
        let states = self
            .workflow
            .latest_revision_job_states(sequences, sequence, &operations)?;
        Ok(operations.into_iter().zip(states).collect())
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工工程の今の改訂で正常に終わったツールパス生成のうち、最も
    /// 新しいものの CL だけを空間モデルに取り込み、そのジョブの番号を返す（切削シミュレーションは読まない）
    ///
    /// # Errors
    ///
    /// 今の改訂の取り込めるツールパス生成がない場合、取り込みの失敗。
    pub fn import_toolpath(
        &mut self,
        document: &mut SpaceModelDocument,
    ) -> Result<u64, CuttingSimulationSessionError> {
        document.target_visible_faces();
        let (_, sequences, sequence, operation) = document.machining();
        let latest = self
            .workflow
            .latest_revision_toolpath(sequences, sequence, operation)?
            .ok_or(CuttingSimulationSessionError::NoSucceededToolpath)?;
        let result = self.workflow.toolpath_result(latest)?;
        document.import_toolpath_result(&result);
        Ok(latest)
    }

    /// 同じプロセスの中で実行するときに、待ちのジョブを実行する。実行したジョブの数を返す（外のジョブマネージャー
    /// は、待ちのジョブを自分で実行するため、何もせず 0）
    ///
    /// # Errors
    ///
    /// 同じプロセスの中で実行するときの、ジョブの状態の更新の失敗。
    pub fn run_queued_jobs(&mut self) -> Result<usize, CuttingSimulationSessionError> {
        Ok(self.workflow.run_queued_jobs()?)
    }

    /// ジョブマネージャーのウィンドウを開く（動いていなければ、ウィンドウを出して起動する）
    ///
    /// # Errors
    ///
    /// ジョブマネージャーがない・起動できない場合。
    pub fn open_job_manager(&mut self) -> Result<(), CuttingSimulationSessionError> {
        Ok(self.workflow.open_job_manager()?)
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工工程の今の改訂（今の計算条件と加工の対象）で正常終了した
    /// 切削シミュレーションのうち、最も新しいものを取り込み、切削アニメーションにする。その CL は、空間モデルに取り込む
    ///
    /// # Errors
    ///
    /// 今の改訂の取り込める切削シミュレーションがない場合、取り込みの失敗、切削アニメーションを作れない場合。
    pub fn import_latest(
        &mut self,
        document: &mut SpaceModelDocument,
    ) -> Result<ImportedCuttingSimulation, CuttingSimulationSessionError> {
        document.target_visible_faces();
        let (_, sequences, sequence, operation) = document.machining();
        let latest = self
            .workflow
            .latest_revision_simulation(sequences, sequence, operation)?
            .ok_or(CuttingSimulationSessionError::NoSucceededSimulation)?;
        let result = self.workflow.import(latest)?;
        // 空間モデルの CL から計算した結果は、ツールパス生成の入力を持たない。その CL は今の条件の結果のため、今の
        // 条件の走査線の条件とトレランスにする
        let (scanline, toolpath_tolerance) = match &result.solver_input {
            Some(solver_input) => {
                let OperationSpec::Scanline(scanline) = solver_input.operation;
                (scanline, solver_input.chord_tolerance)
            }
            None => {
                let OperationSpec::Scanline(scanline) =
                    document.operation_content().conditions.spec;
                (scanline, document.toolpath_tolerance())
            }
        };
        let conditions = ImportedConditions {
            tool: result.simulation_input.tool.clone(),
            scanline,
            toolpath_tolerance,
            stock: result.simulation_input.stock,
            precision: result.simulation_input.precision,
        };
        let playback = CuttingSimulationPlayback::open(&result)
            .map_err(CuttingSimulationSessionError::Playback)?;
        // 再生できる結果だけ、その CL を空間モデルに取り込む（以後は、ジョブの成果物に頼らない）
        document.import_toolpath(&result);
        // 取り込んだことをジョブマネージャーに伝える（待たない。伝えられなくても、取り込みは終わっている）
        if let Err(error) = self.workflow.record_imported(latest) {
            tracing::warn!("Cannot record the imported toolpath in the job manager: {error}");
        }
        let animation = CuttingAnimation::new(playback, result.simulation_input.tool);
        Ok(ImportedCuttingSimulation {
            job_id: latest,
            animation,
            conditions,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use application::cam_simulation_workflow::CamSimulationWorkflowError;
    use application::product_shape::ProductShapeError;

    use super::*;
    use crate::space_model_document::ImportedToolpathState;
    use crate::space_model_document::tests::{NAMES, document_with_pyramid};

    /// 走査線の条件 `content` を `edit` で変えた中身
    fn with_scanline(
        content: MachiningOperationContent,
        edit: impl FnOnce(&mut ScanlineParams),
    ) -> MachiningOperationContent {
        let mut content = content;
        let OperationSpec::Scanline(scanline) = &mut content.conditions.spec;
        edit(scanline);
        content
    }

    /// 文書 `document` の加工工程を、粗い条件（直径 6 mm・工具長 30 mm のボールエンドミル、ピックフィード 6 mm。
    /// テストを速くする）にし、条件を設定したことにする（適用したときと同じ）
    fn coarse(document: &mut SpaceModelDocument) {
        let mut content = document.operation_content().clone();
        content.conditions.tool = CuttingToolKind::BallEndMill.tool(6.0, Some(30.0));
        document.set_operation_content(with_scanline(content, |scanline| scanline.stepover = 6.0));
        document.mark_conditions_set();
    }

    /// 製品形状（四角錐）と粗い条件の文書と、同じプロセスの中でジョブを実行するセッション
    fn coarse_session() -> (CuttingSimulationSession, SpaceModelDocument) {
        let mut document = document_with_pyramid();
        coarse(&mut document);
        (CuttingSimulationSession::in_process(), document)
    }

    /// 表示中の製品形状（四角錐）を粗い条件で投入・実行・取り込みした切削アニメーション
    ///
    /// `cutting_length` を渡すと、工具長をそれにする（`Some(None)` は工具長を指定しない工具）。
    pub(crate) fn imported_animation(cutting_length: Option<Option<f64>>) -> CuttingAnimation {
        let (mut session, mut document) = coarse_session();
        if let Some(length) = cutting_length {
            let mut content = document.operation_content().clone();
            content.conditions.tool.cutting_length = length;
            document.set_operation_content(content);
        }
        session.submit(&mut document, |_| true).unwrap();
        session.run_queued_jobs().unwrap();
        session.import_latest(&mut document).unwrap().animation
    }

    /// 確かめる目安を超え、メモリの上限は超えないピックフィード（直径 0.1 mm のボールエンドミルで、四角錐の
    /// ツールパスが数 GB）
    const LARGE_STEPOVER: f64 = 0.0005;

    /// 文書 `document` の加工工程を、直径 0.1 mm のボールエンドミルとピックフィード `stepover` にする（ツールパスが
    /// 大きくなる）
    fn small_tool(document: &mut SpaceModelDocument, stepover: f64) {
        let mut content = document.operation_content().clone();
        content.conditions.tool = CuttingToolKind::BallEndMill.tool(0.1, Some(30.0));
        document.set_operation_content(with_scanline(content, |scanline| {
            scanline.stepover = stepover
        }));
    }

    #[test]
    fn a_large_toolpath_is_calculated_only_after_it_is_confirmed() {
        let (mut session, mut document) = coarse_session();
        small_tool(&mut document, LARGE_STEPOVER);
        let mut asked = Vec::new();
        assert_eq!(
            session
                .submit(&mut document, |large| {
                    asked.extend_from_slice(large);
                    false
                })
                .unwrap(),
            SubmitOutcome::Declined
        );
        assert_eq!(asked.len(), 1);
        assert_eq!(asked[0].position, 1);
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
        assert_eq!(
            session.redo(&mut document, |_| false).unwrap(),
            RedoOutcome::Declined
        );
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
        // 全工程の計算は、大きくなる工程をまとめて 1 回だけ確かめる
        let mut asked = 0;
        assert_eq!(
            session
                .submit_sequence(&mut document, |_| {
                    asked += 1;
                    false
                })
                .unwrap(),
            SequenceOutcome::Declined
        );
        assert_eq!(asked, 1);
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
        // 確かめて続けると、投入する（計算はしない。大きいため）
        assert_eq!(
            session.submit(&mut document, |_| true).unwrap(),
            SubmitOutcome::Submitted(SubmissionKind::Full)
        );
    }

    #[test]
    fn a_toolpath_over_the_memory_limit_is_not_calculated() {
        let (mut session, mut document) = coarse_session();
        small_tool(&mut document, 0.0001);
        let too_large = |error: &CuttingSimulationSessionError| {
            matches!(
                error,
                CuttingSimulationSessionError::Workflow(
                    CamSimulationWorkflowError::ToolpathTooLarge(_)
                )
            )
        };
        let error = session.submit(&mut document, |_| true).unwrap_err();
        assert!(too_large(&error));
        assert!(too_large(
            &session.redo(&mut document, |_| true).unwrap_err()
        ));
        assert!(matches!(
            session.submit_sequence(&mut document, |_| true),
            Err(SequenceSubmissionError::Conditions { position: 1, ref error, .. })
                if too_large(error)
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }

    #[test]
    fn tools_are_made_from_and_read_back_as_the_selectable_kinds() {
        for kind in [CuttingToolKind::BallEndMill, CuttingToolKind::FlatEndMill] {
            let tool = kind.tool(8.0, Some(25.0));
            assert_eq!(CuttingToolKind::of(&tool), Some(kind));
            assert_eq!(tool.diameter(), 8.0);
            assert_eq!(tool.cutting_length, Some(25.0));
        }
        assert_eq!(
            CuttingToolKind::FlatEndMill.tool(8.0, None).cutting_length,
            None
        );
        let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 30.0);
        assert_eq!(CuttingToolKind::of(&radius), None);
    }

    #[test]
    fn the_whole_sequence_is_submitted_after_every_operation_is_checked() {
        let (mut session, mut document) = coarse_session();
        document.duplicate_operation("仕上げ".to_string());
        // 誤りのある工程があれば、どの工程も投入しない（誤りの工程の名前を返す）
        let mut wrong = document.operation_content().clone();
        let OperationSpec::Scanline(mut scanline) = wrong.conditions.spec;
        scanline.stepover = 1000.0;
        wrong.conditions.spec = OperationSpec::Scanline(scanline);
        document.set_operation_content(wrong);
        assert!(matches!(
            session.submit_sequence(&mut document, |_| true),
            Err(SequenceSubmissionError::Conditions { position: 2, ref operation, .. })
                if operation == "仕上げ"
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);

        // 誤りを直すと、すべての工程を投入する
        let mut fixed = document.operation_content().clone();
        let OperationSpec::Scanline(mut scanline) = fixed.conditions.spec;
        scanline.stepover = 3.0;
        fixed.conditions.spec = OperationSpec::Scanline(scanline);
        document.set_operation_content(fixed);
        assert_eq!(
            session.submit_sequence(&mut document, |_| true).unwrap(),
            SequenceOutcome::Submitted(SequenceSubmission {
                submitted: 2,
                simulation_only: 0,
                skipped: 0
            })
        );
        // 計算待ち・計算済みの工程は、投入し直さない
        assert_eq!(
            session.submit_sequence(&mut document, |_| true).unwrap(),
            SequenceOutcome::Submitted(SequenceSubmission {
                submitted: 0,
                simulation_only: 0,
                skipped: 2
            })
        );
        assert_eq!(session.run_queued_jobs().unwrap(), 4);
        assert_eq!(
            session.submit_sequence(&mut document, |_| true).unwrap(),
            SequenceOutcome::Submitted(SequenceSubmission {
                submitted: 0,
                simulation_only: 0,
                skipped: 2
            })
        );
    }

    #[test]
    fn only_the_simulation_is_submitted_for_a_current_toolpath_without_a_running_or_done_job() {
        use OperationJobStatus::{Canceled, Failed, Queued, Running, Succeeded};
        let not_done = [
            OperationJobState::NotRead,
            OperationJobState::NotSubmitted,
            OperationJobState::Status(Failed),
            OperationJobState::Status(Canceled),
        ];
        let done_or_pending = [Queued, Running, Succeeded].map(OperationJobState::Status);
        for state in not_done {
            assert_eq!(
                SubmissionKind::of(ImportedToolpathState::Current, state),
                Some(SubmissionKind::SimulationOnly)
            );
            assert_eq!(
                SubmissionKind::of(ImportedToolpathState::None, state),
                Some(SubmissionKind::Full)
            );
        }
        // 計算待ち・計算中・計算済みの計算は、重ねて投入しない
        for state in done_or_pending {
            for toolpath in [ImportedToolpathState::None, ImportedToolpathState::Current] {
                assert_eq!(SubmissionKind::of(toolpath, state), None);
            }
        }
    }

    #[test]
    fn only_the_toolpath_is_imported_and_redoing_removes_it_and_submits_again() {
        let (mut session, mut document) = coarse_session();
        assert_eq!(
            session.submit(&mut document, |_| true).unwrap(),
            SubmitOutcome::Submitted(SubmissionKind::Full)
        );
        // 計算待ちの間は、重ねて投入しない
        assert_eq!(
            session.submit(&mut document, |_| true).unwrap(),
            SubmitOutcome::AlreadyCalculated
        );
        // 計算が終わるまでは、CL を取り込めない
        assert!(matches!(
            session.import_toolpath(&mut document),
            Err(CuttingSimulationSessionError::NoSucceededToolpath)
        ));
        session.run_queued_jobs().unwrap();
        // ツールパス生成のジョブ（1 つ目）の CL だけを取り込み、保存していない変更になる
        assert_eq!(session.import_toolpath(&mut document).unwrap(), 1);
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::Current
        );
        assert!(document.is_modified());
        // 今の改訂の計算が正常に終わっているため、投入しない
        assert_eq!(
            session.submit(&mut document, |_| true).unwrap(),
            SubmitOutcome::AlreadyCalculated
        );

        // やり直しは、CL を除いてツールパス生成から投入する（計算待ち・計算中のジョブはない）
        assert_eq!(
            session.redo(&mut document, |_| true).unwrap(),
            RedoOutcome::Submitted { canceled: false }
        );
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::None
        );
        assert_eq!(session.run_queued_jobs().unwrap(), 2);
        // 計算の条件を変えると前の条件の CL は除き、誤った条件では投入できない
        session.import_toolpath(&mut document).unwrap();
        let wrong = with_scanline(document.operation_content().clone(), |scanline| {
            scanline.stepover = 1000.0
        });
        document.set_operation_content(wrong);
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::None
        );
        assert!(session.redo(&mut document, |_| true).is_err());
    }

    #[test]
    fn changing_only_the_attributes_keeps_the_imported_toolpath_current() {
        let (mut session, mut document) = coarse_session();
        session.submit(&mut document, |_| true).unwrap();
        session.run_queued_jobs().unwrap();
        session.import_toolpath(&mut document).unwrap();

        // 名前・加工の段階は計算に使わないため、変えても取り込んだ CL は今の条件の結果のまま（計算し直さない）
        let mut content = document.operation_content().clone();
        content.attributes.name = "荒".to_string();
        content.attributes.stage = MachiningStage::Rough;
        document.set_operation_content(content);
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::Current
        );
        assert_eq!(
            session.submit(&mut document, |_| true).unwrap(),
            SubmitOutcome::AlreadyCalculated
        );

        // 計算の条件を変えると、前の条件の CL を除く（保存していない変更になる）
        document
            .save_as(&temporary_model_file("attributes"), "0.1.0")
            .unwrap();
        let changed = with_scanline(document.operation_content().clone(), |scanline| {
            scanline.stepover = 5.0
        });
        document.set_operation_content(changed);
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::None
        );
        assert!(document.toolpaths().records().is_empty());
        assert!(document.is_modified());
        std::fs::remove_file(document.path().unwrap()).unwrap();
    }

    /// 試験用のモデルファイルの場所（名前に `name` を含める）
    fn temporary_model_file(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "redring_cutting_simulation_session_{name}_{}.rrm",
            std::process::id()
        ))
    }

    #[test]
    fn the_cl_of_previous_conditions_left_in_a_file_is_removed_when_opened() {
        use application::space_model_file::{load_space_model, save_space_model};

        let (mut session, mut document) = coarse_session();
        session.submit(&mut document, |_| true).unwrap();
        session.run_queued_jobs().unwrap();
        session.import_toolpath(&mut document).unwrap();
        let path = temporary_model_file("outdated");
        document.save_as(&path, "0.1.0").unwrap();
        // 前の版で、素材を変えた後に前の条件の CL を残して保存したファイル
        let (mut model, info) = load_space_model(&path).unwrap();
        let sequence = model.sequences.sequence_ids()[0];
        model
            .sequences
            .set_stock(
                sequence,
                MachiningStock {
                    precision: 0.2,
                    ..document.stock()
                },
            )
            .unwrap();
        assert_eq!(model.toolpaths.records().len(), 1);
        save_space_model(&path, &model, &info).unwrap();

        // 開くときに除き、保存していない変更にはしない
        let opened = SpaceModelDocument::open(&path, NAMES).unwrap();
        assert!(opened.toolpaths().records().is_empty());
        assert_eq!(
            opened.imported_toolpath_state(),
            ImportedToolpathState::None
        );
        assert!(!opened.is_modified());
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn stopping_cancels_the_waiting_calculation_of_the_current_conditions() {
        let (mut session, mut document) = coarse_session();
        assert_eq!(
            session.submit(&mut document, |_| true).unwrap(),
            SubmitOutcome::Submitted(SubmissionKind::Full)
        );
        assert_eq!(session.stop(&document).unwrap(), StopOutcome::Stopped);
        // 取り消したジョブは計算しない
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
        assert_eq!(
            session.job_states(&mut document).unwrap()[0].1,
            OperationJobState::Status(OperationJobStatus::Canceled)
        );
        // 計算待ち・計算中のジョブがなければ、何もしない
        assert_eq!(session.stop(&document).unwrap(), StopOutcome::NothingToStop);
    }

    #[test]
    fn recalculating_while_waiting_submits_nothing() {
        let (mut session, mut document) = coarse_session();
        assert_eq!(
            session.submit(&mut document, |_| true).unwrap(),
            SubmitOutcome::Submitted(SubmissionKind::Full)
        );
        // 計算待ちの計算は、まだ始まっていないため、再計算しない（重ねて投入しない）
        assert_eq!(
            session.redo(&mut document, |_| true).unwrap(),
            RedoOutcome::Waiting
        );
        assert_eq!(session.run_queued_jobs().unwrap(), 2);
        // 計算が終わった後の再計算は、取り消すジョブがない
        assert_eq!(
            session.redo(&mut document, |_| true).unwrap(),
            RedoOutcome::Submitted { canceled: false }
        );
    }

    #[test]
    fn submitted_product_shapes_are_imported_as_a_cutting_animation_after_running() {
        let (mut session, mut document) = coarse_session();
        assert_eq!(session.check(&mut document), vec![]);
        session.submit(&mut document, |_| true).unwrap();
        // 実行するまでは取り込めない
        assert!(matches!(
            session.import_latest(&mut document),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));

        assert_eq!(session.run_queued_jobs().unwrap(), 2);
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::None
        );
        let mut imported = session.import_latest(&mut document).unwrap();
        assert_eq!(imported.job_id, 2);
        // CL を空間モデルに取り込み、保存していない変更になる
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::Current
        );
        assert!(document.is_modified());
        assert!(imported.animation.frame_count() > 1);
        assert!(!imported.animation.take_initial_stock_blocks().is_empty());
    }

    #[test]
    fn results_of_previous_conditions_are_not_imported_until_submitted_again() {
        let (mut session, mut document) = coarse_session();
        session.submit(&mut document, |_| true).unwrap();
        session.run_queued_jobs().unwrap();
        assert_eq!(session.import_latest(&mut document).unwrap().job_id, 2);

        // 素材を変えると、前の条件の結果は取り込まない。取り込んだ CL は前の条件の結果のため除く
        document.set_stock(MachiningStock {
            precision: 0.2,
            ..document.stock()
        });
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::None
        );
        assert!(matches!(
            session.import_latest(&mut document),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));

        session.submit(&mut document, |_| true).unwrap();
        session.run_queued_jobs().unwrap();
        let imported = session.import_latest(&mut document).unwrap();
        assert_eq!(imported.job_id, 4);
        assert_eq!(imported.conditions.precision, 0.2);
        // 同じ条件を設定し直しても、今の条件の結果のまま取り込める
        document.set_operation_content(document.operation_content().clone());
        document.set_stock(document.stock());
        assert_eq!(session.import_latest(&mut document).unwrap().job_id, 4);
        // 工程の中身を変えても、前の条件の結果は取り込まない
        let content = with_scanline(document.operation_content().clone(), |scanline| {
            scanline.feed_rate = 600.0
        });
        document.set_operation_content(content);
        assert!(matches!(
            session.import_latest(&mut document),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));
    }

    #[test]
    fn results_of_other_target_faces_are_not_imported() {
        let (mut session, mut document) = coarse_session();
        session.submit(&mut document, |_| true).unwrap();
        session.run_queued_jobs().unwrap();
        // 製品形状を加えると、表示中の面（加工の対象）が変わり、前の対象の結果は取り込まない
        let more = document_with_pyramid();
        let path = std::env::temp_dir().join(format!(
            "redring_cutting_simulation_session_more_{}.stl",
            std::process::id()
        ));
        let mesh = match more.shapes().records().remove(0).geometry {
            application::product_shape::ProductFaceGeometry::Mesh(mesh) => mesh,
            application::product_shape::ProductFaceGeometry::Surface(_) => unreachable!(),
        };
        geo_io::stl::save_stl(&mesh, &path).unwrap();
        document.import_stl(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(matches!(
            session.import_latest(&mut document),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));
    }

    #[test]
    fn the_edited_conditions_are_submitted_and_returned_with_the_imported_result() {
        let (mut session, mut document) = coarse_session();
        let mut content = document.operation_content().clone();
        content.conditions.tool = CuttingToolKind::FlatEndMill.tool(6.0, Some(25.0));
        let content = with_scanline(content, |scanline| {
            scanline.cut_direction = ScanlineCutDirection::ZigZag
        });
        document.set_operation_content(content.clone());
        document.set_stock(MachiningStock {
            margins: StockMargins {
                top: 2.0,
                ..StockMargins::default()
            },
            precision: 0.2,
        });
        session.submit(&mut document, |_| true).unwrap();
        session.run_queued_jobs().unwrap();

        let imported = session.import_latest(&mut document).unwrap().conditions;
        assert_eq!(imported.tool, content.conditions.tool);
        assert_eq!(
            OperationSpec::Scanline(imported.scanline),
            content.conditions.spec
        );
        assert_eq!(imported.toolpath_tolerance, document.toolpath_tolerance());
        assert_eq!(imported.precision, 0.2);
        // 四角錐（0〜20, 0〜20, 0〜6）を囲む箱の上に 2 を足した素材
        assert_eq!(imported.stock.max().z(), 8.0);
        assert_eq!(imported.stock.min().x(), 0.0);
    }

    #[test]
    fn conditions_with_errors_are_reported_by_field_and_not_submitted() {
        let (mut session, mut document) = coarse_session();
        document.set_stock(MachiningStock {
            margins: StockMargins {
                bottom: -1.0,
                ..StockMargins::default()
            },
            precision: 0.0005,
        });
        // ピックフィードが工具径 6 mm を超える
        let content = with_scanline(document.operation_content().clone(), |scanline| {
            scanline.stepover = 7.0
        });
        document.set_operation_content(content);
        let issues = session.check(&mut document);
        assert_eq!(
            issues,
            vec![
                ConditionIssue::Condition(CamConditionError::InvalidStockMargins(StockMargins {
                    bottom: -1.0,
                    ..StockMargins::default()
                })),
                ConditionIssue::Condition(CamConditionError::PrecisionFinerThanTolerance {
                    precision: 0.0005,
                    tolerance: document.toolpath_tolerance()
                }),
                ConditionIssue::Condition(CamConditionError::StepoverExceedsToolDiameter {
                    tool_diameter: 6.0,
                }),
            ]
        );
        assert!(session.submit(&mut document, |_| true).is_err());
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }

    #[test]
    fn conditions_not_applied_yet_are_checked_without_changing_the_document() {
        let (session, mut document) = coarse_session();
        let applied = document.operation_content().clone();
        let stock = document.stock();
        let state = document.imported_toolpath_state();
        let modified = document.is_modified();
        // ピックフィードが工具径 6 mm を超える条件は、適用する前でも誤りとして分かる
        let draft = with_scanline(applied.clone(), |scanline| scanline.stepover = 7.0);
        assert_eq!(
            session.check_draft(&mut document, &draft, stock),
            vec![ConditionIssue::Condition(
                CamConditionError::StepoverExceedsToolDiameter { tool_diameter: 6.0 }
            )]
        );
        assert!(
            session
                .check_draft(&mut document, &applied, stock)
                .is_empty()
        );
        // 文書の工程・手順は替えない（再計算待ちにも、保存していない変更にもならない）
        assert_eq!(*document.operation_content(), applied);
        assert_eq!(document.stock(), stock);
        assert_eq!(document.imported_toolpath_state(), state);
        assert_eq!(document.is_modified(), modified);
        assert!(session.check(&mut document).is_empty());
    }

    #[test]
    fn operations_whose_conditions_are_not_set_are_not_submitted() {
        let mut session = CuttingSimulationSession::in_process();
        let mut document = document_with_pyramid();
        // 作った工程は、条件を設定していない
        assert!(!document.conditions_set_of(document.selected_operation()));
        assert_eq!(session.check(&mut document), vec![ConditionIssue::NotSet]);
        assert!(matches!(
            session.submit(&mut document, |_| true),
            Err(CuttingSimulationSessionError::ConditionsNotSet)
        ));
        assert!(matches!(
            session.redo(&mut document, |_| true),
            Err(CuttingSimulationSessionError::ConditionsNotSet)
        ));
        // 全工程の計算は、設定していない工程があれば、どの工程も投入しない
        coarse(&mut document);
        document.add_operation("追加".to_string());
        assert!(!document.conditions_set_of(document.selected_operation()));
        assert!(matches!(
            session.submit_sequence(&mut document, |_| true),
            Err(SequenceSubmissionError::Conditions {
                position: 2,
                error: CuttingSimulationSessionError::ConditionsNotSet,
                ..
            })
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
        // 工程複写は、設定したかを受け継ぐ
        document.duplicate_operation("複写".to_string());
        assert!(!document.conditions_set_of(document.selected_operation()));
        coarse(&mut document);
        document.duplicate_operation("複写の複写".to_string());
        assert!(document.conditions_set_of(document.selected_operation()));
    }

    #[test]
    fn operations_whose_conditions_are_not_set_stay_so_after_saving_and_opening() {
        let mut document = document_with_pyramid();
        let path = temporary_model_file("not_set");
        document.save_as(&path, "0.1.0").unwrap();
        let mut opened = SpaceModelDocument::open(&path, NAMES).unwrap();
        assert!(!opened.conditions_set_of(opened.selected_operation()));
        // 設定すると保存していない変更になり、保存して開き直しても設定したまま
        opened.mark_conditions_set();
        assert!(opened.is_modified());
        opened.save_as(&path, "0.1.0").unwrap();
        let reopened = SpaceModelDocument::open(&path, NAMES).unwrap();
        assert!(reopened.conditions_set_of(reopened.selected_operation()));
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn operation_condition_issues_are_listed_without_the_ones_not_set() {
        let (mut session, mut document) = coarse_session();
        let first = document.selected_operation();
        assert_eq!(
            session.operation_condition_issues(&mut document),
            Vec::new()
        );
        // ピックフィードが工具径を超える工程
        document.set_operation_content(with_scanline(
            document.operation_content().clone(),
            |scanline| scanline.stepover = 7.0,
        ));
        // 設定していない工程は、条件に誤りがあっても含めない
        document.add_operation("追加".to_string());
        document.set_operation_content(with_scanline(
            document.operation_content().clone(),
            |scanline| scanline.stepover = 1000.0,
        ));
        // 問題は、工程を選んで確かめたときと同じ
        let added = document.selected_operation();
        document.select_operation(first);
        let issues = session.check(&mut document);
        assert!(!issues.is_empty());
        document.select_operation(added);
        assert_eq!(
            session.operation_condition_issues(&mut document),
            vec![(first, issues)]
        );
        // 改訂が進んだ工程だけ確かめ直す（直すと、誤りのある工程から外れる）
        document.select_operation(first);
        coarse(&mut document);
        assert_eq!(
            session.operation_condition_issues(&mut document),
            Vec::new()
        );
    }

    #[test]
    fn nothing_is_submitted_without_visible_product_shapes() {
        let mut session = CuttingSimulationSession::in_process();
        let mut document = SpaceModelDocument::new(NAMES);
        coarse(&mut document);
        assert!(matches!(
            session.check(&mut document)[..],
            [ConditionIssue::NoProductShape]
        ));
        // 工程リストの条件の問題も、工程を選んで確かめたときと同じ
        assert_eq!(
            session.operation_condition_issues(&mut document),
            vec![(document.selected_operation(), session.check(&mut document))]
        );
        let error = session.submit(&mut document, |_| true).unwrap_err();
        assert!(matches!(
            error,
            CuttingSimulationSessionError::Workflow(CamSimulationWorkflowError::ProductShape(
                ProductShapeError::NoFaces
            ))
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }
}
