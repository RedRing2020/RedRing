use viewmodel::message_catalog::{
    TableMessageCatalog, UiLocale, UiMessage, UiMessageArg, resolve_message,
};

const MACHINING_CONDITIONS_TEMPLATE_JA: &[(&str, &str)] = &[
    ("conditions.list.sequence", "加工手順: {name}"),
    ("conditions.list.sequence_settings", "手順の設定"),
    ("conditions.list.operation_settings", "工程の設定"),
    ("conditions.list.column.order", "順番"),
    ("conditions.list.column.name", "名前"),
    ("conditions.list.column.stage", "加工の段階"),
    ("conditions.list.column.kind", "加工の種類"),
    ("conditions.list.column.tool", "工具"),
    ("conditions.list.column.status", "状態"),
    ("conditions.list.tool", "{kind} φ{diameter}"),
    ("conditions.stage.rough", "荒"),
    ("conditions.stage.semi_finish", "中"),
    ("conditions.stage.finish", "仕上げ"),
    ("conditions.kind.scanline", "スキャン加工"),
    ("conditions.page.machining", "加工条件"),
    ("conditions.page.region", "加工領域・加工対象"),
    ("conditions.operation_settings.title", "工程の設定: {name}"),
    ("conditions.sequence_settings.title", "手順の設定"),
    ("conditions.dialog.apply", "適用"),
    ("conditions.dialog.cancel", "キャンセル"),
    ("conditions.dialog.submit_on_apply", "適用と同時に計算する"),
    ("conditions.operation.stage", "加工の段階"),
    ("conditions.operation.kind", "加工の種類: {kind}"),
    (
        "conditions.target.shared",
        "加工の対象は、手順のすべての工程で同じです。変えると、手順のすべての工程の CL が消え、計算し直しになります",
    ),
    ("conditions.header.region", "加工領域"),
    ("conditions.region.target", "加工の対象の XY の範囲"),
    (
        "conditions.region.rectangle",
        "X {min_x}〜{max_x} mm、Y {min_y}〜{max_y} mm",
    ),
    ("conditions.sequence.name", "名前: {name}"),
    (
        "conditions.sequence.note",
        "素材を変えると、手順のすべての工程の CL が消え、計算し直しになります",
    ),
    ("conditions.header.target", "加工の対象"),
    ("conditions.operations.add", "工程追加"),
    ("conditions.operations.duplicate", "工程複写"),
    ("conditions.operations.remove", "工程削除"),
    ("conditions.operations.move_up", "加工の順を前へ"),
    ("conditions.operations.move_down", "加工の順を後へ"),
    ("conditions.operations.name", "名前"),
    ("conditions.operations.new_name", "工程 {number}"),
    ("conditions.operations.default_name", "スキャン加工"),
    ("conditions.sequence.default_name", "加工手順"),
    ("conditions.operations.copy_name", "{name} のコピー"),
    ("conditions.operations.status.none", "未計算"),
    ("conditions.operations.status.not_set", "未設定"),
    ("conditions.operations.status.condition_error", "条件エラー"),
    ("conditions.operations.status.queued", "計算待ち"),
    ("conditions.operations.status.running", "計算中"),
    ("conditions.operations.status.succeeded", "計算済み"),
    ("conditions.operations.status.imported", "取込済み"),
    ("conditions.operations.status.failed", "計算失敗"),
    ("conditions.operations.status.canceled", "取り消し"),
    ("conditions.operations.status.unreadable", "読めません"),
    ("conditions.operations.status.reading", "読んでいます"),
    ("conditions.operations.status.unknown", "状態不明"),
    ("conditions.target.faces", "表示中の製品形状の面: {count}"),
    ("conditions.header.tool", "工具"),
    ("conditions.tool.ball", "ボールエンドミル"),
    ("conditions.tool.flat", "フラットエンドミル"),
    ("conditions.tool.diameter", "工具径"),
    (
        "conditions.tool.specify_length",
        "工具長を指定する（指定しない場合は長さの制限なし）",
    ),
    ("conditions.header.machining", "スキャン加工の条件"),
    ("conditions.machining.stepover", "ピックフィード"),
    ("conditions.machining.feed_rate", "送り"),
    (
        "conditions.machining.angle",
        "走査線の角度（X 軸から反時計回り）",
    ),
    (
        "conditions.machining.cut_direction",
        "送り方（主軸は右回り）",
    ),
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
    ("conditions.submit", "計算"),
    (
        "conditions.submit.hint",
        "選んだ工程を計算します。今の条件の CL があり、今の条件の計算が残っていなければ、その CL から切削シミュレーションだけを計算します",
    ),
    (
        "conditions.navigation.in_progress",
        "今の条件の計算が進んでいます。終わるまでお待ちください",
    ),
    (
        "conditions.navigation.calculated",
        "今の条件の計算は済んでいます。結果を取り込めます",
    ),
    (
        "conditions.navigation.imported_with_simulation",
        "CL 取込済みです。切削シミュレーションを表示できます",
    ),
    (
        "conditions.navigation.imported_toolpath_only",
        "CL 取込済みです。切削シミュレーションを計算できます",
    ),
    (
        "conditions.navigation.imported_manager_not_running",
        "ジョブマネージャーを開くと、切削シミュレーションの結果があるか確かめられます",
    ),
    ("conditions.redo", "再計算"),
    (
        "conditions.redo.hint",
        "選んだ工程の空間モデルの CL を消して、ツールパス生成から計算し直します",
    ),
    ("conditions.stop", "計算停止"),
    (
        "conditions.stop.hint",
        "選んだ工程の今の条件の、計算待ち・計算中の計算を止めます",
    ),
    ("conditions.import_toolpath", "CL 取込"),
    (
        "conditions.import_toolpath.hint",
        "選んだ工程の今の条件の CL だけを、空間モデルに取り込みます（切削シミュレーションは表示しません）",
    ),
    ("conditions.import", "切削シミュレーション表示"),
    (
        "conditions.import.hint",
        "選んだ工程の今の条件の切削シミュレーションを取り込んで表示し、その CL も空間モデルに取り込みます。表示している間に押すと、表示を閉じます",
    ),
    ("conditions.open_job_manager", "ジョブマネージャーを開く"),
    (
        "conditions.status.submitted",
        "ツールパス生成から計算を始めました",
    ),
    (
        "conditions.status.submitted_simulation",
        "今の条件の CL から、切削シミュレーションだけの計算を始めました",
    ),
    (
        "conditions.status.redone",
        "空間モデルの CL を消して、ツールパス生成から再計算を始めました",
    ),
    (
        "conditions.status.redone_canceled",
        "今の条件の計算を取り消し、空間モデルの CL を消して、ツールパス生成から再計算を始めました",
    ),
    (
        "conditions.status.submit_failed",
        "計算を始められません（{reason}）",
    ),
    (
        "conditions.status.already_calculated",
        "今の条件の計算が、計算待ち・計算中・計算済みのため、計算を始めていません",
    ),
    (
        "conditions.status.already_imported",
        "今の条件の CL を取り込み済みのため、計算を始めていません",
    ),
    (
        "conditions.status.large_toolpath_declined",
        "ツールパスが大きくなるため、計算を始めていません",
    ),
    (
        "conditions.confirm.large_toolpath",
        "ツールパスの生成に多くのメモリを使う工程があります。計算に長い時間がかかり、ほかのアプリが遅くなることがあります。計算しますか？\n\n{operations}",
    ),
    (
        "conditions.confirm.large_toolpath_item",
        "{position}. {operation}: 約 {gb} GB",
    ),
    (
        "conditions.confirm.large_toolpath_item_near_limit",
        "{position}. {operation}: 約 {gb} GB（上限に近いため、途中で止まることがあります）",
    ),
    ("conditions.confirm.calculate", "計算する"),
    ("conditions.confirm.cancel", "やめる"),
    (
        "conditions.status.recalculation_waiting",
        "今の条件の計算が計算待ちのため、再計算していません（計算が始まってから再計算できます）",
    ),
    (
        "conditions.status.stopped",
        "今の条件の計算を停止しています（止まると、状態が取り消しになります）",
    ),
    (
        "conditions.status.earlier_conditions_canceled",
        "前の条件の計算待ち・計算中の計算を取り消しました",
    ),
    (
        "conditions.status.stop_not_accepted",
        "今の条件の計算に、まだジョブマネージャーが受け取っていないものがあるため、すべては停止していません（少し待ってから、もう一度停止してください）",
    ),
    (
        "conditions.status.nothing_to_stop",
        "今の条件の計算は、計算待ち・計算中でないため、停止していません",
    ),
    (
        "conditions.status.stop_failed",
        "計算を停止できません（{reason}）",
    ),
    (
        "conditions.status.sequence_submitted",
        "手順の {submitted} 工程の計算を始めました（計算待ち・計算中・計算済みの {skipped} 工程は、計算し直していません）",
    ),
    (
        "conditions.status.sequence_submitted_all",
        "手順の {submitted} 工程の計算を始めました",
    ),
    (
        "conditions.status.sequence_submitted_simulation_all",
        "手順の {submitted} 工程の計算を始めました。うち {simulation_only} 工程は、今の条件の CL から切削シミュレーションだけを計算します",
    ),
    (
        "conditions.status.sequence_submitted_simulation",
        "手順の {submitted} 工程の計算を始めました。うち {simulation_only} 工程は、今の条件の CL から切削シミュレーションだけを計算します（計算待ち・計算中・計算済みの {skipped} 工程は、計算し直していません）",
    ),
    (
        "conditions.status.sequence_nothing_to_submit",
        "手順のすべての工程が、計算待ち・計算中・計算済みのため、計算を始めていません",
    ),
    (
        "conditions.status.sequence_submit_failed",
        "{position} 番目の工程「{operation}」の計算を始められません（{reason}）。その前の {submitted} 工程は計算を始めました",
    ),
    (
        "conditions.status.sequence_status_unreadable",
        "計算の状態を読めないため、全工程を計算していません（{reason}）。少し待ってから、もう一度押してください",
    ),
    ("conditions.submit_sequence", "全工程を計算"),
    (
        "conditions.status.imported",
        "ジョブ {job} を取り込みました",
    ),
    (
        "conditions.status.nothing_to_import",
        "取り込める結果がありません（計算が終わった後に取り込みます）",
    ),
    (
        "conditions.status.toolpath_imported",
        "ジョブ {job} の CL を取り込みました",
    ),
    (
        "conditions.status.no_toolpath_to_import",
        "取り込める CL がありません（ツールパス生成が終わった後に取り込みます）",
    ),
    (
        "conditions.job.none",
        "今の条件の計算: ジョブがありません（計算していない、または消した）",
    ),
    ("conditions.job.queued", "今の条件の計算: 計算待ち"),
    ("conditions.job.running", "今の条件の計算: 計算中"),
    (
        "conditions.job.succeeded",
        "今の条件の計算: 計算完了（取り込んで表示します）",
    ),
    ("conditions.job.failed", "今の条件の計算: 計算失敗"),
    (
        "conditions.job.canceled",
        "今の条件の計算: 取り消されました",
    ),
    (
        "conditions.job.unreadable",
        "今の条件の計算: ジョブマネージャーから状態を読めません",
    ),
    (
        "conditions.job.reading",
        "今の条件の計算: 状態を読んでいます",
    ),
    (
        "conditions.job.manager_not_running",
        "今の条件の計算: ジョブマネージャーが動いていないため、分かりません（計算開始すると起動します）",
    ),
    (
        "conditions.job.manager_not_found",
        "今の条件の計算: ジョブマネージャーが見つからないため、分かりません",
    ),
    (
        "conditions.toolpath.none",
        "空間モデルの CL: まだ取り込んでいません",
    ),
    (
        "conditions.toolpath.current",
        "空間モデルの CL: 今の条件の結果です",
    ),
    (
        "conditions.status.import_failed",
        "取り込めません（{reason}）",
    ),
    (
        "conditions.status.job_manager_failed",
        "ジョブマネージャーを開けません（{reason}）",
    ),
    (
        "conditions.issue.no_product_shape",
        "製品形状がありません（ファイルを開いてください）",
    ),
    (
        "conditions.issue.not_set",
        "工程の条件をまだ設定していません",
    ),
    (
        "conditions.issue.invalid_stock_margins",
        "余裕は 0 以上の数値にしてください",
    ),
    (
        "conditions.issue.invalid_stock",
        "素材が体積を持つ直方体になりません",
    ),
    (
        "conditions.issue.invalid_tool",
        "工具径・工具長は正の数値にしてください",
    ),
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
    (
        "conditions.issue.feed_rate_not_positive",
        "送りは正の数値にしてください",
    ),
    (
        "conditions.issue.invalid_precision",
        "精度は正の数値にしてください",
    ),
    (
        "conditions.issue.precision_finer_than_tolerance",
        "精度はツールパス生成のトレランス（{tolerance} mm）以上にしてください",
    ),
    (
        "conditions.issue.too_many_dexel_lines",
        "素材の大きさに対して精度が細かすぎます。{finest_precision} mm 以上にしてください",
    ),
    ("conditions.issue.other", "計算できません（{reason}）"),
    ("conditions.header.imported", "表示中の結果の条件"),
    ("conditions.imported.job", "ジョブ {job}"),
    (
        "conditions.imported.tool",
        "工具: {kind} φ{diameter} mm・工具長 {length}",
    ),
    ("conditions.imported.length_unlimited", "指定なし"),
    (
        "conditions.imported.scanline",
        "加工: {cut_direction}・ピックフィード {stepover} mm・送り {feed_rate} mm/min・走査線の角度 {angle}°",
    ),
    (
        "conditions.imported.tolerance",
        "ツールパス生成のトレランス: {tolerance} mm",
    ),
    (
        "conditions.imported.stock",
        "素材: X {x} × Y {y} × Z {z} mm",
    ),
    ("conditions.imported.precision", "精度: {precision} mm"),
];

const MACHINING_CONDITIONS_TEMPLATE_EN: &[(&str, &str)] = &[
    ("conditions.list.sequence", "Machining sequence: {name}"),
    ("conditions.list.sequence_settings", "Sequence settings"),
    ("conditions.list.operation_settings", "Operation settings"),
    ("conditions.list.column.order", "No."),
    ("conditions.list.column.name", "Name"),
    ("conditions.list.column.stage", "Stage"),
    ("conditions.list.column.kind", "Machining type"),
    ("conditions.list.column.tool", "Tool"),
    ("conditions.list.column.status", "Status"),
    ("conditions.list.tool", "{kind} D{diameter}"),
    ("conditions.stage.rough", "Rough"),
    ("conditions.stage.semi_finish", "Semi-finish"),
    ("conditions.stage.finish", "Finish"),
    ("conditions.kind.scanline", "Scanline"),
    ("conditions.page.machining", "Machining conditions"),
    ("conditions.page.region", "Region and target"),
    (
        "conditions.operation_settings.title",
        "Operation settings: {name}",
    ),
    ("conditions.sequence_settings.title", "Sequence settings"),
    ("conditions.dialog.apply", "Apply"),
    ("conditions.dialog.cancel", "Cancel"),
    (
        "conditions.dialog.submit_on_apply",
        "Calculate when applied",
    ),
    ("conditions.operation.stage", "Stage"),
    ("conditions.operation.kind", "Machining type: {kind}"),
    (
        "conditions.target.shared",
        "The target is the same for all operations of the sequence. Changing it removes the CL of all operations of the sequence, and they need to be calculated again",
    ),
    ("conditions.header.region", "Machining region"),
    ("conditions.region.target", "XY extent of the target"),
    (
        "conditions.region.rectangle",
        "X {min_x} to {max_x} mm, Y {min_y} to {max_y} mm",
    ),
    ("conditions.sequence.name", "Name: {name}"),
    (
        "conditions.sequence.note",
        "Changing the stock removes the CL of all operations of the sequence, and they need to be calculated again",
    ),
    ("conditions.header.target", "Target"),
    ("conditions.operations.add", "Add operation"),
    ("conditions.operations.duplicate", "Duplicate operation"),
    ("conditions.operations.remove", "Delete operation"),
    (
        "conditions.operations.move_up",
        "Move earlier in the machining order",
    ),
    (
        "conditions.operations.move_down",
        "Move later in the machining order",
    ),
    ("conditions.operations.name", "Name"),
    ("conditions.operations.new_name", "Operation {number}"),
    ("conditions.operations.default_name", "Scanline"),
    ("conditions.sequence.default_name", "Machining sequence"),
    ("conditions.operations.copy_name", "{name} (copy)"),
    ("conditions.operations.status.none", "not calculated"),
    ("conditions.operations.status.not_set", "not set"),
    (
        "conditions.operations.status.condition_error",
        "condition error",
    ),
    ("conditions.operations.status.queued", "waiting"),
    ("conditions.operations.status.running", "calculating"),
    ("conditions.operations.status.succeeded", "calculated"),
    ("conditions.operations.status.imported", "imported"),
    ("conditions.operations.status.failed", "calculation failed"),
    ("conditions.operations.status.canceled", "canceled"),
    ("conditions.operations.status.unreadable", "cannot read"),
    ("conditions.operations.status.reading", "reading"),
    ("conditions.operations.status.unknown", "unknown"),
    (
        "conditions.target.faces",
        "Visible product shape faces: {count}",
    ),
    ("conditions.header.tool", "Tool"),
    ("conditions.tool.ball", "Ball end mill"),
    ("conditions.tool.flat", "Flat end mill"),
    ("conditions.tool.diameter", "Diameter"),
    (
        "conditions.tool.specify_length",
        "Specify the tool length (unlimited if not specified)",
    ),
    ("conditions.header.machining", "Scanline conditions"),
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
    (
        "conditions.stock.margins",
        "Margins around the product bounds",
    ),
    ("conditions.stock.minus_x", "-X side"),
    ("conditions.stock.plus_x", "+X side"),
    ("conditions.stock.minus_y", "-Y side"),
    ("conditions.stock.plus_y", "+Y side"),
    ("conditions.stock.top", "top"),
    ("conditions.stock.bottom", "bottom"),
    (
        "conditions.stock.precision",
        "Precision (stock surface detail)",
    ),
    ("conditions.submit", "Calculate"),
    (
        "conditions.submit.hint",
        "Calculates the selected operation. If the CL of the current conditions exists and no calculation of the current conditions remains, only the cutting simulation is calculated from that CL",
    ),
    (
        "conditions.navigation.in_progress",
        "The calculation for the current conditions is in progress. Wait until it finishes",
    ),
    (
        "conditions.navigation.calculated",
        "The calculation for the current conditions is done. You can import the result",
    ),
    (
        "conditions.navigation.imported_with_simulation",
        "The CL is imported. You can show the cutting simulation",
    ),
    (
        "conditions.navigation.imported_toolpath_only",
        "The CL is imported. You can calculate the cutting simulation",
    ),
    (
        "conditions.navigation.imported_manager_not_running",
        "Open the job manager to check whether there is a cutting simulation result",
    ),
    ("conditions.redo", "Recalculate"),
    (
        "conditions.redo.hint",
        "Deletes the CL of the selected operation from the space model and calculates again from toolpath generation",
    ),
    ("conditions.stop", "Stop calculation"),
    (
        "conditions.stop.hint",
        "Stops the waiting or running calculation for the current conditions of the selected operation",
    ),
    ("conditions.import_toolpath", "Import CL"),
    (
        "conditions.import_toolpath.hint",
        "Imports only the CL of the current conditions of the selected operation into the space model (the cutting simulation is not shown)",
    ),
    ("conditions.import", "Show cutting simulation"),
    (
        "conditions.import.hint",
        "Imports and shows the cutting simulation of the current conditions of the selected operation, and imports its CL into the space model. Press it again while it is shown to close it",
    ),
    ("conditions.open_job_manager", "Open job manager"),
    (
        "conditions.status.submitted",
        "Started the calculation from toolpath generation",
    ),
    (
        "conditions.status.submitted_simulation",
        "Started calculating only the cutting simulation from the CL of the current conditions",
    ),
    (
        "conditions.status.redone",
        "Deleted the CL from the space model and started recalculating from toolpath generation",
    ),
    (
        "conditions.status.redone_canceled",
        "Canceled the calculation for the current conditions, deleted the CL from the space model, and started recalculating from toolpath generation",
    ),
    (
        "conditions.status.submit_failed",
        "Cannot start the calculation ({reason})",
    ),
    (
        "conditions.status.already_calculated",
        "The calculation was not started because the calculation for the current conditions is waiting, calculating, or calculated",
    ),
    (
        "conditions.status.already_imported",
        "The calculation was not started because the toolpath for the current conditions is already imported",
    ),
    (
        "conditions.status.large_toolpath_declined",
        "The calculation was not started because the toolpath would be large",
    ),
    (
        "conditions.confirm.large_toolpath",
        "Some operations need a lot of memory to generate the toolpath. The calculation may take a long time and slow down other applications. Calculate?\n\n{operations}",
    ),
    (
        "conditions.confirm.large_toolpath_item",
        "{position}. {operation}: about {gb} GB",
    ),
    (
        "conditions.confirm.large_toolpath_item_near_limit",
        "{position}. {operation}: about {gb} GB (close to the limit, so the calculation may stop partway)",
    ),
    ("conditions.confirm.calculate", "Calculate"),
    ("conditions.confirm.cancel", "Cancel"),
    (
        "conditions.status.recalculation_waiting",
        "Not recalculated because the calculation for the current conditions is waiting (it can be recalculated after the calculation starts)",
    ),
    (
        "conditions.status.stopped",
        "Stopping the calculation for the current conditions (the status becomes canceled when it stops)",
    ),
    (
        "conditions.status.earlier_conditions_canceled",
        "Canceled the waiting and running calculations for the earlier conditions",
    ),
    (
        "conditions.status.stop_not_accepted",
        "Not all of the calculation for the current conditions was stopped because the job manager has not received part of it yet (wait a moment and stop it again)",
    ),
    (
        "conditions.status.nothing_to_stop",
        "Not stopped because the calculation for the current conditions is not waiting or calculating",
    ),
    (
        "conditions.status.stop_failed",
        "Cannot stop the calculation ({reason})",
    ),
    (
        "conditions.status.sequence_submitted",
        "Started calculating {submitted} operations of the sequence ({skipped} waiting, calculating, or calculated operations were not calculated again)",
    ),
    (
        "conditions.status.sequence_submitted_all",
        "Started calculating {submitted} operations of the sequence",
    ),
    (
        "conditions.status.sequence_submitted_simulation_all",
        "Started calculating {submitted} operations of the sequence. For {simulation_only} of them, only the cutting simulation is calculated from the CL of the current conditions",
    ),
    (
        "conditions.status.sequence_submitted_simulation",
        "Started calculating {submitted} operations of the sequence. For {simulation_only} of them, only the cutting simulation is calculated from the CL of the current conditions ({skipped} waiting, calculating, or calculated operations were not calculated again)",
    ),
    (
        "conditions.status.sequence_nothing_to_submit",
        "No calculation was started because every operation of the sequence is waiting, calculating, or calculated",
    ),
    (
        "conditions.status.sequence_submit_failed",
        "Cannot start calculating operation {position} \"{operation}\" ({reason}). The calculation of the {submitted} operations before it was started",
    ),
    (
        "conditions.status.sequence_status_unreadable",
        "No operation was calculated because the calculation status cannot be read ({reason}). Wait a moment and try again",
    ),
    ("conditions.submit_sequence", "Calculate all operations"),
    ("conditions.status.imported", "Imported job {job}"),
    (
        "conditions.status.nothing_to_import",
        "No result to import (import it after the calculation finishes)",
    ),
    (
        "conditions.status.toolpath_imported",
        "Imported the CL of job {job}",
    ),
    (
        "conditions.status.no_toolpath_to_import",
        "No CL to import (import it after the toolpath generation finishes)",
    ),
    (
        "conditions.job.none",
        "Calculation for the current conditions: no job (not calculated or deleted)",
    ),
    (
        "conditions.job.queued",
        "Calculation for the current conditions: waiting",
    ),
    (
        "conditions.job.running",
        "Calculation for the current conditions: calculating",
    ),
    (
        "conditions.job.succeeded",
        "Calculation for the current conditions: completed (import it to show it)",
    ),
    (
        "conditions.job.failed",
        "Calculation for the current conditions: calculation failed",
    ),
    (
        "conditions.job.canceled",
        "Calculation for the current conditions: canceled",
    ),
    (
        "conditions.job.unreadable",
        "Calculation for the current conditions: cannot read the status from the job manager",
    ),
    (
        "conditions.job.reading",
        "Calculation for the current conditions: reading the status",
    ),
    (
        "conditions.job.manager_not_running",
        "Calculation for the current conditions: unknown because the job manager is not running (it starts when you start a calculation)",
    ),
    (
        "conditions.job.manager_not_found",
        "Calculation for the current conditions: unknown because the job manager is not found",
    ),
    (
        "conditions.toolpath.none",
        "CL in the space model: not imported yet",
    ),
    (
        "conditions.toolpath.current",
        "CL in the space model: result of the current conditions",
    ),
    (
        "conditions.status.import_failed",
        "Cannot import ({reason})",
    ),
    (
        "conditions.status.job_manager_failed",
        "Cannot open the job manager ({reason})",
    ),
    (
        "conditions.issue.no_product_shape",
        "No product shape (open a file)",
    ),
    (
        "conditions.issue.not_set",
        "The conditions of the operation are not set yet",
    ),
    (
        "conditions.issue.invalid_stock_margins",
        "Margins must be zero or more",
    ),
    (
        "conditions.issue.invalid_stock",
        "The stock is not a box with volume",
    ),
    (
        "conditions.issue.invalid_tool",
        "Tool diameter and length must be positive",
    ),
    (
        "conditions.issue.invalid_toolpath_input",
        "Tool or machining conditions are invalid ({reason})",
    ),
    (
        "conditions.issue.stepover_not_positive",
        "Stepover must be positive",
    ),
    (
        "conditions.issue.stepover_exceeds_tool_diameter",
        "Stepover must not exceed the tool diameter ({diameter} mm)",
    ),
    (
        "conditions.issue.feed_rate_not_positive",
        "Feed rate must be positive",
    ),
    (
        "conditions.issue.invalid_precision",
        "Precision must be positive",
    ),
    (
        "conditions.issue.precision_finer_than_tolerance",
        "Precision must not be finer than the toolpath tolerance ({tolerance} mm)",
    ),
    (
        "conditions.issue.too_many_dexel_lines",
        "Precision is too fine for the stock size. Use {finest_precision} mm or coarser",
    ),
    ("conditions.issue.other", "Cannot calculate ({reason})"),
    (
        "conditions.header.imported",
        "Conditions of the displayed result",
    ),
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
    (
        "conditions.imported.tolerance",
        "Toolpath tolerance: {tolerance} mm",
    ),
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
