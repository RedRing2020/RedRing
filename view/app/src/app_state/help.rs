//! AppState のヘルプ（操作の一覧・バージョンの情報）と、画面の下の知らせの行を扱うモジュール。

use super::AppState;
use crate::help_ui::{HelpUiState, Notice};
use crate::machining_conditions_ui::OperationStatus;
use viewmodel::message_catalog::UiLocale;

impl AppState {
    /// 「操作の一覧」のウィンドウを開閉する（F1・h キー）
    pub fn toggle_help_controls(&mut self) {
        self.help_controls_open = !self.help_controls_open;
    }

    /// 「操作の一覧」のウィンドウを開く（「ヘルプ」→「操作の一覧」）
    pub(crate) fn open_help_controls(&mut self) {
        self.help_controls_open = true;
    }

    /// 「バージョンの情報」のウィンドウを開く
    pub(crate) fn open_help_about(&mut self) {
        self.help_about_open = true;
    }

    /// 画面の文言の言語を `locale` に替え、アプリの設定に保存する
    pub(crate) fn set_ui_locale(&mut self, locale: UiLocale) {
        if self.ui_locale == locale {
            return;
        }
        self.ui_locale = locale;
        crate::fatal_error_dialog::set_fatal_error_locale(locale);
        self.refresh_snapshot_window_title();
        self.app_settings.set_locale(locale);
        let Some(path) = &self.app_settings_path else {
            self.set_notice(Notice::SettingsNotSaved);
            return;
        };
        if let Err(error) = crate::app_settings::save_settings(path, &self.app_settings) {
            tracing::warn!(path = %path.display(), kind = ?error.kind(), "Cannot save the app settings: {error}");
            self.set_notice(Notice::SettingsSaveFailed {
                reason: error.to_string(),
            });
        }
    }

    /// 画面の下の知らせの行に、知らせ `notice` を出す（前の知らせに替える）
    pub(crate) fn set_notice(&mut self, notice: Notice) {
        self.notice = notice;
    }

    /// 投入・実行・取り込みの結果 `status` を、加工条件のウィンドウの結果の行と知らせの行に出す
    pub(crate) fn set_operation_status(&mut self, status: OperationStatus) {
        self.notice = Notice::Operation(status.clone());
        self.cutting_simulation_status = Some(status);
    }

    pub(crate) fn prepare_help_state(&self) -> HelpUiState {
        HelpUiState {
            locale: self.ui_locale,
            controls_open: self.help_controls_open,
            about_open: self.help_about_open,
            log_path: crate::logging::log_file_path(),
        }
    }

    pub(crate) fn apply_help_state(&mut self, state: HelpUiState) {
        self.help_controls_open = state.controls_open;
        self.help_about_open = state.about_open;
    }
}
