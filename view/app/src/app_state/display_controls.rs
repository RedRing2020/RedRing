//! AppState の表示モード・カメラ制御を扱うモジュール。

use super::AppState;
use stage::{MeshStage, OctreeStage};

impl AppState {
    /// カメラをリセット
    pub fn reset_camera(&mut self) {
        self.camera.reset();
        self.update_camera_uniforms();
    }

    /// 安全な視点にカメラをリセット（標準CAD視点）
    pub fn reset_camera_to_safe_view(&mut self) {
        self.camera.reset_to_standard_cad_view();
        self.update_camera_uniforms();
        tracing::info!("カメラを標準CAD視点にリセット");
    }

    /// 緊急脱出：最小距離を強制確保
    pub fn emergency_camera_escape(&mut self) {
        self.camera.ensure_minimum_distance();
        self.update_camera_uniforms();
        tracing::warn!("緊急カメラ脱出実行");
    }

    /// カメラ状態をログ出力
    pub fn log_camera_state(&self) {
        self.camera.log_state();
    }

    /// ワイヤーフレーム表示を切り替え
    pub fn toggle_wireframe(&mut self) {
        if self.debug_snapshot.series.is_some()
            && self.debug_snapshot.wireframes.is_some()
            && self.debug_snapshot.solids.is_some()
        {
            if self.debug_snapshot.shaded_mode {
                let Some(snapshot_wireframes) = &self.debug_snapshot.wireframes else {
                    return;
                };
                if snapshot_wireframes.is_empty() {
                    return;
                }

                let mut octree_stage = Box::new(OctreeStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                octree_stage.set_depth_levels(&self.graphic.device, snapshot_wireframes.clone());
                let frame_index = self
                    .debug_snapshot
                    .cursor
                    .min(snapshot_wireframes.len().saturating_sub(1));
                octree_stage.set_depth(&self.graphic.device, frame_index);
                self.renderer.set_stage(octree_stage);
                self.debug_snapshot.shaded_mode = false;
                self.update_camera_uniforms();
                tracing::info!("Octree表示モード: ワイヤーフレーム");
                return;
            }

            let Some(snapshot_solids) = &self.debug_snapshot.solids else {
                return;
            };
            if snapshot_solids.is_empty() {
                return;
            }

            let frame_index = self
                .debug_snapshot
                .cursor
                .min(snapshot_solids.len().saturating_sub(1));
            let (vertices, indices) = snapshot_solids[frame_index].clone();

            let mut mesh_stage = Box::new(MeshStage::new(
                &self.graphic.device,
                self.graphic.config.format,
            ));
            mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);
            mesh_stage.set_mesh_base_color(self.snapshot_shaded_color_settings.work_solid_color);

            let toolpath_lines = self
                .debug_snapshot
                .toolpath_lines
                .clone()
                .unwrap_or_default();
            if let Some(tool_lines_per_frame) = &self.debug_snapshot.tool_lines {
                let overlay_index = frame_index.min(tool_lines_per_frame.len().saturating_sub(1));
                let tool_lines = tool_lines_per_frame[overlay_index].clone();
                if !tool_lines.is_empty() {
                    mesh_stage.set_overlay_tool_line_data(&self.graphic.device, tool_lines);
                }
            }
            if !toolpath_lines.is_empty() {
                mesh_stage.set_overlay_toolpath_line_data(&self.graphic.device, toolpath_lines);
            }
            self.renderer.set_stage(mesh_stage);
            self.debug_snapshot.shaded_mode = true;
            self.update_camera_uniforms();
            tracing::info!("Octree表示モード: シェーディング（ソリッド）");
            return;
        }

        let stage = self.renderer.get_stage_mut();
        if let Some(is_wireframe) = stage.toggle_wireframe_mode() {
            let mode = if is_wireframe {
                "ワイヤーフレーム"
            } else {
                "ソリッド"
            };
            tracing::info!("表示モードを{}に切り替え", mode);
            return;
        }

        tracing::warn!("現在のステージはワイヤーフレーム表示に対応していません");
    }
}
