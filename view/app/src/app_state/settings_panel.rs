use super::AppState;
use crate::settings_panel_ui::SettingsPanelUiState;
use viewmodel::cam_sim_visualization_converter::CamSimulationDemoScenario;

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
                self.load_sample_toolpath_with_scenario(scenario, "設定パネル");
                self.camera = preserved_camera;
                self.update_camera_uniforms();
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
