//! AppState の表示モード・カメラ制御を扱うモジュール。

use super::AppState;

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
