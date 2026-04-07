use crate::app_renderer::{AppRenderer, AppRendererFactory};
use crate::entity_manager::EntityManager;
use crate::graphic::{init_graphic, Graphic};
use crate::mouse_input::MouseInput;
use crate::selection_rect::SelectionRect;
use crate::settings_panel_ui::SettingsPanelTab;
use crate::snapshot_overlay_renderer::SnapshotOverlayStyle;
use analysis::{LengthUnit, Tolerance};
use debug_snapshot_state::DebugSnapshotState;
use std::sync::Arc;
use viewmodel::cam_sim_visualization_converter::{
    CamSimulationDemoScenario, ToolWireframeVisualizationSettings,
};
use viewmodel::message_catalog::UiLocale;
use viewmodel::octree_converter::OctreeVisualizationSettings;
use viewmodel::toolpath_converter::ToolPathVisualizationSettings;
use viewmodel_graphics::{Camera, CameraControlSensitivity};
use winit::window::Window;

// AppState の画面ライフサイクル処理（主にリサイズ）
mod app_lifecycle;
// AppState のデバッグスナップショット状態
mod debug_snapshot_state;
// AppState のデバッグ表示ロード（octree/toolpath/svg/nurbs）
mod debug_scene;
// AppState の表示モード・カメラ制御
mod display_controls;
// AppState の基盤設定（単位系・トレランス）
mod foundation_settings;
// AppState のキーボード入力ハンドリング
mod input_actions;
// AppState のマウス入力ハンドリング
mod mouse_actions;
// AppState の設定アクセサ（種類別: Snapshot/Octree/Camera）
mod settings_accessors;
// AppState の設定パネル状態と適用
mod settings_panel;
// AppState の Snapshot 再生・スクラブ制御
mod snapshot_playback;
// AppState の stage復元スナップショット
mod stage_restore;
// AppState のステージ更新オーケストレーション
mod stage_orchestration;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewingOperationSettings {
    pub camera_control_sensitivity: CameraControlSensitivity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapshotShadedColorSettings {
    // Snapshotシェーディング時のワーク色（RGBA）
    pub work_solid_color: [f32; 4],
    // Snapshotシェーディング時の工具ワイヤー色（RGB）
    pub tool_wire_color: [f32; 3],
}

impl Default for SnapshotShadedColorSettings {
    fn default() -> Self {
        Self {
            work_solid_color: [0.95, 0.55, 0.25, 1.0],
            tool_wire_color: [0.9, 0.95, 1.0],
        }
    }
}

pub struct AppState {
    pub window: Arc<Window>,
    pub graphic: Graphic,
    pub renderer: AppRenderer,
    pub camera: Camera,
    pub mouse_input: MouseInput,
    pub entity_manager: EntityManager,
    pub active_selection_rect: Option<SelectionRect>,
    pub last_selection_rect: Option<SelectionRect>,

    /// アプリケーション単位系（CAD標準: ミリメートル）
    ///
    /// 全ての幾何データはこの単位で解釈されます。
    /// デフォルト: ミリメートル（浮動小数点誤差を最小化）
    pub unit_system: LengthUnit,

    /// 表示トレランス（CAD標準: 0.01mm）
    ///
    /// 曲線のテッセレーション（分割）や近似計算で使用される許容誤差。
    /// この値により、曲線から生成される線分の精度が決まります。
    pub display_tolerance: Tolerance,

    /// Octree可視化設定（setting画面向けの保持値）
    pub octree_visualization_settings: OctreeVisualizationSettings,

    /// ビュー操作設定（setting画面向けの保持値）
    pub viewing_operation_settings: ViewingOperationSettings,

    /// Snapshot進捗バー表示設定
    pub snapshot_overlay_style: SnapshotOverlayStyle,

    /// Snapshotシェーディング時の色設定
    pub snapshot_shaded_color_settings: SnapshotShadedColorSettings,

    /// Toolワイヤーフレーム可視化設定
    tool_wireframe_visualization_settings: ToolWireframeVisualizationSettings,

    /// ToolPath表示設定
    toolpath_visualization_settings: ToolPathVisualizationSettings,

    /// 最後に表示したCAMデモの種別
    current_cam_demo_scenario: Option<CamSimulationDemoScenario>,

    /// 設定パネル表示状態
    settings_panel_open: bool,

    /// 設定パネルの選択中タブ
    settings_panel_active_tab: SettingsPanelTab,

    /// 設定パネルの表示言語
    settings_panel_locale: UiLocale,

    /// デバッグ用: シミュレーションスナップショット状態
    debug_snapshot: DebugSnapshotState,
    snapshot_scrub_active: bool,

    cursor_position: Option<(f32, f32)>,
    last_cursor_position: Option<(f32, f32)>,
    arcball_drag_start: Option<(f32, f32)>,
    arcball_virtual_cursor: Option<(f32, f32)>,
    selection_rect_drag_origin: Option<(f32, f32)>,
}

impl AppState {
    fn apply_viewing_operation_settings(&mut self) {
        self.camera
            .set_control_sensitivity(self.viewing_operation_settings.camera_control_sensitivity);
    }

    pub fn new(window: Arc<Window>) -> Self {
        let graphic = init_graphic(window.clone());
        let renderer = AppRendererFactory::create_draft(&window, &graphic.device, &graphic.config);
        let viewing_operation_settings = ViewingOperationSettings::default();

        let mut app_state = Self {
            window,
            graphic,
            renderer,
            camera: Camera::new(),
            mouse_input: MouseInput::new(),
            entity_manager: EntityManager::new(),
            active_selection_rect: None,
            last_selection_rect: None,
            // CAD標準設定
            unit_system: LengthUnit::Millimeter,
            display_tolerance: Tolerance::default(), // 0.01mm
            octree_visualization_settings: OctreeVisualizationSettings::default(),
            viewing_operation_settings,
            snapshot_overlay_style: SnapshotOverlayStyle::default(),
            snapshot_shaded_color_settings: SnapshotShadedColorSettings::default(),
            tool_wireframe_visualization_settings: ToolWireframeVisualizationSettings::default(),
            toolpath_visualization_settings: ToolPathVisualizationSettings::default(),
            current_cam_demo_scenario: None,
            settings_panel_open: false,
            settings_panel_active_tab: SettingsPanelTab::default(),
            settings_panel_locale: UiLocale::Ja,
            debug_snapshot: DebugSnapshotState::default(),
            snapshot_scrub_active: false,
            cursor_position: None,
            last_cursor_position: None,
            arcball_drag_start: None,
            arcball_virtual_cursor: None,
            selection_rect_drag_origin: None,
        };

        app_state.apply_viewing_operation_settings();
        app_state
    }
}
