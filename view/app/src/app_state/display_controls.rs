//! AppState の表示モード・カメラ制御を扱うモジュール。

use super::AppState;

impl AppState {
    /// カメラをリセット
    pub fn reset_camera(&mut self) {
        self.camera.reset();
        self.update_camera_uniforms();
    }

    /// カメラを、通常の表示で最後に合わせた形状の全体に、+Z から見る向きで合わせ直す。合わせたら `true`。デバッグ
    /// 表示の間と、まだ形状に合わせていないときは、何もせず `false`
    pub(super) fn fit_camera_to_shapes(&mut self) -> bool {
        match self.camera_fit {
            Some(fit) if !self.debug_view_shown() => {
                self.apply_camera_fit(fit);
                self.update_camera_uniforms();
                true
            }
            _ => false,
        }
    }

    /// 安全な視点にカメラをリセット（標準CAD視点）
    pub fn reset_camera_to_safe_view(&mut self) {
        self.camera.reset_to_standard_cad_view();
        self.update_camera_uniforms();
        tracing::info!("Reset the camera to the standard CAD view");
    }

    /// 緊急脱出：最小距離を強制確保
    pub fn emergency_camera_escape(&mut self) {
        self.camera.ensure_minimum_distance();
        self.update_camera_uniforms();
        tracing::warn!("Emergency camera escape");
    }

    /// カメラ状態をログ出力
    pub fn log_camera_state(&self) {
        self.camera.log_state();
    }

    /// ワイヤーフレーム表示を切り替え
    pub fn toggle_wireframe(&mut self) {
        let stage = self.renderer.get_stage_mut();
        if let Some(is_wireframe) = stage.toggle_wireframe_mode() {
            let mode = if is_wireframe { "wireframe" } else { "solid" };
            tracing::info!("Switched the display mode to {}", mode);
            return;
        }

        tracing::warn!("The current stage does not support the wireframe display");
    }
}
