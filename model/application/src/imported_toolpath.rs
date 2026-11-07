//! 空間モデルに取り込んだ CL（ツールパスの成果物）の保管（設計: JOB_MANAGER_STORAGE_DESIGN.md 6 章、
//! SPACE_MODEL_FILE_DESIGN.md 3 章）
//!
//! 加工工程は CL を 1 つだけ持ち、取り込むたびに上書きする。CL は、取り込んだときの工程の改訂の番号と改訂の鍵を
//! 持ち、工程の改訂の鍵と違えば、今の条件の結果ではない。取り込んだ後は、ジョブマネージャーの成果物に頼らない。

use std::fmt;
use std::num::NonZeroU32;

use cam_core::read_toolpath_artifact_v1;

use crate::cam_simulation_workflow::CamSimulationWorkflowResult;
use crate::machining_sequence::{
    MachiningOperationId, MachiningRevisionKey, MachiningSequenceError, MachiningSequenceId,
    MachiningSequenceStore,
};

/// 取り込んだ CL の記録（保管先から取り出し、保管先に戻す中身）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedToolpathRecord {
    /// もとにした加工工程
    pub operation: MachiningOperationId,
    /// 取り込んだときの工程の改訂の番号
    pub revision: NonZeroU32,
    /// 取り込んだときの工程の改訂の鍵
    pub revision_key: MachiningRevisionKey,
    /// CL の編集の番号（0 は、計算の結果から編集していない）
    pub edit_number: u32,
    /// CL（ツールパスの成果物のバイト列）
    pub artifact: Vec<u8>,
}

/// 取り込んだ CL が、加工工程の今の条件の結果か
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedToolpathState {
    /// 取り込んでいない
    None,
    /// 今の条件（工程の今の改訂）の結果
    Current,
    /// 前の条件の結果（条件を変えた後で、計算し直していない）
    Outdated,
}

/// 記録から保管先に戻すときの誤り
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedToolpathRestoreError {
    /// 同じ加工工程の CL が 2 つある
    DuplicateOperation(MachiningOperationId),
    /// CL のバイト列がツールパスの成果物として読めない
    InvalidArtifact(MachiningOperationId),
}

impl fmt::Display for ImportedToolpathRestoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateOperation(id) => {
                write!(f, "duplicate imported toolpath of machining operation {id}")
            }
            Self::InvalidArtifact(id) => {
                write!(f, "invalid imported toolpath of machining operation {id}")
            }
        }
    }
}

impl std::error::Error for ImportedToolpathRestoreError {}

/// 取り込んだ CL の保管先（取り込んだ順に保持する）
#[derive(Debug, Default)]
pub struct ImportedToolpathStore {
    toolpaths: Vec<ImportedToolpathRecord>,
}

impl ImportedToolpathStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 記録 `records` を、その順に持つ保管先を作る
    ///
    /// もとにした加工工程がない CL も、そのまま戻す（加工工程の参照が切れたときと同じく、勝手に消さない）。
    ///
    /// # Errors
    ///
    /// 同じ加工工程の CL が 2 つある場合、CL がツールパスの成果物として読めない場合。
    pub fn from_records(
        records: Vec<ImportedToolpathRecord>,
    ) -> Result<Self, ImportedToolpathRestoreError> {
        let mut store = Self::new();
        for record in records {
            if store.get(record.operation).is_some() {
                return Err(ImportedToolpathRestoreError::DuplicateOperation(
                    record.operation,
                ));
            }
            if read_toolpath_artifact_v1(&mut record.artifact.as_slice()).is_err() {
                return Err(ImportedToolpathRestoreError::InvalidArtifact(
                    record.operation,
                ));
            }
            store.toolpaths.push(record);
        }
        Ok(store)
    }

    /// 保持している CL の記録（取り込んだ順）
    pub fn records(&self) -> &[ImportedToolpathRecord] {
        &self.toolpaths
    }

    /// 加工工程 `operation` の CL
    fn get(&self, operation: MachiningOperationId) -> Option<&ImportedToolpathRecord> {
        self.toolpaths
            .iter()
            .find(|toolpath| toolpath.operation == operation)
    }

    /// 加工手順 `sequence` の加工工程 `operation` の今の改訂で計算した結果 `result` の CL を取り込む（前の CL は
    /// 上書きする）。取り込んだ CL が前と変われば `true`
    ///
    /// `result` のジョブの投入元が、その工程の今の改訂（工程の ID と今の改訂の鍵）でなければ、取り込まずに `false` を
    /// 返す（前の条件の結果を、今の条件の結果として持たない）。
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn import(
        &mut self,
        sequences: &MachiningSequenceStore,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
        result: &CamSimulationWorkflowResult,
    ) -> Result<bool, MachiningSequenceError> {
        let current = sequences.operation(sequence, operation)?;
        let computed_for_current = result.source.as_ref().is_some_and(|source| {
            source.source_id == operation.to_string()
                && source.revision_key.as_deref()
                    == Some(current.revision_key().to_string().as_str())
        });
        if !computed_for_current {
            return Ok(false);
        }
        let record = ImportedToolpathRecord {
            operation,
            revision: NonZeroU32::new(current.revision())
                .expect("the revision of an operation starts at 1 and only increases"),
            revision_key: current.revision_key(),
            edit_number: 0,
            artifact: result.toolpath_artifact.clone(),
        };
        match self
            .toolpaths
            .iter_mut()
            .find(|toolpath| toolpath.operation == operation)
        {
            Some(existing) if *existing == record => Ok(false),
            Some(existing) => {
                *existing = record;
                Ok(true)
            }
            None => {
                self.toolpaths.push(record);
                Ok(true)
            }
        }
    }

    /// 加工手順 `sequence` の加工工程 `operation` の CL が、今の条件の結果か
    ///
    /// # Errors
    ///
    /// 保管していない加工手順、または加工手順にない加工工程の場合。
    pub fn state(
        &self,
        sequences: &MachiningSequenceStore,
        sequence: MachiningSequenceId,
        operation: MachiningOperationId,
    ) -> Result<ImportedToolpathState, MachiningSequenceError> {
        let key = sequences.operation_revision_key(sequence, operation)?;
        Ok(match self.get(operation) {
            None => ImportedToolpathState::None,
            Some(toolpath) if toolpath.revision_key == key => ImportedToolpathState::Current,
            Some(_) => ImportedToolpathState::Outdated,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests;
