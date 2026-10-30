//! AppState のデバッグ表示ロード（octree/toolpath/svg/nurbs）を扱うモジュール群。

pub(super) mod camera_fit;
mod inverse_offset;
mod nurbs;
mod octree;
mod shapes;
mod toolpath;

pub(crate) use inverse_offset::InverseOffsetDebugState;

use super::AppState;
use crate::settings_panel_ui::DebugSceneAction;

impl AppState {
    /// 設定パネルのデバッグタブで押されたデバッグ用の表示を行う
    pub(super) fn show_debug_scene(&mut self, action: DebugSceneAction) {
        match action {
            DebugSceneAction::Line => self.load_debug_line(),
            DebugSceneAction::Circle => self.load_debug_circle(),
            DebugSceneAction::Arc => self.load_debug_arc(),
            DebugSceneAction::Triangle => self.load_debug_triangle(),
            DebugSceneAction::NurbsCurve => self.load_debug_nurbs(),
            DebugSceneAction::NurbsSurface => self.load_debug_nurbs_surface(),
            DebugSceneAction::InverseOffset => self.load_debug_inverse_offset(),
            DebugSceneAction::InverseOffsetTool => self.toggle_debug_inverse_offset_tool(),
            DebugSceneAction::InverseOffsetMode => self.toggle_debug_inverse_offset_mode(),
            DebugSceneAction::Octree => {
                let stage = self.renderer.get_stage_mut();
                match stage.cycle_octree_depth(&self.graphic.device) {
                    Some((current_depth, max_depth)) => {
                        tracing::info!("Octree深さ表示: {}/{}", current_depth, max_depth);
                    }
                    None => self.load_debug_octree(),
                }
            }
            DebugSceneAction::OctreeDepthAnimation => {
                let started = self.renderer.get_stage_mut().start_octree_depth_animation() || {
                    self.load_debug_octree();
                    self.renderer.get_stage_mut().start_octree_depth_animation()
                };
                if started {
                    tracing::info!("Octree深さアニメーション再生開始");
                }
            }
            DebugSceneAction::ToolPathOnly => self.load_sample_toolpath_only(),
        }
    }
}
