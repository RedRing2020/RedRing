//! 加工手順・加工工程のエンティティの保管と編集（設計: CAM_PROCESS_ENTITY_DESIGN.md）
//!
//! エンティティは保管先の中だけで持ち、作成と編集は保管先の関数で行う。関数に出てくる値の型（工程の中身・素材・ID と、
//! 中身の工具・加工の種類と条件・素材の余裕）は、ここから再エクスポートする。

use std::collections::HashSet;
use std::fmt;
use std::num::NonZeroU32;

use cam_core::ToleranceProfile;
pub use cam_core::{
    MachiningBoundary, MachiningStage, OperationSpec, ScanlineCutDirection, ScanlineParams,
    ScanlineToolRange, StockMargins, Tool,
};
use cam_entity::{MachiningOperation, MachiningSequence};
pub use cam_entity::{
    MachiningOperationContent, MachiningOperationId, MachiningRevisionKey, MachiningSequenceId,
    MachiningStock,
};
use geo_entity::FaceId;

/// 加工工程の中身 `content` で、平らな面を隣り合うパスの間がピックフィードで削ったときに残る山の高さ（mm）
///
/// ボールエンドミルは r − √(r² − (s/2)²)、フラットエンドミルは底が平らなので 0 とする。パスの間はピックフィード
/// 以下になるため、平らな面での上限の値になる。傾いた面では山の高さが変わるため、目安の値とする。ピックフィードが
/// 工具径を超える場合と、ほかの種類の工具の場合は `None`。
pub fn flat_surface_cusp_height(content: &MachiningOperationContent) -> Option<f64> {
    let OperationSpec::Scanline(scanline) = content.spec;
    let tool = &content.tool;
    let stepover = scanline.stepover;
    if stepover > tool.diameter() {
        return None;
    }
    if tool.is_ball_end_mill() {
        let radius = tool.radius();
        let half = 0.5 * stepover;
        Some(radius - (radius * radius - half * half).sqrt())
    } else if tool.is_flat_end_mill() {
        Some(0.0)
    } else {
        None
    }
}

/// 加工手順の参照・確認の失敗
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachiningSequenceError {
    /// 保管していない加工手順
    UnknownSequence(MachiningSequenceId),
    /// 加工手順にない加工工程
    UnknownOperation(MachiningOperationId),
}

impl fmt::Display for MachiningSequenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSequence(id) => write!(f, "unknown machining sequence: {id}"),
            Self::UnknownOperation(id) => write!(f, "unknown machining operation: {id}"),
        }
    }
}

impl std::error::Error for MachiningSequenceError {}

/// 加工工程の記録（保管先から取り出し、保管先に戻す中身）
#[derive(Debug, Clone, PartialEq)]
pub struct MachiningOperationRecord {
    pub id: MachiningOperationId,
    pub content: MachiningOperationContent,
    pub revision: NonZeroU32,
    pub revision_key: MachiningRevisionKey,
}

/// 加工手順の記録（保管先から取り出し、保管先に戻す中身）
#[derive(Debug, Clone, PartialEq)]
pub struct MachiningSequenceRecord {
    pub id: MachiningSequenceId,
    pub name: String,
    /// 加工の対象（製品形状の面の ID。同じ面は 1 回だけ）
    pub target_faces: Vec<FaceId>,
    pub stock: MachiningStock,
    /// 加工の順の加工工程
    pub operations: Vec<MachiningOperationRecord>,
}

/// 記録から加工手順の保管先に戻すときの誤り
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachiningSequenceRestoreError {
    /// 加工手順の ID が重なっている
    DuplicateSequence(MachiningSequenceId),
    /// 加工工程の ID が重なっている（ほかの加工手順の工程とも）
    DuplicateOperation(MachiningOperationId),
    /// 加工手順の加工の対象の面の ID が重なっている
    DuplicateTargetFace {
        sequence: MachiningSequenceId,
        face: FaceId,
    },
}

impl fmt::Display for MachiningSequenceRestoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSequence(id) => write!(f, "duplicate machining sequence: {id}"),
            Self::DuplicateOperation(id) => write!(f, "duplicate machining operation: {id}"),
            Self::DuplicateTargetFace { sequence, face } => {
                write!(
                    f,
                    "duplicate target face {face} in machining sequence {sequence}"
                )
            }
        }
    }
}

impl std::error::Error for MachiningSequenceRestoreError {}

/// 走査線の加工工程を作るときの既定の精度プロファイル（工程テンプレートを登録できるようになるまでの既定）
const DEFAULT_TOLERANCE_PROFILE: ToleranceProfile = ToleranceProfile::PressRough;

/// 工具 `tool` で走査線の条件 `scanline` の仕上げを行う加工工程の中身（名前は `name`、加工領域は加工の対象の
/// XY の範囲、精度プロファイルは既定）
pub fn scanline_finish_operation_content(
    name: String,
    tool: Tool<f64>,
    scanline: ScanlineParams,
) -> MachiningOperationContent {
    MachiningOperationContent {
        name,
        stage: MachiningStage::Finish,
        spec: OperationSpec::Scanline(scanline),
        tool,
        boundary: None,
        tolerance_profile: DEFAULT_TOLERANCE_PROFILE,
    }
}

/// 加工手順の保管先（加えた順に保持する。加工工程は、属する加工手順が持つ）
#[derive(Debug, Default)]
pub struct MachiningSequenceStore {
    sequences: Vec<MachiningSequence>,
}

impl MachiningSequenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 記録 `records` を、その順に持つ保管先を作る（ID・改訂の番号・改訂の鍵は記録のまま）
    ///
    /// 加工の対象の面が製品形状にない加工手順も、そのまま戻す（投入前の確認で誤りになる）。
    ///
    /// # Errors
    ///
    /// 加工手順の ID・加工工程の ID・加工手順の加工の対象の面の ID が重なっている場合。
    pub fn from_records(
        records: Vec<MachiningSequenceRecord>,
    ) -> Result<Self, MachiningSequenceRestoreError> {
        let mut sequence_ids = HashSet::new();
        let mut operation_ids = HashSet::new();
        let mut sequences = Vec::with_capacity(records.len());
        for record in records {
            if !sequence_ids.insert(record.id) {
                return Err(MachiningSequenceRestoreError::DuplicateSequence(record.id));
            }
            let mut faces = HashSet::new();
            if let Some(&face) = record
                .target_faces
                .iter()
                .find(|face| !faces.insert(**face))
            {
                return Err(MachiningSequenceRestoreError::DuplicateTargetFace {
                    sequence: record.id,
                    face,
                });
            }
            let mut operations = Vec::with_capacity(record.operations.len());
            for operation in record.operations {
                if !operation_ids.insert(operation.id) {
                    return Err(MachiningSequenceRestoreError::DuplicateOperation(
                        operation.id,
                    ));
                }
                operations.push(MachiningOperation::restore(
                    operation.id,
                    operation.content,
                    operation.revision,
                    operation.revision_key,
                ));
            }
            sequences.push(MachiningSequence::restore(
                record.id,
                record.name,
                record.target_faces,
                record.stock,
                operations,
            ));
        }
        Ok(Self { sequences })
    }

    /// 保管している加工手順の記録（加えた順）
    pub fn records(&self) -> Vec<MachiningSequenceRecord> {
        self.sequences
            .iter()
            .map(|sequence| MachiningSequenceRecord {
                id: sequence.id(),
                name: sequence.name().to_string(),
                target_faces: sequence.target_faces().to_vec(),
                stock: *sequence.stock(),
                operations: sequence
                    .operations()
                    .iter()
                    .map(|operation| MachiningOperationRecord {
                        id: operation.id(),
                        content: operation.content().clone(),
                        revision: NonZeroU32::new(operation.revision())
                            .expect("the revision of an operation starts at 1 and only increases"),
                        revision_key: operation.revision_key(),
                    })
                    .collect(),
            })
            .collect()
    }

    /// 加工工程のない加工手順を作って加え、その ID を返す（加工の対象の同じ面は 1 回だけ持つ）
    pub fn create_sequence(
        &mut self,
        name: String,
        target_faces: Vec<FaceId>,
        stock: MachiningStock,
    ) -> MachiningSequenceId {
        let sequence = MachiningSequence::new(name, target_faces, stock);
        let id = sequence.id();
        self.sequences.push(sequence);
        id
    }

    /// 加工手順 `sequence` の最後に、中身が `content` の加工工程を作って加え、その ID を返す
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub fn add_operation(
        &mut self,
        sequence: MachiningSequenceId,
        content: MachiningOperationContent,
    ) -> Result<MachiningOperationId, MachiningSequenceError> {
        let operation = MachiningOperation::new(content);
        let id = operation.id();
        self.sequence_mut(sequence)?.push_operation(operation);
        Ok(id)
    }

    /// 加工手順 `sequence` の加工の順の位置 `index` に、中身が `content` の加工工程を作って挿み、その ID を返す
    /// （工程の数より大きい位置は、最後に加える）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub fn insert_operation(
        &mut self,
        sequence: MachiningSequenceId,
        index: usize,
        content: MachiningOperationContent,
    ) -> Result<MachiningOperationId, MachiningSequenceError> {
        let operation = MachiningOperation::new(content);
        let id = operation.id();
        self.sequence_mut(sequence)?
            .insert_operation(index, operation);
        Ok(id)
    }

    /// 加工手順 `sequence` から加工工程 `operation` を除く
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn remove_operation(
        &mut self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
    ) -> Result<(), MachiningSequenceError> {
        if self.sequence_mut(sequence)?.remove_operation(operation) {
            Ok(())
        } else {
            Err(MachiningSequenceError::UnknownOperation(operation))
        }
    }

    /// 加工手順 `sequence` の加工工程 `operation` を、加工の順の位置 `index` に移す（工程の数以上なら最後へ）。工程の
    /// 改訂の番号・改訂の鍵は変えない（前の工程の素材の受け継ぎが変わることは、区間の前提で見分ける）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn move_operation(
        &mut self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
        index: usize,
    ) -> Result<(), MachiningSequenceError> {
        if self
            .sequence_mut(sequence)?
            .move_operation(operation, index)
        {
            Ok(())
        } else {
            Err(MachiningSequenceError::UnknownOperation(operation))
        }
    }

    /// 加工手順 `sequence` の加工工程 `operation` の、加工の順の位置
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn operation_position(
        &self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
    ) -> Result<usize, MachiningSequenceError> {
        self.sequence(sequence)?
            .position(operation)
            .ok_or(MachiningSequenceError::UnknownOperation(operation))
    }

    /// 加工手順 `sequence` の加工工程 `operation` の中身を `content` に替える（今と違えば、工程の改訂の番号が増える）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn set_operation_content(
        &mut self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
        content: MachiningOperationContent,
    ) -> Result<(), MachiningSequenceError> {
        self.sequence_mut(sequence)?
            .operation_mut(operation)
            .ok_or(MachiningSequenceError::UnknownOperation(operation))?
            .set_content(content);
        Ok(())
    }

    /// 加工手順 `sequence` の加工の対象を `target_faces` に替える（面の集合として今と違えば、すべての加工工程の
    /// 改訂の番号が増える）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub fn set_target_faces(
        &mut self,
        sequence: MachiningSequenceId,
        target_faces: Vec<FaceId>,
    ) -> Result<(), MachiningSequenceError> {
        self.sequence_mut(sequence)?.set_target_faces(target_faces);
        Ok(())
    }

    /// 加工手順 `sequence` の素材を `stock` に替える（今と違えば、すべての加工工程の改訂の番号が増える）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub fn set_stock(
        &mut self,
        sequence: MachiningSequenceId,
        stock: MachiningStock,
    ) -> Result<(), MachiningSequenceError> {
        self.sequence_mut(sequence)?.set_stock(stock);
        Ok(())
    }

    /// 保管している加工手順の ID（加えた順）
    pub fn sequence_ids(&self) -> Vec<MachiningSequenceId> {
        self.sequences.iter().map(MachiningSequence::id).collect()
    }

    /// 加工手順 `sequence` の加工工程の ID（加工の順）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub fn operation_ids(
        &self,
        sequence: MachiningSequenceId,
    ) -> Result<Vec<MachiningOperationId>, MachiningSequenceError> {
        Ok(self
            .sequence(sequence)?
            .operations()
            .iter()
            .map(MachiningOperation::id)
            .collect())
    }

    /// 加工手順 `sequence` の名前
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub fn sequence_name(
        &self,
        sequence: MachiningSequenceId,
    ) -> Result<&str, MachiningSequenceError> {
        Ok(self.sequence(sequence)?.name())
    }

    /// 加工手順 `sequence` の加工工程 `operation` の改訂の鍵
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn operation_revision_key(
        &self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
    ) -> Result<MachiningRevisionKey, MachiningSequenceError> {
        Ok(self.operation(sequence, operation)?.revision_key())
    }

    /// 加工手順 `sequence` の加工工程 `operation` の中身
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn operation_content(
        &self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
    ) -> Result<&MachiningOperationContent, MachiningSequenceError> {
        Ok(self.operation(sequence, operation)?.content())
    }

    /// 加工手順 `sequence` の素材
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub fn stock(
        &self,
        sequence: MachiningSequenceId,
    ) -> Result<&MachiningStock, MachiningSequenceError> {
        Ok(self.sequence(sequence)?.stock())
    }

    /// 加工手順 `sequence` の加工工程 `operation` のツールパス生成のトレランス（mm。工程の精度プロファイルから決まる）
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn operation_tolerance(
        &self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
    ) -> Result<f64, MachiningSequenceError> {
        Ok(self
            .operation(sequence, operation)?
            .content()
            .tolerance_profile
            .tolerance_mm())
    }

    /// 加工手順 `id`
    ///
    /// # Errors
    ///
    /// 保管していない加工手順の場合。
    pub(crate) fn sequence(
        &self,
        id: MachiningSequenceId,
    ) -> Result<&MachiningSequence, MachiningSequenceError> {
        self.sequences
            .iter()
            .find(|sequence| sequence.id() == id)
            .ok_or(MachiningSequenceError::UnknownSequence(id))
    }

    /// 加工手順 `sequence` の加工工程 `operation`
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub(crate) fn operation(
        &self,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
    ) -> Result<&MachiningOperation, MachiningSequenceError> {
        self.sequence(sequence)?
            .operations()
            .iter()
            .find(|candidate| candidate.id() == operation)
            .ok_or(MachiningSequenceError::UnknownOperation(operation))
    }

    fn sequence_mut(
        &mut self,
        id: MachiningSequenceId,
    ) -> Result<&mut MachiningSequence, MachiningSequenceError> {
        self.sequences
            .iter_mut()
            .find(|sequence| sequence.id() == id)
            .ok_or(MachiningSequenceError::UnknownSequence(id))
    }
}

#[cfg(test)]
mod tests;
