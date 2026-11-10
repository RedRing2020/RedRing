//! ヘルプ（操作の一覧・バージョンの情報）のウィンドウと、画面の下の知らせの行

use crate::help_messages::resolve_help_text;
use crate::machining_conditions_ui::{operation_status_text, OperationStatus};
use crate::settings_panel_renderer::SYMBOL_FONT_FAMILY;
use egui::containers::menu::{MenuButton, MenuConfig};
use egui::{
    Align2, Color32, Context, Grid, Label, Panel, PopupCloseBehavior, Rect, RichText, ScrollArea,
    Ui, Window,
};
use std::collections::VecDeque;
use viewmodel::failure_message_catalog::resolve_failure_message;
use viewmodel::message_catalog::{UiLocale, UiMessage};

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
            ("Ctrl+N", "help.app.new"),
            ("Ctrl+O", "help.app.open"),
            ("Ctrl+S", "help.app.save"),
            ("Ctrl+Shift+S", "help.app.save_as"),
            ("help.input.drop", "help.app.drop"),
            ("Shift+S", "help.app.settings"),
            ("F1 / h", "help.app.help"),
            ("Esc", "help.app.quit"),
        ],
    ),
];

/// 知らせの重さ
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    /// 投入・実行・取り込みの結果（工程リストの結果の行と同じ文言）
    Operation(OperationStatus),
    /// 空の空間モデルにした
    ModelCreated,
    /// モデルファイルを開いた（ファイルの名前）
    ModelOpened { file: String },
    /// モデルファイルを開けない（ファイルの名前と理由）
    ModelOpenFailed { file: String, reason: UiMessage },
    /// ジョブマネージャーが動いていないため、計算の状態が分からない（計算開始すると起動する）
    JobManagerNotRunning,
    /// ジョブマネージャーが見つからないため、計算の投入と状態の確認ができない
    JobManagerNotFound,
    /// 計算待ち・計算中のジョブがあるのに、ジョブマネージャーが停止中のため計算されていない
    JobManagerStopped,
    /// モデルファイルに保存した（ファイルの名前）
    ModelSaved { file: String },
    /// モデルファイルに保存できない（ファイルの名前と理由）
    ModelSaveFailed { file: String, reason: UiMessage },
}

impl Notice {
    pub(crate) fn level(&self) -> NoticeLevel {
        match self {
            Self::Startup
            | Self::ShapeImported { .. }
            | Self::ModelCreated
            | Self::ModelOpened { .. }
            | Self::ModelSaved { .. }
            | Self::JobManagerNotRunning => NoticeLevel::Info,
            Self::ShapeImportFailed { .. }
            | Self::ModelOpenFailed { .. }
            | Self::ModelSaveFailed { .. }
            | Self::JobManagerNotFound => NoticeLevel::Error,
            Self::NotImported
            | Self::NoToolSegments
            | Self::ApplyWithoutAnimation
            | Self::DebugViewShown
            | Self::InverseOffsetToolDiffers
            | Self::SettingsLoadFailed { .. }
            | Self::SettingsNotSaved
            | Self::SettingsSaveFailed { .. }
            | Self::JobManagerStopped => NoticeLevel::Warning,
            Self::Operation(status) => match status {
                OperationStatus::SubmitFailed(_)
                | OperationStatus::SequenceSubmitFailed { .. }
                | OperationStatus::ImportFailed(_)
                | OperationStatus::JobManagerFailed(_)
                | OperationStatus::SequenceStatusUnreadable(_) => NoticeLevel::Error,
                OperationStatus::NothingToImport | OperationStatus::NoToolpathToImport => {
                    NoticeLevel::Warning
                }
                OperationStatus::Submitted { .. }
                | OperationStatus::Redone
                | OperationStatus::SequenceSubmitted { .. }
                | OperationStatus::Imported { .. }
                | OperationStatus::ToolpathImported { .. } => NoticeLevel::Info,
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
            Self::ModelCreated => resolve_help_text(locale, "notice.model_created", &[]),
            Self::JobManagerNotRunning => {
                resolve_help_text(locale, "notice.job_manager_not_running", &[])
            }
            Self::JobManagerNotFound => {
                resolve_help_text(locale, "notice.job_manager_not_found", &[])
            }
            Self::JobManagerStopped => resolve_help_text(locale, "notice.job_manager_stopped", &[]),
            Self::ModelOpened { file } => {
                resolve_help_text(locale, "notice.model_opened", &[("file", file.clone())])
            }
            Self::ModelSaved { file } => {
                resolve_help_text(locale, "notice.model_saved", &[("file", file.clone())])
            }
            Self::ModelOpenFailed { file, reason } => resolve_help_text(
                locale,
                "notice.model_open_failed",
                &[
                    ("file", file.clone()),
                    ("reason", resolve_failure_message(locale, reason)),
                ],
            ),
            Self::ModelSaveFailed { file, reason } => resolve_help_text(
                locale,
                "notice.model_save_failed",
                &[
                    ("file", file.clone()),
                    ("reason", resolve_failure_message(locale, reason)),
                ],
            ),
        }
    }
}

/// 画面の下の行（最後のメッセージを 1 行で出し、📋 から最近のメッセージと選んだ工程の状態の一覧を開く）
#[derive(Debug, Clone)]
pub struct NoticeBarUiState<'a> {
    pub locale: UiLocale,
    pub(crate) notice: &'a Notice,
    /// 最近のメッセージ（新しい順）
    pub(crate) history: &'a VecDeque<Notice>,
    /// まだ一覧を開いていない、注意・失敗のメッセージのうち最も重いもの（`None` はない）
    pub(crate) unread: Option<NoticeLevel>,
    /// 一覧に出す、選んだ工程の状態の行（重さと文言。加工手順を開いていなければ空）
    pub(crate) details: Vec<(NoticeLevel, String)>,
    /// 一覧を開いた（まだ開いていないメッセージの印を消す）
    pub(crate) log_opened: bool,
}

/// 重さ `level` の文字の色（普段の文字は `None`）
fn level_color(ui: &Ui, level: NoticeLevel) -> Option<Color32> {
    match level {
        NoticeLevel::Info => None,
        NoticeLevel::Warning => Some(ui.visuals().warn_fg_color),
        NoticeLevel::Error => Some(ui.visuals().error_fg_color),
    }
}

/// 重さ `level` の色を付けた文字 `text`
fn level_text(ui: &Ui, level: NoticeLevel, text: String) -> RichText {
    match level_color(ui, level) {
        Some(color) => RichText::new(text).color(color),
        None => RichText::new(text),
    }
}

impl NoticeBarUiState<'_> {
    /// 画面の下の行に、一覧を開く 📋 と、最後のメッセージを 1 行で出す（長い文は切り詰める。全文は一覧で見る）
    pub fn show(&mut self, ui: &mut Ui) {
        Panel::bottom("notice_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                // まだ開いていない注意・失敗があれば、その重さの色にする（記号のフォントは単色のため、色を付けられる）
                let icon = match self.unread {
                    Some(level) => level_text(ui, level, "📋".to_string()),
                    None => RichText::new("📋"),
                }
                .family(egui::FontFamily::Name(SYMBOL_FONT_FAMILY.into()));
                // 一覧の中の文字を選ぶ・スクロールするときに閉じないよう、外を押したときだけ閉じる
                let config =
                    MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside);
                let (response, log) = MenuButton::new(icon)
                    .config(config)
                    .ui(ui, |ui| self.show_log(ui));
                if log.is_some() {
                    self.log_opened = true;
                }
                response.on_hover_text(resolve_help_text(self.locale, "notice.log.hint", &[]));
                let text = level_text(ui, self.notice.level(), self.notice.text(self.locale));
                ui.add(Label::new(text).truncate());
            });
        });
    }

    /// 一覧（選んだ工程の状態と、最近のメッセージ。メッセージの文は写せるように選べる）
    fn show_log(&self, ui: &mut Ui) {
        ui.set_max_width(LOG_WIDTH);
        ScrollArea::vertical()
            .max_height(LOG_MAX_HEIGHT)
            .show(ui, |ui| {
                if !self.details.is_empty() {
                    ui.strong(resolve_help_text(self.locale, "notice.log.state", &[]));
                    for (level, text) in &self.details {
                        ui.label(level_text(ui, *level, text.clone()));
                    }
                    ui.separator();
                }
                ui.strong(resolve_help_text(self.locale, "notice.log.history", &[]));
                for notice in self.history {
                    let text = level_text(ui, notice.level(), notice.text(self.locale));
                    ui.add(Label::new(text).wrap().selectable(true));
                }
            });
    }
}

/// 最近のメッセージの一覧 `history`（新しい順）に、メッセージ `notice` を足す（上限 `limit` を超えた古いものは
/// 捨てる）。同じメッセージが続くときは足さない（キーを押し続けたときに、一覧が同じメッセージで埋まらないように）
pub(crate) fn push_notice_history(history: &mut VecDeque<Notice>, notice: &Notice, limit: usize) {
    if history.front() == Some(notice) {
        return;
    }
    history.push_front(notice.clone());
    history.truncate(limit);
}

/// まだ一覧を開いていないメッセージのうち最も重いもの `unread` に、重さ `level` のメッセージを加えた結果（注意・
/// 失敗だけを数える）
pub(crate) fn next_unread(unread: Option<NoticeLevel>, level: NoticeLevel) -> Option<NoticeLevel> {
    if level > NoticeLevel::Info {
        unread.max(Some(level))
    } else {
        unread
    }
}

/// メッセージの一覧の幅・最も高い高さ（論理座標）
const LOG_WIDTH: f32 = 520.0;
const LOG_MAX_HEIGHT: f32 = 360.0;

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
        // マニュアル（GitHub Pages）は、画面の言語のものを開く
        let manual_url = self.text("about.manual.url");
        Window::new(self.text("about.window.title"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .constrain_to(area)
            .resizable(false)
            .collapsible(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.heading(version);
                // ログのファイルの場所は、写せるように選べる（ほかの文字は選べない）
                ui.add(Label::new(log_file).selectable(true));
                ui.hyperlink_to(manual, &manual_url);
            });
        self.about_open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notice_log_has_texts_and_errors_are_heavier_than_warnings() {
        for locale in [UiLocale::Ja, UiLocale::En] {
            for key in ["notice.log.hint", "notice.log.state", "notice.log.history"] {
                assert!(
                    !resolve_help_text(locale, key, &[]).contains("__missing__"),
                    "{key}"
                );
            }
        }
        // 📋 の印は、まだ開いていない注意・失敗のうち最も重いものの色にする（情報は印にしない）
        assert_eq!(next_unread(None, NoticeLevel::Info), None);
        assert_eq!(
            next_unread(None, NoticeLevel::Warning),
            Some(NoticeLevel::Warning)
        );
        assert_eq!(
            next_unread(Some(NoticeLevel::Warning), NoticeLevel::Error),
            Some(NoticeLevel::Error)
        );
        assert_eq!(
            next_unread(Some(NoticeLevel::Error), NoticeLevel::Warning),
            Some(NoticeLevel::Error)
        );
        assert_eq!(
            next_unread(Some(NoticeLevel::Error), NoticeLevel::Info),
            Some(NoticeLevel::Error)
        );
    }

    #[test]
    fn the_history_keeps_the_newest_messages_without_repeating_the_same_one() {
        let mut history = VecDeque::new();
        push_notice_history(&mut history, &Notice::NotImported, 2);
        push_notice_history(&mut history, &Notice::NotImported, 2);
        assert_eq!(history, [Notice::NotImported]);
        push_notice_history(&mut history, &Notice::ModelCreated, 2);
        push_notice_history(&mut history, &Notice::SettingsNotSaved, 2);
        assert_eq!(history, [Notice::SettingsNotSaved, Notice::ModelCreated]);
    }

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
                Notice::Operation(OperationStatus::SubmitFailed(UiMessage {
                    key: "failure.job_manager_not_found".to_string(),
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
            Notice::Operation(OperationStatus::Submitted {
                simulation_only: true
            })
            .level(),
            NoticeLevel::Info
        );
        assert_eq!(Notice::NotImported.level(), NoticeLevel::Warning);
        assert_eq!(Notice::JobManagerStopped.level(), NoticeLevel::Warning);
    }
}
