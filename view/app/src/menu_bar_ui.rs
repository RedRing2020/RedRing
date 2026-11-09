//! メニューバー（ウィンドウの上端）

use crate::menu_bar_messages::resolve_menu_bar_text;
use egui::{MenuBar, Panel, Ui};
use viewmodel::message_catalog::UiLocale;

/// 選べる言語と、その言語での名前（どの言語の表示でも、その言語の名前で出す）
const LANGUAGE_CHOICES: &[(UiLocale, &str)] =
    &[(UiLocale::Ja, "日本語"), (UiLocale::En, "English")];

/// メニューの項目の文言を折り返さない（メニューの幅は親の項目の幅から決まるため、英語の長い項目が折り返される）
fn keep_items_on_one_line(ui: &mut Ui) {
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
}

/// 「ファイル」の項目
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileMenuItem {
    New,
    Open,
    Save,
    SaveAs,
    Load,
    Exit,
}

/// 「ファイル」の項目と、その文言のキー・キーの組み合わせ（`None` は区切り）
const FILE_MENU: &[Option<(FileMenuItem, &str, Option<&str>)>] = &[
    Some((FileMenuItem::New, "menu.file.new", Some("Ctrl+N"))),
    Some((FileMenuItem::Open, "menu.file.open", Some("Ctrl+O"))),
    Some((FileMenuItem::Save, "menu.file.save", Some("Ctrl+S"))),
    Some((
        FileMenuItem::SaveAs,
        "menu.file.save_as",
        Some("Ctrl+Shift+S"),
    )),
    None,
    Some((FileMenuItem::Load, "menu.file.load", None)),
    None,
    Some((FileMenuItem::Exit, "menu.file.exit", None)),
];

/// メニューバーの表示の言語と、選ばれた項目
#[derive(Debug, Clone, Copy)]
pub struct MenuBarUiState {
    pub locale: UiLocale,
    /// 「ファイル」で選ばれた項目
    pub file_requested: Option<FileMenuItem>,
    /// 「表示」→「言語」で選ばれた言語
    pub locale_requested: Option<UiLocale>,
    /// 「ヘルプ」→「操作の一覧」が選ばれた
    pub controls_requested: bool,
    /// 「ヘルプ」→「バージョンの情報」が選ばれた
    pub about_requested: bool,
}

impl MenuBarUiState {
    fn text(&self, key: &str) -> String {
        resolve_menu_bar_text(self.locale, key)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        Panel::top("menu_bar").show(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.text("menu.file"), |ui| {
                    keep_items_on_one_line(ui);
                    for entry in FILE_MENU {
                        let Some((item, key, shortcut)) = entry else {
                            ui.separator();
                            continue;
                        };
                        let mut button = egui::Button::new(self.text(key));
                        if let Some(shortcut) = shortcut {
                            button = button.shortcut_text(*shortcut);
                        }
                        if ui.add(button).clicked() {
                            self.file_requested = Some(*item);
                        }
                    }
                });
                ui.menu_button(self.text("menu.view"), |ui| {
                    keep_items_on_one_line(ui);
                    ui.menu_button(self.text("menu.view.language"), |ui| {
                        keep_items_on_one_line(ui);
                        for (locale, name) in LANGUAGE_CHOICES {
                            if ui.radio(self.locale == *locale, *name).clicked() {
                                self.locale_requested = Some(*locale);
                            }
                        }
                    });
                });
                ui.menu_button(self.text("menu.help"), |ui| {
                    keep_items_on_one_line(ui);
                    if ui.button(self.text("menu.help.controls")).clicked() {
                        self.controls_requested = true;
                    }
                    if ui.button(self.text("menu.help.about")).clicked() {
                        self.about_requested = true;
                    }
                });
            });
        });
    }
}
