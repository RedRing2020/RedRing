//! AppState のステージ更新オーケストレーションを扱うモジュール。

use super::AppState;
use crate::stage_factory;
use stage::{MeshStage, RenderStage};

impl AppState {
    fn set_stage_and_update_camera(&mut self, stage: Box<dyn RenderStage>) {
        self.renderer.set_stage(stage);
        self.update_camera_uniforms();
    }

    pub(super) fn rebuild_stage_from_entities(&mut self) {
        if !self.entity_manager.is_dirty() {
            return;
        }

        let vertices = self.entity_manager.line_vertices();
        if vertices.is_empty() {
            self.entity_manager.clear_dirty();
            return;
        }

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);

        self.set_stage_and_update_camera(mesh_stage);
        self.entity_manager.clear_dirty();
    }

    pub fn render(&mut self) {
        self.renderer.update();
        {
            let stage = self.renderer.get_stage_mut();
            stage.update_with_device(&self.graphic.device);
        }

        self.update_camera_uniforms();

        self.renderer.update_selection_rect_overlay(
            &self.graphic.queue,
            self.active_selection_rect,
            self.graphic.config.width,
            self.graphic.config.height,
        );
        self.renderer.update_snapshot_overlay(
            &self.graphic.queue,
            self.snapshot_progress_ratio(),
            &self.snapshot_overlay_style,
            self.graphic.config.width,
            self.graphic.config.height,
        );
        self.graphic.render(&mut self.renderer);
    }

    pub fn set_stage_draft(&mut self) {
        let stage =
            stage_factory::create_draft_stage(&self.graphic.device, self.graphic.config.format);
        self.set_stage_and_update_camera(stage);
    }

    pub fn set_stage_outline(&mut self) {
        let stage =
            stage_factory::create_outline_stage(&self.graphic.device, self.graphic.config.format);
        self.set_stage_and_update_camera(stage);
    }

    pub fn set_stage_shading(&mut self) {
        let stage =
            stage_factory::create_shading_stage(&self.graphic.device, self.graphic.config.format);
        self.set_stage_and_update_camera(stage);
    }

    /// カメラのユニフォームを更新
    pub fn update_camera_uniforms(&mut self) {
        let view_matrix = self.camera.view_matrix();
        let aspect = self.graphic.config.width as f32 / self.graphic.config.height as f32;
        let projection_matrix = self.camera.projection_matrix(aspect);

        tracing::info!(
            "🎥 カメラ更新: mode={:?}, aspect={:.3}, target=({:.1}, {:.1}, {:.1}), distance={:.1}, bounds={:?}",
            self.camera.projection_mode,
            aspect,
            self.camera.target.x(),
            self.camera.target.y(),
            self.camera.target.z(),
            self.camera.distance,
            self.camera.orthographic_bounds
        );

        let stage = self.renderer.get_stage_mut();
        stage.update_camera(&self.graphic.queue, view_matrix, projection_matrix);
    }
}
