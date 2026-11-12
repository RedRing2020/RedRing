//! 空間モデルの文書（開いたモデルファイル・製品形状・加工手順・保存していない変更）
//!
//! 1 つのウィンドウで扱う空間モデルの状態をまとめて持つ（設計: SPACE_MODEL_FILE_DESIGN.md 6 章。タブで複数の空間
//! モデルを扱うときは、この文書を複数持つ）。加工手順は 1 つを、加工工程はそのうち選んだ 1 つを編集の対象とする。
//! 加工工程は、加えて・複写して・消して・並べ替えられる（手順には、いつも 1 つ以上の工程がある）。

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use application::cam_simulation_workflow::{CamSimulationWorkflowResult, CamToolpathResult};
pub use application::imported_toolpath::ImportedToolpathState;
pub use application::machining_sequence::MachiningOperationId;
use application::machining_sequence::{
    scanline_finish_operation_content, MachiningOperationContent, MachiningSequenceId,
    MachiningSequenceStore, MachiningStage, MachiningStock, OperationSpec, ScanlineCutDirection,
    ScanlineParams, ScanlineToolRange, StockMargins, Tool,
};
use application::product_shape::{EntityDisplayProperties, ProductShapeError, ProductShapeStore};
use application::space_model_file::{
    load_space_model, save_space_model, SpaceModel, SpaceModelFileError, SpaceModelSaveInfo,
};

use crate::cam_sim_visualization_converter::create_toolpath_wireframe_vertices;
use crate::cutting_simulation_session::CuttingToolKind;
use crate::mesh_converter::WireframeVertex;
use crate::product_shape_converter::{
    face_set_entity_to_mesh, ProductShapeDisplaySettings, ProductShapeMesh,
};
use crate::toolpath_converter::ToolPathVisualizationSettings;

/// 空間モデルに加工手順・加工工程がないときに作る、加工手順・加工工程の名前（画面の言語の名前を渡す）
#[derive(Debug, Clone, Copy)]
pub struct DefaultNames<'a> {
    pub sequence: &'a str,
    pub operation: &'a str,
}

/// 新しく作る、名前が `name` の加工工程の中身（直径 10 mm のボールエンドミル、ピックフィード 2 mm）
fn initial_operation_content(name: String) -> MachiningOperationContent {
    scanline_finish_operation_content(
        name,
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

/// 加工工程の一覧の 1 行
#[derive(Debug, Clone, PartialEq)]
pub struct OperationListItem {
    pub id: MachiningOperationId,
    pub name: String,
    pub stage: MachiningStage,
    pub spec: OperationSpec,
    pub tool: Tool<f64>,
    /// 計算の条件を設定したか（設定していない工程は計算しない）
    pub conditions_set: bool,
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

impl SpaceModelDocument {
    /// 空の空間モデル（製品形状なし。加工工程を 1 つ持つ加工手順。名前は `names`）の文書を作る
    pub fn new(names: DefaultNames<'_>) -> Self {
        Self::from_model(
            SpaceModel::new(ProductShapeStore::new(), MachiningSequenceStore::new()),
            None,
            names,
        )
    }

    /// モデルファイル `path` を開く
    ///
    /// 加工手順がなければ、加工工程を 1 つ持つ加工手順を作る。加工工程は、最初の加工手順の最初の加工工程を編集の
    /// 対象にする（なければ作る）。作る加工手順・加工工程の名前は `names`。
    ///
    /// # Errors
    ///
    /// モデルファイルを読めない場合。
    pub fn open(path: &Path, names: DefaultNames<'_>) -> Result<Self, SpaceModelFileError> {
        let (model, _) = load_space_model(path)?;
        Ok(Self::from_model(model, Some(path.to_path_buf()), names))
    }

    fn from_model(mut model: SpaceModel, path: Option<PathBuf>, names: DefaultNames<'_>) -> Self {
        let sequence = match model.sequences.sequence_ids().first() {
            Some(&sequence) => sequence,
            None => model.sequences.create_sequence(
                names.sequence.to_string(),
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
                .add_operation(
                    sequence,
                    initial_operation_content(names.operation.to_string()),
                )
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
        // 前の版で残した、前の条件の CL を除く（すべての加工手順。利用者が変えたものではないため、保存していない
        // 変更にしない）
        for sequence in document.model.sequences.sequence_ids() {
            document
                .model
                .toolpaths
                .remove_outdated(&document.model.sequences, sequence)
                .expect("the sequence is held");
        }
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

    /// 編集の対象の加工手順の加工工程（加工の順）
    pub fn operations(&self) -> Vec<OperationListItem> {
        self.model
            .sequences
            .operation_ids(self.sequence)
            .expect("the document holds its machining sequence")
            .into_iter()
            .map(|id| {
                let content = self
                    .model
                    .sequences
                    .operation_content(self.sequence, id)
                    .expect("the listed operation is in the sequence");
                OperationListItem {
                    id,
                    name: content.attributes.name.clone(),
                    stage: content.attributes.stage,
                    spec: content.conditions.spec,
                    tool: content.conditions.tool.clone(),
                    conditions_set: self.conditions_set_of(id),
                }
            })
            .collect()
    }

    /// 編集の対象の加工手順の名前
    pub fn sequence_name(&self) -> &str {
        self.model
            .sequences
            .sequence_name(self.sequence)
            .expect("the document holds its machining sequence")
    }

    /// 編集の対象の加工工程（選んだ工程）
    pub fn selected_operation(&self) -> MachiningOperationId {
        self.operation
    }

    /// 加工工程 `id` を編集の対象にする（手順にない工程は選ばずに `false`。選んでも、保存していない変更にしない）
    pub fn select_operation(&mut self, id: MachiningOperationId) -> bool {
        let held = self
            .model
            .sequences
            .operation_position(self.sequence, id)
            .is_ok();
        if held {
            self.operation = id;
        }
        held
    }

    /// 編集の対象の加工工程のすぐ後に、名前が `name` の新しい加工工程（既定の工具・走査線の条件）を加え、編集の対象に
    /// する
    pub fn add_operation(&mut self, name: String) {
        let content = initial_operation_content(name);
        self.insert_after_selected(content);
    }

    /// 編集の対象の加工工程と同じ中身で、名前が `name` の新しい加工工程をすぐ後に加え、編集の対象にする（新しい工程の
    /// 改訂の番号は 1 から。計算の条件を設定したかは受け継ぎ、計算の結果・取り込んだ CL は受け継がない）
    pub fn duplicate_operation(&mut self, name: String) {
        let mut content = self.operation_content().clone();
        content.attributes.name = name;
        let conditions_set = self.conditions_set_of(self.operation);
        self.insert_after_selected(content);
        if conditions_set {
            self.mark_conditions_set();
        }
    }

    /// 加工工程 `operation` の計算の条件を設定したか（手順にない工程は、設定していないとする）
    pub fn conditions_set_of(&self, operation: MachiningOperationId) -> bool {
        self.model
            .sequences
            .operation_conditions_set(self.sequence, operation)
            .unwrap_or(false)
    }

    /// 編集の対象の加工工程の計算の条件を、設定したことにする（設定していなかったら、保存していない変更にして `true`
    /// を返す。改訂は進めない）
    pub fn mark_conditions_set(&mut self) -> bool {
        let changed = self
            .model
            .sequences
            .mark_operation_conditions_set(self.sequence, self.operation)
            .expect("the document holds its machining operation");
        if changed {
            self.modified = true;
        }
        changed
    }

    /// 加工工程 `operation` の今の改訂の鍵（手順にない工程は `None`）
    pub(crate) fn revision_key_of(
        &self,
        operation: MachiningOperationId,
    ) -> Option<application::machining_sequence::MachiningRevisionKey> {
        self.model
            .sequences
            .operation_revision_key(self.sequence, operation)
            .ok()
    }

    fn insert_after_selected(&mut self, content: MachiningOperationContent) {
        let index = self.selected_position() + 1;
        self.operation = self
            .model
            .sequences
            .insert_operation(self.sequence, index, content)
            .expect("the document holds its machining sequence");
        self.modified = true;
    }

    /// 編集の対象の加工工程の名前を `name` にする（前後の空白を除く。空なら替えない）。替えたら保存していない変更にする
    /// （名前は計算に使わないため、工程の改訂は進めない）
    pub fn rename_selected_operation(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() || name == self.operation_content().attributes.name {
            return;
        }
        let mut content = self.operation_content().clone();
        content.attributes.name = name.to_string();
        self.model
            .sequences
            .set_operation_content(self.sequence, self.operation, content)
            .expect("the document holds its machining operation");
        self.modified = true;
    }

    /// 編集の対象の加工工程を消し、その工程の取り込んだ CL も消す。次の工程（最後の工程を消したときは前の工程）を
    /// 編集の対象にする。工程が 1 つだけなら消さずに `false`
    pub fn remove_selected_operation(&mut self) -> bool {
        let ids = self
            .model
            .sequences
            .operation_ids(self.sequence)
            .expect("the document holds its machining sequence");
        if ids.len() <= 1 {
            return false;
        }
        let index = self.selected_position();
        let next = ids
            .get(index + 1)
            .or_else(|| index.checked_sub(1).and_then(|before| ids.get(before)))
            .copied()
            .expect("another operation is held");
        self.model
            .remove_operation(self.sequence, self.operation)
            .expect("the document holds its machining operation");
        self.operation = next;
        self.modified = true;
        true
    }

    /// 編集の対象の加工工程を、加工の順で `offset` だけ動かす（負は前へ。端を越える分は端で止める）。動いたら `true`
    pub fn move_selected_operation(&mut self, offset: isize) -> bool {
        let count = self
            .model
            .sequences
            .operation_ids(self.sequence)
            .expect("the document holds its machining sequence")
            .len();
        let from = self.selected_position();
        let to = from.saturating_add_signed(offset).min(count - 1);
        if to == from {
            return false;
        }
        self.model
            .sequences
            .move_operation(self.sequence, self.operation, to)
            .expect("the document holds its machining operation");
        self.modified = true;
        true
    }

    /// 編集の対象の加工工程の、加工の順の位置
    fn selected_position(&self) -> usize {
        self.model
            .sequences
            .operation_position(self.sequence, self.operation)
            .expect("the document holds its machining operation")
    }

    /// 編集の対象の加工工程の中身（工具・走査線の条件）
    pub fn operation_content(&self) -> &MachiningOperationContent {
        self.model
            .sequences
            .operation_content(self.sequence, self.operation)
            .expect("the document holds its machining operation")
    }

    /// 編集の対象の加工工程の中身を `content` にする（今と違えば保存していない変更になり、計算の条件が違えば工程の
    /// 改訂の番号も増える）
    pub fn set_operation_content(&mut self, content: MachiningOperationContent) {
        // 属性（名前・加工の段階）だけを替えたときは改訂しないため、中身を比べて保存していない変更にする
        if *self.operation_content() != content {
            self.modified = true;
        }
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

    /// 加工工程 `operation` に取り込んだ CL が、今の条件の結果か（手順にない工程は、取り込んでいないとする）
    pub fn imported_toolpath_state_of(
        &self,
        operation: MachiningOperationId,
    ) -> ImportedToolpathState {
        self.model
            .toolpaths
            .state(&self.model.sequences, self.sequence, operation)
            .unwrap_or(ImportedToolpathState::None)
    }

    /// 編集の対象の加工工程に取り込んだ、今の条件の CL の線（今の表示の設定 `settings` で作る。取り込んでいなければ
    /// `None`）
    pub fn selected_toolpath_wireframe(
        &self,
        settings: &ToolPathVisualizationSettings,
    ) -> Option<Vec<WireframeVertex>> {
        if self.imported_toolpath_state() != ImportedToolpathState::Current {
            return None;
        }
        let toolpath = self.model.toolpaths.toolpath(self.operation)?;
        Some(create_toolpath_wireframe_vertices(&toolpath, settings))
    }

    /// 編集の対象の加工工程に取り込んだ CL が、今の条件の結果か
    pub fn imported_toolpath_state(&self) -> ImportedToolpathState {
        self.imported_toolpath_state_of(self.operation)
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

    /// 編集の対象の加工工程の今の改訂で計算したツールパス生成の結果 `result` の CL を取り込む（前の CL は上書きする。
    /// 変われば、保存していない変更になる）
    pub(crate) fn import_toolpath_result(&mut self, result: &CamToolpathResult) {
        let changed = self
            .model
            .toolpaths
            .import_toolpath(&self.model.sequences, self.sequence, self.operation, result)
            .expect("the document holds its machining operation");
        if changed {
            self.modified = true;
        }
    }

    /// 編集の対象の加工工程に取り込んだ CL を除く（除いたら、保存していない変更になる）
    pub(crate) fn remove_toolpath(&mut self) {
        if self.model.toolpaths.remove(self.operation) {
            self.modified = true;
        }
    }

    /// 取り込んだ CL
    pub(crate) fn toolpaths(&self) -> &application::imported_toolpath::ImportedToolpathStore {
        &self.model.toolpaths
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

    /// 加工手順を `edit` で編集し、編集の対象の加工工程の改訂が変われば、保存していない変更にして、改訂が進んだ工程の
    /// 前の条件の CL を除く（前の条件の CL は使わない。計算の条件・素材・加工の対象のどれを変えても、編集の対象の
    /// 工程の改訂は進む）
    fn edit_sequences(
        &mut self,
        edit: impl FnOnce(&mut MachiningSequenceStore, MachiningSequenceId, MachiningOperationId),
    ) {
        let before = self.revision_key();
        edit(&mut self.model.sequences, self.sequence, self.operation);
        if self.revision_key() != before {
            self.modified = true;
            self.model
                .toolpaths
                .remove_outdated(&self.model.sequences, self.sequence)
                .expect("the document holds its machining sequence");
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
