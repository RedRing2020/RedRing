use viewmodel::message_catalog::{
    resolve_message, TableMessageCatalog, UiLocale, UiMessage, UiMessageArg,
};

const SETTINGS_PANEL_TEMPLATE_JA: &[(&str, &str)] = &[
    ("settings.window.title", "設定"),
    ("settings.intro", "カテゴリごとに設定を確認・調整します。"),
    ("settings.tab.display", "表示"),
    ("settings.tab.tool", "工具"),
    ("settings.current_demo", "現在のデモ: {label}"),
    ("settings.apply", "設定を反映"),
    (
        "settings.no_active_demo",
        "現在アクティブな切削デモはありません。変更は次回のデモ開始時に反映されます。",
    ),
    ("settings.header.view_controls", "View 操作設定"),
    (
        "settings.view.controls_summary",
        "rotate={rotate}, arcball={arcball}, pan={pan}",
    ),
    (
        "settings.view.zoom_summary",
        "zoom_drag={zoom_drag}, zoom_wheel={zoom_wheel}, pixel_to_line={pixel_to_line}",
    ),
    ("settings.header.octree", "Octree 可視化設定"),
    ("settings.octree.max_depth", "max_depth={max_depth}"),
    (
        "settings.octree.gradient_start",
        "gradient_start=({r}, {g}, {b})",
    ),
    (
        "settings.octree.gradient_end",
        "gradient_end=({r}, {g}, {b})",
    ),
    (
        "settings.octree.tolerance",
        "tol: point={point}, query={query}, nearest={nearest}",
    ),
    ("settings.header.snapshot_overlay", "Snapshot Overlay 設定"),
    ("settings.snapshot_overlay.block_count", "block_count={count}"),
    (
        "settings.snapshot_overlay.track",
        "track=({r}, {g}, {b}, {a})",
    ),
    (
        "settings.snapshot_overlay.done",
        "done=({r}, {g}, {b}, {a})",
    ),
    ("settings.header.snapshot_shading", "Snapshot Shading 設定"),
    ("settings.snapshot_shading.work", "work=({r}, {g}, {b}, {a})"),
    ("settings.snapshot_shading.tool", "tool=({r}, {g}, {b})"),
    ("settings.header.tool_wireframe", "Tool Wireframe 設定"),
    ("settings.tool.manual_apply", "このタブの設定は手動で反映します"),
    ("settings.tool.flat_circle_divisions", "flat 円周分割"),
    ("settings.tool.ball_circle_divisions", "ball 円周分割"),
    ("settings.tool.ball_latitudes", "ball 半球緯線分割"),
    ("settings.tool.ball_meridians", "ball 半球経線本数"),
    (
        "settings.header.toolpath_discretization",
        "ToolPath 離散化設定",
    ),
    ("settings.toolpath.display_only", "表示ワイヤーフレーム専用"),
    ("settings.toolpath.preset", "プリセット"),
    ("settings.toolpath.preset.performance", "軽量"),
    ("settings.toolpath.preset.balanced", "標準"),
    ("settings.toolpath.preset.fine", "精細"),
    (
        "settings.toolpath.current_values",
        "現在値: 弦誤差 {chord_tolerance_mm} mm / 最大角度 {max_angle_deg} 度 / 分割 {min_divisions}..{max_divisions}",
    ),
    ("settings.toolpath.chord_tolerance", "弦誤差(mm)"),
    ("settings.toolpath.max_angle_deg", "最大角度(度)"),
    ("settings.toolpath.min_divisions", "最小分割数"),
    ("settings.toolpath.max_divisions", "最大分割数"),
];

const SETTINGS_PANEL_TEMPLATE_EN: &[(&str, &str)] = &[
    ("settings.window.title", "Settings"),
    ("settings.intro", "Review and adjust settings by category."),
    ("settings.tab.display", "Display"),
    ("settings.tab.tool", "Tool"),
    ("settings.current_demo", "Current demo: {label}"),
    ("settings.apply", "Apply Settings"),
    (
        "settings.no_active_demo",
        "No CAM demo is active. Changes will be applied the next time a demo starts.",
    ),
    ("settings.header.view_controls", "View Controls"),
    (
        "settings.view.controls_summary",
        "rotate={rotate}, arcball={arcball}, pan={pan}",
    ),
    (
        "settings.view.zoom_summary",
        "zoom_drag={zoom_drag}, zoom_wheel={zoom_wheel}, pixel_to_line={pixel_to_line}",
    ),
    ("settings.header.octree", "Octree Visualization"),
    ("settings.octree.max_depth", "max_depth={max_depth}"),
    (
        "settings.octree.gradient_start",
        "gradient_start=({r}, {g}, {b})",
    ),
    (
        "settings.octree.gradient_end",
        "gradient_end=({r}, {g}, {b})",
    ),
    (
        "settings.octree.tolerance",
        "tol: point={point}, query={query}, nearest={nearest}",
    ),
    ("settings.header.snapshot_overlay", "Snapshot Overlay"),
    ("settings.snapshot_overlay.block_count", "block_count={count}"),
    (
        "settings.snapshot_overlay.track",
        "track=({r}, {g}, {b}, {a})",
    ),
    (
        "settings.snapshot_overlay.done",
        "done=({r}, {g}, {b}, {a})",
    ),
    ("settings.header.snapshot_shading", "Snapshot Shading"),
    ("settings.snapshot_shading.work", "work=({r}, {g}, {b}, {a})"),
    ("settings.snapshot_shading.tool", "tool=({r}, {g}, {b})"),
    ("settings.header.tool_wireframe", "Tool Wireframe"),
    ("settings.tool.manual_apply", "Changes on this tab are applied manually."),
    ("settings.tool.flat_circle_divisions", "flat circle divisions"),
    ("settings.tool.ball_circle_divisions", "ball circle divisions"),
    ("settings.tool.ball_latitudes", "ball hemisphere latitude divisions"),
    ("settings.tool.ball_meridians", "ball hemisphere meridian count"),
    (
        "settings.header.toolpath_discretization",
        "ToolPath Discretization",
    ),
    ("settings.toolpath.display_only", "For display wireframe only"),
    ("settings.toolpath.preset", "Preset"),
    ("settings.toolpath.preset.performance", "Light"),
    ("settings.toolpath.preset.balanced", "Balanced"),
    ("settings.toolpath.preset.fine", "Fine"),
    (
        "settings.toolpath.current_values",
        "Current: chord {chord_tolerance_mm} mm / max angle {max_angle_deg} deg / divisions {min_divisions}..{max_divisions}",
    ),
    ("settings.toolpath.chord_tolerance", "Chord tolerance (mm)"),
    ("settings.toolpath.max_angle_deg", "Max angle (deg)"),
    ("settings.toolpath.min_divisions", "Min divisions"),
    ("settings.toolpath.max_divisions", "Max divisions"),
];

pub const SETTINGS_PANEL_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    SETTINGS_PANEL_TEMPLATE_JA,
    SETTINGS_PANEL_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

pub fn resolve_settings_panel_text(locale: UiLocale, key: &str) -> String {
    resolve_settings_panel_text_with_args(locale, key, &[])
}

pub fn resolve_settings_panel_text_with_args(
    locale: UiLocale,
    key: &str,
    args: &[(&str, String)],
) -> String {
    let message = UiMessage {
        key: key.to_string(),
        args: args
            .iter()
            .map(|(name, value)| UiMessageArg {
                name: (*name).to_string(),
                value: value.clone(),
            })
            .collect(),
    };

    resolve_message(&SETTINGS_PANEL_MESSAGE_CATALOG, locale, &message)
}

#[cfg(test)]
mod tests {
    use super::{resolve_settings_panel_text, resolve_settings_panel_text_with_args};
    use viewmodel::message_catalog::UiLocale;

    #[test]
    fn resolves_settings_panel_text_for_ja_and_en() {
        assert_eq!(
            resolve_settings_panel_text(UiLocale::Ja, "settings.window.title"),
            "設定"
        );
        assert_eq!(
            resolve_settings_panel_text(UiLocale::En, "settings.window.title"),
            "Settings"
        );
    }

    #[test]
    fn resolves_settings_panel_text_with_args() {
        let ja = resolve_settings_panel_text_with_args(
            UiLocale::Ja,
            "settings.current_demo",
            &[("label", "ボールエンドミル".to_string())],
        );
        let en = resolve_settings_panel_text_with_args(
            UiLocale::En,
            "settings.current_demo",
            &[("label", "Ball End Mill".to_string())],
        );

        assert_eq!(ja, "現在のデモ: ボールエンドミル");
        assert_eq!(en, "Current demo: Ball End Mill");
    }
}
