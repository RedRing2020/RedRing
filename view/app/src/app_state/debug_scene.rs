//! AppState のデバッグ表示（形状・逆オフセット形状）を扱うモジュール群。

pub(super) mod camera_fit;
mod inverse_offset_shape;
mod shapes;

use super::AppState;
use crate::settings_panel_ui::DebugSceneAction;

impl AppState {
    /// 設定パネルのデバッグタブで押されたデバッグ用の表示を行う
    pub(super) fn show_debug_scene(&mut self, action: DebugSceneAction) {
        match action {
            DebugSceneAction::MainView => self.restore_main_view(),
            DebugSceneAction::Line => self.load_debug_line(),
            DebugSceneAction::Circle => self.load_debug_circle(),
            DebugSceneAction::Arc => self.load_debug_arc(),
            DebugSceneAction::Triangle => self.load_debug_triangle(),
            DebugSceneAction::InverseOffsetShape => self.load_debug_inverse_offset_shape(),
        }
    }
}
