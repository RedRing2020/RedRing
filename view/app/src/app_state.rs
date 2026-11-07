use crate::app_renderer::{AppRenderer, AppRendererFactory};
use crate::app_settings::AppSettings;
use crate::entity_manager::EntityManager;
use crate::graphic::{init_graphic, Graphic};
use crate::help_ui::Notice;
use crate::machining_conditions_ui::OperationStatus;
use crate::mouse_input::MouseInput;
use crate::selection_rect::SelectionRect;
use crate::settings_panel_ui::SettingsPanelTab;
use crate::snapshot_overlay_renderer::SnapshotOverlayStyle;
use analysis::{LengthUnit, Tolerance};
use debug_snapshot_state::DebugSnapshotState;
use std::path::PathBuf;
use std::sync::Arc;
use viewmodel::cam_sim_visualization_converter::ToolWireframeVisualizationSettings;
use viewmodel::cutting_animation::CuttingAnimation;
use viewmodel::cutting_simulation_session::{
    ConditionIssue, CuttingSimulationSession, ImportedConditions,
};
use viewmodel::message_catalog::UiLocale;
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
// AppState のメニューバーの項目
mod menu_bar;

mod help;
// AppState のデバッグスナップショット状態
mod debug_snapshot_state;

// AppState の通常の表示とデバッグ表示の切り替え
mod main_view;
// AppState のデバッグ表示（形状・逆オフセット形状）
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
// AppState の設定アクセサ（種類別: Snapshot/Camera）
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
    /// 最後の時点の素材の面の、製品形状との比較による色分け
    pub deviations: DeviationColorSettings,
}

impl Default for SnapshotShadedColorSettings {
    fn default() -> Self {
        Self {
            work_solid_color: [0.95, 0.55, 0.25, 1.0],
            tool_wire_color: [0.9, 0.95, 1.0],
            deviations: DeviationColorSettings::default(),
        }
    }
}

/// 最後の時点の素材の面の、製品形状との比較（削り込み・トレランスの中・削り残し）による色分け（比べていない所は
/// 素材の色）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviationColorSettings {
    /// 色分けするか
    pub shown: bool,
    /// 削り込みの色（RGBA）
    pub gouge_color: [f32; 4],
    /// トレランスの中の色（RGBA）
    pub within_color: [f32; 4],
    /// 削り残しの色（RGBA）
    pub uncut_color: [f32; 4],
}

impl Default for DeviationColorSettings {
    fn default() -> Self {
        Self {
            shown: true,
            gouge_color: [0.9, 0.15, 0.15, 1.0],
            within_color: [0.2, 0.75, 0.3, 1.0],
            uncut_color: [0.25, 0.45, 0.95, 1.0],
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

    /// 取り込んだ製品形状
    product_shapes: ProductShapeScene,

    /// 切削シミュレーションの投入・実行・取り込み
    cutting_simulation: CuttingSimulationSession,

    /// 取り込んだ切削シミュレーションの切削アニメーション
    cutting_animation: Option<CuttingAnimation>,

    /// 切削アニメーションを描いている色（取り込みと「設定を反映」で、設定の色にする）
    cutting_animation_colors: SnapshotShadedColorSettings,

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

    /// メニューバーの「ファイル」→「読み込み…」が選ばれ、まだファイルを開く画面を開いていない
    shape_file_load_requested: bool,

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

    /// 画面の文言の言語（「表示」→「言語」で選び、アプリの設定に保存する）
    ui_locale: UiLocale,
    /// アプリの設定（表示の言語）と、その保存のファイル（利用者ごとのフォルダが分からないとき・読めなかったときは
    /// `None`。読めなかったファイルは上書きしない）
    app_settings: AppSettings,
    app_settings_path: Option<PathBuf>,

    /// ヘルプの「操作の一覧」のウィンドウを開いているか
    help_controls_open: bool,
    /// ヘルプの「バージョンの情報」のウィンドウを開いているか
    help_about_open: bool,
    /// 画面の下の知らせの行に出している、最後の知らせ
    notice: Notice,

    /// デバッグ用: シミュレーションスナップショット状態
    debug_snapshot: DebugSnapshotState,
    /// デバッグ表示の間、取っておく通常の表示（ステージとカメラ。通常の表示の間は `None`）
    main_view: Option<main_view::MainView>,
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
        // 描画の準備で異常になっても選んだ言語で知らせられるよう、設定を先に読む
        let (app_settings, app_settings_path, settings_notice) = load_app_settings();
        let ui_locale = app_settings.locale();
        crate::fatal_error_dialog::set_fatal_error_locale(ui_locale);
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
            viewing_operation_settings,
            snapshot_overlay_style: SnapshotOverlayStyle::default(),
            snapshot_shaded_color_settings: SnapshotShadedColorSettings::default(),
            tool_wireframe_visualization_settings: ToolWireframeVisualizationSettings::default(),
            toolpath_visualization_settings: ToolPathVisualizationSettings::default(),
            product_shapes: ProductShapeScene::default(),
            cutting_simulation: CuttingSimulationSession::default(),
            cutting_animation: None,
            cutting_animation_colors: SnapshotShadedColorSettings::default(),
            tool_length_input: 50.0,
            machining_conditions_open: false,
            condition_issues: None,
            imported_conditions: None,
            run_after_submit: true,
            shape_file_load_requested: false,
            cutting_simulation_status: None,
            toolpath_lines_visible: true,
            tool_lines_visible: true,
            cutting_animation_job_id: None,
            settings_panel_open: false,
            settings_panel_active_tab: SettingsPanelTab::default(),
            ui_locale,
            app_settings,
            app_settings_path,
            help_controls_open: false,
            help_about_open: false,
            notice: settings_notice.unwrap_or(Notice::Startup),
            debug_snapshot: DebugSnapshotState::default(),
            main_view: None,
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

/// アプリの設定を読む。読めないときは既定の設定にし（そのファイルは上書きしないため、保存の場所を返さない）、
/// 知らせの行に出す知らせを返す
fn load_app_settings() -> (AppSettings, Option<PathBuf>, Option<Notice>) {
    let Some(path) = crate::app_settings::settings_file_path() else {
        tracing::warn!("The per-user folder is unknown, so the app settings are not saved");
        return (AppSettings::default(), None, None);
    };
    match crate::app_settings::load_settings(&path) {
        Ok(settings) => (settings, Some(path), None),
        Err(error) => {
            tracing::warn!(path = %path.display(), kind = ?error.kind(), "Cannot read the app settings; starting with the defaults: {error}");
            let notice = Notice::SettingsLoadFailed {
                reason: viewmodel::failure_message_mapper::file_failure_message(error.kind()),
            };
            (AppSettings::default(), None, Some(notice))
        }
    }
}
