use viewmodel::message_catalog::{
    resolve_message, TableMessageCatalog, UiLocale, UiMessage, UiMessageArg,
};

const MACHINING_CONDITIONS_TEMPLATE_JA: &[(&str, &str)] = &[
    ("conditions.window.title", "加工条件"),
    (
        "conditions.intro",
        "次に投入する切削シミュレーションの計算条件です。「投入」で投入するとジョブマネージャーが計算し、終わった後に「取り込み」で結果を表示します。",
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
    ("conditions.machining.stepover", "ピックフィード"),
    ("conditions.machining.feed_rate", "送り"),
    ("conditions.machining.angle", "走査線の角度（X 軸から反時計回り）"),
    ("conditions.machining.cut_direction", "送り方（主軸は右回り）"),
    ("conditions.machining.cut_direction.down", "ダウンカット"),
    ("conditions.machining.cut_direction.up", "アップカット"),
    (
        "conditions.machining.cut_direction.zigzag",
        "ジグザグ（ダウンカットから）",
    ),
    ("conditions.machining.tool_range", "工具の移動範囲"),
    (
        "conditions.machining.tool_range.extend",
        "形状の範囲を工具の半径だけ広げる",
    ),
    ("conditions.machining.tool_range.geometry", "形状の範囲"),
    (
        "conditions.machining.cusp_height",
        "カスプハイト（平らな面）: {height} mm 以下",
    ),
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
    ("conditions.status.submitted", "投入しました"),
    ("conditions.status.submit_failed", "投入できません（{reason}）"),
    ("conditions.status.imported", "ジョブ {job} を取り込みました"),
    (
        "conditions.status.nothing_to_import",
        "取り込める結果がありません（投入して計算が終わった後に取り込みます）",
    ),
    ("conditions.job.none", "今の条件の計算: まだ投入していません"),
    ("conditions.job.queued", "今の条件の計算: 計算待ち"),
    ("conditions.job.running", "今の条件の計算: 計算中"),
    (
        "conditions.job.succeeded",
        "今の条件の計算: 計算完了（「取り込み」で表示します）",
    ),
    ("conditions.job.failed", "今の条件の計算: 失敗しました"),
    ("conditions.job.canceled", "今の条件の計算: 取り消されました"),
    ("conditions.job.unreadable", "今の条件の計算: ジョブの保存先を読めません"),
    ("conditions.status.import_failed", "取り込めません（{reason}）"),
    ("conditions.issue.no_product_shape", "製品形状がありません（ファイルを開いてください）"),
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
    (
        "conditions.issue.stepover_not_positive",
        "ピックフィードは正の数値にしてください",
    ),
    (
        "conditions.issue.stepover_exceeds_tool_diameter",
        "ピックフィードは工具径（{diameter} mm）以下にしてください",
    ),
    ("conditions.issue.feed_rate_not_positive", "送りは正の数値にしてください"),
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
        "加工: {cut_direction}・ピックフィード {stepover} mm・送り {feed_rate} mm/min・走査線の角度 {angle}°",
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
        "Calculation conditions for the next cutting simulation. Submit to have the job manager calculate it, then press Import after it finishes to show the result.",
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
    ("conditions.machining.feed_rate", "Feed rate"),
    (
        "conditions.machining.angle",
        "Scan angle (counterclockwise from X)",
    ),
    (
        "conditions.machining.cut_direction",
        "Cut direction (spindle clockwise)",
    ),
    ("conditions.machining.cut_direction.down", "Climb"),
    ("conditions.machining.cut_direction.up", "Conventional"),
    (
        "conditions.machining.cut_direction.zigzag",
        "Zigzag (from climb)",
    ),
    ("conditions.machining.tool_range", "Tool range"),
    (
        "conditions.machining.tool_range.extend",
        "Shape extent widened by the tool radius",
    ),
    ("conditions.machining.tool_range.geometry", "Shape extent"),
    (
        "conditions.machining.cusp_height",
        "Cusp height (flat surface): up to {height} mm",
    ),
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
    ("conditions.status.submitted", "Submitted"),
    ("conditions.status.submit_failed", "Cannot submit ({reason})"),
    ("conditions.status.imported", "Imported job {job}"),
    (
        "conditions.status.nothing_to_import",
        "No result to import (submit and wait for the calculation to finish first)",
    ),
    ("conditions.job.none", "Calculation for the current conditions: not submitted"),
    ("conditions.job.queued", "Calculation for the current conditions: waiting"),
    ("conditions.job.running", "Calculation for the current conditions: calculating"),
    (
        "conditions.job.succeeded",
        "Calculation for the current conditions: completed (press Import to show it)",
    ),
    ("conditions.job.failed", "Calculation for the current conditions: failed"),
    ("conditions.job.canceled", "Calculation for the current conditions: canceled"),
    (
        "conditions.job.unreadable",
        "Calculation for the current conditions: cannot read the job store",
    ),
    ("conditions.status.import_failed", "Cannot import ({reason})"),
    ("conditions.issue.no_product_shape", "No product shape (open a file)"),
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
    ("conditions.issue.stepover_not_positive", "Stepover must be positive"),
    (
        "conditions.issue.stepover_exceeds_tool_diameter",
        "Stepover must not exceed the tool diameter ({diameter} mm)",
    ),
    ("conditions.issue.feed_rate_not_positive", "Feed rate must be positive"),
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
        "Machining: {cut_direction}, stepover {stepover} mm, feed {feed_rate} mm/min, scan angle {angle}°",
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

        let args = [("diameter", "10".to_string())];
        assert_eq!(
            resolve_machining_conditions_text(
                UiLocale::Ja,
                "conditions.issue.stepover_exceeds_tool_diameter",
                &args
            ),
            "ピックフィードは工具径（10 mm）以下にしてください"
        );
        assert_eq!(
            resolve_machining_conditions_text(
                UiLocale::En,
                "conditions.issue.stepover_exceeds_tool_diameter",
                &args
            ),
            "Stepover must not exceed the tool diameter (10 mm)"
        );

        let args = [("height", "0.1010".to_string())];
        assert_eq!(
            resolve_machining_conditions_text(
                UiLocale::Ja,
                "conditions.machining.cusp_height",
                &args
            ),
            "カスプハイト（平らな面）: 0.1010 mm 以下"
        );
        assert_eq!(
            resolve_machining_conditions_text(
                UiLocale::En,
                "conditions.machining.cusp_height",
                &args
            ),
            "Cusp height (flat surface): up to 0.1010 mm"
        );
    }
}
