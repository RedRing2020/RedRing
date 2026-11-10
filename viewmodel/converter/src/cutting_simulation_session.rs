//! 製品形状からの切削シミュレーションの投入・実行・取り込み
//!
//! 空間モデルの文書（`SpaceModelDocument`）の、編集の対象の加工工程から投入・取り込みを行う。加工の対象は、確かめ・
//! 投入・取り込みのときに表示中の製品形状のすべての面にする。計算はアプリの外のジョブマネージャーが行い、セッションは
//! 投入・状態の読み取り・取り込みだけを行う。

use crate::space_model_document::MachiningOperationId;
use application::cam_simulation_workflow::{
    Aabb3D, CamSimulationWorkflow, CamSimulationWorkflowError,
};
pub use application::cam_simulation_workflow::{
    CamConditionError, CamConditionField, OperationJobState, OperationJobStatus,
};
pub use application::machining_sequence::{
    flat_surface_cusp_height, MachiningBoundary, MachiningOperationContent, MachiningStage,
    MachiningStock, OperationSpec, ScanlineCutDirection, ScanlineToolRange, StockMargins, Tool,
};
use application::machining_sequence::{MachiningSequenceStore, ScanlineParams};

use application::cutting_simulation_playback::{
    CuttingSimulationPlayback, CuttingSimulationPlaybackError,
};

use crate::cutting_animation::CuttingAnimation;
use crate::failure_message_mapper::workflow_failure_message;
use crate::inverse_offset_shape_converter::{inverse_offset_shape_meshes, InverseOffsetShapeMesh};
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

/// 計算条件の確かめで見つかった問題（製品形状がない・条件の誤り・そのほか）
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionIssue {
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
}

impl CuttingSimulationSession {
    /// 同じプロセスの中でジョブを実行するセッション（試験用）
    #[cfg(test)]
    pub(crate) fn in_process() -> Self {
        Self {
            workflow: CamSimulationWorkflow::in_process(),
        }
    }

    /// 表示中の製品形状と加工工程・加工手順の条件を、投入と同じ判定で確かめ、問題をすべて返す（問題がなければ空）
    pub fn check(&self, document: &mut SpaceModelDocument) -> Vec<ConditionIssue> {
        document.target_visible_faces();
        let (shapes, sequences, sequence, operation) = document.machining();
        match CamSimulationWorkflow::check_operation(shapes, sequences, sequence, operation) {
            Ok(()) => Vec::new(),
            Err(error) => ConditionIssue::from_workflow_error(&error),
        }
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
    /// 今の改訂の計算が計算待ち・計算中・正常に終わっていれば、重ねて投入せずに `None` を返す。そうでなく、空間モデル
    /// に今の条件の CL があれば、その CL を入力にして切削シミュレーションだけを投入する。どちらでもなければ、ツール
    /// パス生成から投入する。
    ///
    /// # Errors
    ///
    /// 表示中の製品形状がない場合・計算条件に誤りがある場合を含む、投入の失敗。
    pub fn submit(
        &mut self,
        document: &mut SpaceModelDocument,
    ) -> Result<Option<SubmissionKind>, CuttingSimulationSessionError> {
        document.target_visible_faces();
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
            return Ok(None);
        };
        self.submit_kind(document, operation, kind)?;
        Ok(Some(kind))
    }

    /// 表示中の製品形状のすべての面を加工の対象として、加工工程の空間モデルの CL を除き、ツールパス生成から投入し直す
    /// （実行しない。CL は、投入できたときだけ除く）
    ///
    /// # Errors
    ///
    /// 表示中の製品形状がない場合・計算条件に誤りがある場合を含む、投入の失敗。
    pub fn redo(
        &mut self,
        document: &mut SpaceModelDocument,
    ) -> Result<(), CuttingSimulationSessionError> {
        document.target_visible_faces();
        let (shapes, sequences, sequence, operation) = document.machining();
        self.workflow
            .submit_operation(shapes, sequences, sequence, operation)?;
        document.remove_toolpath();
        Ok(())
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
    /// と同じく決める。
    ///
    /// # Errors
    ///
    /// 工程の条件の誤り・計算の状態を読めていない（どの工程も投入しない）、投入の失敗（その工程の前までは投入した）の
    /// 場合。
    pub fn submit_sequence(
        &mut self,
        document: &mut SpaceModelDocument,
    ) -> Result<SequenceSubmission, SequenceSubmissionError> {
        document.target_visible_faces();
        let operations = document.operations();
        let (shapes, sequences, sequence, _) = document.machining();
        for (index, item) in operations.iter().enumerate() {
            CamSimulationWorkflow::check_operation(shapes, sequences, sequence, item.id).map_err(
                |error| SequenceSubmissionError::Conditions {
                    position: index + 1,
                    operation: item.name.clone(),
                    error: error.into(),
                },
            )?;
        }
        let ids: Vec<MachiningOperationId> = operations.iter().map(|item| item.id).collect();
        // 計算待ち・計算中・計算済みの工程を投入し直さないよう、状態を今読んでから決める（ジョブマネージャーが動いて
        // いなければ起動する）
        let states = self
            .workflow
            .latest_revision_job_states_now(sequences, sequence, &ids)
            .map_err(|error| SequenceSubmissionError::StatusUnreadable(error.into()))?;
        let mut submission = SequenceSubmission {
            submitted: 0,
            simulation_only: 0,
            skipped: 0,
        };
        for (index, (item, state)) in operations.iter().zip(states).enumerate() {
            let Some(kind) =
                SubmissionKind::of(document.imported_toolpath_state_of(item.id), state)
            else {
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
        Ok(submission)
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
                let OperationSpec::Scanline(scanline) = document.operation_content().spec;
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
    use crate::space_model_document::tests::{document_with_pyramid, NAMES};
    use crate::space_model_document::ImportedToolpathState;

    /// 走査線の条件 `content` を `edit` で変えた中身
    fn with_scanline(
        content: MachiningOperationContent,
        edit: impl FnOnce(&mut ScanlineParams),
    ) -> MachiningOperationContent {
        let OperationSpec::Scanline(mut scanline) = content.spec;
        edit(&mut scanline);
        MachiningOperationContent {
            spec: OperationSpec::Scanline(scanline),
            ..content
        }
    }

    /// 文書 `document` の加工工程を、粗い条件（直径 6 mm・工具長 30 mm のボールエンドミル、ピックフィード 6 mm。
    /// テストを速くする）にする
    fn coarse(document: &mut SpaceModelDocument) {
        let content = MachiningOperationContent {
            tool: CuttingToolKind::BallEndMill.tool(6.0, Some(30.0)),
            ..document.operation_content().clone()
        };
        document.set_operation_content(with_scanline(content, |scanline| scanline.stepover = 6.0));
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
            content.tool.cutting_length = length;
            document.set_operation_content(content);
        }
        session.submit(&mut document).unwrap();
        session.run_queued_jobs().unwrap();
        session.import_latest(&mut document).unwrap().animation
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
        let OperationSpec::Scanline(mut scanline) = wrong.spec;
        scanline.stepover = 1000.0;
        wrong.spec = OperationSpec::Scanline(scanline);
        document.set_operation_content(wrong);
        assert!(matches!(
            session.submit_sequence(&mut document),
            Err(SequenceSubmissionError::Conditions { position: 2, ref operation, .. })
                if operation == "仕上げ"
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);

        // 誤りを直すと、すべての工程を投入する
        let mut fixed = document.operation_content().clone();
        let OperationSpec::Scanline(mut scanline) = fixed.spec;
        scanline.stepover = 3.0;
        fixed.spec = OperationSpec::Scanline(scanline);
        document.set_operation_content(fixed);
        assert_eq!(
            session.submit_sequence(&mut document).unwrap(),
            SequenceSubmission {
                submitted: 2,
                simulation_only: 0,
                skipped: 0
            }
        );
        // 計算待ち・計算済みの工程は、投入し直さない
        assert_eq!(
            session.submit_sequence(&mut document).unwrap(),
            SequenceSubmission {
                submitted: 0,
                simulation_only: 0,
                skipped: 2
            }
        );
        assert_eq!(session.run_queued_jobs().unwrap(), 4);
        assert_eq!(
            session.submit_sequence(&mut document).unwrap(),
            SequenceSubmission {
                submitted: 0,
                simulation_only: 0,
                skipped: 2
            }
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
            for toolpath in [ImportedToolpathState::None, ImportedToolpathState::Outdated] {
                assert_eq!(
                    SubmissionKind::of(toolpath, state),
                    Some(SubmissionKind::Full)
                );
            }
        }
        // 計算待ち・計算中・計算済みの計算は、重ねて投入しない
        for state in done_or_pending {
            for toolpath in [
                ImportedToolpathState::None,
                ImportedToolpathState::Outdated,
                ImportedToolpathState::Current,
            ] {
                assert_eq!(SubmissionKind::of(toolpath, state), None);
            }
        }
    }

    #[test]
    fn only_the_toolpath_is_imported_and_redoing_removes_it_and_submits_again() {
        let (mut session, mut document) = coarse_session();
        assert_eq!(
            session.submit(&mut document).unwrap(),
            Some(SubmissionKind::Full)
        );
        // 計算待ちの間は、重ねて投入しない
        assert_eq!(session.submit(&mut document).unwrap(), None);
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
        assert_eq!(session.submit(&mut document).unwrap(), None);

        // やり直しは、CL を除いてツールパス生成から投入する
        session.redo(&mut document).unwrap();
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::None
        );
        assert_eq!(session.run_queued_jobs().unwrap(), 2);
        // 投入できなければ、CL を除かない
        session.import_toolpath(&mut document).unwrap();
        let wrong = with_scanline(document.operation_content().clone(), |scanline| {
            scanline.stepover = 1000.0
        });
        document.set_operation_content(wrong);
        assert!(session.redo(&mut document).is_err());
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::Outdated
        );
    }

    #[test]
    fn submitted_product_shapes_are_imported_as_a_cutting_animation_after_running() {
        let (mut session, mut document) = coarse_session();
        assert_eq!(session.check(&mut document), vec![]);
        session.submit(&mut document).unwrap();
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
        session.submit(&mut document).unwrap();
        session.run_queued_jobs().unwrap();
        assert_eq!(session.import_latest(&mut document).unwrap().job_id, 2);

        // 素材を変えると、前の条件の結果は取り込まない（再計算待ち）。取り込んだ CL は前の条件の結果になる
        document.set_stock(MachiningStock {
            precision: 0.2,
            ..document.stock()
        });
        assert_eq!(
            document.imported_toolpath_state(),
            ImportedToolpathState::Outdated
        );
        assert!(matches!(
            session.import_latest(&mut document),
            Err(CuttingSimulationSessionError::NoSucceededSimulation)
        ));

        session.submit(&mut document).unwrap();
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
        session.submit(&mut document).unwrap();
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
        let content = with_scanline(
            MachiningOperationContent {
                tool: CuttingToolKind::FlatEndMill.tool(6.0, Some(25.0)),
                ..document.operation_content().clone()
            },
            |scanline| scanline.cut_direction = ScanlineCutDirection::ZigZag,
        );
        document.set_operation_content(content.clone());
        document.set_stock(MachiningStock {
            margins: StockMargins {
                top: 2.0,
                ..StockMargins::default()
            },
            precision: 0.2,
        });
        session.submit(&mut document).unwrap();
        session.run_queued_jobs().unwrap();

        let imported = session.import_latest(&mut document).unwrap().conditions;
        assert_eq!(imported.tool, content.tool);
        assert_eq!(OperationSpec::Scanline(imported.scanline), content.spec);
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
        assert!(session.submit(&mut document).is_err());
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
        assert!(session
            .check_draft(&mut document, &applied, stock)
            .is_empty());
        // 文書の工程・手順は替えない（再計算待ちにも、保存していない変更にもならない）
        assert_eq!(*document.operation_content(), applied);
        assert_eq!(document.stock(), stock);
        assert_eq!(document.imported_toolpath_state(), state);
        assert_eq!(document.is_modified(), modified);
        assert!(session.check(&mut document).is_empty());
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
        let error = session.submit(&mut document).unwrap_err();
        assert!(matches!(
            error,
            CuttingSimulationSessionError::Workflow(CamSimulationWorkflowError::ProductShape(
                ProductShapeError::NoFaces
            ))
        ));
        assert_eq!(session.run_queued_jobs().unwrap(), 0);
    }
}
