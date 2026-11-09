//! AppState のメニューバーの項目を扱うモジュール。

use super::model_file::FileAction;
use super::AppState;
use crate::menu_bar_ui::{FileMenuItem, MachiningMenuItem, MenuBarUiState};

impl AppState {
    pub(crate) fn prepare_menu_bar_state(&self) -> MenuBarUiState {
        MenuBarUiState {
            locale: self.ui_locale,
            machining_sequence_open: self.machining_sequence_open,
            machining_conditions_shown: self.machining_conditions_open,
            file_requested: None,
            machining_requested: None,
            locale_requested: None,
            controls_requested: false,
            about_requested: false,
        }
    }

    pub(crate) fn apply_menu_bar_state(&mut self, state: MenuBarUiState) {
        if let Some(item) = state.file_requested {
            self.request_file_action(match item {
                FileMenuItem::New => FileAction::New,
                FileMenuItem::Open => FileAction::Open,
                FileMenuItem::Save => FileAction::Save,
                FileMenuItem::SaveAs => FileAction::SaveAs,
                FileMenuItem::Load => FileAction::Load,
                FileMenuItem::Exit => FileAction::Exit,
            });
        }
        match state.machining_requested {
            Some(MachiningMenuItem::OpenSequence) => self.open_machining_sequence(),
            Some(MachiningMenuItem::CloseSequence) => self.close_machining_sequence(),
            Some(MachiningMenuItem::ConditionsWindow) => self.toggle_machining_conditions(),
            None => {}
        }
        if let Some(locale) = state.locale_requested {
            self.set_ui_locale(locale);
        }
        if state.controls_requested {
            self.open_help_controls();
        }
        if state.about_requested {
            self.open_help_about();
        }
    }

    /// 選ばれたメニューの項目・要求されたモデルファイルの操作を行う（ファイルを選ぶ画面・確かめの画面は描画を止める
    /// ため、描画の途中ではなく、イベントの処理が終わった後に呼ぶ）
    pub fn run_requested_menu_actions(&mut self) {
        self.run_file_actions();
        self.refresh_window_title();
    }
}
