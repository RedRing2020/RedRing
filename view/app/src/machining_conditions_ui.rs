//! 加工条件のウィンドウ（切削シミュレーションの計算条件の入力と投入）
//!
//! 表示の設定（設定パネル）とは分けて、加工手順 1 つの加工工程 1 つの条件（工程の中身と手順の素材）と、表示中の結果の
//! 計算に使った条件を扱う。

use crate::machining_conditions_messages::resolve_machining_conditions_text;
use egui::{Align2, CollapsingHeader, Context, DragValue, Rect, Window};
use viewmodel::cutting_simulation_session::{
    flat_surface_cusp_height, CamConditionError, CamConditionField, ConditionIssue,
    CuttingToolKind, ImportedConditions, MachiningOperationContent, MachiningStock, OperationSpec,
    ScanlineCutDirection, ScanlineToolRange, StockMargins,
};
use viewmodel::message_catalog::UiLocale;

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
        ConditionIssue::NoProductShape(_) => ConditionField::Target,
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
    SubmitFailed(String),
    /// 実行したジョブの数と、かかった秒数
    Ran {
        executed: usize,
        seconds: f64,
    },
    RunFailed(String),
    /// 取り込んだ切削シミュレーションのジョブの番号
    Imported {
        job: u64,
    },
    /// 取り込める（正常終了した）切削シミュレーションがない
    NothingToImport,
    ImportFailed(String),
}

/// 結果 `status` を出す文言のキーと引数
pub(crate) fn operation_status_message(
    status: &OperationStatus,
) -> (&'static str, Vec<(&'static str, String)>) {
    match status {
        OperationStatus::Submitted => ("conditions.status.submitted", vec![]),
        OperationStatus::SubmitFailed(reason) => (
            "conditions.status.submit_failed",
            vec![("reason", reason.clone())],
        ),
        OperationStatus::Ran { executed, seconds } => (
            "conditions.status.ran",
            vec![
                ("executed", executed.to_string()),
                ("seconds", format!("{seconds:.1}")),
            ],
        ),
        OperationStatus::RunFailed(reason) => (
            "conditions.status.run_failed",
            vec![("reason", reason.clone())],
        ),
        OperationStatus::Imported { job } => {
            ("conditions.status.imported", vec![("job", job.to_string())])
        }
        OperationStatus::NothingToImport => ("conditions.status.nothing_to_import", vec![]),
        OperationStatus::ImportFailed(reason) => (
            "conditions.status.import_failed",
            vec![("reason", reason.clone())],
        ),
    }
}

#[derive(Debug, Clone)]
pub struct MachiningConditionsUiState {
    pub open: bool,
    pub locale: UiLocale,
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
    /// 「投入」で、投入した後に続けて実行するか
    pub run_after_submit: bool,
    /// 最後の操作の結果
    pub(crate) status: Option<OperationStatus>,
    pub submit_requested: bool,
    pub import_requested: bool,
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
            ConditionIssue::NoProductShape(_) => self.text("conditions.issue.no_product_shape"),
            ConditionIssue::Condition(error) => {
                let (key, args) = condition_error_message(error);
                self.text_with_args(key, &args)
            }
            ConditionIssue::Other(reason) => {
                self.text_with_args("conditions.issue.other", &[("reason", reason.clone())])
            }
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
                ui.label(self.text("conditions.intro"));
                ui.separator();
                self.show_target(ui);
                self.show_tool(ui);
                self.show_machining(ui);
                self.show_stock(ui);
                ui.separator();
                let run_after_submit_label = self.text("conditions.run_after_submit");
                ui.checkbox(&mut self.run_after_submit, run_after_submit_label);
                ui.horizontal(|ui| {
                    let submit = ui.add_enabled(
                        self.issues.is_empty(),
                        egui::Button::new(self.text("conditions.submit")),
                    );
                    if submit.clicked() {
                        self.submit_requested = true;
                    }
                    if ui.button(self.text("conditions.import")).clicked() {
                        self.import_requested = true;
                    }
                });
                self.show_issues(ui, ConditionField::Submission);
                if let Some(status) = &self.status {
                    let (key, args) = operation_status_message(status);
                    ui.label(self.text_with_args(key, &args));
                }
                ui.separator();
                self.show_imported(ui);
            });
        self.open = open;
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
                let mut kind = CuttingToolKind::of(tool).unwrap_or_default();
                let mut diameter = tool.diameter();
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
                let mut specified = self.operation.tool.cutting_length.is_some();
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
                let tool = kind.tool(diameter, specified.then_some(self.tool_length_input));
                if tool != self.operation.tool {
                    self.operation.tool = tool;
                }
                self.show_issues(ui, ConditionField::Tool);
            });
    }

    fn show_machining(&mut self, ui: &mut egui::Ui) {
        let stepover_label = self.text("conditions.machining.stepover");
        let feed_rate_label = self.text("conditions.machining.feed_rate");
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
        condition_error_message, condition_field, operation_status_message, ConditionField,
        OperationStatus,
    };
    use crate::machining_conditions_messages::resolve_machining_conditions_text;
    use viewmodel::cutting_simulation_session::{CamConditionError, ConditionIssue, StockMargins};
    use viewmodel::message_catalog::UiLocale;

    #[test]
    fn every_operation_status_has_a_message() {
        for status in [
            OperationStatus::Submitted,
            OperationStatus::SubmitFailed("reason".to_string()),
            OperationStatus::Ran {
                executed: 2,
                seconds: 1.25,
            },
            OperationStatus::RunFailed("reason".to_string()),
            OperationStatus::Imported { job: 2 },
            OperationStatus::NothingToImport,
            OperationStatus::ImportFailed("reason".to_string()),
        ] {
            let (key, args) = operation_status_message(&status);
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text = resolve_machining_conditions_text(locale, key, &args);
                assert!(
                    !text.contains("__missing__") && !text.contains('{'),
                    "{status:?}: {text}"
                );
            }
        }
        let (key, args) = operation_status_message(&OperationStatus::Ran {
            executed: 2,
            seconds: 1.25,
        });
        assert_eq!(
            resolve_machining_conditions_text(UiLocale::Ja, key, &args),
            "実行しました（2 件・1.2 秒）。「取り込み」で結果を表示します"
        );
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
            condition_field(&ConditionIssue::NoProductShape(String::new())),
            ConditionField::Target
        );
        assert_eq!(
            condition_field(&ConditionIssue::Other(String::new())),
            ConditionField::Submission
        );
    }
}
