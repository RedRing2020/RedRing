use crate::app_renderer::{AppRenderer, AppRendererFactory};
use crate::entity_manager::EntityManager;
use crate::graphic::{init_graphic, Graphic};
use crate::machining_conditions_ui::OperationStatus;
use crate::mouse_input::MouseInput;
use crate::selection_rect::SelectionRect;
use crate::settings_panel_ui::SettingsPanelTab;
use crate::snapshot_overlay_renderer::SnapshotOverlayStyle;
use analysis::{LengthUnit, Tolerance};
use debug_scene::InverseOffsetDebugState;
use debug_snapshot_state::DebugSnapshotState;
use std::sync::Arc;
use viewmodel::cam_sim_visualization_converter::ToolWireframeVisualizationSettings;
use viewmodel::cutting_animation::CuttingAnimation;
use viewmodel::cutting_simulation_session::{
    ConditionIssue, CuttingSimulationSession, ImportedConditions,
};
use viewmodel::inverse_offset_converter::InverseOffsetVisualizationSettings;
use viewmodel::message_catalog::UiLocale;
use viewmodel::octree_converter::OctreeVisualizationSettings;
use viewmodel::product_shape_converter::ProductShapeScene;
use viewmodel::toolpath_converter::ToolPathVisualizationSettings;
use viewmodel_graphics::{Camera, CameraControlSensitivity};
use winit::window::Window;

// AppState の画面ライフサイクル処理（主にリサイズ）
mod app_lifecycle;
// AppState の切削シミュレーションの投入・実行・取り込みと切削アニメーションの描画
mod cutting_simulation;
// AppState の加工条件のウィンドウ（計算条件の入力と投入）
mod machining_conditions;
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
// AppState の製品形状の取り込みと表示
mod product_shapes;
// AppState の設定アクセサ（種類別: Snapshot/Octree/Camera）
mod settings_accessors;
// AppState の設定パネル状態と適用
mod settings_panel;
// AppState の Snapshot 再生・スクラブ制御
mod snapshot_playback;
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

    /// 逆オフセット包絡面デバッグ表示設定
    inverse_offset_visualization_settings: InverseOffsetVisualizationSettings,

    /// 逆オフセット包絡面デバッグ表示の状態（工具種別・表示モード）
    inverse_offset_debug: InverseOffsetDebugState,

    /// 取り込んだ製品形状
    product_shapes: ProductShapeScene,

    /// 切削シミュレーションの投入・実行・取り込み
    cutting_simulation: CuttingSimulationSession,

    /// 取り込んだ切削シミュレーションの切削アニメーション
    cutting_animation: Option<CuttingAnimation>,

    /// 加工条件のウィンドウの、工具長を指定するときの入力値（mm）
    tool_length_input: f64,

    /// 加工条件のウィンドウの表示状態
    machining_conditions_open: bool,

    /// 次に投入する計算条件の問題（`None` は、条件・製品形状が変わって確かめ直す必要がある）
    condition_issues: Option<Vec<ConditionIssue>>,

    /// 取り込んだ切削シミュレーションの計算に使った条件
    imported_conditions: Option<ImportedConditions>,

    /// 加工条件のウィンドウの「投入」で、投入した後に続けて実行するか
    run_after_submit: bool,

    /// 切削シミュレーションの最後の操作（投入・実行・取り込み）の結果
    cutting_simulation_status: Option<OperationStatus>,

    /// 切削アニメーションのツールパスの線を描くか
    toolpath_lines_visible: bool,

    /// 切削アニメーションの工具の線を描くか
    tool_lines_visible: bool,

    /// 取り込んだ切削シミュレーションのジョブの番号
    cutting_animation_job_id: Option<u64>,

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
    /// 設定パネルか加工条件のウィンドウ（egui のウィンドウ）が開いている
    pub fn is_ui_window_open(&self) -> bool {
        self.is_settings_panel_open() || self.is_machining_conditions_open()
    }

    fn apply_viewing_operation_settings(&mut self) {
        self.camera
            .set_control_sensitivity(self.viewing_operation_settings.camera_control_sensitivity);
    }

    pub fn new(window: Arc<Window>) -> Self {
        let graphic = init_graphic(window.clone());
        let renderer = AppRendererFactory::create_empty(&window, &graphic.device, &graphic.config);
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
            inverse_offset_visualization_settings: InverseOffsetVisualizationSettings::default(),
            inverse_offset_debug: InverseOffsetDebugState::default(),
            product_shapes: ProductShapeScene::default(),
            cutting_simulation: CuttingSimulationSession::default(),
            cutting_animation: None,
            tool_length_input: 50.0,
            machining_conditions_open: false,
            condition_issues: None,
            imported_conditions: None,
            run_after_submit: true,
            cutting_simulation_status: None,
            toolpath_lines_visible: true,
            tool_lines_visible: true,
            cutting_animation_job_id: None,
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
