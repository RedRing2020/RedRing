//! 加工条件のウィンドウ（切削シミュレーションの計算条件の入力と投入）
//!
//! 表示の設定（設定パネル）とは分けて、加工手順 1 つの加工工程の一覧（加える・複写する・消す・並べ替える・選ぶ）と、
//! 選んだ工程の条件（工程の中身と手順の素材）と、表示中の結果の計算に使った条件を扱う。

use crate::machining_conditions_messages::resolve_machining_conditions_text;
use crate::settings_panel_renderer::window_body_max_height;
use egui::{Align2, CollapsingHeader, Context, DragValue, Rect, ScrollArea, Window};
use viewmodel::cutting_simulation_session::{
    flat_surface_cusp_height, CamConditionError, CamConditionField, ConditionIssue,
    CuttingToolKind, ImportedConditions, MachiningOperationContent, MachiningStock,
    OperationJobStatus, OperationSpec, ScanlineCutDirection, ScanlineToolRange, StockMargins,
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
    Submitted,
    /// 投入できない（画面に出す理由）
    SubmitFailed(UiMessage),
    /// 取り込んだ切削シミュレーションのジョブの番号
    Imported {
        job: u64,
    },
    /// 取り込める（正常終了した）切削シミュレーションがない
    NothingToImport,
    /// 取り込めない（画面に出す理由）
    ImportFailed(UiMessage),
    /// ジョブマネージャーを開けない（画面に出す理由）
    JobManagerFailed(UiMessage),
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
        OperationStatus::Submitted => ("conditions.status.submitted", vec![]),
        OperationStatus::SubmitFailed(message) => {
            ("conditions.status.submit_failed", reason(message))
        }
        OperationStatus::Imported { job } => {
            ("conditions.status.imported", vec![("job", job.to_string())])
        }
        OperationStatus::NothingToImport => ("conditions.status.nothing_to_import", vec![]),
        OperationStatus::ImportFailed(message) => {
            ("conditions.status.import_failed", reason(message))
        }
        OperationStatus::JobManagerFailed(message) => {
            ("conditions.status.job_manager_failed", reason(message))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MachiningConditionsUiState {
    pub open: bool,
    pub locale: UiLocale,
    /// 加工手順の加工工程の一覧（加工の順）
    pub(crate) operations: Vec<OperationRow>,
    /// 選んだ加工工程
    pub(crate) selected_operation: MachiningOperationId,
    /// 選んだ加工工程の、入力している名前
    pub(crate) operation_name: String,
    /// 加工工程の一覧の操作
    pub(crate) operation_command: Option<OperationCommand>,
    /// 表示中の製品形状の面の数
    pub face_count: usize,
    /// 加工工程の中身（工具・走査線の条件）
    pub operation: MachiningOperationContent,
    /// 加工手順の素材（素材の余裕・精度）
    pub stock: MachiningStock,
    /// 工具長を指定するときの入力値（指定しない間も保つ）
    pub tool_length_input: f64,
    /// ツールパス生成のトレランス（mm）
    pub toolpath_tolerance: f64,
    /// 加工工程・加工手順の条件の問題（投入前の確認の結果）
    pub issues: Vec<ConditionIssue>,
    /// 表示中の結果のジョブの番号と、計算に使った条件
    pub imported: Option<(u64, ImportedConditions)>,
    /// 加工工程の今の改訂の計算の状態
    pub(crate) job_status: JobStatusLine,
    /// 空間モデルに取り込んだ CL が、今の条件の結果か
    pub imported_toolpath: ImportedToolpathState,
    /// 最後の操作の結果
    pub(crate) status: Option<OperationStatus>,
    pub submit_requested: bool,
    /// 空間モデルの CL から、切削シミュレーションだけを投入する
    pub simulate_toolpath_requested: bool,
    pub import_requested: bool,
    pub open_job_manager_requested: bool,
}

/// 加工工程の一覧の 1 行
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OperationRow {
    pub(crate) id: MachiningOperationId,
    pub(crate) name: String,
    pub(crate) status: JobStatusLine,
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
    /// 選んだ工程の名前を替える
    Rename(String),
}

/// 加工工程の一覧に出す、工程の計算の状態の短い文言のキー
fn operation_row_status_key(status: &JobStatusLine) -> &'static str {
    match status {
        JobStatusLine::NotSubmitted => "conditions.operations.status.none",
        JobStatusLine::Unreadable => "conditions.operations.status.unreadable",
        JobStatusLine::Reading => "conditions.operations.status.reading",
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

/// 加工条件のウィンドウに出す、今の条件の計算の状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JobStatusLine {
    /// 今の条件で投入していない
    NotSubmitted,
    Status(OperationJobStatus),
    /// ジョブマネージャーから状態を読めない
    Unreadable,
    /// 状態をまだ読めていない（読み終えるのを待たないため、初めて読む前）
    Reading,
}

impl JobStatusLine {
    /// 投入し直すか条件を変えるまで変わらない状態か
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

    /// 計算を待っている・計算している
    pub(crate) fn is_pending(&self) -> bool {
        matches!(
            self,
            Self::Status(OperationJobStatus::Queued | OperationJobStatus::Running)
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
        match issue {
            ConditionIssue::NoProductShape => self.text("conditions.issue.no_product_shape"),
            ConditionIssue::Condition(error) => {
                let (key, args) = condition_error_message(error);
                self.text_with_args(key, &args)
            }
            ConditionIssue::Other(message) => self.text_with_args(
                "conditions.issue.other",
                &[("reason", resolve_failure_message(self.locale, message))],
            ),
        }
    }

    /// 欄 `field` の問題を出す
    fn show_issues(&self, ui: &mut egui::Ui, field: ConditionField) {
        for issue in self
            .issues
            .iter()
            .filter(|issue| condition_field(issue) == field)
        {
            ui.colored_label(ui.visuals().error_fg_color, self.issue_text(issue));
        }
    }

    /// 切削シミュレーションだけを計算し直せるか（CL が今の条件の結果で、条件に誤りがなく、今の条件の計算が待ち・計算中・
    /// 完了でない。結果がジョブマネージャーに残っている間は、同じ計算を重ねない）
    fn can_simulate_toolpath(&self) -> bool {
        self.issues.is_empty()
            && self.imported_toolpath == ImportedToolpathState::Current
            && !matches!(
                self.job_status,
                JobStatusLine::Status(
                    OperationJobStatus::Queued
                        | OperationJobStatus::Running
                        | OperationJobStatus::Succeeded
                )
            )
    }

    /// ウィンドウを、範囲 `area`（メニューバーの下）の左上に置く
    pub fn show(&mut self, ctx: &Context, area: Rect) {
        if !self.open {
            return;
        }

        let mut open = self.open;
        Window::new(self.text("conditions.window.title"))
            .anchor(Align2::LEFT_TOP, [16.0, 16.0])
            .constrain_to(area)
            .default_width(360.0)
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                // すべての分類を開いても下まで見られるよう、中身は範囲 `area` に収まる高さでスクロールする
                ScrollArea::vertical()
                    .max_height(window_body_max_height(ui, area))
                    .show(ui, |ui| {
                        ui.label(self.text("conditions.intro"));
                        ui.separator();
                        self.show_operations(ui);
                        self.show_target(ui);
                        self.show_tool(ui);
                        self.show_machining(ui);
                        self.show_stock(ui);
                        ui.separator();
                        ui.horizontal(|ui| {
                            let submit = ui.add_enabled(
                                self.issues.is_empty(),
                                egui::Button::new(self.text("conditions.submit")),
                            );
                            if submit.clicked() {
                                self.submit_requested = true;
                            }
                            let simulate_toolpath = ui.add_enabled(
                                self.can_simulate_toolpath(),
                                egui::Button::new(self.text("conditions.simulate_toolpath")),
                            );
                            if simulate_toolpath.clicked() {
                                self.simulate_toolpath_requested = true;
                            }
                            if ui.button(self.text("conditions.import")).clicked() {
                                self.import_requested = true;
                            }
                            if ui
                                .button(self.text("conditions.open_job_manager"))
                                .clicked()
                            {
                                self.open_job_manager_requested = true;
                            }
                        });
                        self.show_issues(ui, ConditionField::Submission);
                        if let Some(status) = &self.status {
                            ui.label(operation_status_text(self.locale, status));
                        }
                        ui.label(self.text(job_status_key(&self.job_status)));
                        ui.label(self.text(imported_toolpath_key(self.imported_toolpath)));
                        ui.separator();
                        self.show_imported(ui);
                    });
            });
        self.open = open;
    }

    /// 加工工程の一覧（選ぶと、その工程の条件を下に出す）と、一覧の操作
    fn show_operations(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new(self.text("conditions.header.operations"))
            .default_open(true)
            .show(ui, |ui| {
                let mut command = None;
                for row in &self.operations {
                    let label = self.text_with_args(
                        "conditions.operations.row",
                        &[
                            ("name", row.name.clone()),
                            ("status", self.text(operation_row_status_key(&row.status))),
                        ],
                    );
                    let selected = row.id == self.selected_operation;
                    if ui.selectable_label(selected, label).clicked() && !selected {
                        command = Some(OperationCommand::Select(row.id));
                    }
                }
                let position = self
                    .operations
                    .iter()
                    .position(|row| row.id == self.selected_operation);
                let count = self.operations.len();
                ui.horizontal(|ui| {
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
                            count > 1,
                            egui::Button::new(self.text("conditions.operations.remove")),
                        )
                        .clicked()
                    {
                        command = Some(OperationCommand::Remove);
                    }
                    let up = ui
                        .add_enabled(position.is_some_and(|at| at > 0), egui::Button::new("▲"))
                        .on_hover_text(self.text("conditions.operations.move_up"));
                    if up.clicked() {
                        command = Some(OperationCommand::Move(-1));
                    }
                    let down = ui
                        .add_enabled(
                            position.is_some_and(|at| at + 1 < count),
                            egui::Button::new("▼"),
                        )
                        .on_hover_text(self.text("conditions.operations.move_down"));
                    if down.clicked() {
                        command = Some(OperationCommand::Move(1));
                    }
                });
                // 名前は、入力を終えたとき（ほかを押した・Enter）に決める（打つたびに工程の改訂を進めない）
                ui.horizontal(|ui| {
                    ui.label(self.text("conditions.operations.name"));
                    let edit = ui.text_edit_singleline(&mut self.operation_name);
                    if edit.lost_focus() {
                        command = Some(OperationCommand::Rename(self.operation_name.clone()));
                    }
                });
                if command.is_some() {
                    self.operation_command = command;
                }
            });
    }

    fn show_target(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new(self.text("conditions.header.target"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(self.text_with_args(
                    "conditions.target.faces",
                    &[("count", self.face_count.to_string())],
                ));
                self.show_issues(ui, ConditionField::Target);
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
                let tool = &self.operation.tool;
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
                    self.operation.tool =
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
        let cusp_height_label = flat_surface_cusp_height(&self.operation).map(|height| {
            self.text_with_args(
                "conditions.machining.cusp_height",
                &[("height", format!("{height:.4}"))],
            )
        });
        CollapsingHeader::new(self.text("conditions.header.machining"))
            .default_open(true)
            .show(ui, |ui| {
                let OperationSpec::Scanline(scanline) = &mut self.operation.spec;
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

    fn show_imported(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new(self.text("conditions.header.imported"))
            .default_open(true)
            .show(ui, |ui| {
                let Some((job, conditions)) = &self.imported else {
                    ui.label(self.text("conditions.imported.none"));
                    return;
                };
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
        condition_error_message, condition_field, imported_toolpath_key, job_status_key,
        operation_row_status_key, operation_status_text, ConditionField, JobStatusLine,
        OperationJobStatus, OperationStatus,
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
            "投入できません（ジョブを投入・実行できません）"
        );
        assert_eq!(
            operation_status_text(UiLocale::En, &status),
            "Cannot submit (Cannot submit or run the job)"
        );
    }

    #[test]
    fn every_operation_status_has_a_message() {
        for status in [
            OperationStatus::Submitted,
            OperationStatus::SubmitFailed(failure("failure.orchestration")),
            OperationStatus::Imported { job: 2 },
            OperationStatus::NothingToImport,
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
    fn the_cl_can_be_simulated_only_when_no_result_of_the_current_conditions_remains() {
        use super::MachiningConditionsUiState;
        use viewmodel::space_model_document::{ImportedToolpathState, SpaceModelDocument};
        let document = SpaceModelDocument::new();
        let state = |imported_toolpath, job_status| MachiningConditionsUiState {
            open: true,
            locale: UiLocale::Ja,
            operations: Vec::new(),
            selected_operation: document.selected_operation(),
            operation_name: document.operation_content().name.clone(),
            operation_command: None,
            face_count: 1,
            operation: document.operation_content().clone(),
            stock: document.stock(),
            tool_length_input: 50.0,
            toolpath_tolerance: document.toolpath_tolerance(),
            issues: Vec::new(),
            imported: None,
            job_status,
            imported_toolpath,
            status: None,
            submit_requested: false,
            simulate_toolpath_requested: false,
            import_requested: false,
            open_job_manager_requested: false,
        };
        let current = ImportedToolpathState::Current;
        for (job_status, allowed) in [
            (JobStatusLine::NotSubmitted, true),
            (JobStatusLine::Status(OperationJobStatus::Failed), true),
            (JobStatusLine::Status(OperationJobStatus::Canceled), true),
            (JobStatusLine::Status(OperationJobStatus::Queued), false),
            (JobStatusLine::Status(OperationJobStatus::Running), false),
            (JobStatusLine::Status(OperationJobStatus::Succeeded), false),
        ] {
            assert_eq!(
                state(current, job_status.clone()).can_simulate_toolpath(),
                allowed,
                "{job_status:?}"
            );
        }
        // 空間モデルの CL が今の条件の結果でなければ、計算し直さない
        for imported in [ImportedToolpathState::None, ImportedToolpathState::Outdated] {
            assert!(!state(imported, JobStatusLine::NotSubmitted).can_simulate_toolpath());
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
