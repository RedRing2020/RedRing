//! 空間モデルの文書（開いたモデルファイル・製品形状・加工手順・保存していない変更）
//!
//! 1 つのウィンドウで扱う空間モデルの状態をまとめて持つ（設計: SPACE_MODEL_FILE_DESIGN.md 6 章。タブで複数の空間
//! モデルを扱うときは、この文書を複数持つ）。加工手順は 1 つ、加工工程は 1 つを編集の対象とする。

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use application::cam_simulation_workflow::CamSimulationWorkflowResult;
pub use application::imported_toolpath::ImportedToolpathState;
use application::machining_sequence::{
    scanline_finish_operation_content, MachiningOperationContent, MachiningOperationId,
    MachiningSequenceId, MachiningSequenceStore, MachiningStock, ScanlineCutDirection,
    ScanlineParams, ScanlineToolRange, StockMargins,
};
use application::product_shape::{EntityDisplayProperties, ProductShapeError, ProductShapeStore};
use application::space_model_file::{
    load_space_model, save_space_model, SpaceModel, SpaceModelFileError, SpaceModelSaveInfo,
};

use crate::cutting_simulation_session::CuttingToolKind;
use crate::product_shape_converter::{
    face_set_entity_to_mesh, ProductShapeDisplaySettings, ProductShapeMesh,
};

/// 加工手順・加工工程の名前（名前を編集できるようになるまでの固定の名前）
const SEQUENCE_NAME: &str = "加工手順";
const OPERATION_NAME: &str = "スキャン加工";

/// 新しく作る加工工程の中身（直径 10 mm のボールエンドミル、ピックフィード 2 mm）
fn initial_operation_content() -> MachiningOperationContent {
    scanline_finish_operation_content(
        OPERATION_NAME.to_string(),
        CuttingToolKind::BallEndMill.tool(10.0, None),
        ScanlineParams {
            stepover: 2.0,
            feed_rate: 1200.0,
            tool_range: ScanlineToolRange::default(),
            cut_direction: ScanlineCutDirection::default(),
            angle_degrees: 0.0,
        },
    )
}

/// 新しく作る加工手順の素材（製品形状を囲む箱、精度 0.05 mm）
fn initial_stock() -> MachiningStock {
    MachiningStock {
        margins: StockMargins::default(),
        precision: 0.05,
    }
}

/// 空間モデルの文書
#[derive(Debug)]
pub struct SpaceModelDocument {
    model: SpaceModel,
    /// 編集の対象の加工手順と加工工程
    sequence: MachiningSequenceId,
    operation: MachiningOperationId,
    /// 開いた・保存したモデルファイル（まだなければ `None`）
    path: Option<PathBuf>,
    /// 保存していない変更があるか
    modified: bool,
    display: ProductShapeDisplaySettings,
}

impl Default for SpaceModelDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceModelDocument {
    /// 空の空間モデル（製品形状なし。加工工程を 1 つ持つ加工手順）の文書を作る
    pub fn new() -> Self {
        Self::from_model(
            SpaceModel::new(ProductShapeStore::new(), MachiningSequenceStore::new()),
            None,
        )
    }

    /// モデルファイル `path` を開く
    ///
    /// 加工手順がなければ、加工工程を 1 つ持つ加工手順を作る。加工工程は、最初の加工手順の最初の加工工程を編集の
    /// 対象にする（なければ作る）。
    ///
    /// # Errors
    ///
    /// モデルファイルを読めない場合。
    pub fn open(path: &Path) -> Result<Self, SpaceModelFileError> {
        let (model, _) = load_space_model(path)?;
        Ok(Self::from_model(model, Some(path.to_path_buf())))
    }

    fn from_model(mut model: SpaceModel, path: Option<PathBuf>) -> Self {
        let sequence = match model.sequences.sequence_ids().first() {
            Some(&sequence) => sequence,
            None => model.sequences.create_sequence(
                SEQUENCE_NAME.to_string(),
                Vec::new(),
                initial_stock(),
            ),
        };
        let operation = match model
            .sequences
            .operation_ids(sequence)
            .expect("the sequence is held")
            .first()
        {
            Some(&operation) => operation,
            None => model
                .sequences
                .add_operation(sequence, initial_operation_content())
                .expect("the sequence is held"),
        };
        let mut document = Self {
            model,
            sequence,
            operation,
            path,
            modified: false,
            display: ProductShapeDisplaySettings::default(),
        };
        // 加工の対象は表示中の面にするため、開いたときに合わせる（作った加工手順・前に合わせていない加工の対象。
        // 利用者が変えたものではないため、保存していない変更にしない）
        document.target_visible_faces();
        document.modified = false;
        document
    }

    /// 開いた・保存したモデルファイル
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// 保存していない変更があるか
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// モデルファイル `path` に保存し（プログラムの版 `program_version` を記録する）、以後はそのファイルを開いた
    /// ファイルとする
    ///
    /// # Errors
    ///
    /// 書き込みの失敗。
    pub fn save_as(
        &mut self,
        path: &Path,
        program_version: &str,
    ) -> Result<(), SpaceModelFileError> {
        let saved_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis() as u64);
        let info = SpaceModelSaveInfo {
            program_version: program_version.to_string(),
            saved_at_ms,
        };
        save_space_model(path, &self.model, &info)?;
        self.path = Some(path.to_path_buf());
        self.modified = false;
        Ok(())
    }

    /// 製品形状
    pub fn shapes(&self) -> &ProductShapeStore {
        &self.model.shapes
    }

    /// STL ファイルを製品形状として取り込み、今の空間モデルに加える
    ///
    /// # Errors
    ///
    /// STL の読み込みの失敗。
    pub fn import_stl(&mut self, path: &Path) -> Result<(), ProductShapeError> {
        self.model.shapes.import_stl(path)?;
        self.modified = true;
        Ok(())
    }

    /// 表示中のエンティティの描画用のメッシュ（表示用の三角形を作れないエンティティは除き、警告を出す）
    pub fn visible_meshes(&self) -> Vec<ProductShapeMesh> {
        self.model
            .shapes
            .entities()
            .iter()
            .filter(|entity| entity.entity_visible())
            .filter_map(|entity| {
                face_set_entity_to_mesh(entity, self.display.chord_tolerance)
                    .inspect_err(|error| {
                        tracing::warn!(entity_id = %entity.id(), "product shape is not displayed: {error}");
                    })
                    .ok()
            })
            .collect()
    }

    /// 編集の対象の加工工程の中身（工具・走査線の条件）
    pub fn operation_content(&self) -> &MachiningOperationContent {
        self.model
            .sequences
            .operation_content(self.sequence, self.operation)
            .expect("the document holds its machining operation")
    }

    /// 編集の対象の加工工程の中身を `content` にする（今と違えば、工程の改訂の番号が増え、保存していない変更になる）
    pub fn set_operation_content(&mut self, content: MachiningOperationContent) {
        self.edit_sequences(|sequences, sequence, operation| {
            sequences
                .set_operation_content(sequence, operation, content)
                .expect("the document holds its machining operation");
        });
    }

    /// 編集の対象の加工手順の素材（素材の余裕・精度）
    pub fn stock(&self) -> MachiningStock {
        *self
            .model
            .sequences
            .stock(self.sequence)
            .expect("the document holds its machining sequence")
    }

    /// 編集の対象の加工手順の素材を `stock` にする（今と違えば、工程の改訂の番号が増え、保存していない変更になる）
    pub fn set_stock(&mut self, stock: MachiningStock) {
        self.edit_sequences(|sequences, sequence, _| {
            sequences
                .set_stock(sequence, stock)
                .expect("the document holds its machining sequence");
        });
    }

    /// 編集の対象の加工工程のツールパス生成のトレランス（mm）
    pub fn toolpath_tolerance(&self) -> f64 {
        self.model
            .sequences
            .operation_tolerance(self.sequence, self.operation)
            .expect("the document holds its machining operation")
    }

    /// 加工の対象を、表示中の製品形状のすべての面にする（面の集合が変わった場合は、工程の改訂の番号が増え、保存して
    /// いない変更になる）
    pub(crate) fn target_visible_faces(&mut self) {
        let faces = self.model.shapes.visible_face_ids();
        self.edit_sequences(|sequences, sequence, _| {
            sequences
                .set_target_faces(sequence, faces)
                .expect("the document holds its machining sequence");
        });
    }

    /// 編集の対象の加工工程に取り込んだ CL が、今の条件の結果か
    pub fn imported_toolpath_state(&self) -> ImportedToolpathState {
        self.model
            .toolpaths
            .state(&self.model.sequences, self.sequence, self.operation)
            .expect("the document holds its machining operation")
    }

    /// 編集の対象の加工工程の今の改訂で計算した結果 `result` の CL を取り込む（前の CL は上書きする。変われば、保存して
    /// いない変更になる）
    pub(crate) fn import_toolpath(&mut self, result: &CamSimulationWorkflowResult) {
        let changed = self
            .model
            .toolpaths
            .import(&self.model.sequences, self.sequence, self.operation, result)
            .expect("the document holds its machining operation");
        if changed {
            self.modified = true;
        }
    }

    /// 製品形状・加工手順と、編集の対象の加工手順・加工工程
    pub(crate) fn machining(
        &self,
    ) -> (
        &ProductShapeStore,
        &MachiningSequenceStore,
        MachiningSequenceId,
        MachiningOperationId,
    ) {
        (
            &self.model.shapes,
            &self.model.sequences,
            self.sequence,
            self.operation,
        )
    }

    /// 加工手順を `edit` で編集し、編集の対象の加工工程の改訂が変われば、保存していない変更にする
    fn edit_sequences(
        &mut self,
        edit: impl FnOnce(&mut MachiningSequenceStore, MachiningSequenceId, MachiningOperationId),
    ) {
        let before = self.revision_key();
        edit(&mut self.model.sequences, self.sequence, self.operation);
        if self.revision_key() != before {
            self.modified = true;
        }
    }

    fn revision_key(&self) -> application::machining_sequence::MachiningRevisionKey {
        self.model
            .sequences
            .operation_revision_key(self.sequence, self.operation)
            .expect("the document holds its machining operation")
    }
}

#[cfg(test)]
pub(crate) mod tests;
