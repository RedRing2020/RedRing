//! AppState のキーボード入力ハンドリングを扱うモジュール。

use super::AppState;

impl AppState {
    /// キーボード入力を処理
    pub fn handle_keyboard_input(&mut self, key: &winit::keyboard::Key, pressed: bool) {
        self.mouse_input.update_key(key, pressed);

        if !pressed {
            return;
        }

        // 設定パネル・加工条件のウィンドウの入力欄の編集中は、打った文字をキー操作にしない
        // F1 は文字を打たないため、入力欄の編集中も受ける
        if matches!(
            key,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F1)
        ) {
            self.toggle_help_controls();
            return;
        }

        if self.renderer.is_settings_using_keyboard() {
            return;
        }

        if matches!(
            key,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space)
        ) {
            self.toggle_auto_snapshot_playback();
            return;
        }

        if let winit::keyboard::Key::Character(ch) = key {
            match ch.as_str() {
                "M" => {
                    self.toggle_machining_conditions();
                }
                "R" => {
                    self.run_cutting_simulation_jobs();
                }
                "L" => {
                    self.import_cutting_simulation();
                }
                "r" => {
                    self.camera.reset();
                    self.update_camera_uniforms();
                    tracing::info!("Reset the camera (r key)");
                }
                "t" => {
                    self.camera.reset_to_standard_cad_view();
                    self.update_camera_uniforms();
                    tracing::info!("Set the standard CAD view (t key)");
                }
                "f" => {
                    self.camera.reset_to_front_view();
                    self.update_camera_uniforms();
                    tracing::info!("Set the front view (f key)");
                }
                "e" => {
                    self.camera.emergency_camera_escape();
                    self.update_camera_uniforms();
                    tracing::warn!("Emergency camera escape (e key)");
                }
                "h" => {
                    self.toggle_help_controls();
                }
                "w" => {
                    self.toggle_wireframe();
                }
                "v" => {
                    self.toggle_toolpath_lines();
                }
                "V" => {
                    self.toggle_tool_lines();
                }
                "S" => {
                    self.toggle_settings_panel();
                }
                "k" => {
                    self.cycle_debug_simulation_snapshot();
                }
                "j" => {
                    self.rewind_debug_simulation_snapshot();
                }
                "." => {
                    self.step_tool_to_segment_endpoint(true);
                }
                "," => {
                    self.step_tool_to_segment_endpoint(false);
                }
                "J" => {
                    self.stop_auto_snapshot_playback();
                }
                "+" | "=" => {
                    self.scale_snapshot_playback_speed(2.0);
                }
                "-" | "_" => {
                    self.scale_snapshot_playback_speed(0.5);
                }
                _ => {}
            }
        }
    }
}
