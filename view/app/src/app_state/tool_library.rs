//! AppState のツールライブラリのウィンドウ（使うライブラリの一覧の保存と、ライブラリ・ツールセットの操作）

use super::AppState;
use crate::help_ui::Notice;
use crate::tool_library_messages::resolve_tool_library_text;
use crate::tool_library_ui::{ToolLibraryAction, ToolLibraryUiState};
use std::path::PathBuf;
use viewmodel::failure_message_catalog::resolve_failure_message;
use viewmodel::message_catalog::{UiLocale, UiMessage};
use viewmodel::tool_library_session::{
    ToolLibraryFailure, ToolLibrarySession, ToolLibraryState, file_label, locked_message,
};

/// ツールライブラリのファイルの拡張子
const TOOL_LIBRARY_EXTENSIONS: &[&str] = &["rrtl"];

/// ロックのファイルに書く書き手（利用者と PC の名前。分からないものは除く）
fn lock_owner() -> String {
    let user = platform_foundation::identity::user_name();
    let computer = platform_foundation::identity::computer_name();
    match (user, computer) {
        (Some(user), Some(computer)) => format!("{user} @ {computer}"),
        (Some(name), None) | (None, Some(name)) => name,
        (None, None) => String::new(),
    }
}

/// 使うライブラリのファイル `paths` の一覧を読む（読めないライブラリは、ログに出す）
pub(super) fn open_tool_libraries(paths: Vec<PathBuf>) -> ToolLibrarySession {
    let session =
        ToolLibrarySession::open(paths, lock_owner(), env!("CARGO_PKG_VERSION").to_string());
    for row in session.libraries() {
        if let ToolLibraryState::Unreadable(reason) = &row.state {
            tracing::warn!(path = %row.path.display(), reason = %reason.key, "Cannot read the tool library");
        }
    }
    session
}

/// ツールライブラリの操作の結果
enum Outcome {
    /// 書いた（結果の文言）
    Done(String),
    /// 書けなかった（ライブラリのファイルの名前と理由）
    Failed { file: String, reason: UiMessage },
    /// 何もしなかった（選ぶ画面で選ばなかったなど）
    Nothing,
}

impl AppState {
    /// 「加工」→「ツールライブラリ…」: ウィンドウを開き、変わったライブラリを読み直す
    pub(crate) fn open_tool_library_window(&mut self) {
        self.tool_library.refresh();
        self.tool_library_window.open = true;
        let count = self.tool_library.libraries().len();
        if self.tool_library_window.selected_library >= count {
            self.tool_library_window.selected_library = count.saturating_sub(1);
        }
    }

    pub(crate) fn prepare_tool_library_state(&self) -> ToolLibraryUiState {
        let window = self.tool_library_window.clone();
        let toolsets = if window.open {
            self.tool_library.toolsets(window.selected_library)
        } else {
            Vec::new()
        };
        let selected_usage = window
            .selected_toolset
            .filter(|_| window.open)
            .map_or(0, |id| {
                self.tool_library
                    .toolset_usage(window.selected_library, id, &self.document)
            });
        ToolLibraryUiState {
            locale: self.ui_locale,
            selected_usage,
            libraries: if window.open {
                self.tool_library.libraries()
            } else {
                Vec::new()
            },
            toolsets,
            window,
            requested: None,
        }
    }

    pub(crate) fn apply_tool_library_state(&mut self, state: ToolLibraryUiState) {
        self.tool_library_window = state.window;
        if state.requested.is_some() {
            self.tool_library_action = state.requested;
        }
    }

    /// 要求されたツールライブラリの操作を行う（ファイルを選ぶ画面・確かめの画面は描画を止めるため、描画の途中では
    /// なく、イベントの処理が終わった後に呼ぶ）
    pub(super) fn run_tool_library_action(&mut self) {
        let Some(action) = self.tool_library_action.take() else {
            return;
        };
        let outcome = match action {
            ToolLibraryAction::CreateLibrary => self.create_tool_library(),
            ToolLibraryAction::OpenLibraries => self.add_tool_libraries(),
            ToolLibraryAction::DuplicateLibrary(index) => self.duplicate_tool_library(index),
            ToolLibraryAction::RemoveLibrary(index) => {
                self.tool_library.remove_library(index);
                let window = &mut self.tool_library_window;
                window.selected_library = window.selected_library.saturating_sub(1);
                window.selected_toolset = None;
                window.draft = None;
                window.rename = None;
                self.save_tool_library_list();
                Outcome::Nothing
            }
            edit => self.edit_tool_library(edit),
        };
        let locale = self.ui_locale;
        match outcome {
            Outcome::Done(text) => self.tool_library_window.result = Some((text, false)),
            Outcome::Failed { file, reason } => {
                let text = resolve_tool_library_text(
                    locale,
                    "tool_library.result.failed",
                    &[("reason", resolve_failure_message(locale, &reason))],
                );
                tracing::warn!(file = %file, reason = %reason.key, "Cannot write the tool library");
                self.tool_library_window.result = Some((text, true));
                self.set_notice(Notice::ToolLibraryFailed { file, reason });
            }
            Outcome::Nothing => {}
        }
    }

    /// ライブラリのファイルを選ぶ画面
    fn tool_library_dialog(&self) -> rfd::FileDialog {
        rfd::FileDialog::new().set_parent(&*self.window).add_filter(
            resolve_tool_library_text(self.ui_locale, "tool_library.file_type", &[]),
            TOOL_LIBRARY_EXTENSIONS,
        )
    }

    /// 場所を選んで、ファイルの名前を名前にした新しいライブラリを作り、一覧に足す
    fn create_tool_library(&mut self) -> Outcome {
        let Some((path, file, name)) = self.pick_new_tool_library_file() else {
            return Outcome::Nothing;
        };
        match self.tool_library.create_library(path, &name) {
            Ok(index) => {
                self.select_tool_library(index);
                self.save_tool_library_list();
                Outcome::Done(resolve_tool_library_text(
                    self.ui_locale,
                    "tool_library.result.created",
                    &[("file", file)],
                ))
            }
            Err(reason) => Outcome::Failed { file, reason },
        }
    }

    /// 場所を選んで、一覧の `index` 番目のライブラリを、ファイルの名前を名前にした別のライブラリとして複写し、一覧に
    /// 足す
    fn duplicate_tool_library(&mut self, index: usize) -> Outcome {
        let Some((path, file, name)) = self.pick_new_tool_library_file() else {
            return Outcome::Nothing;
        };
        match self.tool_library.duplicate_library(index, path, &name) {
            Ok(index) => {
                self.select_tool_library(index);
                self.save_tool_library_list();
                Outcome::Done(resolve_tool_library_text(
                    self.ui_locale,
                    "tool_library.result.created",
                    &[("file", file)],
                ))
            }
            Err(reason) => Outcome::Failed { file, reason },
        }
    }

    /// 新しいライブラリのファイルの場所を選ぶ（選んだ場所・ファイルの名前・拡張子を除いた名前。選ばなければ `None`）
    fn pick_new_tool_library_file(&self) -> Option<(PathBuf, String, String)> {
        let path = self.tool_library_dialog().save_file()?;
        let path = if path.extension().is_none() {
            path.with_extension(TOOL_LIBRARY_EXTENSIONS[0])
        } else {
            path
        };
        let file = file_label(&path);
        let name = path
            .file_stem()
            .map_or_else(|| file.clone(), |stem| stem.to_string_lossy().into_owned());
        Some((path, file, name))
    }

    /// 選んだライブラリのファイルを一覧に足す
    fn add_tool_libraries(&mut self) -> Outcome {
        let Some(paths) = self.tool_library_dialog().pick_files() else {
            return Outcome::Nothing;
        };
        let mut last = None;
        for path in paths {
            last = Some(self.tool_library.add_library(path));
        }
        if let Some(index) = last {
            self.select_tool_library(index);
            self.save_tool_library_list();
        }
        Outcome::Nothing
    }

    fn select_tool_library(&mut self, index: usize) {
        let window = &mut self.tool_library_window;
        window.selected_library = index;
        window.selected_toolset = None;
        window.draft = None;
        window.rename = None;
    }

    /// ライブラリへの書き込み（古いロックは、確かめてから消して 1 回だけ書き直す）
    fn edit_tool_library(&mut self, action: ToolLibraryAction) -> Outcome {
        let library = match &action {
            ToolLibraryAction::RenameLibrary { library, .. }
            | ToolLibraryAction::AddToolSet { library, .. }
            | ToolLibraryAction::UpdateToolSet { library, .. }
            | ToolLibraryAction::RemoveToolSet { library, .. }
            | ToolLibraryAction::MoveToolSet { library, .. } => *library,
            ToolLibraryAction::CreateLibrary
            | ToolLibraryAction::OpenLibraries
            | ToolLibraryAction::DuplicateLibrary(_)
            | ToolLibraryAction::RemoveLibrary(_) => return Outcome::Nothing,
        };
        let file = self
            .tool_library
            .libraries()
            .get(library)
            .map(|row| file_label(&row.path))
            .unwrap_or_default();
        let mut retried = false;
        loop {
            match self.apply_tool_library_edit(&action) {
                Ok(()) => {
                    return Outcome::Done(resolve_tool_library_text(
                        self.ui_locale,
                        "tool_library.result.saved",
                        &[],
                    ));
                }
                Err(ToolLibraryFailure::Locked { owner, stale }) => {
                    if stale && !retried && self.confirm_remove_stale_lock(&owner) {
                        if let Err(reason) = self.tool_library.remove_stale_lock(library) {
                            return Outcome::Failed { file, reason };
                        }
                        retried = true;
                        continue;
                    }
                    return Outcome::Failed {
                        file,
                        reason: locked_message(&owner),
                    };
                }
                Err(ToolLibraryFailure::Failed(reason)) => {
                    return Outcome::Failed { file, reason };
                }
            }
        }
    }

    fn apply_tool_library_edit(
        &mut self,
        action: &ToolLibraryAction,
    ) -> Result<(), ToolLibraryFailure> {
        let session = &mut self.tool_library;
        let window = &mut self.tool_library_window;
        match action {
            ToolLibraryAction::RenameLibrary { library, name } => {
                session.rename_library(*library, name)?;
                window.rename = None;
            }
            ToolLibraryAction::AddToolSet { library, draft } => {
                let id = session.add_toolset(*library, draft)?;
                window.selected_toolset = Some(id);
                window.draft = None;
            }
            ToolLibraryAction::UpdateToolSet { library, id, draft } => {
                session.update_toolset(*library, *id, draft)?;
                window.draft = None;
            }
            ToolLibraryAction::RemoveToolSet { library, id } => {
                session.remove_toolset(*library, *id)?;
                window.selected_toolset = None;
                window.draft = None;
            }
            ToolLibraryAction::MoveToolSet { library, id, to } => {
                session.move_toolset(*library, *id, *to)?;
            }
            ToolLibraryAction::CreateLibrary
            | ToolLibraryAction::OpenLibraries
            | ToolLibraryAction::DuplicateLibrary(_)
            | ToolLibraryAction::RemoveLibrary(_) => {}
        }
        Ok(())
    }

    /// 古いロックを消して書くかを確かめる
    fn confirm_remove_stale_lock(&self, owner: &str) -> bool {
        let description = stale_lock_text(self.ui_locale, owner);
        rfd::MessageDialog::new()
            .set_parent(&*self.window)
            .set_level(rfd::MessageLevel::Warning)
            .set_title(crate::app::APP_TITLE)
            .set_description(description)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes
    }

    /// 使うライブラリの一覧を、アプリの設定に保存する
    fn save_tool_library_list(&mut self) {
        self.app_settings
            .set_tool_libraries(self.tool_library.paths());
        self.save_app_settings();
    }
}

fn stale_lock_text(locale: UiLocale, owner: &str) -> String {
    resolve_tool_library_text(
        locale,
        "tool_library.lock.stale",
        &[("owner", owner.to_string())],
    )
}
