//! メニューバー（ウィンドウの上端）

use crate::menu_bar_messages::resolve_menu_bar_text;
use egui::{MenuBar, Panel, Ui};
use viewmodel::message_catalog::UiLocale;

/// 選べる言語と、その言語での名前（どの言語の表示でも、その言語の名前で出す）
const LANGUAGE_CHOICES: &[(UiLocale, &str)] =
    &[(UiLocale::Ja, "日本語"), (UiLocale::En, "English")];

/// メニューバーの表示の言語と、選ばれた項目
#[derive(Debug, Clone, Copy)]
pub struct MenuBarUiState {
    pub locale: UiLocale,
    /// 「ファイル」→「読み込み…」が選ばれた
    pub load_requested: bool,
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
        Panel::top("menu_bar").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.text("menu.file"), |ui| {
                    if ui.button(self.text("menu.file.load")).clicked() {
                        self.load_requested = true;
                    }
                });
                ui.menu_button(self.text("menu.view"), |ui| {
                    ui.menu_button(self.text("menu.view.language"), |ui| {
                        for (locale, name) in LANGUAGE_CHOICES {
                            if ui.radio(self.locale == *locale, *name).clicked() {
                                self.locale_requested = Some(*locale);
                            }
                        }
                    });
                });
                ui.menu_button(self.text("menu.help"), |ui| {
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
