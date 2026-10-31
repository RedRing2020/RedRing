use viewmodel::message_catalog::{
    resolve_message, TableMessageCatalog, UiLocale, UiMessage, UiMessageArg,
};

const MACHINING_CONDITIONS_TEMPLATE_JA: &[(&str, &str)] = &[
    ("conditions.window.title", "加工条件"),
    (
        "conditions.intro",
        "次に投入する切削シミュレーションの計算条件です。「投入」で投入し、実行が終わった後に「取り込み」で結果を表示します。",
    ),
    ("conditions.header.target", "加工の対象"),
    ("conditions.target.faces", "表示中の製品形状の面: {count}"),
    ("conditions.header.tool", "工具"),
    ("conditions.tool.ball", "ボールエンドミル"),
    ("conditions.tool.flat", "フラットエンドミル"),
    ("conditions.tool.diameter", "工具径"),
    (
        "conditions.tool.specify_length",
        "工具長を指定する（指定しない場合は長さの制限なし）",
    ),
    ("conditions.header.machining", "加工（スキャン加工）"),
    ("conditions.machining.stepover", "ステップオーバー"),
    ("conditions.machining.sample_pitch", "サンプル間隔"),
    ("conditions.machining.feed_rate", "送り"),
    ("conditions.machining.tool_range", "工具の移動範囲"),
    (
        "conditions.machining.tool_range.extend",
        "形状の範囲を工具の半径だけ広げる",
    ),
    ("conditions.machining.tool_range.geometry", "形状の範囲"),
    (
        "conditions.machining.tolerance",
        "ツールパス生成のトレランス: {tolerance} mm（変えられません）",
    ),
    ("conditions.header.stock", "素材"),
    ("conditions.stock.margins", "製品形状を囲む箱からの余裕"),
    ("conditions.stock.minus_x", "−X 側"),
    ("conditions.stock.plus_x", "+X 側"),
    ("conditions.stock.minus_y", "−Y 側"),
    ("conditions.stock.plus_y", "+Y 側"),
    ("conditions.stock.top", "上"),
    ("conditions.stock.bottom", "底"),
    ("conditions.stock.precision", "精度（素材の面の細かさ）"),
    ("conditions.submit", "投入"),
    ("conditions.import", "取り込み"),
    ("conditions.run_after_submit", "投入したらすぐ実行する（実行が終わるまで画面は止まります）"),
    ("conditions.status.submitted", "投入しました。Shift+R で実行します"),
    ("conditions.status.submit_failed", "投入できません（{reason}）"),
    (
        "conditions.status.ran",
        "実行しました（{executed} 件・{seconds} 秒）。「取り込み」で結果を表示します",
    ),
    ("conditions.status.run_failed", "実行できません（{reason}）"),
    ("conditions.status.imported", "ジョブ {job} を取り込みました"),
    (
        "conditions.status.nothing_to_import",
        "取り込める結果がありません（投入して実行した後に取り込みます）",
    ),
    ("conditions.status.import_failed", "取り込めません（{reason}）"),
    ("conditions.issue.no_product_shape", "製品形状がありません（STL をドロップしてください）"),
    (
        "conditions.issue.invalid_stock_margins",
        "余裕は 0 以上の数値にしてください",
    ),
    ("conditions.issue.invalid_stock", "素材が体積を持つ直方体になりません"),
    ("conditions.issue.invalid_tool", "工具径・工具長は正の数値にしてください"),
    (
        "conditions.issue.invalid_toolpath_input",
        "工具・加工の条件が正しくありません（{reason}）",
    ),
    ("conditions.issue.invalid_precision", "精度は正の数値にしてください"),
    (
        "conditions.issue.precision_finer_than_tolerance",
        "精度はツールパス生成のトレランス（{tolerance} mm）以上にしてください",
    ),
    (
        "conditions.issue.too_many_dexel_lines",
        "素材の大きさに対して精度が細かすぎます。{finest_precision} mm 以上にしてください",
    ),
    ("conditions.issue.other", "投入できません（{reason}）"),
    ("conditions.header.imported", "表示中の結果の条件"),
    ("conditions.imported.none", "切削シミュレーションが取り込まれていません"),
    ("conditions.imported.job", "ジョブ {job}"),
    ("conditions.imported.tool", "工具: {kind} φ{diameter} mm・工具長 {length}"),
    ("conditions.imported.length_unlimited", "指定なし"),
    (
        "conditions.imported.scanline",
        "加工: ステップオーバー {stepover} mm・サンプル間隔 {sample_pitch} mm・送り {feed_rate} mm/min",
    ),
    ("conditions.imported.tolerance", "ツールパス生成のトレランス: {tolerance} mm"),
    (
        "conditions.imported.stock",
        "素材: X {x} × Y {y} × Z {z} mm",
    ),
    ("conditions.imported.precision", "精度: {precision} mm"),
];

const MACHINING_CONDITIONS_TEMPLATE_EN: &[(&str, &str)] = &[
    ("conditions.window.title", "Machining Conditions"),
    (
        "conditions.intro",
        "Calculation conditions for the next cutting simulation. Submit, then press Import after the run to show the result.",
    ),
    ("conditions.header.target", "Target"),
    ("conditions.target.faces", "Visible product shape faces: {count}"),
    ("conditions.header.tool", "Tool"),
    ("conditions.tool.ball", "Ball end mill"),
    ("conditions.tool.flat", "Flat end mill"),
    ("conditions.tool.diameter", "Diameter"),
    (
        "conditions.tool.specify_length",
        "Specify the tool length (unlimited if not specified)",
    ),
    ("conditions.header.machining", "Machining (scanline)"),
    ("conditions.machining.stepover", "Stepover"),
    ("conditions.machining.sample_pitch", "Sample pitch"),
    ("conditions.machining.feed_rate", "Feed rate"),
    ("conditions.machining.tool_range", "Tool range"),
    (
        "conditions.machining.tool_range.extend",
        "Shape extent widened by the tool radius",
    ),
    ("conditions.machining.tool_range.geometry", "Shape extent"),
    (
        "conditions.machining.tolerance",
        "Toolpath tolerance: {tolerance} mm (fixed)",
    ),
    ("conditions.header.stock", "Stock"),
    ("conditions.stock.margins", "Margins around the product bounds"),
    ("conditions.stock.minus_x", "-X side"),
    ("conditions.stock.plus_x", "+X side"),
    ("conditions.stock.minus_y", "-Y side"),
    ("conditions.stock.plus_y", "+Y side"),
    ("conditions.stock.top", "top"),
    ("conditions.stock.bottom", "bottom"),
    ("conditions.stock.precision", "Precision (stock surface detail)"),
    ("conditions.submit", "Submit"),
    ("conditions.import", "Import"),
    (
        "conditions.run_after_submit",
        "Run right after submitting (the screen stops until the run ends)",
    ),
    ("conditions.status.submitted", "Submitted. Run with Shift+R"),
    ("conditions.status.submit_failed", "Cannot submit ({reason})"),
    (
        "conditions.status.ran",
        "Ran {executed} jobs in {seconds} s. Press Import to show the result",
    ),
    ("conditions.status.run_failed", "Cannot run ({reason})"),
    ("conditions.status.imported", "Imported job {job}"),
    (
        "conditions.status.nothing_to_import",
        "No result to import (submit and run first)",
    ),
    ("conditions.status.import_failed", "Cannot import ({reason})"),
    ("conditions.issue.no_product_shape", "No product shape (drop an STL file)"),
    (
        "conditions.issue.invalid_stock_margins",
        "Margins must be zero or more",
    ),
    ("conditions.issue.invalid_stock", "The stock is not a box with volume"),
    (
        "conditions.issue.invalid_tool",
        "Tool diameter and length must be positive",
    ),
    (
        "conditions.issue.invalid_toolpath_input",
        "Tool or machining conditions are invalid ({reason})",
    ),
    ("conditions.issue.invalid_precision", "Precision must be positive"),
    (
        "conditions.issue.precision_finer_than_tolerance",
        "Precision must not be finer than the toolpath tolerance ({tolerance} mm)",
    ),
    (
        "conditions.issue.too_many_dexel_lines",
        "Precision is too fine for the stock size. Use {finest_precision} mm or coarser",
    ),
    ("conditions.issue.other", "Cannot submit ({reason})"),
    ("conditions.header.imported", "Conditions of the displayed result"),
    ("conditions.imported.none", "No cutting simulation is imported"),
    ("conditions.imported.job", "Job {job}"),
    (
        "conditions.imported.tool",
        "Tool: {kind} D{diameter} mm, length {length}",
    ),
    ("conditions.imported.length_unlimited", "unlimited"),
    (
        "conditions.imported.scanline",
        "Machining: stepover {stepover} mm, sample pitch {sample_pitch} mm, feed {feed_rate} mm/min",
    ),
    ("conditions.imported.tolerance", "Toolpath tolerance: {tolerance} mm"),
    (
        "conditions.imported.stock",
        "Stock: X {x} x Y {y} x Z {z} mm",
    ),
    ("conditions.imported.precision", "Precision: {precision} mm"),
];

pub const MACHINING_CONDITIONS_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    MACHINING_CONDITIONS_TEMPLATE_JA,
    MACHINING_CONDITIONS_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

pub fn resolve_machining_conditions_text(
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

    resolve_message(&MACHINING_CONDITIONS_MESSAGE_CATALOG, locale, &message)
}

#[cfg(test)]
mod tests {
    use super::resolve_machining_conditions_text;
    use viewmodel::message_catalog::UiLocale;

    #[test]
    fn resolves_machining_conditions_text_for_ja_and_en() {
        let args = [("tolerance", "0.001".to_string())];
        assert_eq!(
            resolve_machining_conditions_text(
                UiLocale::Ja,
                "conditions.issue.precision_finer_than_tolerance",
                &args
            ),
            "精度はツールパス生成のトレランス（0.001 mm）以上にしてください"
        );
        assert_eq!(
            resolve_machining_conditions_text(
                UiLocale::En,
                "conditions.issue.precision_finer_than_tolerance",
                &args
            ),
            "Precision must not be finer than the toolpath tolerance (0.001 mm)"
        );
    }
}
