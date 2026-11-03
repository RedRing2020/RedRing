//! 加工手順・加工工程のエンティティの保管と、加工の対象の参照の確認（設計: CAM_PROCESS_ENTITY_DESIGN.md）

use std::collections::HashSet;
use std::fmt;

use cam_entity::{MachiningSequence, MachiningSequenceId};
use geo_entity::FaceId;

use crate::product_shape::ProductShapeStore;

/// 加工手順の参照・確認の失敗
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachiningSequenceError {
    /// 保管していない加工手順
    UnknownSequence(MachiningSequenceId),
    /// 製品形状の保管先にない、加工の対象の面（面を削除した等で、参照が切れた）
    MissingTargetFaces(Vec<FaceId>),
}

impl fmt::Display for MachiningSequenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSequence(id) => write!(f, "unknown machining sequence: {id}"),
            Self::MissingTargetFaces(ids) => {
                write!(f, "target faces are missing from the product shape:")?;
                for id in ids {
                    write!(f, " {id}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for MachiningSequenceError {}

/// 加工手順の保管先（加えた順に保持する。加工工程は、属する加工手順が持つ）
#[derive(Debug, Default)]
pub struct MachiningSequenceStore {
    sequences: Vec<MachiningSequence>,
}

impl MachiningSequenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 加工手順を加え、その ID を返す
    pub fn add(&mut self, sequence: MachiningSequence) -> MachiningSequenceId {
        let id = sequence.id();
        self.sequences.push(sequence);
        id
    }

    pub fn sequence(&self, id: MachiningSequenceId) -> Option<&MachiningSequence> {
        self.sequences.iter().find(|sequence| sequence.id() == id)
    }

    pub fn sequence_mut(&mut self, id: MachiningSequenceId) -> Option<&mut MachiningSequence> {
        self.sequences
            .iter_mut()
            .find(|sequence| sequence.id() == id)
    }

    /// 加工手順 `id` の加工の対象の面が、すべて製品形状の保管先 `shapes` にあることを確かめる
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または `shapes` にない面がある場合（ない面をすべて返す）。
    pub fn check_target_faces(
        &self,
        id: MachiningSequenceId,
        shapes: &ProductShapeStore,
    ) -> Result<(), MachiningSequenceError> {
        let sequence = self
            .sequence(id)
            .ok_or(MachiningSequenceError::UnknownSequence(id))?;
        let held: HashSet<FaceId> = shapes
            .entities()
            .iter()
            .map(|entity| entity.face().id())
            .collect();
        let missing: Vec<FaceId> = sequence
            .target_faces()
            .iter()
            .copied()
            .filter(|face_id| !held.contains(face_id))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(MachiningSequenceError::MissingTargetFaces(missing))
        }
    }
}

#[cfg(test)]
mod tests;
