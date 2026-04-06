use crate::app_state::SnapshotShadedColorSettings;
use crate::settings_panel_messages::{
    resolve_settings_panel_text, resolve_settings_panel_text_with_args,
};
use crate::snapshot_overlay_renderer::SnapshotOverlayStyle;
use egui::{Align2, CollapsingHeader, Context, RichText, Slider, Window};
use viewmodel::cam_sim_visualization_converter::ToolWireframeVisualizationSettings;
use viewmodel::message_catalog::UiLocale;
use viewmodel::octree_converter::OctreeVisualizationSettings;
use viewmodel::toolpath_converter::{
    DisplayCurveDiscretizationSettings, ToolPathVisualizationSettings,
};
use viewmodel_graphics::CameraControlSensitivity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsPanelTab {
    #[default]
    Display,
    Tool,
}

#[derive(Debug, Clone)]
pub struct SettingsPanelUiState {
    pub open: bool,
    pub active_tab: SettingsPanelTab,
    pub locale: UiLocale,
    pub camera_control_sensitivity: CameraControlSensitivity,
    pub octree_settings: OctreeVisualizationSettings,
    pub snapshot_overlay_style: SnapshotOverlayStyle,
    pub snapshot_shaded_colors: SnapshotShadedColorSettings,
    pub tool_wireframe_settings: ToolWireframeVisualizationSettings,
    pub toolpath_settings: ToolPathVisualizationSettings,
    pub current_demo_label: Option<String>,
    pub reload_demo_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurveDiscretizationPreset {
    Performance,
    Balanced,
    Fine,
}

fn apply_curve_discretization_preset(
    settings: &mut ToolPathVisualizationSettings,
    preset: CurveDiscretizationPreset,
) {
    settings.curve_discretization = match preset {
        CurveDiscretizationPreset::Performance => {
            DisplayCurveDiscretizationSettings::performance_preset()
        }
        CurveDiscretizationPreset::Balanced => {
            DisplayCurveDiscretizationSettings::balanced_preset()
        }
        CurveDiscretizationPreset::Fine => DisplayCurveDiscretizationSettings::fine_preset(),
    };
}

fn set_curve_discretization_max_angle_deg(
    settings: &mut ToolPathVisualizationSettings,
    max_angle_deg: f64,
) {
    settings.curve_discretization.max_angle_step_rad = max_angle_deg.to_radians();
}

fn set_curve_discretization_min_divisions(
    settings: &mut ToolPathVisualizationSettings,
    min_divisions: usize,
) {
    settings.curve_discretization.min_divisions = min_divisions;
    settings.curve_discretization.max_divisions = settings
        .curve_discretization
        .max_divisions
        .max(min_divisions);
}

fn set_curve_discretization_max_divisions(
    settings: &mut ToolPathVisualizationSettings,
    max_divisions: usize,
) {
    settings.curve_discretization.max_divisions = max_divisions;
}

impl SettingsPanelUiState {
    fn text(&self, key: &str) -> String {
        resolve_settings_panel_text(self.locale, key)
    }

    fn text_with_args(&self, key: &str, args: &[(&str, String)]) -> String {
        resolve_settings_panel_text_with_args(self.locale, key, args)
    }

    pub fn show(&mut self, ctx: &Context) {
        if !self.open {
            return;
        }

        let mut open = self.open;

        Window::new(self.text("settings.window.title"))
            .anchor(Align2::RIGHT_TOP, [-16.0, 16.0])
            .default_width(360.0)
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                let display_tab_label = self.text("settings.tab.display");
                let tool_tab_label = self.text("settings.tab.tool");

                ui.label(self.text("settings.intro"));
                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.active_tab,
                        SettingsPanelTab::Display,
                        display_tab_label,
                    );
                    ui.selectable_value(
                        &mut self.active_tab,
                        SettingsPanelTab::Tool,
                        tool_tab_label,
                    );
                });
                ui.separator();

                match self.active_tab {
                    SettingsPanelTab::Display => self.show_display_tab(ui),
                    SettingsPanelTab::Tool => self.show_tool_tab(ui),
                }

                ui.separator();
                match &self.current_demo_label {
                    Some(label) => {
                        ui.label(
                            self.text_with_args(
                                "settings.current_demo",
                                &[("label", label.clone())],
                            ),
                        );
                        if ui.button(self.text("settings.apply")).clicked() {
                            self.reload_demo_requested = true;
                        }
                    }
                    None => {
                        ui.label(self.text("settings.no_active_demo"));
                    }
                }
            });

        self.open = open;
    }

    fn show_display_tab(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new(self.text("settings.header.view_controls"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(self.text_with_args(
                    "settings.view.controls_summary",
                    &[
                        (
                            "rotate",
                            format!("{:.3}", self.camera_control_sensitivity.rotate),
                        ),
                        (
                            "arcball",
                            format!("{:.3}", self.camera_control_sensitivity.arcball),
                        ),
                        ("pan", format!("{:.3}", self.camera_control_sensitivity.pan)),
                    ],
                ));
                ui.label(self.text_with_args(
                    "settings.view.zoom_summary",
                    &[
                        (
                            "zoom_drag",
                            format!("{:.3}", self.camera_control_sensitivity.zoom_drag),
                        ),
                        (
                            "zoom_wheel",
                            format!("{:.3}", self.camera_control_sensitivity.zoom_wheel),
                        ),
                        (
                            "pixel_to_line",
                            format!("{:.3}", self.camera_control_sensitivity.wheel_pixel_to_line),
                        ),
                    ],
                ));
            });

        CollapsingHeader::new(self.text("settings.header.octree"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(self.text_with_args(
                    "settings.octree.max_depth",
                    &[("max_depth", self.octree_settings.max_depth.to_string())],
                ));
                ui.label(self.text_with_args(
                    "settings.octree.gradient_start",
                    &[
                        (
                            "r",
                            format!("{:.2}", self.octree_settings.gradient_start[0]),
                        ),
                        (
                            "g",
                            format!("{:.2}", self.octree_settings.gradient_start[1]),
                        ),
                        (
                            "b",
                            format!("{:.2}", self.octree_settings.gradient_start[2]),
                        ),
                    ],
                ));
                ui.label(self.text_with_args(
                    "settings.octree.gradient_end",
                    &[
                        ("r", format!("{:.2}", self.octree_settings.gradient_end[0])),
                        ("g", format!("{:.2}", self.octree_settings.gradient_end[1])),
                        ("b", format!("{:.2}", self.octree_settings.gradient_end[2])),
                    ],
                ));
                ui.label(self.text_with_args(
                    "settings.octree.tolerance",
                    &[
                        (
                            "point",
                            format!(
                                "{:.4}",
                                self.octree_settings.octree_tolerance.point_aabb_half_extent
                            ),
                        ),
                        (
                            "query",
                            format!("{:.4}", self.octree_settings.octree_tolerance.query_expand),
                        ),
                        (
                            "nearest",
                            format!(
                                "{:.4}",
                                self.octree_settings.octree_tolerance.nearest_prune_margin
                            ),
                        ),
                    ],
                ));
            });

        CollapsingHeader::new(self.text("settings.header.snapshot_overlay"))
            .default_open(false)
            .show(ui, |ui| {
                ui.label(self.text_with_args(
                    "settings.snapshot_overlay.block_count",
                    &[("count", self.snapshot_overlay_style.block_count.to_string())],
                ));
                ui.label(self.text_with_args(
                    "settings.snapshot_overlay.track",
                    &[
                        (
                            "r",
                            format!("{:.2}", self.snapshot_overlay_style.track_color[0]),
                        ),
                        (
                            "g",
                            format!("{:.2}", self.snapshot_overlay_style.track_color[1]),
                        ),
                        (
                            "b",
                            format!("{:.2}", self.snapshot_overlay_style.track_color[2]),
                        ),
                        (
                            "a",
                            format!("{:.2}", self.snapshot_overlay_style.track_color[3]),
                        ),
                    ],
                ));
                ui.label(self.text_with_args(
                    "settings.snapshot_overlay.done",
                    &[
                        (
                            "r",
                            format!("{:.2}", self.snapshot_overlay_style.done_color[0]),
                        ),
                        (
                            "g",
                            format!("{:.2}", self.snapshot_overlay_style.done_color[1]),
                        ),
                        (
                            "b",
                            format!("{:.2}", self.snapshot_overlay_style.done_color[2]),
                        ),
                        (
                            "a",
                            format!("{:.2}", self.snapshot_overlay_style.done_color[3]),
                        ),
                    ],
                ));
            });

        CollapsingHeader::new(self.text("settings.header.snapshot_shading"))
            .default_open(false)
            .show(ui, |ui| {
                ui.label(self.text_with_args(
                    "settings.snapshot_shading.work",
                    &[
                        (
                            "r",
                            format!("{:.2}", self.snapshot_shaded_colors.work_solid_color[0]),
                        ),
                        (
                            "g",
                            format!("{:.2}", self.snapshot_shaded_colors.work_solid_color[1]),
                        ),
                        (
                            "b",
                            format!("{:.2}", self.snapshot_shaded_colors.work_solid_color[2]),
                        ),
                        (
                            "a",
                            format!("{:.2}", self.snapshot_shaded_colors.work_solid_color[3]),
                        ),
                    ],
                ));
                ui.label(self.text_with_args(
                    "settings.snapshot_shading.tool",
                    &[
                        (
                            "r",
                            format!("{:.2}", self.snapshot_shaded_colors.tool_wire_color[0]),
                        ),
                        (
                            "g",
                            format!("{:.2}", self.snapshot_shaded_colors.tool_wire_color[1]),
                        ),
                        (
                            "b",
                            format!("{:.2}", self.snapshot_shaded_colors.tool_wire_color[2]),
                        ),
                    ],
                ));
            });
    }

    fn show_tool_tab(&mut self, ui: &mut egui::Ui) {
        let flat_circle_label = self.text("settings.tool.flat_circle_divisions");
        let ball_circle_label = self.text("settings.tool.ball_circle_divisions");
        let ball_latitudes_label = self.text("settings.tool.ball_latitudes");
        let ball_meridians_label = self.text("settings.tool.ball_meridians");
        let preset_label = self.text("settings.toolpath.preset");
        let preset_performance_label = self.text("settings.toolpath.preset.performance");
        let preset_balanced_label = self.text("settings.toolpath.preset.balanced");
        let preset_fine_label = self.text("settings.toolpath.preset.fine");
        let chord_tolerance_label = self.text("settings.toolpath.chord_tolerance");
        let max_angle_deg_label = self.text("settings.toolpath.max_angle_deg");
        let min_divisions_label = self.text("settings.toolpath.min_divisions");
        let max_divisions_label = self.text("settings.toolpath.max_divisions");

        CollapsingHeader::new(self.text("settings.header.tool_wireframe"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(RichText::new(self.text("settings.tool.manual_apply")).strong());

                let mut flat_circle = self.tool_wireframe_settings.flat_circle_divisions as u32;
                if ui
                    .add(Slider::new(&mut flat_circle, 8..=96).text(flat_circle_label))
                    .changed()
                {
                    self.tool_wireframe_settings.flat_circle_divisions = flat_circle as usize;
                }

                let mut ball_circle = self.tool_wireframe_settings.ball_circle_divisions as u32;
                if ui
                    .add(Slider::new(&mut ball_circle, 8..=128).text(ball_circle_label))
                    .changed()
                {
                    self.tool_wireframe_settings.ball_circle_divisions = ball_circle as usize;
                }

                let mut ball_latitudes =
                    self.tool_wireframe_settings.ball_hemisphere_divisions as u32;
                if ui
                    .add(Slider::new(&mut ball_latitudes, 2..=24).text(ball_latitudes_label))
                    .changed()
                {
                    self.tool_wireframe_settings.ball_hemisphere_divisions =
                        ball_latitudes as usize;
                }

                let mut ball_meridians = self.tool_wireframe_settings.ball_meridian_count as u32;
                if ui
                    .add(Slider::new(&mut ball_meridians, 4..=32).text(ball_meridians_label))
                    .changed()
                {
                    self.tool_wireframe_settings.ball_meridian_count = ball_meridians as usize;
                }
            });

        CollapsingHeader::new(self.text("settings.header.toolpath_discretization"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(RichText::new(self.text("settings.toolpath.display_only")).strong());

                ui.horizontal(|ui| {
                    ui.label(preset_label);
                    if ui.button(preset_performance_label).clicked() {
                        apply_curve_discretization_preset(
                            &mut self.toolpath_settings,
                            CurveDiscretizationPreset::Performance,
                        );
                    }
                    if ui.button(preset_balanced_label).clicked() {
                        apply_curve_discretization_preset(
                            &mut self.toolpath_settings,
                            CurveDiscretizationPreset::Balanced,
                        );
                    }
                    if ui.button(preset_fine_label).clicked() {
                        apply_curve_discretization_preset(
                            &mut self.toolpath_settings,
                            CurveDiscretizationPreset::Fine,
                        );
                    }
                });

                let angle_deg = self
                    .toolpath_settings
                    .curve_discretization
                    .max_angle_step_rad
                    .to_degrees();
                ui.label(
                    self.text_with_args(
                        "settings.toolpath.current_values",
                        &[
                            (
                                "chord_tolerance_mm",
                                format!(
                                    "{:.3}",
                                    self.toolpath_settings
                                        .curve_discretization
                                        .chord_tolerance_mm
                                ),
                            ),
                            ("max_angle_deg", format!("{:.1}", angle_deg)),
                            (
                                "min_divisions",
                                self.toolpath_settings
                                    .curve_discretization
                                    .min_divisions
                                    .to_string(),
                            ),
                            (
                                "max_divisions",
                                self.toolpath_settings
                                    .curve_discretization
                                    .max_divisions
                                    .to_string(),
                            ),
                        ],
                    ),
                );

                ui.add(
                    Slider::new(
                        &mut self
                            .toolpath_settings
                            .curve_discretization
                            .chord_tolerance_mm,
                        0.01..=1.0,
                    )
                    .text(chord_tolerance_label),
                );

                let mut max_angle_deg = self
                    .toolpath_settings
                    .curve_discretization
                    .max_angle_step_rad
                    .to_degrees();
                ui.add(Slider::new(&mut max_angle_deg, 3.0..=90.0).text(max_angle_deg_label));
                set_curve_discretization_max_angle_deg(&mut self.toolpath_settings, max_angle_deg);

                let mut min_divisions =
                    self.toolpath_settings.curve_discretization.min_divisions as u32;
                if ui
                    .add(Slider::new(&mut min_divisions, 4..=64).text(min_divisions_label))
                    .changed()
                {
                    set_curve_discretization_min_divisions(
                        &mut self.toolpath_settings,
                        min_divisions as usize,
                    );
                }

                let min_max = self.toolpath_settings.curve_discretization.min_divisions as u32;
                let mut max_divisions =
                    self.toolpath_settings.curve_discretization.max_divisions as u32;
                if ui
                    .add(Slider::new(&mut max_divisions, min_max..=256).text(max_divisions_label))
                    .changed()
                {
                    set_curve_discretization_max_divisions(
                        &mut self.toolpath_settings,
                        max_divisions as usize,
                    );
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_curve_discretization_preset, set_curve_discretization_max_angle_deg,
        set_curve_discretization_max_divisions, set_curve_discretization_min_divisions,
        CurveDiscretizationPreset,
    };
    use viewmodel::toolpath_converter::ToolPathVisualizationSettings;

    #[test]
    fn curve_discretization_preset_updates_settings() {
        let mut settings = ToolPathVisualizationSettings::default();

        apply_curve_discretization_preset(&mut settings, CurveDiscretizationPreset::Performance);
        let performance = settings.curve_discretization;

        apply_curve_discretization_preset(&mut settings, CurveDiscretizationPreset::Fine);
        let fine = settings.curve_discretization;

        assert!(performance.chord_tolerance_mm > fine.chord_tolerance_mm);
        assert!(performance.max_divisions < fine.max_divisions);
    }

    #[test]
    fn set_curve_discretization_max_angle_deg_converts_to_radians() {
        let mut settings = ToolPathVisualizationSettings::default();

        set_curve_discretization_max_angle_deg(&mut settings, 45.0);

        assert!(
            (settings.curve_discretization.max_angle_step_rad - std::f64::consts::FRAC_PI_4).abs()
                < 1e-12
        );
    }

    #[test]
    fn set_curve_discretization_min_divisions_clamps_max_divisions() {
        let mut settings = ToolPathVisualizationSettings::default();
        settings.curve_discretization.max_divisions = 12;

        set_curve_discretization_min_divisions(&mut settings, 24);

        assert_eq!(settings.curve_discretization.min_divisions, 24);
        assert_eq!(settings.curve_discretization.max_divisions, 24);
    }

    #[test]
    fn set_curve_discretization_max_divisions_updates_value() {
        let mut settings = ToolPathVisualizationSettings::default();

        set_curve_discretization_max_divisions(&mut settings, 72);

        assert_eq!(settings.curve_discretization.max_divisions, 72);
    }
}
