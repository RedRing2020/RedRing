use crate::app_state::{AppState, SnapshotShadedColorSettings};
use stage::MeshStage;

impl AppState {
    /// Snapshotシェーディング時のワークソリッド色を設定
    pub fn set_snapshot_work_solid_color(&mut self, color: [f32; 4]) {
        self.snapshot_shaded_color_settings.work_solid_color = color;

        if self.debug_snapshot.shaded_mode {
            let stage = self.renderer.get_stage_mut();
            if let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() {
                mesh_stage.set_mesh_base_color(color);
            }
        }
    }

    /// Snapshotシェーディング時の工具ワイヤー色を設定
    pub fn set_snapshot_tool_wire_color(&mut self, color: [f32; 3]) {
        self.snapshot_shaded_color_settings.tool_wire_color = color;

        if let Some(tool_lines_per_frame) = &mut self.debug_snapshot.tool_lines {
            for frame in tool_lines_per_frame.iter_mut() {
                for vertex in frame.iter_mut() {
                    vertex.normal = color;
                }
            }
        }

        if self.debug_snapshot.shaded_mode {
            self.sync_snapshot_visual_frame();
        }
    }

    /// Snapshotシェーディング色設定を一括更新
    pub fn set_snapshot_shaded_color_settings(&mut self, settings: SnapshotShadedColorSettings) {
        self.set_snapshot_work_solid_color(settings.work_solid_color);
        self.set_snapshot_tool_wire_color(settings.tool_wire_color);
    }

    /// Snapshotシェーディング時のワークソリッド色を取得
    pub fn snapshot_work_solid_color(&self) -> [f32; 4] {
        self.snapshot_shaded_color_settings.work_solid_color
    }

    /// Snapshotシェーディング時の工具ワイヤー色を取得
    pub fn snapshot_tool_wire_color(&self) -> [f32; 3] {
        self.snapshot_shaded_color_settings.tool_wire_color
    }

    /// Snapshotシェーディング色設定を取得
    pub fn snapshot_shaded_color_settings(&self) -> SnapshotShadedColorSettings {
        self.snapshot_shaded_color_settings
    }
}
