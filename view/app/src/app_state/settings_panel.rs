use super::AppState;
use crate::settings_panel_ui::SettingsPanelUiState;
use viewmodel::cam_sim_visualization_converter::CamSimulationDemoScenario;

fn restore_cursor_with_fallback(previous_cursor: usize, frame_count: usize) -> usize {
    if frame_count == 0 {
        0
    } else {
        previous_cursor.min(frame_count.saturating_sub(1))
    }
}

impl AppState {
    pub fn toggle_settings_panel(&mut self) {
        self.settings_panel_open = !self.settings_panel_open;
        tracing::info!(
            "設定パネル{}（Shift+S）",
            if self.settings_panel_open {
                "表示"
            } else {
                "非表示"
            }
        );
    }

    pub fn handle_settings_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.renderer
            .handle_settings_window_event(&self.window, event)
    }

    pub fn handle_settings_mouse_motion(&mut self, delta: (f64, f64)) {
        self.renderer.handle_settings_mouse_motion(delta);
    }

    pub(crate) fn prepare_settings_panel_state(&self) -> SettingsPanelUiState {
        SettingsPanelUiState {
            open: self.settings_panel_open,
            camera_control_sensitivity: self.viewing_operation_settings.camera_control_sensitivity,
            octree_settings: self.octree_visualization_settings.clone(),
            snapshot_overlay_style: self.snapshot_overlay_style,
            snapshot_shaded_colors: self.snapshot_shaded_color_settings,
            tool_wireframe_settings: self.tool_wireframe_visualization_settings,
            current_demo_label: self.current_cam_demo_scenario.map(demo_scenario_label),
            reload_demo_requested: false,
        }
    }

    pub(crate) fn apply_settings_panel_state(&mut self, panel_state: SettingsPanelUiState) {
        self.settings_panel_open = panel_state.open;
        self.tool_wireframe_visualization_settings = panel_state.tool_wireframe_settings;

        if panel_state.reload_demo_requested {
            if let Some(scenario) = self.current_cam_demo_scenario {
                let preserved_camera = self.camera.clone();
                let preserved_cursor = self.debug_snapshot.cursor;
                let preserved_shaded_mode = self.debug_snapshot.shaded_mode;

                self.load_sample_toolpath_with_scenario(scenario, "設定パネル");
                self.camera = preserved_camera;
                self.update_camera_uniforms();

                if let Some(series) = &self.debug_snapshot.series {
                    let restored_cursor =
                        restore_cursor_with_fallback(preserved_cursor, series.frames.len());
                    let fallback_applied = restored_cursor != preserved_cursor;

                    self.debug_snapshot.cursor = restored_cursor;

                    if fallback_applied {
                        tracing::info!(
                            "設定再読込: 旧カーソル {} をフレーム末尾 {} へclampして復元",
                            preserved_cursor,
                            restored_cursor
                        );
                    } else {
                        tracing::info!("設定再読込: カーソル {} を維持して復元", restored_cursor);
                    }

                    self.log_current_snapshot_frame(true);

                    // 再読込は一旦ワイヤー初期化されるため、保存前がソリッドなら実ステージごと戻す。
                    if preserved_shaded_mode {
                        self.toggle_wireframe();
                    }
                } else {
                    tracing::warn!(
                        "設定再読込後にスナップショット系列が空のため、カーソルを先頭へ初期化"
                    );
                    self.debug_snapshot.cursor = 0;
                }
            }
        }
    }
}

fn demo_scenario_label(scenario: CamSimulationDemoScenario) -> String {
    match scenario {
        CamSimulationDemoScenario::Success => "ボールエンドミル".to_string(),
        CamSimulationDemoScenario::SuccessFlatEndMill => "フラットエンドミル".to_string(),
        CamSimulationDemoScenario::FailureEmptyToolpath => "空ToolPath".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::restore_cursor_with_fallback;

    #[test]
    fn restore_cursor_keeps_index_when_in_range() {
        assert_eq!(restore_cursor_with_fallback(5, 10), 5);
    }

    #[test]
    fn restore_cursor_clamps_to_tail_when_out_of_range() {
        assert_eq!(restore_cursor_with_fallback(12, 8), 7);
    }

    #[test]
    fn restore_cursor_resets_to_zero_when_empty() {
        assert_eq!(restore_cursor_with_fallback(3, 0), 0);
    }
}
