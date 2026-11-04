//! 加工手順・加工工程のエンティティの保管と編集（設計: CAM_PROCESS_ENTITY_DESIGN.md）
//!
//! エンティティは保管先の中だけで持ち、作成と編集は保管先の関数で行う。関数に出てくる値の型（工程の中身・素材・ID）は
//! ここから再エクスポートする。

use std::fmt;

use cam_core::{MachiningStage, OperationSpec, ScanlineParams, ToleranceProfile, Tool};
use cam_entity::{MachiningOperation, MachiningSequence};
pub use cam_entity::{
    MachiningOperationContent, MachiningOperationId, MachiningSequenceId, MachiningStock,
};
use geo_entity::FaceId;

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

/// 走査線の加工工程を作るときの既定の精度プロファイル（工程テンプレートを登録できるようになるまでの既定）
const DEFAULT_TOLERANCE_PROFILE: ToleranceProfile = ToleranceProfile::PressRough;

/// 工具 `tool` で走査線の条件 `scanline` の仕上げを行う加工工程の中身（名前は `name`、加工領域は加工の対象の
/// XY の範囲、精度プロファイルは既定）
pub fn scanline_operation_content(
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
