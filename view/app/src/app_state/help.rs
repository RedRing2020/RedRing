//! AppState のヘルプ（操作の一覧・バージョンの情報）と、画面の下の知らせの行を扱うモジュール。

use super::AppState;
use crate::help_ui::{next_unread, push_notice_history, HelpUiState, Notice, NoticeLevel};
use crate::machining_conditions_ui::{
    condition_issue_text, imported_toolpath_text, job_status_text, JobStatusLine, OperationStatus,
};
use viewmodel::cutting_simulation_session::OperationJobStatus;
use viewmodel::message_catalog::UiLocale;
use viewmodel::space_model_document::ImportedToolpathState;

/// 最近のメッセージの一覧に残す数
const NOTICE_HISTORY_LIMIT: usize = 50;

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
                reason: viewmodel::failure_message_mapper::file_failure_message(error.kind()),
            });
        }
    }

    /// 画面の下の行に、メッセージ `notice` を出し（前のメッセージに替える）、最近のメッセージの一覧に足す。注意・失敗は、一覧を
    /// 開くまで 📋 に印を付ける
    pub(crate) fn set_notice(&mut self, notice: Notice) {
        self.unread_notice = next_unread(self.unread_notice, notice.level());
        push_notice_history(&mut self.notice_history, &notice, NOTICE_HISTORY_LIMIT);
        self.notice = notice;
    }

    /// 投入・実行・取り込みの結果 `status` を、画面の下の行に出す
    pub(crate) fn set_operation_status(&mut self, status: OperationStatus) {
        self.set_notice(Notice::Operation(status));
    }

    /// メッセージの一覧に出す、選んだ工程の状態（今の条件の計算の状態・空間モデルの CL の状態・今の条件の誤り）。加工手順を
    /// 開いていなければ空
    pub(crate) fn notice_details(&self) -> Vec<(NoticeLevel, String)> {
        if !self.machining_sequence_open {
            return Vec::new();
        }
        let locale = self.ui_locale;
        let job_status = &self.cutting_simulation_job_status;
        // 失敗・読めない計算は失敗、前の条件の CL は注意の色で出す
        let job_level = match job_status {
            JobStatusLine::Status(OperationJobStatus::Failed)
            | JobStatusLine::Unreadable
            | JobStatusLine::ManagerNotFound => NoticeLevel::Error,
            JobStatusLine::Status(OperationJobStatus::Canceled) => NoticeLevel::Warning,
            _ => NoticeLevel::Info,
        };
        let toolpath = self.document.imported_toolpath_state();
        let toolpath_level = match toolpath {
            ImportedToolpathState::Outdated => NoticeLevel::Warning,
            ImportedToolpathState::None | ImportedToolpathState::Current => NoticeLevel::Info,
        };
        let mut details = vec![
            (job_level, job_status_text(locale, job_status)),
            (toolpath_level, imported_toolpath_text(locale, toolpath)),
        ];
        details.extend(
            self.condition_issues
                .iter()
                .flatten()
                .map(|issue| (NoticeLevel::Error, condition_issue_text(locale, issue))),
        );
        details
    }

    /// メッセージの一覧を開いたら、まだ開いていないメッセージの印を消す
    pub(crate) fn apply_notice_bar_state(&mut self, log_opened: bool) {
        if log_opened {
            self.unread_notice = None;
        }
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
