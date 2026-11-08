//! AppState の製品形状の取り込みと表示を扱うモジュール。

use super::cutting_simulation::line_vertices;
use super::AppState;
use render::mesh_list::MeshListItem;
use render::vertex_3d::convert_vertex_data_to_mesh_vertices;
use stage::MeshStage;
use std::path::Path;
use viewmodel::space_model_document::ImportedToolpathState;

/// 今の条件の結果でない CL の線の色
const OUTDATED_TOOLPATH_COLOR: [f32; 3] = [0.55, 0.55, 0.55];

impl AppState {
    /// STL ファイルを製品形状として取り込み、表示中の製品形状を表示する
    pub fn import_product_shape(&mut self, path: &Path) {
        let file = super::model_file::file_name(path);
        if let Err(error) = self.document.import_stl(path) {
            tracing::warn!(path = %path.display(), "product shape import failed: {error}");
            self.set_notice(crate::help_ui::Notice::ShapeImportFailed {
                file,
                reason: viewmodel::failure_message_mapper::product_shape_failure_message(&error),
            });
            return;
        }
        tracing::info!(path = %path.display(), "product shape imported");
        self.set_notice(crate::help_ui::Notice::ShapeImported { file });
        self.invalidate_conditions();
        self.show_product_shapes();
    }

    /// 表示中の製品形状をメッシュステージに描き、全体が見えるようにカメラを合わせる
    pub(super) fn show_product_shapes(&mut self) {
        self.build_product_shape_stage(true);
    }

    /// 選んだ加工工程に合わせて表示する。取り込んだ切削アニメーションの工程を選んでいればそのまま出し、ほかの工程を
    /// 選んだら、切削アニメーションを閉じて（結果は大きいため、持ち続けない）、製品形状とその工程の CL の線を出す
    /// （カメラは動かさない）
    pub(super) fn show_selected_operation(&mut self) {
        let animation_of_selected = self.cutting_animation.is_some()
            && self.imported_operation == Some(self.document.selected_operation());
        if animation_of_selected {
            return;
        }
        if self.cutting_animation.is_some() {
            // デバッグ表示は取り込んだ切削アニメーションのものなので、通常の表示に戻してから閉じる
            self.restore_main_view();
            self.build_product_shape_stage(false);
        } else {
            self.refresh_operation_toolpath_lines();
        }
    }

    /// 切削アニメーションを出していないときに、選んだ加工工程の空間モデルの CL の線を描き直す（今の条件の結果でない
    /// CL は灰色にする。CL を取り込んでいなければ、線を消す）
    pub(super) fn refresh_operation_toolpath_lines(&mut self) {
        if self.cutting_animation.is_some() {
            return;
        }
        self.drawn_toolpath_state = Some((
            self.document.selected_operation(),
            self.document.imported_toolpath_state(),
        ));
        let lines = match self
            .document
            .selected_toolpath_wireframe(&self.toolpath_visualization_settings)
        {
            Some((vertices, ImportedToolpathState::Current)) => line_vertices(&vertices, None),
            Some((vertices, _)) => line_vertices(&vertices, Some(OUTDATED_TOOLPATH_COLOR)),
            None => Vec::new(),
        };
        let visible = self.toolpath_lines_visible;
        let device = self.graphic.device.clone();
        if let Some(mesh_stage) = self.main_mesh_stage() {
            mesh_stage.set_overlay_toolpath_visible(visible);
            mesh_stage.set_overlay_toolpath_line_data(&device, lines);
        }
    }

    /// 表示中の製品形状をメッシュステージに描く（`fit_camera` のときは、全体が見えるようにカメラを合わせる）。選んだ
    /// 加工工程の CL の線も描く
    fn build_product_shape_stage(&mut self, fit_camera: bool) {
        self.discard_main_view();
        self.cutting_animation = None;
        self.cutting_animation_job_id = None;
        self.imported_conditions = None;
        self.imported_operation = None;
        self.debug_snapshot.clear();
        let meshes = self.document.visible_meshes();
        let positions: Vec<[f32; 3]> = meshes
            .iter()
            .flat_map(|mesh| mesh.vertices.iter().map(|vertex| vertex.position))
            .collect();
        let items = meshes
            .into_iter()
            .enumerate()
            .map(|(index, mesh)| {
                let item = MeshListItem {
                    vertices: convert_vertex_data_to_mesh_vertices(&mesh.vertices),
                    indices: mesh.indices,
                    color: mesh.color,
                };
                (index as u64, item)
            })
            .collect();

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_list(&self.graphic.device, items);
        if fit_camera {
            if let Some(fit) = Self::build_camera_fit(&positions) {
                self.apply_camera_fit(fit);
            }
        }
        self.renderer.set_stage(mesh_stage);
        self.refresh_operation_toolpath_lines();
        self.update_camera_uniforms();
    }
}
