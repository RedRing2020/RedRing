use crate::app_state::{AppState, SnapshotShadedColorSettings};

impl AppState {
    /// 切削アニメーションの素材の色を設定（取り込んだ切削アニメーションの描き直しで反映する）
    pub fn set_snapshot_work_solid_color(&mut self, color: [f32; 4]) {
        self.snapshot_shaded_color_settings.work_solid_color = color;
    }

    /// 切削アニメーションの工具の線の色を設定（取り込んだ切削アニメーションの描き直しで反映する）
    pub fn set_snapshot_tool_wire_color(&mut self, color: [f32; 3]) {
        self.snapshot_shaded_color_settings.tool_wire_color = color;
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
