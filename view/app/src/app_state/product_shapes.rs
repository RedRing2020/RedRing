//! AppState の製品形状の取り込みと表示を扱うモジュール。

use super::AppState;
use super::cutting_simulation::line_vertices;
use render::mesh_list::MeshListItem;
use render::vertex_3d::convert_vertex_data_to_mesh_vertices;
use stage::MeshStage;
use std::path::Path;

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
            self.close_cutting_animation();
        } else {
            self.refresh_operation_toolpath_lines();
        }
    }

    /// 切削アニメーションを閉じて、製品形状と選んだ加工工程の CL の線を出す（カメラは動かさない）
    pub(super) fn close_cutting_animation(&mut self) {
        // デバッグ表示は取り込んだ切削アニメーションのものなので、通常の表示に戻してから閉じる
        self.restore_main_view();
        self.build_product_shape_stage(false);
    }

    /// 切削アニメーションを出していないときに、選んだ加工工程の空間モデルの CL の線を描き直す。CL の線は、加工手順を
    /// 開いている間だけ出す（閉じている・CL を取り込んでいなければ、線を消して放す）
    pub(super) fn refresh_operation_toolpath_lines(&mut self) {
        if self.cutting_animation.is_some() {
            return;
        }
        let shown = self.machining_sequence_open;
        self.drawn_toolpath_state = Some((
            self.document.selected_operation(),
            self.document.imported_toolpath_state(),
            shown,
        ));
        let wireframe = if shown {
            self.document
                .selected_toolpath_wireframe(&self.toolpath_visualization_settings)
        } else {
            None
        };
        let lines = wireframe
            .map(|vertices| line_vertices(&vertices, None))
            .unwrap_or_default();
        let visible = self.toolpath_lines_visible;
        let device = self.graphic.device.clone();
        if let Some(mesh_stage) = self.main_mesh_stage() {
            mesh_stage.set_overlay_toolpath_visible(visible);
            let lines = mesh_stage.set_overlay_toolpath_line_data(&device, &lines);
            self.report_not_drawn(lines, crate::help_ui::Notice::LinesNotDrawn);
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
        let meshes = mesh_stage.set_mesh_list(&self.graphic.device, items);
        self.report_not_drawn(meshes, crate::help_ui::Notice::ShapesNotDrawn);
        if fit_camera && let Some(fit) = Self::build_camera_fit(&positions) {
            self.apply_camera_fit(fit);
        }
        self.renderer.set_stage(mesh_stage);
        self.refresh_operation_toolpath_lines();
        self.update_camera_uniforms();
    }
}
