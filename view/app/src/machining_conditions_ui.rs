//! 加工手順の画面（切削シミュレーションの計算条件の入力と投入）
//!
//! 表示の設定（設定パネル）とは分けて、次の 3 つを扱う。
//! - 工程リスト（画面の下のパネル）: 加工手順 1 つの加工工程の表（選ぶ・加える・複写する・消す・並べ替える）と、
//!   計算の操作・その結果・選んだ工程の計算の状態
//! - 工程の設定のダイアログ: 選んだ工程の条件（工程の中身）と、表示中の結果の計算に使った条件
//! - 手順の設定のダイアログ: 手順の名前と素材
//!
//! 条件は、ダイアログで編集し、「適用」するまで工程・手順に入れない（触っただけで再計算待ちにしない）。ダイアログを
//! 開いている間は、見る操作（カメラ・切削アニメーション・表示の設定）だけを受け、工程リストは押せない。

use crate::machining_conditions_messages::resolve_machining_conditions_text;
use egui::{
    CollapsingHeader, Color32, Context, DragValue, Id, Panel, Rect, RichText, ScrollArea, Stroke,
    Window,
};
use viewmodel::cutting_simulation_session::{
    flat_surface_cusp_height, CamConditionError, CamConditionField, ConditionIssue,
    CuttingToolKind, ImportedConditions, MachiningBoundary, MachiningOperationContent,
    MachiningStage, MachiningStock, OperationJobStatus, OperationSpec, ScanlineCutDirection,
    ScanlineToolRange, StockMargins, Tool,
};
use viewmodel::failure_message_catalog::resolve_failure_message;
use viewmodel::message_catalog::{UiLocale, UiMessage};
use viewmodel::space_model_document::{ImportedToolpathState, MachiningOperationId};

/// 計算条件の問題を出す欄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConditionField {
    Target,
    Tool,
    Machining,
    Stock,
    Precision,
    Submission,
}

/// 問題 `issue` を出す欄
pub(crate) fn condition_field(issue: &ConditionIssue) -> ConditionField {
    match issue {
        ConditionIssue::NoProductShape => ConditionField::Target,
        ConditionIssue::Condition(error) => match error.field() {
            CamConditionField::Tool => ConditionField::Tool,
            CamConditionField::Machining => ConditionField::Machining,
            CamConditionField::Stock => ConditionField::Stock,
            CamConditionField::Precision => ConditionField::Precision,
            CamConditionField::Other => ConditionField::Submission,
        },
        ConditionIssue::Other(_) => ConditionField::Submission,
    }
}

/// 切削シミュレーションの最後の操作（投入・実行・取り込み）の結果
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum OperationStatus {
    /// 投入した（今の条件の CL から、切削シミュレーションだけを投入したか）
    Submitted { simulation_only: bool },
    /// 空間モデルの CL を消して、ツールパス生成から投入し直した（今の条件の計算待ち・計算中のジョブを取り消すよう
    /// 頼んだか）
    Redone { canceled: bool },
    /// 今の条件の計算が計算待ち・計算中・計算済みのため、投入しない
    AlreadyCalculated,
    /// 計算の状態は分からないが、今の条件の CL を取り込み済みのため、投入しない
    AlreadyImported,
    /// 今の条件の計算が計算待ちのため、再計算しない（計算が始まってから再計算できる）
    RecalculationWaiting,
    /// 今の条件の計算待ち・計算中の計算を止めている（取り消すよう頼んだ。止まるのは待たない）
    Stopped,
    /// 今の条件の計算に、待ってもジョブマネージャーが受け取らない投入があり、すべては止めていない
    StopNotYetAccepted,
    /// 今の条件の計算は、計算待ち・計算中でないため、止めていない
    NothingToStop,
    /// 計算を止められない（画面に出す理由）
    StopFailed(UiMessage),
    /// 投入できない（画面に出す理由）
    SubmitFailed(UiMessage),
    /// 手順の全体を投入した（投入した工程・そのうち切削シミュレーションだけを投入した工程・投入し直さなかった工程の数）
    SequenceSubmitted {
        submitted: usize,
        simulation_only: usize,
        skipped: usize,
    },
    /// 手順の全体のうち、工程を投入できない（工程の加工の順の番号・名前、その前に投入した工程の数、画面に出す理由）
    SequenceSubmitFailed {
        position: usize,
        operation: String,
        submitted: usize,
        reason: UiMessage,
    },
    /// 計算の状態を読めないため、手順の全体を投入しない（画面に出す理由）
    SequenceStatusUnreadable(UiMessage),
    /// 取り込んだ切削シミュレーションのジョブの番号
    Imported { job: u64 },
    /// 取り込める（正常終了した）切削シミュレーションがない
    NothingToImport,
    /// CL を取り込んだツールパス生成のジョブの番号
    ToolpathImported { job: u64 },
    /// 取り込める（正常終了した）ツールパス生成がない
    NoToolpathToImport,
    /// 取り込めない（画面に出す理由）
    ImportFailed(UiMessage),
    /// ジョブマネージャーを開けない（画面に出す理由）
    JobManagerFailed(UiMessage),
}

/// 計算条件の問題 `issue` の、言語 `locale` の文言
pub(crate) fn condition_issue_text(locale: UiLocale, issue: &ConditionIssue) -> String {
    match issue {
        ConditionIssue::NoProductShape => {
            resolve_machining_conditions_text(locale, "conditions.issue.no_product_shape", &[])
        }
        ConditionIssue::Condition(error) => {
            let (key, args) = condition_error_message(error);
            resolve_machining_conditions_text(locale, key, &args)
        }
        ConditionIssue::Other(message) => resolve_machining_conditions_text(
            locale,
            "conditions.issue.other",
            &[("reason", resolve_failure_message(locale, message))],
        ),
    }
}

/// 選んだ工程の今の条件の計算の状態 `status` の、言語 `locale` の文言
pub(crate) fn job_status_text(locale: UiLocale, status: &JobStatusLine) -> String {
    resolve_machining_conditions_text(locale, job_status_key(status), &[])
}

/// 空間モデルの CL の状態 `state` の、言語 `locale` の文言
pub(crate) fn imported_toolpath_text(locale: UiLocale, state: ImportedToolpathState) -> String {
    resolve_machining_conditions_text(locale, imported_toolpath_key(state), &[])
}

/// 結果 `status` の、言語 `locale` の文言
pub(crate) fn operation_status_text(locale: UiLocale, status: &OperationStatus) -> String {
    let (key, args) = operation_status_message(locale, status);
    resolve_machining_conditions_text(locale, key, &args)
}

/// 結果 `status` を出す文言のキーと引数（失敗の理由は、言語 `locale` の文言にして渡す）
fn operation_status_message(
    locale: UiLocale,
    status: &OperationStatus,
) -> (&'static str, Vec<(&'static str, String)>) {
    let reason = |message: &UiMessage| vec![("reason", resolve_failure_message(locale, message))];
    match status {
        OperationStatus::Submitted {
            simulation_only: false,
        } => ("conditions.status.submitted", vec![]),
        OperationStatus::Submitted {
            simulation_only: true,
        } => ("conditions.status.submitted_simulation", vec![]),
        OperationStatus::Redone { canceled: false } => ("conditions.status.redone", vec![]),
        OperationStatus::Redone { canceled: true } => ("conditions.status.redone_canceled", vec![]),
        OperationStatus::SubmitFailed(message) => {
            ("conditions.status.submit_failed", reason(message))
        }
        OperationStatus::SequenceSubmitted { submitted: 0, .. } => {
            ("conditions.status.sequence_nothing_to_submit", vec![])
        }
        OperationStatus::SequenceSubmitted {
            submitted,
            simulation_only: 0,
            skipped: 0,
        } => (
            "conditions.status.sequence_submitted_all",
            vec![("submitted", submitted.to_string())],
        ),
        OperationStatus::SequenceSubmitted {
            submitted,
            simulation_only: 0,
            skipped,
        } => (
            "conditions.status.sequence_submitted",
            vec![
                ("submitted", submitted.to_string()),
                ("skipped", skipped.to_string()),
            ],
        ),
        OperationStatus::SequenceSubmitted {
            submitted,
            simulation_only,
            skipped: 0,
        } => (
            "conditions.status.sequence_submitted_simulation_all",
            vec![
                ("submitted", submitted.to_string()),
                ("simulation_only", simulation_only.to_string()),
            ],
        ),
        OperationStatus::SequenceSubmitted {
            submitted,
            simulation_only,
            skipped,
        } => (
            "conditions.status.sequence_submitted_simulation",
            vec![
                ("submitted", submitted.to_string()),
                ("simulation_only", simulation_only.to_string()),
                ("skipped", skipped.to_string()),
            ],
        ),
        OperationStatus::SequenceSubmitFailed {
            position,
            operation,
            submitted,
            reason: message,
        } => {
            let mut args = reason(message);
            args.push(("position", position.to_string()));
            args.push(("operation", operation.clone()));
            args.push(("submitted", submitted.to_string()));
            ("conditions.status.sequence_submit_failed", args)
        }
        OperationStatus::SequenceStatusUnreadable(message) => (
            "conditions.status.sequence_status_unreadable",
            reason(message),
        ),
        OperationStatus::Imported { job } => {
            ("conditions.status.imported", vec![("job", job.to_string())])
        }
        OperationStatus::NothingToImport => ("conditions.status.nothing_to_import", vec![]),
        OperationStatus::AlreadyCalculated => ("conditions.status.already_calculated", vec![]),
        OperationStatus::AlreadyImported => ("conditions.status.already_imported", vec![]),
        OperationStatus::RecalculationWaiting => {
            ("conditions.status.recalculation_waiting", vec![])
        }
        OperationStatus::Stopped => ("conditions.status.stopped", vec![]),
        OperationStatus::StopNotYetAccepted => ("conditions.status.stop_not_accepted", vec![]),
        OperationStatus::NothingToStop => ("conditions.status.nothing_to_stop", vec![]),
        OperationStatus::StopFailed(message) => ("conditions.status.stop_failed", reason(message)),
        OperationStatus::ToolpathImported { job } => (
            "conditions.status.toolpath_imported",
            vec![("job", job.to_string())],
        ),
        OperationStatus::NoToolpathToImport => ("conditions.status.no_toolpath_to_import", vec![]),
        OperationStatus::ImportFailed(message) => {
            ("conditions.status.import_failed", reason(message))
        }
        OperationStatus::JobManagerFailed(message) => {
            ("conditions.status.job_manager_failed", reason(message))
        }
    }
}

/// 工程リストの初めの高さ（論理座標）
const LIST_DEFAULT_HEIGHT: f32 = 220.0;
/// 条件の編集のダイアログの幅（論理座標）
const DIALOG_WIDTH: f32 = 380.0;
/// 条件の編集のダイアログを置く、範囲の左上からの間（論理座標）
const DIALOG_MARGIN: f32 = 16.0;
/// 条件の編集のダイアログの、スクロールする中身のほかの高さ（題・頁の切り替え・区切り・ボタンの行。論理座標）
const DIALOG_CHROME_HEIGHT: f32 = 140.0;
/// 条件の編集のダイアログの、スクロールする中身の最小の高さ（論理座標）
const DIALOG_BODY_MIN_HEIGHT: f32 = 120.0;

/// 工程の表の選んだ行の、選択の色の濃さ（1 は選択の色のまま）
const SELECTED_ROW_OPACITY: f32 = 0.45;

/// 工程の表の列の見出しの文言のキー
const COLUMN_KEYS: [&str; 6] = [
    "conditions.list.column.order",
    "conditions.list.column.name",
    "conditions.list.column.stage",
    "conditions.list.column.kind",
    "conditions.list.column.tool",
    "conditions.list.column.status",
];

/// 条件の編集のダイアログ（適用するまで、工程・手順の条件を変えない）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConditionDialog {
    /// 選んだ工程の設定
    Operation,
    /// 手順の設定（素材）
    Sequence,
}

/// 条件の編集のダイアログで選んだ操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DialogResult {
    /// 編集した条件を工程・手順に入れる
    Apply,
    /// 編集した条件を捨てる
    Cancel,
}

/// 範囲 `area` に置くダイアログの、スクロールする中身の最大の高さ（題・ボタンの行を範囲に収める）
fn dialog_body_max_height(area: Rect) -> f32 {
    (area.height() - DIALOG_CHROME_HEIGHT).max(DIALOG_BODY_MIN_HEIGHT)
}

/// 工程の設定のダイアログの頁
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OperationSettingsPage {
    /// 加工条件（名前・加工の段階・工具・加工の種類と条件）
    #[default]
    Machining,
    /// 加工領域・加工対象
    Region,
}

#[derive(Debug, Clone)]
pub struct MachiningConditionsUiState {
    /// 工程リストを出しているか
    pub(crate) list_shown: bool,
    /// 開いている条件の編集のダイアログ（`None` は開いていない）と、工程の設定の出している頁
    pub(crate) dialog: Option<ConditionDialog>,
    pub(crate) dialog_page: OperationSettingsPage,
    /// 開くように頼んだダイアログ（工程リストのボタン・行のダブルクリック・右クリックのメニュー）
    pub(crate) open_dialog_requested: Option<ConditionDialog>,
    /// ダイアログで選んだ操作
    pub(crate) dialog_result: Option<DialogResult>,
    /// 工程の設定を適用すると同時に、選んだ工程を投入する
    pub(crate) submit_on_apply: bool,
    pub locale: UiLocale,
    /// 加工手順の名前
    pub(crate) sequence_name: String,
    /// 加工手順の加工工程の一覧（加工の順）
    pub(crate) operations: Vec<OperationRow>,
    /// 選んだ加工工程
    pub(crate) selected_operation: MachiningOperationId,
    /// 加工工程の一覧の操作
    pub(crate) operation_command: Option<OperationCommand>,
    /// 表示中の製品形状の面の数
    pub face_count: usize,
    /// ダイアログで編集している、選んだ加工工程の中身（名前・工具・走査線の条件）と加工手順の素材（余裕・精度）。
    /// 適用するまで、工程・手順に入れない
    pub operation: MachiningOperationContent,
    pub stock: MachiningStock,
    /// 工具長を指定するときの入力値（指定しない間も保つ）
    pub tool_length_input: f64,
    /// ツールパス生成のトレランス（mm）
    pub toolpath_tolerance: f64,
    /// 今の（適用した）加工工程・加工手順の条件の問題（投入前の確認の結果）
    pub issues: Vec<ConditionIssue>,
    /// ダイアログで編集している条件の問題
    pub(crate) dialog_issues: Vec<ConditionIssue>,
    /// 表示中の結果のジョブの番号と、計算に使った条件
    pub imported: Option<(u64, ImportedConditions)>,
    pub submit_requested: bool,
    /// 選んだ工程の空間モデルの CL を消して、ツールパス生成から投入し直す
    pub(crate) redo_requested: bool,
    /// 選んだ工程の今の条件の、計算待ち・計算中の計算を止める
    pub(crate) stop_requested: bool,
    /// 手順の全体を投入する
    pub(crate) submit_sequence_requested: bool,
    /// 選んだ工程の、ツールパス生成の結果（CL）だけを取り込む
    pub(crate) import_toolpath_requested: bool,
    /// 選んだ工程の切削シミュレーションの表示を切り替える（出していれば閉じ、出していなければ取り込んで出す）
    pub(crate) toggle_simulation_requested: bool,
    /// 選んだ工程の切削アニメーションを出しているか（「切削シミュレーション表示」の選んだ印）
    pub(crate) simulation_shown: bool,
    pub open_job_manager_requested: bool,
}

/// 加工工程の一覧の 1 行
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OperationRow {
    pub(crate) id: MachiningOperationId,
    pub(crate) name: String,
    pub(crate) stage: MachiningStage,
    pub(crate) spec: OperationSpec,
    pub(crate) tool: Tool<f64>,
    pub(crate) status: JobStatusLine,
    /// 空間モデルの CL が、今の条件の結果（取込済み）
    pub(crate) toolpath_current: bool,
}

/// 押せないボタン `response` に乗せたときに、押せない理由 `reasons` を出す（理由がないとき、計算の状態のせいで
/// 押せないときは、空の吹き出しを出さないように付けない）
fn with_disabled_reasons(response: egui::Response, reasons: &str) -> egui::Response {
    if reasons.is_empty() {
        response
    } else {
        response.on_disabled_hover_text(reasons)
    }
}

/// 「適用と同時に計算する」で適用したときに、選んだ工程を計算するか
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ApplySubmission {
    /// 計算する
    Submit,
    /// 計算しない（画面に出す理由）
    Skip(OperationStatus),
}

/// 「適用と同時に計算する」で適用したときに、計算の条件が変わったか `conditions_changed`・最後に読んだ計算の状態
/// `status`・空間モデルの CL が今の条件の結果か `toolpath_current` から、選んだ工程を計算するか
///
/// 計算の条件が変わったときは、最後に読んだ状態は前の条件のものなので、比べずに計算する。変わらないときは、計算の
/// ボタンが押せる状態のときだけ計算する（取込済みの工程の属性だけを替えたときに、ジョブマネージャーを起動しない）。
pub(crate) fn submission_on_apply(
    conditions_changed: bool,
    status: &JobStatusLine,
    toolpath_current: bool,
) -> ApplySubmission {
    if conditions_changed || !calculation_blocked(status, toolpath_current) {
        return ApplySubmission::Submit;
    }
    match status {
        JobStatusLine::Status(_) => ApplySubmission::Skip(OperationStatus::AlreadyCalculated),
        // ジョブマネージャーが動いていない・見つからないため状態は分からないが、今の条件の CL がある
        JobStatusLine::ManagerNotRunning | JobStatusLine::ManagerNotFound => {
            ApplySubmission::Skip(OperationStatus::AlreadyImported)
        }
        // 今は計算のボタンを止めない状態（止めるようにするときは、計算しない理由の文言を足す）
        JobStatusLine::NotSubmitted | JobStatusLine::Unreadable | JobStatusLine::Reading => {
            ApplySubmission::Submit
        }
    }
}

/// 計算の状態 `status` と、空間モデルの CL が今の条件の結果か `toolpath_current` から、計算を押せないか（今の条件の
/// 計算が計算待ち・計算中・計算済みのとき。重ねて投入しないように。ジョブマネージャーが動いていない・見つからない
/// ときは、今の条件の CL があれば取込済みとして出すため、計算済みと同じに扱う。動けば状態を読み直し、未計算と分かれば
/// 切削シミュレーションだけを計算できる）
fn calculation_blocked(status: &JobStatusLine, toolpath_current: bool) -> bool {
    match status {
        JobStatusLine::Status(status) => status.is_waiting_or_done(),
        JobStatusLine::ManagerNotRunning | JobStatusLine::ManagerNotFound => toolpath_current,
        _ => false,
    }
}

/// 選んだ工程の、計算の状態 `status` と、空間モデルの CL が今の条件の結果か `toolpath_current` に応じた、次に
/// できることの案内（画面の下の行の右に出す。案内のない状態は `None`）
pub(crate) fn navigation_text(
    locale: UiLocale,
    status: &JobStatusLine,
    toolpath_current: bool,
) -> Option<String> {
    navigation_key(status, toolpath_current)
        .map(|key| resolve_machining_conditions_text(locale, key, &[]))
}

/// [`navigation_text`] の文言のキー
fn navigation_key(status: &JobStatusLine, toolpath_current: bool) -> Option<&'static str> {
    match status {
        JobStatusLine::Status(OperationJobStatus::Queued | OperationJobStatus::Running) => {
            Some("conditions.navigation.in_progress")
        }
        JobStatusLine::Status(OperationJobStatus::Succeeded) if toolpath_current => {
            Some("conditions.navigation.imported_with_simulation")
        }
        JobStatusLine::Status(OperationJobStatus::Succeeded) => {
            Some("conditions.navigation.calculated")
        }
        // ジョブマネージャーの結果を消した後・別の PC で開いたとき（CL から切削シミュレーションだけを計算する）
        JobStatusLine::NotSubmitted if toolpath_current => {
            Some("conditions.navigation.imported_toolpath_only")
        }
        // 取込済みと出し、計算を押せない（状態を読むだけでは、ジョブマネージャーを起動しない）
        JobStatusLine::ManagerNotRunning if toolpath_current => {
            Some("conditions.navigation.imported_manager_not_running")
        }
        _ => None,
    }
}

/// 加工工程の一覧の行に出す状態の文言のキー（計算済み・未計算（ジョブマネージャーの結果を消したとき）・状態不明
/// （ジョブマネージャーが動いていない・見つからない）の工程は、空間モデルの CL が今の条件の結果なら取込済み。CL が正本
/// のため、ジョブの側を見なくてよい。計算待ち・計算中・失敗などは、そのまま出す）
fn operation_row_key(row: &OperationRow) -> &'static str {
    let finished_or_gone = matches!(
        row.status,
        JobStatusLine::Status(OperationJobStatus::Succeeded)
            | JobStatusLine::NotSubmitted
            | JobStatusLine::ManagerNotRunning
            | JobStatusLine::ManagerNotFound
    );
    if row.toolpath_current && finished_or_gone {
        "conditions.operations.status.imported"
    } else {
        operation_row_status_key(&row.status)
    }
}

/// 加工工程の一覧の操作
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OperationCommand {
    /// 工程を選ぶ
    Select(MachiningOperationId),
    /// 選んだ工程のすぐ後に、新しい工程を加える
    Add,
    /// 選んだ工程を複写して、すぐ後に加える
    Duplicate,
    /// 選んだ工程を消す
    Remove,
    /// 選んだ工程を、加工の順で動かす（負は前へ）
    Move(isize),
}

/// 工程の表の状態の欄の、状態の文言のキー `key` ごとの色（ジョブマネージャーの一覧と同じ色。色を付けない状態は
/// `None`）
fn row_status_color(key: &str) -> Option<Color32> {
    match key {
        "conditions.operations.status.queued" => Some(Color32::GRAY),
        "conditions.operations.status.running" => Some(Color32::from_rgb(80, 160, 255)),
        "conditions.operations.status.succeeded" | "conditions.operations.status.imported" => {
            Some(Color32::from_rgb(80, 200, 120))
        }
        "conditions.operations.status.failed" | "conditions.operations.status.unreadable" => {
            Some(Color32::from_rgb(230, 90, 90))
        }
        "conditions.operations.status.canceled" => Some(Color32::from_rgb(200, 160, 60)),
        _ => None,
    }
}

/// 計算の状態 `status` と、空間モデルの CL が今の条件の結果か `toolpath_current` から決める、次に押す操作（目立たせる。
/// 未計算・計算失敗・取り消しは計算（取込済みで切削シミュレーションの結果がない工程も、切削シミュレーションだけを
/// 計算するため計算。画面の下の案内と同じ）、計算済みで取り込んでいなければ CL 取込。計算待ち・計算中・計算済みで
/// 取込済み、状態が分からない工程はない）
fn next_action_of(status: &JobStatusLine, toolpath_current: bool) -> Option<RowAction> {
    match status {
        JobStatusLine::NotSubmitted => Some(RowAction::Submit),
        JobStatusLine::Status(OperationJobStatus::Failed | OperationJobStatus::Canceled) => {
            Some(RowAction::Submit)
        }
        JobStatusLine::Status(OperationJobStatus::Succeeded) if !toolpath_current => {
            Some(RowAction::ImportToolpath)
        }
        _ => None,
    }
}

/// 欄の文字を、行の高さの中央にそろえて置き、欄の範囲を返す
fn centered_label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> Rect {
    ui.horizontal_centered(|ui| ui.label(text)).response.rect
}

/// 加工の段階 `stage` の文言のキー
fn stage_key(stage: MachiningStage) -> &'static str {
    match stage {
        MachiningStage::Rough => "conditions.stage.rough",
        MachiningStage::SemiFinish => "conditions.stage.semi_finish",
        MachiningStage::Finish => "conditions.stage.finish",
    }
}

/// 加工の種類 `spec` の文言のキー
fn spec_kind_key(spec: &OperationSpec) -> &'static str {
    match spec {
        OperationSpec::Scanline(_) => "conditions.kind.scanline",
    }
}

/// 工程の表の行の操作（右クリックのメニュー）
#[derive(Debug, Clone, PartialEq, Eq)]
enum RowAction {
    OpenSettings,
    Submit,
    Redo,
    Stop,
    ImportToolpath,
    Import,
    Command(OperationCommand),
}

/// 加工工程の一覧に出す、工程の計算の状態の短い文言のキー
fn operation_row_status_key(status: &JobStatusLine) -> &'static str {
    match status {
        JobStatusLine::NotSubmitted => "conditions.operations.status.none",
        JobStatusLine::Unreadable => "conditions.operations.status.unreadable",
        JobStatusLine::Reading => "conditions.operations.status.reading",
        JobStatusLine::ManagerNotRunning | JobStatusLine::ManagerNotFound => {
            "conditions.operations.status.unknown"
        }
        JobStatusLine::Status(OperationJobStatus::Queued) => "conditions.operations.status.queued",
        JobStatusLine::Status(OperationJobStatus::Running) => {
            "conditions.operations.status.running"
        }
        JobStatusLine::Status(OperationJobStatus::Succeeded) => {
            "conditions.operations.status.succeeded"
        }
        JobStatusLine::Status(OperationJobStatus::Failed) => "conditions.operations.status.failed",
        JobStatusLine::Status(OperationJobStatus::Canceled) => {
            "conditions.operations.status.canceled"
        }
    }
}

/// 工程リストに出す、今の条件の計算の状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JobStatusLine {
    /// 今の条件で投入していない
    NotSubmitted,
    Status(OperationJobStatus),
    /// ジョブマネージャーから状態を読めない
    Unreadable,
    /// 状態をまだ読めていない（読み終えるのを待たないため、初めて読む前）
    Reading,
    /// ジョブマネージャーが動いていないため、状態が分からない（読むときは起動しない）
    ManagerNotRunning,
    /// ジョブマネージャーが見つからないため、状態が分からない
    ManagerNotFound,
}

impl JobStatusLine {
    /// 投入し直すか条件を変えるまで変わらない状態か（ジョブマネージャーが動いていない・見つからない間は、外から
    /// 起動したときにすぐ気づけるよう、短い間隔で読み直す。つなげるかを確かめるだけで軽い）
    pub(crate) fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Status(
                OperationJobStatus::Succeeded
                    | OperationJobStatus::Failed
                    | OperationJobStatus::Canceled
            )
        )
    }
}

/// 空間モデルに取り込んだ CL の状態 `state` の文言のキー
fn imported_toolpath_key(state: ImportedToolpathState) -> &'static str {
    match state {
        ImportedToolpathState::None => "conditions.toolpath.none",
        ImportedToolpathState::Current => "conditions.toolpath.current",
        ImportedToolpathState::Outdated => "conditions.toolpath.outdated",
    }
}

/// 計算の状態 `status` の文言のキー
fn job_status_key(status: &JobStatusLine) -> &'static str {
    match status {
        JobStatusLine::NotSubmitted => "conditions.job.none",
        JobStatusLine::Unreadable => "conditions.job.unreadable",
        JobStatusLine::Reading => "conditions.job.reading",
        JobStatusLine::ManagerNotRunning => "conditions.job.manager_not_running",
        JobStatusLine::ManagerNotFound => "conditions.job.manager_not_found",
        JobStatusLine::Status(OperationJobStatus::Queued) => "conditions.job.queued",
        JobStatusLine::Status(OperationJobStatus::Running) => "conditions.job.running",
        JobStatusLine::Status(OperationJobStatus::Succeeded) => "conditions.job.succeeded",
        JobStatusLine::Status(OperationJobStatus::Failed) => "conditions.job.failed",
        JobStatusLine::Status(OperationJobStatus::Canceled) => "conditions.job.canceled",
    }
}

/// 送り方 `direction` の文言のキー
fn cut_direction_key(direction: ScanlineCutDirection) -> &'static str {
    match direction {
        ScanlineCutDirection::DownCut => "conditions.machining.cut_direction.down",
        ScanlineCutDirection::UpCut => "conditions.machining.cut_direction.up",
        ScanlineCutDirection::ZigZag => "conditions.machining.cut_direction.zigzag",
    }
}

/// 計算条件の誤り `error` を出す文言のキーと引数
fn condition_error_message(
    error: &CamConditionError,
) -> (&'static str, Vec<(&'static str, String)>) {
    match error {
        CamConditionError::InvalidStockMargins(_) => {
            ("conditions.issue.invalid_stock_margins", Vec::new())
        }
        CamConditionError::InvalidStock => ("conditions.issue.invalid_stock", Vec::new()),
        CamConditionError::InvalidTool => ("conditions.issue.invalid_tool", Vec::new()),
        CamConditionError::StepoverNotPositive => {
            ("conditions.issue.stepover_not_positive", Vec::new())
        }
        CamConditionError::StepoverExceedsToolDiameter { tool_diameter } => (
            "conditions.issue.stepover_exceeds_tool_diameter",
            vec![("diameter", tool_diameter.to_string())],
        ),
        CamConditionError::FeedRateNotPositive => {
            ("conditions.issue.feed_rate_not_positive", Vec::new())
        }
        CamConditionError::InvalidToolpathInput(reason) => (
            "conditions.issue.invalid_toolpath_input",
            vec![("reason", reason.clone())],
        ),
        CamConditionError::InvalidPrecision => ("conditions.issue.invalid_precision", Vec::new()),
        CamConditionError::PrecisionFinerThanTolerance { tolerance, .. } => (
            "conditions.issue.precision_finer_than_tolerance",
            vec![("tolerance", tolerance.to_string())],
        ),
        CamConditionError::TooManyDexelLines { finest_precision } => (
            "conditions.issue.too_many_dexel_lines",
            vec![("finest_precision", finest_precision.to_string())],
        ),
        CamConditionError::InvalidSnapshotInterval => (
            "conditions.issue.other",
            vec![("reason", error.to_string())],
        ),
    }
}

impl MachiningConditionsUiState {
    fn text(&self, key: &str) -> String {
        resolve_machining_conditions_text(self.locale, key, &[])
    }

    fn text_with_args(&self, key: &str, args: &[(&str, String)]) -> String {
        resolve_machining_conditions_text(self.locale, key, args)
    }

    fn issue_text(&self, issue: &ConditionIssue) -> String {
        condition_issue_text(self.locale, issue)
    }

    /// ダイアログで編集している条件の、欄 `field` の問題を出す
    fn show_issues(&self, ui: &mut egui::Ui, field: ConditionField) {
        for issue in self
            .dialog_issues
            .iter()
            .filter(|issue| condition_field(issue) == field)
        {
            ui.colored_label(ui.visuals().error_fg_color, self.issue_text(issue));
        }
    }

    /// 工具 `tool` の短い文言（種類と径）
    fn tool_text(&self, tool: &Tool<f64>) -> String {
        let kind = if tool.is_ball_end_mill() {
            self.text("conditions.tool.ball")
        } else {
            self.text("conditions.tool.flat")
        };
        self.text_with_args(
            "conditions.list.tool",
            &[("kind", kind), ("diameter", tool.diameter().to_string())],
        )
    }

    /// 工程リストを、画面の下のパネルに置く（出していなければ何もしない）
    pub fn show_list(&mut self, ui: &mut egui::Ui) {
        if !self.list_shown {
            return;
        }
        // 条件の編集のダイアログを開いている間は、押せない（選んだ工程が替わると、編集している工程と食い違うため）
        let enabled = self.dialog.is_none();
        Panel::bottom("operation_list")
            .resizable(true)
            .default_size(LIST_DEFAULT_HEIGHT)
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    self.show_list_toolbar(ui);
                    self.show_job_commands(ui);
                    ui.separator();
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.show_operation_table(ui));
                });
            });
    }

    /// 開いている条件の編集のダイアログを、範囲 `area`（メニューバーと工程リストの間）の左上に出す（タイトルの行で
    /// 動かせる）。中身は範囲に収まる高さでスクロールする
    pub fn show_windows(&mut self, ctx: &Context, area: Rect) {
        let Some(dialog) = self.dialog else {
            return;
        };
        let title = match dialog {
            ConditionDialog::Operation => self.text_with_args(
                "conditions.operation_settings.title",
                &[("name", self.operation.attributes.name.clone())],
            ),
            ConditionDialog::Sequence => self.text("conditions.sequence_settings.title"),
        };
        let mut open = true;
        Window::new(title)
            // 題に工程の名前を出すため、ID は題から決めない（工程・ダイアログを替えても、同じ場所に出す）
            .id(Id::unique("condition_dialog"))
            .default_pos(area.left_top() + egui::vec2(DIALOG_MARGIN, DIALOG_MARGIN))
            .constrain_to(area)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_width(DIALOG_WIDTH);
                match dialog {
                    ConditionDialog::Operation => self.show_operation_dialog(ui, area),
                    ConditionDialog::Sequence => self.show_sequence_dialog(ui, area),
                }
                ui.separator();
                ui.horizontal(|ui| {
                    // 適用と同時に投入するときは、誤りのある条件を適用しない（投入できないため）
                    let can_apply = dialog == ConditionDialog::Sequence
                        || !self.submit_on_apply
                        || self.dialog_issues.is_empty();
                    if ui
                        .add_enabled(
                            can_apply,
                            egui::Button::new(self.text("conditions.dialog.apply")),
                        )
                        .clicked()
                    {
                        self.dialog_result = Some(DialogResult::Apply);
                    }
                    if ui.button(self.text("conditions.dialog.cancel")).clicked() {
                        self.dialog_result = Some(DialogResult::Cancel);
                    }
                    if dialog == ConditionDialog::Operation {
                        let label = self.text("conditions.dialog.submit_on_apply");
                        ui.checkbox(&mut self.submit_on_apply, label);
                    }
                });
            });
        // 閉じるボタンは、適用しない（Esc は、キー操作の側でキャンセルにする）
        if !open && self.dialog_result.is_none() {
            self.dialog_result = Some(DialogResult::Cancel);
        }
    }

    /// 手順の名前と、設定のダイアログを開くボタンと、工程の一覧の操作（加える・複写・消す・並べ替え）
    fn show_list_toolbar(&mut self, ui: &mut egui::Ui) {
        let mut command = None;
        ui.horizontal_wrapped(|ui| {
            ui.strong(self.text_with_args(
                "conditions.list.sequence",
                &[("name", self.sequence_name.clone())],
            ));
            if ui
                .button(self.text("conditions.list.sequence_settings"))
                .clicked()
            {
                self.open_dialog_requested = Some(ConditionDialog::Sequence);
            }
            if ui
                .button(self.text("conditions.list.operation_settings"))
                .clicked()
            {
                self.open_dialog_requested = Some(ConditionDialog::Operation);
            }
            ui.separator();
            if ui.button(self.text("conditions.operations.add")).clicked() {
                command = Some(OperationCommand::Add);
            }
            if ui
                .button(self.text("conditions.operations.duplicate"))
                .clicked()
            {
                command = Some(OperationCommand::Duplicate);
            }
            if ui
                .add_enabled(
                    self.command_enabled(&OperationCommand::Remove, self.selected_position()),
                    egui::Button::new(self.text("conditions.operations.remove")),
                )
                .clicked()
            {
                command = Some(OperationCommand::Remove);
            }
            for (offset, label, key) in [
                (-1, "▲", "conditions.operations.move_up"),
                (1, "▼", "conditions.operations.move_down"),
            ] {
                let moved = OperationCommand::Move(offset);
                let response = ui
                    .add_enabled(
                        self.command_enabled(&moved, self.selected_position()),
                        egui::Button::new(label),
                    )
                    .on_hover_text(self.text(key));
                if response.clicked() {
                    command = Some(moved);
                }
            }
        });
        if command.is_some() {
            self.operation_command = command;
        }
    }

    /// 選んだ工程・手順の全体の計算の操作（投入・やり直し・計算停止・手順の全体を投入・CL 取込・切削シミュレーション表示・
    /// ジョブマネージャーを開く）。結果と状態は、行の数で工程リストの高さを変えないよう、ステータスバーに出す
    fn show_job_commands(&mut self, ui: &mut egui::Ui) {
        // 計算・再計算を押せない理由（計算の状態の案内・条件の誤り）は、押せないボタンに乗せたときに出す
        let reasons = self.issue_reasons();
        // 選んだ工程の状態で次に押すボタンは、枠を選択の色で太くして目立たせる（表示している間の「切削シミュレーション
        // 表示」は地の色が変わるため、見分けられる）。枠はボタンを置いた後に上から描く（ボタンの枠の太さを変えると、
        // ボタンが大きくなり、ボタンの行と工程の表が動くため）
        let next = self.next_action(self.selected_position());
        let emphasis = Stroke::new(2.0, ui.visuals().selection.stroke.color);
        let emphasize = |ui: &egui::Ui, response: &egui::Response, action: &RowAction| {
            if next.as_ref() == Some(action) {
                ui.painter().rect_stroke(
                    response.rect,
                    ui.visuals().widgets.inactive.corner_radius,
                    emphasis,
                    egui::StrokeKind::Inside,
                );
            }
        };
        ui.horizontal_wrapped(|ui| {
            let position = self.selected_position();
            let can_submit = self.action_enabled(&RowAction::Submit, position);
            let submit = with_disabled_reasons(
                ui.add_enabled(
                    can_submit,
                    egui::Button::new(self.text("conditions.submit")),
                )
                .on_hover_text(self.text("conditions.submit.hint")),
                &reasons,
            );
            emphasize(ui, &submit, &RowAction::Submit);
            if submit.clicked() {
                self.submit_requested = true;
            }
            let redo = with_disabled_reasons(
                ui.add_enabled(
                    self.action_enabled(&RowAction::Redo, self.selected_position()),
                    egui::Button::new(self.text("conditions.redo")),
                )
                .on_hover_text(self.text("conditions.redo.hint")),
                &reasons,
            );
            if redo.clicked() {
                self.redo_requested = true;
            }
            let stop = ui
                .add_enabled(
                    self.action_enabled(&RowAction::Stop, self.selected_position()),
                    egui::Button::new(self.text("conditions.stop")),
                )
                .on_hover_text(self.text("conditions.stop.hint"));
            if stop.clicked() {
                self.stop_requested = true;
            }
            // 手順の全体は、すべての工程を確かめてから投入する（誤りは、工程の名前とともに出す）
            if ui.button(self.text("conditions.submit_sequence")).clicked() {
                self.submit_sequence_requested = true;
            }
            let import_toolpath = ui
                .add_enabled(
                    self.action_enabled(&RowAction::ImportToolpath, self.selected_position()),
                    egui::Button::new(self.text("conditions.import_toolpath")),
                )
                .on_hover_text(self.text("conditions.import_toolpath.hint"));
            emphasize(ui, &import_toolpath, &RowAction::ImportToolpath);
            if import_toolpath.clicked() {
                self.import_toolpath_requested = true;
            }
            let simulation = ui
                .add(
                    egui::Button::new(self.text("conditions.import"))
                        .selected(self.simulation_shown),
                )
                .on_hover_text(self.text("conditions.import.hint"));
            if simulation.clicked() {
                self.toggle_simulation_requested = true;
            }
            if ui
                .button(self.text("conditions.open_job_manager"))
                .clicked()
            {
                self.open_job_manager_requested = true;
            }
        });
    }

    /// 加工の順の `position` 番目の工程に対して行う操作 `action` を、今押せるか（ボタンと右クリックのメニューで同じ
    /// 決め方にする）
    fn action_enabled(&self, action: &RowAction, position: Option<usize>) -> bool {
        match action {
            RowAction::Submit => {
                self.issues.is_empty()
                    && position
                        .and_then(|at| self.operations.get(at))
                        .is_none_or(|row| !calculation_blocked(&row.status, row.toolpath_current))
            }
            // 計算待ちの計算は、まだ始まっていない（ジョブマネージャーがまだ受け取っていない、取り消せない投入も
            // 計算待ち）。計算が始まってから再計算できる
            RowAction::Redo => {
                self.issues.is_empty()
                    && position
                        .and_then(|at| self.operations.get(at))
                        .is_none_or(|row| {
                            row.status != JobStatusLine::Status(OperationJobStatus::Queued)
                        })
            }
            RowAction::Command(command) => self.command_enabled(command, position),
            // 止められるのは、今の条件の計算が計算待ち・計算中のときだけ
            RowAction::Stop => position
                .and_then(|at| self.operations.get(at))
                .is_some_and(|row| {
                    matches!(
                        row.status,
                        JobStatusLine::Status(
                            OperationJobStatus::Queued | OperationJobStatus::Running
                        )
                    )
                }),
            // 空間モデルの CL が今の条件の結果（取込済み）なら、取り込み直しても同じ CL のため押せない
            RowAction::ImportToolpath => position
                .and_then(|at| self.operations.get(at))
                .is_none_or(|row| !row.toolpath_current),
            RowAction::OpenSettings | RowAction::Import => true,
        }
    }

    /// 今の条件の誤りの文（1 つ 1 行）
    fn issue_reasons(&self) -> String {
        self.issues
            .iter()
            .map(|issue| self.issue_text(issue))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 加工の順の `position` 番目の工程の状態で、次に押す操作（押せないとき・ダイアログを開いていて工程リストを押せない
    /// ときは、目立たせない）
    fn next_action(&self, position: Option<usize>) -> Option<RowAction> {
        if self.dialog.is_some() {
            return None;
        }
        let row = position.and_then(|at| self.operations.get(at))?;
        next_action_of(&row.status, row.toolpath_current)
            .filter(|action| self.action_enabled(action, position))
    }

    /// 選んだ工程の、加工の順の位置
    fn selected_position(&self) -> Option<usize> {
        self.operations
            .iter()
            .position(|row| row.id == self.selected_operation)
    }

    /// 加工の順の `position` 番目の工程に対して行う一覧の操作 `command` を、今押せるか（消すのは工程が 2 つ以上、
    /// 動かすのは端を越えないとき）
    fn command_enabled(&self, command: &OperationCommand, position: Option<usize>) -> bool {
        let count = self.operations.len();
        match command {
            OperationCommand::Remove => count > 1,
            OperationCommand::Move(offset) => position
                .and_then(|at| at.checked_add_signed(*offset))
                .is_some_and(|to| to < count),
            OperationCommand::Select(_) | OperationCommand::Add | OperationCommand::Duplicate => {
                true
            }
        }
    }

    /// 工程の表（1 行が 1 工程。上から加工の順。ジョブマネージャーの一覧と同じ見た目）。行のどこを押しても選び、
    /// ダブルクリックで工程の設定のダイアログを開き、右クリックで行の操作のメニューを出す。選んだ行・マウスが乗った
    /// 行は、行の全体に色を付ける
    fn show_operation_table(&mut self, ui: &mut egui::Ui) {
        let mut select = None;
        let mut action = None;
        // 乗った行は、前の描画のもの（行を描き終えるまで分からないため）。見出しの行が 0 のため、工程の行は 1 から
        let hovered_id = ui.scope_id().with("operation_table_hovered");
        let hovered_row = ui.data(|data| data.get_temp::<Option<usize>>(hovered_id).flatten());
        let selected_row = self.selected_position().map(|at| at + 1);
        let mut hovered_now = None;
        let row_spacing = ui.spacing().item_spacing.y;
        egui::Grid::new("operation_table")
            .num_columns(COLUMN_KEYS.len())
            .with_row_color(move |row, style| {
                if Some(row) == selected_row {
                    // 選んだ行は、選択の色を薄めて、文字との差を抑える
                    Some(
                        style
                            .visuals
                            .selection
                            .bg_fill
                            .gamma_multiply(SELECTED_ROW_OPACITY),
                    )
                } else if row > 0 && Some(row) == hovered_row {
                    Some(style.visuals.widgets.hovered.weak_bg_fill)
                } else if row % 2 == 1 {
                    Some(style.visuals.faint_bg_color)
                } else {
                    None
                }
            })
            .show(ui, |ui| {
                for key in COLUMN_KEYS {
                    ui.label(RichText::new(self.text(key)).strong());
                }
                ui.end_row();
                for (index, row) in self.operations.iter().enumerate() {
                    let selected = row.id == self.selected_operation;
                    let texts = [
                        (index + 1).to_string(),
                        row.name.clone(),
                        self.text(stage_key(row.stage)),
                        self.text(spec_kind_key(&row.spec)),
                        self.tool_text(&row.tool),
                    ];
                    // 欄はすべて、行の高さの中央にそろえる。文字は押せない（画面の設定）ため、行のどこを押しても行を選べる
                    let mut area = Rect::NOTHING;
                    for text in texts {
                        area = area.union(centered_label(ui, text));
                    }
                    let status_key = operation_row_key(row);
                    let status = ui.horizontal_centered(|ui| {
                        let text = RichText::new(self.text(status_key));
                        ui.label(match row_status_color(status_key) {
                            Some(color) => text.color(color),
                            None => text,
                        });
                        // 進み具合の割合はまだないため、計算中は動いていることだけを出す
                        if row.status == JobStatusLine::Status(OperationJobStatus::Running) {
                            // 印は文字の高さにする（計算が始まる・終わるたびに、行の高さが変わらないように）
                            let size = ui.text_style_height(&egui::TextStyle::Body);
                            ui.add(egui::Spinner::new().size(size));
                        }
                    });
                    // 行のすべての欄と、行の間の半分を、押せる範囲にする
                    let area = area
                        .union(status.response.rect)
                        .expand2(egui::vec2(0.0, row_spacing / 2.0));
                    let response = ui.interact(
                        area,
                        ui.scope_id().with(("operation_row", row.id)),
                        egui::Sense::click(),
                    );
                    // 右クリックも、その行を選んでからメニューを出す（メニューの操作は、選んだ工程に対して行う）
                    if !selected && (response.clicked() || response.secondary_clicked()) {
                        select = Some(row.id);
                    }
                    if response.double_clicked() {
                        action = Some(RowAction::OpenSettings);
                    }
                    if response.hovered() {
                        hovered_now = Some(index + 1);
                    }
                    response.context_menu(|ui| {
                        if let Some(chosen) = self.row_menu(ui, index) {
                            action = Some(chosen);
                            ui.close();
                        }
                    });
                    ui.end_row();
                }
            });
        if hovered_now != hovered_row {
            ui.data_mut(|data| data.insert_temp(hovered_id, hovered_now));
            // 乗った行の色を、すぐに描き直す
            ui.ctx().request_repaint();
        }
        if let Some(id) = select {
            self.operation_command = Some(OperationCommand::Select(id));
        }
        match action {
            Some(RowAction::OpenSettings) => {
                self.open_dialog_requested = Some(ConditionDialog::Operation);
            }
            Some(RowAction::Submit) => self.submit_requested = true,
            Some(RowAction::Redo) => self.redo_requested = true,
            Some(RowAction::Stop) => self.stop_requested = true,
            Some(RowAction::ImportToolpath) => self.import_toolpath_requested = true,
            Some(RowAction::Import) => self.toggle_simulation_requested = true,
            Some(RowAction::Command(command)) => self.operation_command = Some(command),
            None => {}
        }
    }

    /// 加工の順の `index` 番目の行の右クリックのメニュー（その行を選んでから行う）。選んだ操作を返す。押せるかは、
    /// 右クリックした行で決める（行を選ぶ操作を受け取る前のフレームでも、その行の状態で出す）
    fn row_menu(&self, ui: &mut egui::Ui, index: usize) -> Option<RowAction> {
        let items = [
            (
                RowAction::OpenSettings,
                "conditions.list.operation_settings",
            ),
            (RowAction::Submit, "conditions.submit"),
            (RowAction::Redo, "conditions.redo"),
            (RowAction::Stop, "conditions.stop"),
            (RowAction::ImportToolpath, "conditions.import_toolpath"),
            (RowAction::Import, "conditions.import"),
            (
                RowAction::Command(OperationCommand::Add),
                "conditions.operations.add",
            ),
            (
                RowAction::Command(OperationCommand::Duplicate),
                "conditions.operations.duplicate",
            ),
            (
                RowAction::Command(OperationCommand::Remove),
                "conditions.operations.remove",
            ),
            (
                RowAction::Command(OperationCommand::Move(-1)),
                "conditions.operations.move_up",
            ),
            (
                RowAction::Command(OperationCommand::Move(1)),
                "conditions.operations.move_down",
            ),
        ];
        let mut chosen = None;
        for (action, key) in items {
            let enabled = self.action_enabled(&action, Some(index));
            // 工程の設定・計算・一覧の操作の間に区切りを入れる
            if matches!(
                action,
                RowAction::Submit | RowAction::Command(OperationCommand::Add)
            ) {
                ui.separator();
            }
            if ui
                .add_enabled(enabled, egui::Button::new(self.text(key)))
                .clicked()
            {
                chosen = Some(action);
            }
        }
        chosen
    }

    /// 工程の設定のダイアログの中身（頁: 加工条件・加工領域と加工対象）
    fn show_operation_dialog(&mut self, ui: &mut egui::Ui, area: Rect) {
        let machining_label = self.text("conditions.page.machining");
        let region_label = self.text("conditions.page.region");
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.dialog_page,
                OperationSettingsPage::Machining,
                machining_label,
            );
            ui.selectable_value(
                &mut self.dialog_page,
                OperationSettingsPage::Region,
                region_label,
            );
        });
        ui.separator();
        // すべての分類を開いても下まで見られるよう、中身は範囲 `area` に収まる高さでスクロールする
        ScrollArea::vertical()
            .max_height(dialog_body_max_height(area))
            .show(ui, |ui| {
                match self.dialog_page {
                    OperationSettingsPage::Machining => {
                        self.show_operation_common(ui);
                        self.show_tool(ui);
                        self.show_machining(ui);
                    }
                    OperationSettingsPage::Region => {
                        self.show_target(ui);
                        self.show_region(ui);
                    }
                }
                self.show_imported(ui);
            });
    }

    /// 手順の設定のダイアログの中身（手順の名前と素材）
    fn show_sequence_dialog(&mut self, ui: &mut egui::Ui, area: Rect) {
        ui.label(self.text_with_args(
            "conditions.sequence.name",
            &[("name", self.sequence_name.clone())],
        ));
        ui.label(self.text("conditions.sequence.note"));
        ui.separator();
        ScrollArea::vertical()
            .max_height(dialog_body_max_height(area))
            .show(ui, |ui| self.show_stock(ui));
    }

    /// 工程の名前・加工の段階・加工の種類
    fn show_operation_common(&mut self, ui: &mut egui::Ui) {
        let stage_labels = [
            MachiningStage::Rough,
            MachiningStage::SemiFinish,
            MachiningStage::Finish,
        ]
        .map(|stage| (stage, self.text(stage_key(stage))));
        ui.horizontal(|ui| {
            ui.label(self.text("conditions.operations.name"));
            ui.text_edit_singleline(&mut self.operation.attributes.name);
        });
        ui.horizontal(|ui| {
            ui.label(self.text("conditions.operation.stage"));
            for (stage, label) in stage_labels {
                ui.radio_value(&mut self.operation.attributes.stage, stage, label);
            }
        });
        ui.label(self.text_with_args(
            "conditions.operation.kind",
            &[(
                "kind",
                self.text(spec_kind_key(&self.operation.conditions.spec)),
            )],
        ));
    }

    fn show_target(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new(self.text("conditions.header.target"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(self.text_with_args(
                    "conditions.target.faces",
                    &[("count", self.face_count.to_string())],
                ));
                ui.label(self.text("conditions.target.shared"));
                self.show_issues(ui, ConditionField::Target);
            });
    }

    /// 加工領域（今は出すだけ。指定は後で足す）
    fn show_region(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new(self.text("conditions.header.region"))
            .default_open(true)
            .show(ui, |ui| {
                let text = match &self.operation.conditions.boundary {
                    None => self.text("conditions.region.target"),
                    Some(MachiningBoundary::Rectangle(rectangle)) => self.text_with_args(
                        "conditions.region.rectangle",
                        &[
                            ("min_x", rectangle.rect_min[0].to_string()),
                            ("max_x", rectangle.rect_max[0].to_string()),
                            ("min_y", rectangle.rect_min[1].to_string()),
                            ("max_y", rectangle.rect_max[1].to_string()),
                        ],
                    ),
                };
                ui.label(text);
            });
    }

    fn show_tool(&mut self, ui: &mut egui::Ui) {
        let ball_label = self.text("conditions.tool.ball");
        let flat_label = self.text("conditions.tool.flat");
        let diameter_label = self.text("conditions.tool.diameter");
        let specify_label = self.text("conditions.tool.specify_length");
        CollapsingHeader::new(self.text("conditions.header.tool"))
            .default_open(true)
            .show(ui, |ui| {
                let tool = &self.operation.conditions.tool;
                let shown_kind = CuttingToolKind::of(tool).unwrap_or_default();
                let shown_diameter = tool.diameter();
                let shown_specified = tool.cutting_length.is_some();
                let shown_length = self.tool_length_input;
                let mut kind = shown_kind;
                let mut diameter = shown_diameter;
                ui.horizontal(|ui| {
                    ui.radio_value(&mut kind, CuttingToolKind::BallEndMill, ball_label);
                    ui.radio_value(&mut kind, CuttingToolKind::FlatEndMill, flat_label);
                });
                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut diameter)
                            .range(0.1..=f64::MAX)
                            .speed(0.1)
                            .suffix(" mm"),
                    );
                    ui.label(diameter_label);
                });
                let mut specified = shown_specified;
                ui.horizontal(|ui| {
                    ui.checkbox(&mut specified, specify_label);
                    ui.add_enabled(
                        specified,
                        DragValue::new(&mut self.tool_length_input)
                            .range(0.1..=f64::MAX)
                            .speed(0.5)
                            .suffix(" mm"),
                    );
                });
                // 入力を変えたときだけ工具を作り直す（入力欄で表せない工具を、開いただけで書き換えない）
                let length_changed = specified && self.tool_length_input != shown_length;
                if kind != shown_kind
                    || diameter != shown_diameter
                    || specified != shown_specified
                    || length_changed
                {
                    self.operation.conditions.tool =
                        kind.tool(diameter, specified.then_some(self.tool_length_input));
                }
                self.show_issues(ui, ConditionField::Tool);
            });
    }

    fn show_machining(&mut self, ui: &mut egui::Ui) {
        let stepover_label = self.text("conditions.machining.stepover");
        let feed_rate_label = self.text("conditions.machining.feed_rate");
        let angle_label = self.text("conditions.machining.angle");
        let tool_range_label = self.text("conditions.machining.tool_range");
        let extend_label = self.text("conditions.machining.tool_range.extend");
        let geometry_label = self.text("conditions.machining.tool_range.geometry");
        let cut_direction_label = self.text("conditions.machining.cut_direction");
        let cut_direction_labels = [
            ScanlineCutDirection::DownCut,
            ScanlineCutDirection::UpCut,
            ScanlineCutDirection::ZigZag,
        ]
        .map(|direction| (direction, self.text(cut_direction_key(direction))));
        let tolerance_label = self.text_with_args(
            "conditions.machining.tolerance",
            &[("tolerance", self.toolpath_tolerance.to_string())],
        );
        let cusp_height_label =
            flat_surface_cusp_height(&self.operation.conditions).map(|height| {
                self.text_with_args(
                    "conditions.machining.cusp_height",
                    &[("height", format!("{height:.4}"))],
                )
            });
        CollapsingHeader::new(self.text("conditions.header.machining"))
            .default_open(true)
            .show(ui, |ui| {
                let OperationSpec::Scanline(scanline) = &mut self.operation.conditions.spec;
                for (value, label, suffix, speed) in [
                    (&mut scanline.stepover, stepover_label, " mm", 0.1),
                    (&mut scanline.feed_rate, feed_rate_label, " mm/min", 10.0),
                ] {
                    ui.horizontal(|ui| {
                        ui.add(
                            DragValue::new(value)
                                .range(0.001..=f64::MAX)
                                .speed(speed)
                                .suffix(suffix),
                        );
                        ui.label(label);
                    });
                }
                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut scanline.angle_degrees)
                            .range(-180.0..=180.0)
                            .speed(1.0)
                            .max_decimals(1)
                            .suffix("°"),
                    );
                    ui.label(angle_label);
                });
                ui.label(cut_direction_label);
                ui.horizontal(|ui| {
                    for (direction, label) in cut_direction_labels {
                        ui.radio_value(&mut scanline.cut_direction, direction, label);
                    }
                });
                ui.label(tool_range_label);
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut scanline.tool_range,
                        ScanlineToolRange::ExtendByToolRadius,
                        extend_label,
                    );
                    ui.radio_value(
                        &mut scanline.tool_range,
                        ScanlineToolRange::GeometryExtent,
                        geometry_label,
                    );
                });
                if let Some(label) = cusp_height_label {
                    ui.label(label);
                }
                ui.label(tolerance_label);
                self.show_issues(ui, ConditionField::Machining);
            });
    }

    fn show_stock(&mut self, ui: &mut egui::Ui) {
        let margins_label = self.text("conditions.stock.margins");
        let labels = [
            "conditions.stock.minus_x",
            "conditions.stock.plus_x",
            "conditions.stock.minus_y",
            "conditions.stock.plus_y",
            "conditions.stock.top",
            "conditions.stock.bottom",
        ]
        .map(|key| self.text(key));
        let precision_label = self.text("conditions.stock.precision");
        CollapsingHeader::new(self.text("conditions.header.stock"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(margins_label);
                let StockMargins {
                    minus_x,
                    plus_x,
                    minus_y,
                    plus_y,
                    bottom,
                    top,
                } = &mut self.stock.margins;
                for (label, margin) in labels
                    .into_iter()
                    .zip([minus_x, plus_x, minus_y, plus_y, top, bottom])
                {
                    ui.horizontal(|ui| {
                        ui.add(
                            DragValue::new(margin)
                                .range(0.0..=f64::MAX)
                                .speed(0.1)
                                .suffix(" mm"),
                        );
                        ui.label(label);
                    });
                }
                self.show_issues(ui, ConditionField::Stock);
                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut self.stock.precision)
                            .range(0.0001..=f64::MAX)
                            .speed(0.001)
                            .max_decimals(4)
                            .suffix(" mm"),
                    );
                    ui.label(precision_label);
                });
                self.show_issues(ui, ConditionField::Precision);
            });
    }

    /// 表示中の結果の計算に使った条件（選んだ工程の切削アニメーションを出しているときだけ）
    fn show_imported(&mut self, ui: &mut egui::Ui) {
        let Some((job, conditions)) = &self.imported else {
            return;
        };
        ui.separator();
        CollapsingHeader::new(self.text("conditions.header.imported"))
            .default_open(true)
            .show(ui, |ui| {
                let tool = &conditions.tool;
                let kind = if tool.is_ball_end_mill() {
                    self.text("conditions.tool.ball")
                } else {
                    self.text("conditions.tool.flat")
                };
                let length = tool.cutting_length.map_or_else(
                    || self.text("conditions.imported.length_unlimited"),
                    |length| format!("{length} mm"),
                );
                let stock = &conditions.stock;
                let lines = [
                    self.text_with_args("conditions.imported.job", &[("job", job.to_string())]),
                    self.text_with_args(
                        "conditions.imported.tool",
                        &[
                            ("kind", kind),
                            ("diameter", tool.diameter().to_string()),
                            ("length", length),
                        ],
                    ),
                    self.text_with_args(
                        "conditions.imported.scanline",
                        &[
                            ("stepover", conditions.scanline.stepover.to_string()),
                            ("feed_rate", conditions.scanline.feed_rate.to_string()),
                            ("angle", conditions.scanline.angle_degrees.to_string()),
                            (
                                "cut_direction",
                                self.text(cut_direction_key(conditions.scanline.cut_direction)),
                            ),
                        ],
                    ),
                    self.text_with_args(
                        "conditions.imported.tolerance",
                        &[("tolerance", conditions.toolpath_tolerance.to_string())],
                    ),
                    self.text_with_args(
                        "conditions.imported.stock",
                        &[
                            ("x", format!("{:.3}", stock.width())),
                            ("y", format!("{:.3}", stock.height())),
                            ("z", format!("{:.3}", stock.depth())),
                        ],
                    ),
                    self.text_with_args(
                        "conditions.imported.precision",
                        &[("precision", conditions.precision.to_string())],
                    ),
                ];
                for line in lines {
                    ui.label(line);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        calculation_blocked, condition_error_message, condition_field, imported_toolpath_key,
        job_status_key, navigation_key, navigation_text, next_action_of, operation_row_key,
        operation_row_status_key, operation_status_text, row_status_color, stage_key,
        submission_on_apply, ApplySubmission, ConditionField, JobStatusLine, MachiningStage,
        OperationJobStatus, OperationRow, OperationStatus, RowAction, COLUMN_KEYS,
    };
    use crate::machining_conditions_messages::resolve_machining_conditions_text;
    use viewmodel::cutting_simulation_session::{CamConditionError, ConditionIssue, StockMargins};
    use viewmodel::message_catalog::{UiLocale, UiMessage};

    fn failure(key: &str) -> UiMessage {
        UiMessage {
            key: key.to_string(),
            args: Vec::new(),
        }
    }

    #[test]
    fn failure_reasons_follow_the_locale() {
        let status = OperationStatus::SubmitFailed(failure("failure.orchestration"));
        assert_eq!(
            operation_status_text(UiLocale::Ja, &status),
            "計算を始められません（計算のジョブを扱えません）"
        );
        assert_eq!(
            operation_status_text(UiLocale::En, &status),
            "Cannot start the calculation (Cannot handle the calculation job)"
        );
    }

    #[test]
    fn every_operation_status_has_a_message() {
        for status in [
            OperationStatus::Submitted {
                simulation_only: false,
            },
            OperationStatus::Submitted {
                simulation_only: true,
            },
            OperationStatus::Redone { canceled: false },
            OperationStatus::Redone { canceled: true },
            OperationStatus::SubmitFailed(failure("failure.orchestration")),
            OperationStatus::SequenceSubmitted {
                submitted: 2,
                simulation_only: 0,
                skipped: 1,
            },
            OperationStatus::SequenceSubmitted {
                submitted: 0,
                simulation_only: 0,
                skipped: 2,
            },
            OperationStatus::SequenceSubmitted {
                submitted: 2,
                simulation_only: 0,
                skipped: 0,
            },
            OperationStatus::SequenceSubmitted {
                submitted: 2,
                simulation_only: 1,
                skipped: 0,
            },
            OperationStatus::SequenceSubmitted {
                submitted: 2,
                simulation_only: 1,
                skipped: 1,
            },
            OperationStatus::SequenceSubmitFailed {
                position: 2,
                operation: "仕上げ".to_string(),
                submitted: 1,
                reason: failure("failure.conditions"),
            },
            OperationStatus::SequenceStatusUnreadable(failure("failure.job_manager_not_found")),
            OperationStatus::Imported { job: 2 },
            OperationStatus::NothingToImport,
            OperationStatus::AlreadyCalculated,
            OperationStatus::AlreadyImported,
            OperationStatus::RecalculationWaiting,
            OperationStatus::Stopped,
            OperationStatus::StopNotYetAccepted,
            OperationStatus::NothingToStop,
            OperationStatus::StopFailed(failure("failure.job_manager_not_found")),
            OperationStatus::ToolpathImported { job: 1 },
            OperationStatus::NoToolpathToImport,
            OperationStatus::ImportFailed(failure("failure.animation.replay")),
            OperationStatus::JobManagerFailed(failure("failure.job_manager_not_found")),
        ] {
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text = operation_status_text(locale, &status);
                assert!(
                    !text.contains("__missing__") && !text.contains('{'),
                    "{status:?}: {text}"
                );
            }
        }
    }

    #[test]
    fn every_list_and_settings_text_exists_in_both_languages() {
        let keys = COLUMN_KEYS
            .into_iter()
            .chain(
                [
                    MachiningStage::Rough,
                    MachiningStage::SemiFinish,
                    MachiningStage::Finish,
                ]
                .map(stage_key),
            )
            .chain([
                "conditions.list.sequence",
                "conditions.list.sequence_settings",
                "conditions.list.operation_settings",
                "conditions.list.tool",
                "conditions.kind.scanline",
                "conditions.page.machining",
                "conditions.page.region",
                "conditions.operation_settings.title",
                "conditions.sequence_settings.title",
                "conditions.operation.stage",
                "conditions.operation.kind",
                "conditions.target.shared",
                "conditions.header.region",
                "conditions.region.target",
                "conditions.region.rectangle",
                "conditions.sequence.name",
                "conditions.sequence.note",
                "conditions.dialog.apply",
                "conditions.dialog.cancel",
                "conditions.dialog.submit_on_apply",
            ]);
        for key in keys {
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text = resolve_machining_conditions_text(locale, key, &[]);
                assert!(!text.contains("__missing__"), "{key} ({locale:?})");
            }
        }
    }

    #[test]
    fn rows_whose_cl_is_current_are_shown_as_imported_unless_pending() {
        let names = viewmodel::space_model_document::DefaultNames {
            sequence: "加工手順",
            operation: "スキャン加工",
        };
        let item = viewmodel::space_model_document::SpaceModelDocument::new(names)
            .operations()
            .remove(0);
        let row = |status, toolpath_current| OperationRow {
            id: item.id,
            name: item.name.clone(),
            stage: item.stage,
            spec: item.spec,
            tool: item.tool.clone(),
            status,
            toolpath_current,
        };
        let succeeded = JobStatusLine::Status(OperationJobStatus::Succeeded);
        let queued = JobStatusLine::Status(OperationJobStatus::Queued);
        assert_eq!(
            operation_row_key(&row(succeeded.clone(), true)),
            "conditions.operations.status.imported"
        );
        // ジョブマネージャーの結果を消しても、空間モデルの CL が今の条件の結果なら取込済み
        assert_eq!(
            operation_row_key(&row(JobStatusLine::NotSubmitted, true)),
            "conditions.operations.status.imported"
        );
        assert_eq!(
            operation_row_key(&row(succeeded, false)),
            "conditions.operations.status.succeeded"
        );
        // ジョブマネージャーが動いていない・見つからないときは、CL が今の条件の結果なら取込済み、そうでなければ状態不明
        for unavailable in [
            JobStatusLine::ManagerNotRunning,
            JobStatusLine::ManagerNotFound,
        ] {
            assert_eq!(
                operation_row_key(&row(unavailable.clone(), true)),
                "conditions.operations.status.imported"
            );
            assert_eq!(
                operation_row_key(&row(unavailable, false)),
                "conditions.operations.status.unknown"
            );
        }
        // 計算し直している間・失敗したときは、計算の状態を出す
        assert_eq!(
            operation_row_key(&row(queued, true)),
            "conditions.operations.status.queued"
        );
        assert_eq!(
            operation_row_key(&row(
                JobStatusLine::Status(OperationJobStatus::Failed),
                true
            )),
            "conditions.operations.status.failed"
        );
    }

    #[test]
    fn calculation_is_blocked_only_while_waiting_calculating_or_calculated() {
        use OperationJobStatus::{Canceled, Failed, Queued, Running, Succeeded};
        for status in [Queued, Running, Succeeded] {
            for toolpath_current in [false, true] {
                assert!(calculation_blocked(
                    &JobStatusLine::Status(status),
                    toolpath_current
                ));
            }
        }
        // ジョブマネージャーが動いていない・見つからないときは、取込済みと出す工程（今の条件の CL がある）だけ押せない
        for status in [
            JobStatusLine::ManagerNotRunning,
            JobStatusLine::ManagerNotFound,
        ] {
            assert!(calculation_blocked(&status, true), "{status:?}");
            assert!(!calculation_blocked(&status, false), "{status:?}");
        }
        // 計算失敗・取り消しは計算し直せる。未計算（CL だけの工程は切削シミュレーションだけ）と、状態が分からない・
        // まだ読めていないときは、押したときに読み直して決める
        for status in [
            JobStatusLine::Status(Failed),
            JobStatusLine::Status(Canceled),
            JobStatusLine::NotSubmitted,
            JobStatusLine::Unreadable,
            JobStatusLine::Reading,
        ] {
            for toolpath_current in [false, true] {
                assert!(
                    !calculation_blocked(&status, toolpath_current),
                    "{status:?}"
                );
            }
        }
    }

    #[test]
    fn applying_submits_unless_unchanged_conditions_are_already_calculated_or_imported() {
        use OperationJobStatus::{Canceled, Failed, Queued, Running, Succeeded};
        let statuses = [
            JobStatusLine::Status(Queued),
            JobStatusLine::Status(Running),
            JobStatusLine::Status(Succeeded),
            JobStatusLine::Status(Failed),
            JobStatusLine::Status(Canceled),
            JobStatusLine::NotSubmitted,
            JobStatusLine::ManagerNotRunning,
            JobStatusLine::ManagerNotFound,
            JobStatusLine::Unreadable,
            JobStatusLine::Reading,
        ];
        // 計算の条件が変わったときは、最後に読んだ状態によらず計算する
        for status in &statuses {
            for toolpath_current in [false, true] {
                assert_eq!(
                    submission_on_apply(true, status, toolpath_current),
                    ApplySubmission::Submit,
                    "{status:?}"
                );
            }
        }
        // 変わらないときは、計算待ち・計算中・計算済みなら計算しない
        for status in [Queued, Running, Succeeded] {
            for toolpath_current in [false, true] {
                assert_eq!(
                    submission_on_apply(false, &JobStatusLine::Status(status), toolpath_current),
                    ApplySubmission::Skip(OperationStatus::AlreadyCalculated)
                );
            }
        }
        // ジョブマネージャーが動いていない・見つからないときは、取込済みの工程を計算しない（起動しない）
        for status in [
            JobStatusLine::ManagerNotRunning,
            JobStatusLine::ManagerNotFound,
        ] {
            assert_eq!(
                submission_on_apply(false, &status, true),
                ApplySubmission::Skip(OperationStatus::AlreadyImported),
                "{status:?}"
            );
            assert_eq!(
                submission_on_apply(false, &status, false),
                ApplySubmission::Submit,
                "{status:?}"
            );
        }
        // 未計算・計算失敗・取り消しと、状態が分からない・まだ読めていないときは計算する（計算するときに読み直す）
        for status in [
            JobStatusLine::Status(Failed),
            JobStatusLine::Status(Canceled),
            JobStatusLine::NotSubmitted,
            JobStatusLine::Unreadable,
            JobStatusLine::Reading,
        ] {
            for toolpath_current in [false, true] {
                assert_eq!(
                    submission_on_apply(false, &status, toolpath_current),
                    ApplySubmission::Submit,
                    "{status:?}"
                );
            }
        }
    }

    #[test]
    fn the_next_button_follows_the_status_of_the_selected_operation() {
        use OperationJobStatus::{Canceled, Failed, Queued, Running, Succeeded};
        let cases = [
            (JobStatusLine::NotSubmitted, false, Some(RowAction::Submit)),
            (
                JobStatusLine::Status(Failed),
                false,
                Some(RowAction::Submit),
            ),
            (JobStatusLine::Status(Failed), true, Some(RowAction::Submit)),
            (
                JobStatusLine::Status(Canceled),
                false,
                Some(RowAction::Submit),
            ),
            (
                JobStatusLine::Status(Succeeded),
                false,
                Some(RowAction::ImportToolpath),
            ),
            // 計算待ち・計算中・計算済みで取込済み・状態が分からないときは、目立たせない
            (JobStatusLine::Status(Queued), false, None),
            (JobStatusLine::Status(Running), false, None),
            (JobStatusLine::Status(Succeeded), true, None),
            // 取込済みで切削シミュレーションの結果がない工程は、切削シミュレーションだけを計算する
            (JobStatusLine::NotSubmitted, true, Some(RowAction::Submit)),
            (JobStatusLine::ManagerNotRunning, false, None),
            (JobStatusLine::Reading, false, None),
        ];
        for (status, toolpath_current, action) in cases {
            assert_eq!(
                next_action_of(&status, toolpath_current),
                action,
                "{status:?} {toolpath_current}"
            );
        }
    }

    #[test]
    fn row_statuses_are_colored_like_the_job_manager() {
        for key in [
            "conditions.operations.status.queued",
            "conditions.operations.status.running",
            "conditions.operations.status.succeeded",
            "conditions.operations.status.imported",
            "conditions.operations.status.failed",
            "conditions.operations.status.canceled",
            "conditions.operations.status.unreadable",
        ] {
            assert!(row_status_color(key).is_some(), "{key}");
        }
        // 未計算・状態不明・読んでいる間は、色を付けない
        for key in [
            "conditions.operations.status.none",
            "conditions.operations.status.unknown",
            "conditions.operations.status.reading",
        ] {
            assert!(row_status_color(key).is_none(), "{key}");
        }
    }

    #[test]
    fn the_navigation_tells_what_can_be_done_next_for_the_selected_operation() {
        let succeeded = JobStatusLine::Status(OperationJobStatus::Succeeded);
        let cases = [
            (
                JobStatusLine::Status(OperationJobStatus::Running),
                false,
                Some("conditions.navigation.in_progress"),
            ),
            (
                succeeded.clone(),
                false,
                Some("conditions.navigation.calculated"),
            ),
            (
                succeeded,
                true,
                Some("conditions.navigation.imported_with_simulation"),
            ),
            (
                JobStatusLine::NotSubmitted,
                true,
                Some("conditions.navigation.imported_toolpath_only"),
            ),
            (JobStatusLine::NotSubmitted, false, None),
            (
                JobStatusLine::ManagerNotRunning,
                true,
                Some("conditions.navigation.imported_manager_not_running"),
            ),
            (JobStatusLine::ManagerNotRunning, false, None),
            (JobStatusLine::ManagerNotFound, true, None),
            (
                JobStatusLine::Status(OperationJobStatus::Failed),
                true,
                None,
            ),
        ];
        for (status, toolpath_current, key) in cases {
            assert_eq!(navigation_key(&status, toolpath_current), key, "{status:?}");
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text = navigation_text(locale, &status, toolpath_current);
                assert_eq!(text.is_some(), key.is_some());
                assert!(!text.unwrap_or_default().contains("__missing__"));
            }
        }
    }

    #[test]
    fn every_operation_row_status_has_a_message_in_both_languages() {
        let statuses = [
            JobStatusLine::NotSubmitted,
            JobStatusLine::Unreadable,
            JobStatusLine::Reading,
            JobStatusLine::Status(OperationJobStatus::Queued),
            JobStatusLine::Status(OperationJobStatus::Running),
            JobStatusLine::Status(OperationJobStatus::Succeeded),
            JobStatusLine::Status(OperationJobStatus::Failed),
            JobStatusLine::Status(OperationJobStatus::Canceled),
        ];
        for locale in [UiLocale::Ja, UiLocale::En] {
            for status in &statuses {
                let key = operation_row_status_key(status);
                assert_ne!(resolve_machining_conditions_text(locale, key, &[]), key);
            }
            let imported = "conditions.operations.status.imported";
            assert_ne!(
                resolve_machining_conditions_text(locale, imported, &[]),
                imported
            );
        }
    }

    #[test]
    fn every_job_status_has_a_message() {
        let statuses = [
            JobStatusLine::NotSubmitted,
            JobStatusLine::Unreadable,
            JobStatusLine::Reading,
            JobStatusLine::Status(OperationJobStatus::Queued),
            JobStatusLine::Status(OperationJobStatus::Running),
            JobStatusLine::Status(OperationJobStatus::Succeeded),
            JobStatusLine::Status(OperationJobStatus::Failed),
            JobStatusLine::Status(OperationJobStatus::Canceled),
        ];
        for status in statuses {
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text = resolve_machining_conditions_text(locale, job_status_key(&status), &[]);
                assert!(!text.contains("__missing__"), "{status:?}: {text}");
            }
        }
    }

    #[test]
    fn every_imported_toolpath_state_has_a_message() {
        use viewmodel::space_model_document::ImportedToolpathState;
        for state in [
            ImportedToolpathState::None,
            ImportedToolpathState::Current,
            ImportedToolpathState::Outdated,
        ] {
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text =
                    resolve_machining_conditions_text(locale, imported_toolpath_key(state), &[]);
                assert!(!text.contains("__missing__"), "{state:?}: {text}");
            }
        }
    }

    /// 計算条件の誤りの種類を 1 つずつ
    fn every_condition_error() -> Vec<CamConditionError> {
        vec![
            CamConditionError::InvalidStockMargins(StockMargins::default()),
            CamConditionError::InvalidStock,
            CamConditionError::InvalidTool,
            CamConditionError::StepoverNotPositive,
            CamConditionError::StepoverExceedsToolDiameter { tool_diameter: 6.0 },
            CamConditionError::FeedRateNotPositive,
            CamConditionError::InvalidToolpathInput("reason".to_string()),
            CamConditionError::InvalidPrecision,
            CamConditionError::PrecisionFinerThanTolerance {
                precision: 0.0005,
                tolerance: 0.001,
            },
            CamConditionError::TooManyDexelLines {
                finest_precision: 0.02,
            },
            CamConditionError::InvalidSnapshotInterval,
        ]
    }

    #[test]
    fn every_condition_error_has_a_message() {
        for error in every_condition_error() {
            let (key, args) = condition_error_message(&error);
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text = resolve_machining_conditions_text(locale, key, &args);
                assert!(
                    !text.contains("__missing__") && !text.contains('{'),
                    "{error:?}: {text}"
                );
            }
        }
    }

    #[test]
    fn issues_are_shown_next_to_the_field_to_fix() {
        let fields: Vec<ConditionField> = every_condition_error()
            .into_iter()
            .map(|error| condition_field(&ConditionIssue::Condition(error)))
            .collect();
        assert_eq!(
            fields,
            [
                ConditionField::Stock,
                ConditionField::Stock,
                ConditionField::Tool,
                ConditionField::Machining,
                ConditionField::Machining,
                ConditionField::Machining,
                ConditionField::Machining,
                ConditionField::Precision,
                ConditionField::Precision,
                ConditionField::Precision,
                ConditionField::Submission,
            ]
        );
        assert_eq!(
            condition_field(&ConditionIssue::NoProductShape),
            ConditionField::Target
        );
        assert_eq!(
            condition_field(&ConditionIssue::Other(failure("failure.orchestration"))),
            ConditionField::Submission
        );
    }
}
