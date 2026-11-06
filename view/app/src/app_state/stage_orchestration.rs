//! AppState のステージ更新オーケストレーションを扱うモジュール。

use super::AppState;
use crate::help_ui::NoticeBarUiState;
use stage::MeshStage;

impl AppState {
    pub(super) fn rebuild_stage_from_entities(&mut self) {
        if !self.entity_manager.is_dirty() {
            return;
        }

        let vertices = self.entity_manager.line_list_vertices();
        if vertices.is_empty() {
            self.entity_manager.clear_dirty();
            return;
        }

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);

        // エンティティの線は、デバッグ用の表示として描く（通常の表示は取っておく）
        self.show_debug_stage(mesh_stage);
        self.entity_manager.clear_dirty();
    }

    pub fn render(&mut self) {
        if self.debug_snapshot.frame_update_required && !self.debug_view_shown() {
            self.sync_snapshot_visual_frame();
        }
        self.apply_cutting_animation_update();
        self.renderer.update();
        {
            let stage = self.renderer.get_stage_mut();
            stage.update_with_device(&self.graphic.device);
        }

        let mut menu_bar_state = self.prepare_menu_bar_state();
        let mut settings_panel_state = self.prepare_settings_panel_state();
        let mut machining_conditions_state = self.prepare_machining_conditions_state();
        let mut help_state = self.prepare_help_state();
        self.renderer.prepare_ui_windows(
            &self.window,
            &mut menu_bar_state,
            NoticeBarUiState {
                locale: self.ui_locale,
                notice: &self.notice,
            },
            &mut settings_panel_state,
            &mut machining_conditions_state,
            &mut help_state,
        );
        self.apply_help_state(help_state);
        self.apply_settings_panel_state(settings_panel_state);
        self.apply_machining_conditions_state(machining_conditions_state);
        self.apply_menu_bar_state(menu_bar_state);

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

    /// カメラのユニフォームを更新
    pub fn update_camera_uniforms(&mut self) {
        let view_matrix = self.camera.view_matrix();
        let aspect = self.graphic.config.width as f32 / self.graphic.config.height as f32;
        let projection_matrix = self.camera.projection_matrix(aspect);

        tracing::debug!(
            "Camera updated: mode={:?}, aspect={:.3}, target=({:.1}, {:.1}, {:.1}), distance={:.1}, bounds={:?}",
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
