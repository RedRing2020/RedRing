//! 加工条件のウィンドウ（切削シミュレーションの計算条件の入力と投入）
//!
//! 表示の設定（設定パネル）とは分けて、次に投入する計算条件と、表示中の結果の計算に使った条件を扱う。

use crate::machining_conditions_messages::resolve_machining_conditions_text;
use egui::{Align2, CollapsingHeader, Context, DragValue, Window};
use viewmodel::cutting_simulation_session::{
    ConditionIssue, CuttingSimulationConditions, CuttingToolKind, ImportedConditions,
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
        ConditionIssue::InvalidTool => ConditionField::Tool,
        ConditionIssue::InvalidToolpathInput(_) => ConditionField::Machining,
        ConditionIssue::InvalidStockMargins | ConditionIssue::InvalidStock => ConditionField::Stock,
        ConditionIssue::InvalidPrecision
        | ConditionIssue::PrecisionFinerThanTolerance { .. }
        | ConditionIssue::TooManyDexelLines { .. } => ConditionField::Precision,
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
    /// 次に投入する計算条件
    pub conditions: CuttingSimulationConditions,
    /// 工具長を指定するときの入力値（指定しない間も保つ）
    pub tool_length_input: f64,
    /// ツールパス生成のトレランス（mm）
    pub toolpath_tolerance: f64,
    /// 次に投入する計算条件の問題（投入前の確認の結果）
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
            ConditionIssue::InvalidStockMargins => {
                self.text("conditions.issue.invalid_stock_margins")
            }
            ConditionIssue::InvalidStock => self.text("conditions.issue.invalid_stock"),
            ConditionIssue::InvalidTool => self.text("conditions.issue.invalid_tool"),
            ConditionIssue::InvalidToolpathInput(reason) => self.text_with_args(
                "conditions.issue.invalid_toolpath_input",
                &[("reason", reason.clone())],
            ),
            ConditionIssue::InvalidPrecision => self.text("conditions.issue.invalid_precision"),
            ConditionIssue::PrecisionFinerThanTolerance { tolerance } => self.text_with_args(
                "conditions.issue.precision_finer_than_tolerance",
                &[("tolerance", tolerance.to_string())],
            ),
            ConditionIssue::TooManyDexelLines { finest_precision } => self.text_with_args(
                "conditions.issue.too_many_dexel_lines",
                &[("finest_precision", finest_precision.to_string())],
            ),
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

    pub fn show(&mut self, ctx: &Context) {
        if !self.open {
            return;
        }

        let mut open = self.open;
        Window::new(self.text("conditions.window.title"))
            .anchor(Align2::LEFT_TOP, [16.0, 16.0])
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
                let conditions = &mut self.conditions;
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut conditions.tool_kind,
                        CuttingToolKind::BallEndMill,
                        ball_label,
                    );
                    ui.radio_value(
                        &mut conditions.tool_kind,
                        CuttingToolKind::FlatEndMill,
                        flat_label,
                    );
                });
                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut conditions.tool_diameter)
                            .range(0.1..=f64::MAX)
                            .speed(0.1)
                            .suffix(" mm"),
                    );
                    ui.label(diameter_label);
                });
                let mut specified = conditions.cutting_length.is_some();
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
                conditions.cutting_length = specified.then_some(self.tool_length_input);
                self.show_issues(ui, ConditionField::Tool);
            });
    }

    fn show_machining(&mut self, ui: &mut egui::Ui) {
        let stepover_label = self.text("conditions.machining.stepover");
        let sample_pitch_label = self.text("conditions.machining.sample_pitch");
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
        CollapsingHeader::new(self.text("conditions.header.machining"))
            .default_open(true)
            .show(ui, |ui| {
                let scanline = &mut self.conditions.scanline;
                for (value, label, suffix, speed) in [
                    (&mut scanline.stepover, stepover_label, " mm", 0.1),
                    (&mut scanline.sample_pitch, sample_pitch_label, " mm", 0.1),
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
                } = &mut self.conditions.stock_margins;
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
                        DragValue::new(&mut self.conditions.precision)
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
                            ("sample_pitch", conditions.scanline.sample_pitch.to_string()),
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
    use super::{condition_field, operation_status_message, ConditionField, OperationStatus};
    use crate::machining_conditions_messages::resolve_machining_conditions_text;
    use viewmodel::cutting_simulation_session::ConditionIssue;
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

    #[test]
    fn issues_are_shown_next_to_the_field_to_fix() {
        for (issue, field) in [
            (
                ConditionIssue::NoProductShape(String::new()),
                ConditionField::Target,
            ),
            (ConditionIssue::InvalidTool, ConditionField::Tool),
            (
                ConditionIssue::InvalidToolpathInput(String::new()),
                ConditionField::Machining,
            ),
            (ConditionIssue::InvalidStockMargins, ConditionField::Stock),
            (ConditionIssue::InvalidStock, ConditionField::Stock),
            (ConditionIssue::InvalidPrecision, ConditionField::Precision),
            (
                ConditionIssue::PrecisionFinerThanTolerance { tolerance: 0.001 },
                ConditionField::Precision,
            ),
            (
                ConditionIssue::TooManyDexelLines {
                    finest_precision: 0.02,
                },
                ConditionField::Precision,
            ),
            (
                ConditionIssue::Other(String::new()),
                ConditionField::Submission,
            ),
        ] {
            assert_eq!(condition_field(&issue), field, "{issue:?}");
        }
    }
}
