//! ヘルプ（操作の一覧・バージョンの情報）と、画面の下のステータスバーの文言（日本語・英語）

use viewmodel::message_catalog::{
    resolve_message, TableMessageCatalog, UiLocale, UiMessage, UiMessageArg,
};

const HELP_TEMPLATE_JA: &[(&str, &str)] = &[
    ("help.window.title", "操作の一覧"),
    (
        "help.note.text_input",
        "設定パネル・加工条件のウィンドウの入力欄を編集している間は、打った文字はキー操作になりません（Esc での終了を除く）。",
    ),
    ("help.header.camera", "カメラ"),
    ("help.header.cutting", "切削シミュレーション"),
    ("help.header.display", "表示"),
    ("help.header.app", "アプリ"),
    ("help.input.ctrl_left_drag", "Ctrl + 左ドラッグ"),
    ("help.input.ctrl_middle_drag", "Ctrl + 中ドラッグ"),
    ("help.input.ctrl_right_drag", "Ctrl + 右ドラッグ / Ctrl + ホイール"),
    ("help.input.progress_drag", "進捗バーを左ドラッグ"),
    ("help.input.drop", "STL をドロップ"),
    ("help.camera.rotate", "回転"),
    ("help.camera.pan", "パン"),
    ("help.camera.zoom", "ズーム"),
    ("help.camera.reset", "カメラをリセット（形状に合わせ直す）"),
    ("help.camera.standard", "標準 CAD 視点"),
    ("help.camera.front", "正面視点"),
    ("help.camera.escape", "緊急脱出（めり込んだカメラを戻す）"),
    ("help.cutting.conditions", "加工条件のウィンドウの表示・非表示（「投入」で投入）"),
    ("help.cutting.run", "キューにあるジョブを実行"),
    ("help.cutting.import", "今の条件で計算した最も新しい結果を取り込み"),
    ("help.cutting.step", "コマ送りの移動量だけ進む / 戻る"),
    ("help.cutting.endpoint", "工具を次 / 前の区間の端点へ移す"),
    ("help.cutting.play", "自動再生 / 一時停止"),
    ("help.cutting.stop", "自動再生を停止して先頭へ"),
    ("help.cutting.speed", "再生速度を 2 倍 / 半分"),
    ("help.cutting.scrub", "左上の進捗バーで、任意の時点へ移す"),
    ("help.display.wireframe", "ソリッド表示 / ワイヤーフレーム表示"),
    ("help.display.toolpath", "ツールパスの線の表示・非表示"),
    ("help.display.tool", "工具の線の表示・非表示"),
    ("help.app.settings", "設定パネルの表示・非表示"),
    ("help.app.help", "この一覧の表示・非表示"),
    ("help.app.load", "形状を読み込む（「ファイル」→「読み込み…」と同じ）"),
    ("help.app.quit", "終了"),
    ("about.window.title", "バージョンの情報"),
    ("about.version", "RedRing {version}（プレビュー）"),
    ("about.log_file", "ログのファイル: {path}"),
    ("about.no_log_file", "ログのファイルを作れなかったため、ログはコンソールにだけ出しています"),
    ("about.manual", "マニュアル"),
    ("notice.startup", "F1 キー、または「ヘルプ」→「操作の一覧」で、操作を確かめられます"),
    ("notice.shape_imported", "形状を読み込みました: {file}"),
    ("notice.shape_import_failed", "形状を読み込めません: {file}（{reason}）"),
    (
        "notice.not_imported",
        "切削シミュレーションを取り込んでから操作してください",
    ),
    ("notice.no_tool_segments", "ツールパスに区間がないため、工具を区間の端点へ移せません"),
    (
        "notice.apply_without_animation",
        "切削アニメーションの表示中ではないため、設定を反映できません",
    ),
    (
        "notice.debug_view_shown",
        "デバッグの表示中です。通常の表示に戻してから操作してください",
    ),
    (
        "notice.settings_load_failed",
        "アプリの設定を読めないため、既定の設定で起動しました（{reason}）",
    ),
    (
        "notice.settings_not_saved",
        "設定のファイルを使えないため、選んだ設定は次の起動に残りません",
    ),
    ("notice.settings_save_failed", "アプリの設定を保存できません（{reason}）"),
    ("fatal.panic.title", "RedRing の異常終了"),
    (
        "fatal.panic.body",
        "思わぬ異常のため、RedRing を終了します。\n\n{detail}\n\n{log_file}",
    ),
    ("fatal.device_lost.title", "RedRing の描画の停止"),
    (
        "fatal.device_lost.body",
        "画面の描画に使う GPU の装置が使えなくなったため、RedRing を終了します。{detail}\n\n{log_file}",
    ),
    ("fatal.log_file", "ログのファイル: {path}"),
    (
        "fatal.no_log_file",
        "ログのファイルは作れませんでした",
    ),
];

const HELP_TEMPLATE_EN: &[(&str, &str)] = &[
    ("help.window.title", "Controls"),
    (
        "help.note.text_input",
        "While editing a field in the settings panel or the machining conditions window, typed characters are not used as keys (except Esc to quit).",
    ),
    ("help.header.camera", "Camera"),
    ("help.header.cutting", "Cutting simulation"),
    ("help.header.display", "Display"),
    ("help.header.app", "Application"),
    ("help.input.ctrl_left_drag", "Ctrl + left drag"),
    ("help.input.ctrl_middle_drag", "Ctrl + middle drag"),
    ("help.input.ctrl_right_drag", "Ctrl + right drag / Ctrl + wheel"),
    ("help.input.progress_drag", "Drag the progress bar"),
    ("help.input.drop", "Drop an STL file"),
    ("help.camera.rotate", "Rotate"),
    ("help.camera.pan", "Pan"),
    ("help.camera.zoom", "Zoom"),
    ("help.camera.reset", "Reset the camera (fit to the shapes)"),
    ("help.camera.standard", "Standard CAD view"),
    ("help.camera.front", "Front view"),
    ("help.camera.escape", "Emergency escape (bring back a camera inside a shape)"),
    (
        "help.cutting.conditions",
        "Show or hide the machining conditions window (submit with \"Submit\")",
    ),
    ("help.cutting.run", "Run the queued jobs"),
    ("help.cutting.import", "Import the latest result of the current conditions"),
    ("help.cutting.step", "Step forward / back by the step distance"),
    ("help.cutting.endpoint", "Move the tool to the next / previous segment endpoint"),
    ("help.cutting.play", "Play / pause"),
    ("help.cutting.stop", "Stop playback and go to the start"),
    ("help.cutting.speed", "Double / halve the playback speed"),
    ("help.cutting.scrub", "Move to any time with the progress bar at the top left"),
    ("help.display.wireframe", "Solid / wireframe"),
    ("help.display.toolpath", "Show or hide the toolpath lines"),
    ("help.display.tool", "Show or hide the tool lines"),
    ("help.app.settings", "Show or hide the settings panel"),
    ("help.app.help", "Show or hide this list"),
    ("help.app.load", "Load the shape (same as File → Load…)"),
    ("help.app.quit", "Quit"),
    ("about.window.title", "About"),
    ("about.version", "RedRing {version} (preview)"),
    ("about.log_file", "Log file: {path}"),
    ("about.no_log_file", "The log file could not be created, so the log goes only to the console"),
    ("about.manual", "Manual"),
    ("notice.startup", "Press F1 or choose Help → Controls to see the controls"),
    ("notice.shape_imported", "Loaded the shape: {file}"),
    ("notice.shape_import_failed", "Cannot load the shape: {file} ({reason})"),
    (
        "notice.not_imported",
        "Import a cutting simulation first",
    ),
    (
        "notice.no_tool_segments",
        "The toolpath has no segments to move the tool to",
    ),
    (
        "notice.apply_without_animation",
        "Settings can be applied only while a cutting animation is shown",
    ),
    (
        "notice.debug_view_shown",
        "A debug view is shown. Go back to the normal view first",
    ),
    (
        "notice.settings_load_failed",
        "Started with the default settings because the settings could not be read ({reason})",
    ),
    (
        "notice.settings_not_saved",
        "The settings file cannot be used, so the choice is not kept for the next start",
    ),
    ("notice.settings_save_failed", "Cannot save the settings ({reason})"),
    ("fatal.panic.title", "RedRing stopped"),
    (
        "fatal.panic.body",
        "RedRing stops because of an unexpected error.\n\n{detail}\n\n{log_file}",
    ),
    ("fatal.device_lost.title", "RedRing cannot draw"),
    (
        "fatal.device_lost.body",
        "RedRing stops because the GPU device used for drawing is no longer available.{detail}\n\n{log_file}",
    ),
    ("fatal.log_file", "Log file: {path}"),
    (
        "fatal.no_log_file",
        "The log file could not be created",
    ),
];

pub const HELP_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    HELP_TEMPLATE_JA,
    HELP_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

pub fn resolve_help_text(locale: UiLocale, key: &str, args: &[(&str, String)]) -> String {
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
    resolve_message(&HELP_MESSAGE_CATALOG, locale, &message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_both_japanese_and_english_texts() {
        let ja: Vec<&str> = HELP_TEMPLATE_JA.iter().map(|(key, _)| *key).collect();
        let en: Vec<&str> = HELP_TEMPLATE_EN.iter().map(|(key, _)| *key).collect();
        assert_eq!(ja, en);
    }

    #[test]
    fn arguments_are_filled_in() {
        let args = [
            ("file", "a.stl".to_string()),
            ("reason", "broken".to_string()),
        ];
        for locale in [UiLocale::Ja, UiLocale::En] {
            let text = resolve_help_text(locale, "notice.shape_import_failed", &args);
            assert!(text.contains("a.stl") && text.contains("broken"), "{text}");
            assert!(!text.contains('{'), "{text}");
        }
    }
}
