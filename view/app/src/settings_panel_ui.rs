use crate::app_state::SnapshotShadedColorSettings;
use crate::snapshot_overlay_renderer::SnapshotOverlayStyle;
use egui::{Align2, CollapsingHeader, Context, RichText, Slider, Window};
use viewmodel::cam_sim_visualization_converter::ToolWireframeVisualizationSettings;
use viewmodel::octree_converter::OctreeVisualizationSettings;
use viewmodel_graphics::CameraControlSensitivity;

#[derive(Debug, Clone)]
pub struct SettingsPanelUiState {
    pub open: bool,
    pub camera_control_sensitivity: CameraControlSensitivity,
    pub octree_settings: OctreeVisualizationSettings,
    pub snapshot_overlay_style: SnapshotOverlayStyle,
    pub snapshot_shaded_colors: SnapshotShadedColorSettings,
    pub tool_wireframe_settings: ToolWireframeVisualizationSettings,
    pub current_demo_label: Option<String>,
    pub reload_demo_requested: bool,
}

impl SettingsPanelUiState {
    pub fn show(&mut self, ctx: &Context) {
        if !self.open {
            return;
        }

        Window::new("Settings")
            .anchor(Align2::RIGHT_TOP, [-16.0, 16.0])
            .default_width(360.0)
            .resizable(true)
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.label("登録済み設定の棚卸しと、工具ワイヤーフレーム分割数の調整を行います。");
                ui.separator();

                CollapsingHeader::new("View 操作設定")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!(
                            "rotate={:.3}, arcball={:.3}, pan={:.3}",
                            self.camera_control_sensitivity.rotate,
                            self.camera_control_sensitivity.arcball,
                            self.camera_control_sensitivity.pan,
                        ));
                        ui.label(format!(
                            "zoom_drag={:.3}, zoom_wheel={:.3}, pixel_to_line={:.3}",
                            self.camera_control_sensitivity.zoom_drag,
                            self.camera_control_sensitivity.zoom_wheel,
                            self.camera_control_sensitivity.wheel_pixel_to_line,
                        ));
                    });

                CollapsingHeader::new("Octree 可視化設定")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("max_depth={}", self.octree_settings.max_depth));
                        ui.label(format!(
                            "gradient_start=({:.2}, {:.2}, {:.2})",
                            self.octree_settings.gradient_start[0],
                            self.octree_settings.gradient_start[1],
                            self.octree_settings.gradient_start[2],
                        ));
                        ui.label(format!(
                            "gradient_end=({:.2}, {:.2}, {:.2})",
                            self.octree_settings.gradient_end[0],
                            self.octree_settings.gradient_end[1],
                            self.octree_settings.gradient_end[2],
                        ));
                        ui.label(format!(
                            "tol: point={:.4}, query={:.4}, nearest={:.4}",
                            self.octree_settings.octree_tolerance.point_aabb_half_extent,
                            self.octree_settings.octree_tolerance.query_expand,
                            self.octree_settings.octree_tolerance.nearest_prune_margin,
                        ));
                    });

                CollapsingHeader::new("Snapshot Overlay 設定")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(format!("block_count={}", self.snapshot_overlay_style.block_count));
                        ui.label(format!(
                            "track=({:.2}, {:.2}, {:.2}, {:.2})",
                            self.snapshot_overlay_style.track_color[0],
                            self.snapshot_overlay_style.track_color[1],
                            self.snapshot_overlay_style.track_color[2],
                            self.snapshot_overlay_style.track_color[3],
                        ));
                        ui.label(format!(
                            "done=({:.2}, {:.2}, {:.2}, {:.2})",
                            self.snapshot_overlay_style.done_color[0],
                            self.snapshot_overlay_style.done_color[1],
                            self.snapshot_overlay_style.done_color[2],
                            self.snapshot_overlay_style.done_color[3],
                        ));
                    });

                CollapsingHeader::new("Snapshot Shading 設定")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(format!(
                            "work=({:.2}, {:.2}, {:.2}, {:.2})",
                            self.snapshot_shaded_colors.work_solid_color[0],
                            self.snapshot_shaded_colors.work_solid_color[1],
                            self.snapshot_shaded_colors.work_solid_color[2],
                            self.snapshot_shaded_colors.work_solid_color[3],
                        ));
                        ui.label(format!(
                            "tool=({:.2}, {:.2}, {:.2})",
                            self.snapshot_shaded_colors.tool_wire_color[0],
                            self.snapshot_shaded_colors.tool_wire_color[1],
                            self.snapshot_shaded_colors.tool_wire_color[2],
                        ));
                    });

                CollapsingHeader::new("Tool Wireframe 設定")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("このセクションのみ編集可能").strong());

                        let mut flat_circle = self.tool_wireframe_settings.flat_circle_divisions as u32;
                        if ui
                            .add(Slider::new(&mut flat_circle, 8..=96).text("flat 円周分割"))
                            .changed()
                        {
                            self.tool_wireframe_settings.flat_circle_divisions = flat_circle as usize;
                        }

                        let mut ball_circle = self.tool_wireframe_settings.ball_circle_divisions as u32;
                        if ui
                            .add(Slider::new(&mut ball_circle, 8..=128).text("ball 円周分割"))
                            .changed()
                        {
                            self.tool_wireframe_settings.ball_circle_divisions = ball_circle as usize;
                        }

                        let mut ball_latitudes =
                            self.tool_wireframe_settings.ball_hemisphere_divisions as u32;
                        if ui
                            .add(Slider::new(&mut ball_latitudes, 2..=24).text("ball 半球緯線分割"))
                            .changed()
                        {
                            self.tool_wireframe_settings.ball_hemisphere_divisions =
                                ball_latitudes as usize;
                        }

                        let mut ball_meridians = self.tool_wireframe_settings.ball_meridian_count as u32;
                        if ui
                            .add(Slider::new(&mut ball_meridians, 4..=32).text("ball 半球経線本数"))
                            .changed()
                        {
                            self.tool_wireframe_settings.ball_meridian_count = ball_meridians as usize;
                        }

                        ui.separator();
                        match &self.current_demo_label {
                            Some(label) => {
                                ui.label(format!("現在のデモ: {}", label));
                                if ui.button(format!("現在の{}デモを再読込", label)).clicked() {
                                    self.reload_demo_requested = true;
                                }
                            }
                            None => {
                                ui.label("現在アクティブな切削デモはありません。変更は次回のデモ開始時に反映されます。");
                            }
                        }
                    });
            });
    }
}
