//! ツールライブラリのウィンドウ（「加工」→「ツールライブラリ…」。設計: TOOL_LIBRARY_DESIGN.md 6.1）
//!
//! 左に使うライブラリの一覧、右に選んだライブラリのツールセットの表と、選んだツールセットの値の欄を出す。ファイルを
//! 選ぶ画面・確かめの画面は描画を止めるため、操作はここでは行わず、要求として返す（`ToolLibraryAction`）。

use crate::machining_conditions_ui::{SELECTED_ROW_OPACITY, centered_label};
use crate::tool_library_messages::resolve_tool_library_text;
use egui::{Color32, Context, DragValue, Grid, Id, Rect, ScrollArea, Window};
use viewmodel::cutting_simulation_session::CuttingToolKind;
use viewmodel::failure_message_catalog::resolve_failure_message;
use viewmodel::message_catalog::UiLocale;
use viewmodel::tool_library_session::{
    ToolLibraryRow, ToolLibraryState, ToolSetDraft, ToolSetDraftIssue, ToolSetId, ToolSetRow,
    file_label,
};

/// ウィンドウの幅（点）
const WINDOW_WIDTH: f32 = 780.0;
/// ライブラリの一覧の幅（点）
const LIBRARY_LIST_WIDTH: f32 = 220.0;
/// ツールセットの表の高さ（点）
const TOOLSET_TABLE_HEIGHT: f32 = 260.0;
/// 径・工具長の入力の上限（mm）
const MAX_LENGTH: f64 = 10_000.0;

/// ウィンドウの操作の要求（ファイルを選ぶ・確かめる画面を出すため、描画の後に行う）
#[derive(Debug, Clone, PartialEq)]
pub enum ToolLibraryAction {
    /// ライブラリのファイルを新しく作る（場所を選ぶ画面を出す）
    CreateLibrary,
    /// ライブラリのファイルを一覧に足す（ファイルを選ぶ画面を出す）
    OpenLibraries,
    /// 一覧の番号のライブラリを一覧から外す
    RemoveLibrary(usize),
    /// 一覧の番号のライブラリの名前を替える
    RenameLibrary { library: usize, name: String },
    /// 一覧の番号のライブラリに、値のツールセットを足す
    AddToolSet { library: usize, draft: ToolSetDraft },
    /// ツールセットの値を替える
    UpdateToolSet {
        library: usize,
        id: ToolSetId,
        draft: ToolSetDraft,
    },
    /// ツールセットを消す
    RemoveToolSet { library: usize, id: ToolSetId },
    /// ツールセットを位置に動かす
    MoveToolSet {
        library: usize,
        id: ToolSetId,
        to: usize,
    },
}

/// ウィンドウの、フレームをまたいで持つ状態
#[derive(Debug, Clone, Default)]
pub struct ToolLibraryWindow {
    pub open: bool,
    /// 選んだライブラリ（一覧の番号）
    pub selected_library: usize,
    /// 選んだツールセット
    pub selected_toolset: Option<ToolSetId>,
    /// 選んだツールセットの、編集中の値（選んだときの値から変えていなければ `None`）
    pub draft: Option<ToolSetDraft>,
    /// 編集中のライブラリの名前（選んだライブラリの名前から変えていなければ `None`）
    pub rename: Option<String>,
    /// 最後の操作の結果（画面に出す文言と、失敗か）
    pub result: Option<(String, bool)>,
}

/// ウィンドウの 1 フレームの状態
pub struct ToolLibraryUiState {
    pub locale: UiLocale,
    pub window: ToolLibraryWindow,
    /// 使うライブラリの一覧
    pub libraries: Vec<ToolLibraryRow>,
    /// 選んだライブラリのツールセット
    pub toolsets: Vec<ToolSetRow>,
    /// 要求された操作
    pub requested: Option<ToolLibraryAction>,
}

impl ToolLibraryUiState {
    fn text(&self, key: &str) -> String {
        resolve_tool_library_text(self.locale, key, &[])
    }

    fn text_with(&self, key: &str, args: &[(&str, String)]) -> String {
        resolve_tool_library_text(self.locale, key, args)
    }

    /// 選んだライブラリの行
    fn library(&self) -> Option<&ToolLibraryRow> {
        self.libraries.get(self.window.selected_library)
    }

    /// 選んだライブラリを編集できるか
    fn editable(&self) -> bool {
        self.library().is_some_and(ToolLibraryRow::editable)
    }

    /// 選んだツールセットの行と、その位置
    fn toolset(&self) -> Option<(usize, &ToolSetRow)> {
        let id = self.window.selected_toolset?;
        self.toolsets
            .iter()
            .enumerate()
            .find(|(_, row)| row.id == id)
    }

    pub fn show(&mut self, ctx: &Context, area: Rect) {
        if !self.window.open {
            return;
        }
        let mut open = true;
        Window::new(self.text("tool_library.window.title"))
            .id(Id::unique("tool_library_window"))
            .default_pos(area.left_top() + egui::vec2(24.0, 24.0))
            .constrain_to(area)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_width(WINDOW_WIDTH);
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(LIBRARY_LIST_WIDTH);
                        self.show_libraries(ui);
                    });
                    ui.separator();
                    ui.vertical(|ui| self.show_toolsets(ui));
                });
                if let Some((text, failed)) = &self.window.result {
                    ui.separator();
                    let color = if *failed {
                        ui.visuals().error_fg_color
                    } else {
                        ui.visuals().text_color()
                    };
                    ui.colored_label(color, text);
                }
            });
        self.window.open = open;
    }

    fn library_label(&self, row: &ToolLibraryRow) -> String {
        let name = row.name.clone().unwrap_or_else(|| file_label(&row.path));
        let state = match row.state {
            ToolLibraryState::Writable => return name,
            ToolLibraryState::ReadOnly => "tool_library.library.read_only",
            ToolLibraryState::Unreadable(_) => "tool_library.library.unreadable",
            ToolLibraryState::Duplicate { .. } => "tool_library.library.duplicate",
        };
        format!("{name}（{}）", self.text(state))
    }

    fn show_libraries(&mut self, ui: &mut egui::Ui) {
        ui.strong(self.text("tool_library.libraries"));
        ui.horizontal(|ui| {
            if ui
                .button(self.text("tool_library.library.create"))
                .clicked()
            {
                self.requested = Some(ToolLibraryAction::CreateLibrary);
            }
            if ui.button(self.text("tool_library.library.open")).clicked() {
                self.requested = Some(ToolLibraryAction::OpenLibraries);
            }
        });
        if self.libraries.is_empty() {
            ui.colored_label(Color32::GRAY, self.text("tool_library.library.none"));
            return;
        }
        let labels: Vec<String> = self
            .libraries
            .iter()
            .map(|row| self.library_label(row))
            .collect();
        ScrollArea::vertical()
            .id_salt("tool_library_list")
            .max_height(TOOLSET_TABLE_HEIGHT)
            .show(ui, |ui| {
                for (index, label) in labels.into_iter().enumerate() {
                    let selected = index == self.window.selected_library;
                    if ui.selectable_label(selected, label).clicked() && !selected {
                        self.window.selected_library = index;
                        self.window.selected_toolset = None;
                        self.window.draft = None;
                        self.window.rename = None;
                    }
                }
            });
        let Some(row) = self.library().cloned() else {
            return;
        };
        ui.separator();
        ui.label(row.path.display().to_string());
        let reason = match &row.state {
            ToolLibraryState::Writable => None,
            ToolLibraryState::ReadOnly => Some(self.text("tool_library.library.read_only.reason")),
            ToolLibraryState::Unreadable(reason) => Some(self.text_with(
                "tool_library.library.unreadable.reason",
                &[("reason", resolve_failure_message(self.locale, reason))],
            )),
            ToolLibraryState::Duplicate { first } => Some(self.text_with(
                "tool_library.library.duplicate.reason",
                &[("first", first.display().to_string())],
            )),
        };
        if let Some(reason) = reason {
            ui.colored_label(ui.visuals().warn_fg_color, reason);
        }
        // 名前を替える（書けるライブラリだけ）
        if row.editable() {
            let shown = row.name.clone().unwrap_or_default();
            let mut name = self.window.rename.clone().unwrap_or_else(|| shown.clone());
            ui.text_edit_singleline(&mut name);
            self.window.rename = (name != shown).then(|| name.clone());
            let can_rename = name != shown && !name.trim().is_empty();
            if ui
                .add_enabled(
                    can_rename,
                    egui::Button::new(self.text("tool_library.library.rename")),
                )
                .clicked()
            {
                self.requested = Some(ToolLibraryAction::RenameLibrary {
                    library: self.window.selected_library,
                    name,
                });
            }
        }
        if ui
            .button(self.text("tool_library.library.remove"))
            .clicked()
        {
            self.requested = Some(ToolLibraryAction::RemoveLibrary(
                self.window.selected_library,
            ));
        }
    }

    fn kind_text(&self, kind: Option<CuttingToolKind>) -> String {
        self.text(match kind {
            Some(CuttingToolKind::BallEndMill) => "tool_library.kind.ball",
            Some(CuttingToolKind::FlatEndMill) => "tool_library.kind.flat",
            None => "tool_library.kind.other",
        })
    }

    fn show_toolsets(&mut self, ui: &mut egui::Ui) {
        ui.strong(self.text("tool_library.toolsets"));
        let library = self.window.selected_library;
        let editable = self.editable();
        let selected = self.toolset().map(|(at, row)| (at, row.clone()));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editable,
                    egui::Button::new(self.text("tool_library.toolset.add")),
                )
                .clicked()
            {
                self.requested = Some(ToolLibraryAction::AddToolSet {
                    library,
                    draft: ToolSetDraft {
                        name: self.text("tool_library.toolset.new_name"),
                        kind: CuttingToolKind::BallEndMill,
                        diameter: 6.0,
                        cutting_length: Some(30.0),
                    },
                });
            }
            let has_selection = editable && selected.is_some();
            if ui
                .add_enabled(
                    has_selection,
                    egui::Button::new(self.text("tool_library.toolset.duplicate")),
                )
                .clicked()
                && let Some((_, row)) = &selected
            {
                let mut draft = ToolSetDraft::of(row);
                draft.name = self.text_with(
                    "tool_library.toolset.copy_name",
                    &[("name", row.name.clone())],
                );
                self.requested = Some(ToolLibraryAction::AddToolSet { library, draft });
            }
            if ui
                .add_enabled(
                    has_selection,
                    egui::Button::new(self.text("tool_library.toolset.remove")),
                )
                .clicked()
                && let Some((_, row)) = &selected
            {
                self.requested = Some(ToolLibraryAction::RemoveToolSet {
                    library,
                    id: row.id,
                });
            }
            let at = selected.as_ref().map(|(at, _)| *at);
            let can_up = has_selection && at.is_some_and(|at| at > 0);
            let can_down = has_selection && at.is_some_and(|at| at + 1 < self.toolsets.len());
            for (enabled, key, step) in [
                (can_up, "tool_library.toolset.up", -1_isize),
                (can_down, "tool_library.toolset.down", 1),
            ] {
                if ui
                    .add_enabled(enabled, egui::Button::new(self.text(key)))
                    .clicked()
                    && let Some((at, row)) = &selected
                {
                    self.requested = Some(ToolLibraryAction::MoveToolSet {
                        library,
                        id: row.id,
                        to: at.saturating_add_signed(step),
                    });
                }
            }
        });
        if self.toolsets.is_empty() {
            ui.colored_label(Color32::GRAY, self.text("tool_library.toolset.none"));
        } else {
            self.show_toolset_table(ui);
        }
        if let Some((_, row)) = selected {
            ui.separator();
            self.show_toolset_editor(ui, &row, editable);
        }
    }

    fn show_toolset_table(&mut self, ui: &mut egui::Ui) {
        let headers = [
            "tool_library.column.name",
            "tool_library.column.kind",
            "tool_library.column.diameter",
            "tool_library.column.corner_radius",
            "tool_library.column.cutting_length",
        ]
        .map(|key| self.text(key));
        let unlimited = self.text("tool_library.length.unlimited");
        let rows: Vec<(ToolSetId, [String; 5])> = self
            .toolsets
            .iter()
            .map(|row| {
                let length = row
                    .cutting_length
                    .map_or_else(|| unlimited.clone(), |length| format!("{length} mm"));
                (
                    row.id,
                    [
                        row.name.clone(),
                        self.kind_text(row.kind),
                        format!("{} mm", row.diameter),
                        format!("{} mm", row.corner_radius),
                        length,
                    ],
                )
            })
            .collect();
        // 選んだ行・乗った行は、工程リストと同じく行の全体に色を付ける。乗った行は前の描画のもの（行を描き終えるまで
        // 分からないため）。見出しの行が 0 のため、ツールセットの行は 1 から
        let hovered_id = ui.scope_id().with("tool_library_toolset_hovered");
        let hovered_row = ui.data(|data| data.get_temp::<Option<usize>>(hovered_id).flatten());
        let selected_row = self
            .window
            .selected_toolset
            .and_then(|id| rows.iter().position(|(row, _)| *row == id))
            .map(|at| at + 1);
        let mut hovered_now = None;
        let mut clicked = None;
        let row_spacing = ui.spacing().item_spacing.y;
        ScrollArea::vertical()
            .id_salt("tool_library_toolsets")
            .max_height(TOOLSET_TABLE_HEIGHT)
            .show(ui, |ui| {
                Grid::new("tool_library_toolset_table")
                    .num_columns(headers.len())
                    .with_row_color(move |row, style| {
                        if Some(row) == selected_row {
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
                        for header in &headers {
                            ui.strong(header);
                        }
                        ui.end_row();
                        for (index, (id, cells)) in rows.into_iter().enumerate() {
                            // 欄の文字は押せないため、行のすべての欄と行の間の半分を押せる範囲にする
                            let mut area = Rect::NOTHING;
                            for cell in cells {
                                area = area.union(centered_label(ui, cell));
                            }
                            let area = area.expand2(egui::vec2(0.0, row_spacing / 2.0));
                            let response = ui.interact(
                                area,
                                ui.scope_id().with(("tool_library_toolset_row", id)),
                                egui::Sense::click(),
                            );
                            if response.clicked() {
                                clicked = Some(id);
                            }
                            if response.hovered() {
                                hovered_now = Some(index + 1);
                            }
                            ui.end_row();
                        }
                    });
            });
        if hovered_now != hovered_row {
            ui.data_mut(|data| data.insert_temp(hovered_id, hovered_now));
            // 乗った行の色を、すぐに描き直す
            ui.ctx().request_repaint();
        }
        if let Some(id) = clicked
            && self.window.selected_toolset != Some(id)
        {
            self.window.selected_toolset = Some(id);
            self.window.draft = None;
        }
    }

    fn show_toolset_editor(&mut self, ui: &mut egui::Ui, row: &ToolSetRow, editable: bool) {
        let shown = ToolSetDraft::of(row);
        let mut draft = self.window.draft.clone().unwrap_or_else(|| shown.clone());
        ui.add_enabled_ui(editable, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.text("tool_library.edit.name"));
                ui.text_edit_singleline(&mut draft.name);
            });
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut draft.kind,
                    CuttingToolKind::BallEndMill,
                    self.text("tool_library.kind.ball"),
                );
                ui.radio_value(
                    &mut draft.kind,
                    CuttingToolKind::FlatEndMill,
                    self.text("tool_library.kind.flat"),
                );
            });
            ui.horizontal(|ui| {
                ui.label(self.text("tool_library.edit.diameter"));
                ui.add(
                    DragValue::new(&mut draft.diameter)
                        .range(0.0..=MAX_LENGTH)
                        .speed(0.1)
                        .suffix(" mm"),
                );
            });
            ui.horizontal(|ui| {
                let mut specified = draft.cutting_length.is_some();
                ui.checkbox(
                    &mut specified,
                    self.text("tool_library.edit.specify_length"),
                );
                let mut length = draft.cutting_length.or(row.cutting_length).unwrap_or(30.0);
                ui.add_enabled(
                    specified,
                    DragValue::new(&mut length)
                        .range(0.0..=MAX_LENGTH)
                        .speed(0.5)
                        .suffix(" mm"),
                );
                draft.cutting_length = specified.then_some(length);
            });
        });
        let issues = draft.issues();
        for issue in &issues {
            let key = match issue {
                ToolSetDraftIssue::EmptyName => "tool_library.issue.empty_name",
                ToolSetDraftIssue::DiameterNotPositive => "tool_library.issue.diameter",
                ToolSetDraftIssue::CuttingLengthNotPositive => "tool_library.issue.cutting_length",
            };
            ui.colored_label(ui.visuals().error_fg_color, self.text(key));
        }
        let changed = draft != shown;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editable && changed && issues.is_empty(),
                    egui::Button::new(self.text("tool_library.edit.apply")),
                )
                .clicked()
            {
                self.requested = Some(ToolLibraryAction::UpdateToolSet {
                    library: self.window.selected_library,
                    id: row.id,
                    draft: draft.clone(),
                });
            }
            if ui
                .add_enabled(
                    changed,
                    egui::Button::new(self.text("tool_library.edit.revert")),
                )
                .clicked()
            {
                draft = shown.clone();
            }
        });
        self.window.draft = (draft != shown).then_some(draft);
    }
}
