//! ヘルプ（操作の一覧・バージョンの情報）のウィンドウと、画面の下の知らせの行

use crate::help_messages::resolve_help_text;
use crate::machining_conditions_ui::{operation_status_text, OperationStatus};
use egui::{Align2, Color32, Context, Grid, Panel, Rect, Ui, Window};
use viewmodel::failure_message_catalog::resolve_failure_message;
use viewmodel::message_catalog::{UiLocale, UiMessage};

/// マニュアル（GitHub Pages）の URL
const MANUAL_URL: &str = "https://redring2020.github.io/RedRing/";

/// アプリの版
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 操作の一覧の分類（見出しの文言のキーと、行の操作と説明。操作は、`help.` で始まるものは文言のキー、ほかはキーの名前）
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "help.header.camera",
        &[
            ("help.input.ctrl_left_drag", "help.camera.rotate"),
            ("help.input.ctrl_middle_drag", "help.camera.pan"),
            ("help.input.ctrl_right_drag", "help.camera.zoom"),
            ("r", "help.camera.reset"),
            ("t", "help.camera.standard"),
            ("f", "help.camera.front"),
            ("e", "help.camera.escape"),
        ],
    ),
    (
        "help.header.cutting",
        &[
            ("Shift+M", "help.cutting.conditions"),
            ("Shift+R", "help.cutting.run"),
            ("Shift+L", "help.cutting.import"),
            ("k / j", "help.cutting.step"),
            (". / ,", "help.cutting.endpoint"),
            ("Space", "help.cutting.play"),
            ("Shift+J", "help.cutting.stop"),
            ("+ / -", "help.cutting.speed"),
            ("help.input.progress_drag", "help.cutting.scrub"),
        ],
    ),
    (
        "help.header.display",
        &[
            ("w", "help.display.wireframe"),
            ("v", "help.display.toolpath"),
            ("Shift+V", "help.display.tool"),
        ],
    ),
    (
        "help.header.app",
        &[
            ("Shift+S", "help.app.settings"),
            ("F1 / h", "help.app.help"),
            ("help.input.drop", "help.app.load"),
            ("Esc", "help.app.quit"),
        ],
    ),
];

/// 知らせの重さ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoticeLevel {
    Info,
    Warning,
    Error,
}

/// 画面の下の知らせの行に出す知らせ
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Notice {
    /// 起動したときの案内
    Startup,
    /// 形状を読み込んだ（ファイルの名前）
    ShapeImported { file: String },
    /// 形状を読み込めない（ファイルの名前と理由）
    ShapeImportFailed { file: String, reason: UiMessage },
    /// 切削シミュレーションを取り込む前に、再生の操作をした
    NotImported,
    /// ツールパスに区間がないため、工具を区間の端点へ移せない
    NoToolSegments,
    /// 切削アニメーションの表示中ではないため、設定を反映できない
    ApplyWithoutAnimation,
    /// デバッグ表示の間は、切削アニメーションのコマを動かさない
    DebugViewShown,
    /// 取り込んだ結果の工具と今の計算条件の工具が違い、ツールパスの線は逆オフセット形状に乗らない
    InverseOffsetToolDiffers,
    /// アプリの設定を読めないため、既定の設定で起動した（理由）
    SettingsLoadFailed { reason: UiMessage },
    /// 設定のファイルを使えない（利用者ごとのフォルダが分からない・読めなかった）ため、設定を保存しない
    SettingsNotSaved,
    /// アプリの設定を保存できない（理由）
    SettingsSaveFailed { reason: UiMessage },
    /// 投入・実行・取り込みの結果（加工条件のウィンドウの結果の行と同じ文言）
    Operation(OperationStatus),
}

impl Notice {
    pub(crate) fn level(&self) -> NoticeLevel {
        match self {
            Self::Startup | Self::ShapeImported { .. } => NoticeLevel::Info,
            Self::ShapeImportFailed { .. } => NoticeLevel::Error,
            Self::NotImported
            | Self::NoToolSegments
            | Self::ApplyWithoutAnimation
            | Self::DebugViewShown
            | Self::InverseOffsetToolDiffers
            | Self::SettingsLoadFailed { .. }
            | Self::SettingsNotSaved
            | Self::SettingsSaveFailed { .. } => NoticeLevel::Warning,
            Self::Operation(status) => match status {
                OperationStatus::SubmitFailed(_)
                | OperationStatus::RunFailed(_)
                | OperationStatus::ImportFailed(_) => NoticeLevel::Error,
                OperationStatus::NothingToImport => NoticeLevel::Warning,
                OperationStatus::Submitted
                | OperationStatus::Ran { .. }
                | OperationStatus::Imported { .. } => NoticeLevel::Info,
            },
        }
    }

    pub(crate) fn text(&self, locale: UiLocale) -> String {
        match self {
            Self::Startup => resolve_help_text(locale, "notice.startup", &[]),
            Self::ShapeImported { file } => {
                resolve_help_text(locale, "notice.shape_imported", &[("file", file.clone())])
            }
            Self::ShapeImportFailed { file, reason } => resolve_help_text(
                locale,
                "notice.shape_import_failed",
                &[
                    ("file", file.clone()),
                    ("reason", resolve_failure_message(locale, reason)),
                ],
            ),
            Self::NotImported => resolve_help_text(locale, "notice.not_imported", &[]),
            Self::NoToolSegments => resolve_help_text(locale, "notice.no_tool_segments", &[]),
            Self::ApplyWithoutAnimation => {
                resolve_help_text(locale, "notice.apply_without_animation", &[])
            }
            Self::DebugViewShown => resolve_help_text(locale, "notice.debug_view_shown", &[]),
            Self::InverseOffsetToolDiffers => {
                resolve_help_text(locale, "notice.inverse_offset_tool_differs", &[])
            }
            Self::SettingsLoadFailed { reason } => resolve_help_text(
                locale,
                "notice.settings_load_failed",
                &[("reason", resolve_failure_message(locale, reason))],
            ),
            Self::SettingsNotSaved => resolve_help_text(locale, "notice.settings_not_saved", &[]),
            Self::SettingsSaveFailed { reason } => resolve_help_text(
                locale,
                "notice.settings_save_failed",
                &[("reason", resolve_failure_message(locale, reason))],
            ),
            Self::Operation(status) => operation_status_text(locale, status),
        }
    }
}

/// 画面の下の知らせの行（最後の知らせを 1 行で出す）
#[derive(Debug, Clone, Copy)]
pub struct NoticeBarUiState<'a> {
    pub locale: UiLocale,
    pub(crate) notice: &'a Notice,
}

impl NoticeBarUiState<'_> {
    pub fn show(&self, ui: &mut Ui) {
        let notice = self.notice;
        Panel::bottom("notice_bar").show_inside(ui, |ui| {
            let text = notice.text(self.locale);
            match notice.level() {
                NoticeLevel::Info => ui.label(text),
                NoticeLevel::Warning => ui.colored_label(ui.visuals().warn_fg_color, text),
                NoticeLevel::Error => ui.colored_label(ui.visuals().error_fg_color, text),
            };
        });
    }
}

/// ヘルプのウィンドウ（操作の一覧・バージョンの情報）の開閉と中身
#[derive(Debug, Clone)]
pub struct HelpUiState {
    pub locale: UiLocale,
    pub controls_open: bool,
    pub about_open: bool,
    /// ログのファイルの場所（ファイルに書いていないときは `None`）
    pub log_path: Option<&'static str>,
}

impl HelpUiState {
    fn text(&self, key: &str) -> String {
        resolve_help_text(self.locale, key, &[])
    }

    /// ウィンドウを、範囲 `area`（メニューバーと知らせの行の間）に置く
    pub fn show(&mut self, ctx: &Context, area: Rect) {
        self.show_controls(ctx, area);
        self.show_about(ctx, area);
    }

    fn show_controls(&mut self, ctx: &Context, area: Rect) {
        if !self.controls_open {
            return;
        }
        let mut open = self.controls_open;
        let sections: Vec<(String, Vec<(String, String)>)> = HELP_SECTIONS
            .iter()
            .map(|(header, rows)| {
                (
                    self.text(header),
                    rows.iter()
                        .map(|(input, description)| {
                            let input = if input.starts_with("help.") {
                                self.text(input)
                            } else {
                                (*input).to_string()
                            };
                            (input, self.text(description))
                        })
                        .collect(),
                )
            })
            .collect();
        let note = self.text("help.note.text_input");
        Window::new(self.text("help.window.title"))
            .anchor(Align2::RIGHT_TOP, [-16.0, 16.0])
            .constrain_to(area)
            .default_width(420.0)
            // 一覧が画面の高さに収まらないときは、ウィンドウの中をスクロールする
            .default_height((area.height() - 32.0).max(120.0))
            .vscroll(true)
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                for (header, rows) in &sections {
                    ui.strong(header);
                    Grid::new(header.as_str())
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for (input, description) in rows {
                                ui.monospace(input);
                                ui.label(description);
                                ui.end_row();
                            }
                        });
                    ui.add_space(8.0);
                }
                ui.separator();
                ui.colored_label(Color32::GRAY, note);
            });
        self.controls_open = open;
    }

    fn show_about(&mut self, ctx: &Context, area: Rect) {
        if !self.about_open {
            return;
        }
        let mut open = self.about_open;
        let version = resolve_help_text(
            self.locale,
            "about.version",
            &[("version", APP_VERSION.to_string())],
        );
        let log_file = match self.log_path {
            Some(path) => {
                resolve_help_text(self.locale, "about.log_file", &[("path", path.to_string())])
            }
            None => self.text("about.no_log_file"),
        };
        let manual = self.text("about.manual");
        Window::new(self.text("about.window.title"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .constrain_to(area)
            .resizable(false)
            .collapsible(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.heading(version);
                ui.label(log_file);
                ui.hyperlink_to(manual, MANUAL_URL);
            });
        self.about_open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_help_row_and_notice_has_a_text() {
        for locale in [UiLocale::Ja, UiLocale::En] {
            for (header, rows) in HELP_SECTIONS {
                assert!(!resolve_help_text(locale, header, &[]).contains("__missing__"));
                for (input, description) in *rows {
                    let text = resolve_help_text(locale, description, &[]);
                    assert!(!text.contains("__missing__"), "{description}");
                    if input.starts_with("help.") {
                        let text = resolve_help_text(locale, input, &[]);
                        assert!(!text.contains("__missing__"), "{input}");
                    }
                }
            }
            let notices = [
                Notice::Startup,
                Notice::ShapeImported {
                    file: "a.stl".to_string(),
                },
                Notice::ShapeImportFailed {
                    file: "a.stl".to_string(),
                    reason: UiMessage {
                        key: "failure.shape.stl".to_string(),
                        args: Vec::new(),
                    },
                },
                Notice::NotImported,
                Notice::NoToolSegments,
                Notice::ApplyWithoutAnimation,
                Notice::DebugViewShown,
                Notice::InverseOffsetToolDiffers,
                Notice::SettingsLoadFailed {
                    reason: UiMessage {
                        key: "failure.file.invalid_data".to_string(),
                        args: Vec::new(),
                    },
                },
                Notice::SettingsNotSaved,
                Notice::SettingsSaveFailed {
                    reason: UiMessage {
                        key: "failure.file.permission_denied".to_string(),
                        args: Vec::new(),
                    },
                },
                Notice::Operation(OperationStatus::RunFailed(UiMessage {
                    key: "failure.orchestration".to_string(),
                    args: Vec::new(),
                })),
            ];
            for notice in notices {
                let text = notice.text(locale);
                assert!(
                    !text.contains("__missing__") && !text.contains('{'),
                    "{notice:?}: {text}"
                );
            }
        }
    }

    #[test]
    fn failures_are_errors_and_results_are_information() {
        assert_eq!(
            Notice::ShapeImportFailed {
                file: String::new(),
                reason: UiMessage {
                    key: String::new(),
                    args: Vec::new()
                }
            }
            .level(),
            NoticeLevel::Error
        );
        assert_eq!(
            Notice::Operation(OperationStatus::ImportFailed(UiMessage {
                key: String::new(),
                args: Vec::new()
            }))
            .level(),
            NoticeLevel::Error
        );
        assert_eq!(
            Notice::Operation(OperationStatus::NothingToImport).level(),
            NoticeLevel::Warning
        );
        assert_eq!(
            Notice::Operation(OperationStatus::Submitted).level(),
            NoticeLevel::Info
        );
        assert_eq!(Notice::NotImported.level(), NoticeLevel::Warning);
    }
}
