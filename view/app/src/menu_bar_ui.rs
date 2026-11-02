//! メニューバー（ウィンドウの上端）

use crate::menu_bar_messages::resolve_menu_bar_text;
use egui::{MenuBar, Panel, Ui};
use viewmodel::message_catalog::UiLocale;

/// メニューバーの表示の言語と、選ばれた項目
#[derive(Debug, Clone, Copy)]
pub struct MenuBarUiState {
    pub locale: UiLocale,
    /// 「ファイル」→「読み込み…」が選ばれた
    pub load_requested: bool,
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
            });
        });
    }
}
