//! AppState のモデルファイルの操作（新規・開く・保存・名前を付けて保存・読み込み・終了）と、保存していない変更の
//! 確かめを扱うモジュール。
//!
//! 確かめの画面は描画を止めるため、操作は要求として積み、イベントの処理が終わった後に行う。ファイルを選ぶ画面は
//! 別のスレッドで開き、閉じるまで残りの操作を待たせる。

use std::path::{Path, PathBuf};
use std::time::Instant;

use super::AppState;
use super::file_dialog::{FileDialogPurpose, one_file};
use crate::help_ui::Notice;
use crate::menu_bar_messages::resolve_menu_bar_text;
use viewmodel::space_model_document::SpaceModelDocument;

/// 読み込める形状ファイルの形式（ファイルを開く画面の種類の名前と拡張子。大文字・小文字を区別する OS のため両方）
/// STEP ファイルの拡張子（大文字と小文字は区別しない）
const STEP_EXTENSIONS: &[&str] = &["step", "stp", "STEP", "STP"];
const SHAPE_FILE_FORMATS: &[(&str, &[&str])] =
    &[("STL", &["stl", "STL"]), ("STEP", STEP_EXTENSIONS)];

/// STEP ファイルか（拡張子で決める）
pub(super) fn is_step_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| {
        STEP_EXTENSIONS
            .iter()
            .any(|step| extension.eq_ignore_ascii_case(step))
    })
}

/// モデルファイルの拡張子
const MODEL_FILE_EXTENSION: &str = "rrm";

/// モデルファイルを選ぶ画面の拡張子（大文字・小文字を区別する OS のため両方）
const MODEL_FILE_EXTENSIONS: &[&str] = &["rrm", "RRM"];

/// モデルファイルの操作
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileAction {
    /// 空の空間モデルにする
    New,
    /// モデルファイルを選んで開く
    Open,
    /// モデルファイル（ドロップしたファイル）を開く
    OpenPath(PathBuf),
    /// 開いたファイルに保存する（まだなければ、名前を付けて保存）
    Save,
    /// ファイルを選んで保存する
    SaveAs,
    /// 形状ファイルを選んで、今の空間モデルに加える
    Load,
    /// 形状ファイル（ドロップしたファイル）を、今の空間モデルに加える
    LoadPath(PathBuf),
    /// アプリを終える
    Exit,
}

/// 保存していない変更の確かめの答え
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscardAnswer {
    Save,
    DontSave,
    Cancel,
}

impl AppState {
    /// モデルファイルの操作 `action` を要求する（イベントの処理が終わった後に、要求の順に行う。同じ操作がまだ残って
    /// いれば、重ねない）
    pub(crate) fn request_file_action(&mut self, action: FileAction) {
        // 工程の表で編集している名前は、確定してから行う（保存する文書に入れるため）。確定は、まだ入力欄が受け取って
        // いない打った文字を入れるよう、次の画面の描画で行う
        if self.is_operation_name_editing() {
            self.operation_name_confirm_requested = true;
        }
        if !self.file_actions.contains(&action) {
            self.file_actions.push(action);
        }
    }

    /// ウィンドウに落としたファイルを、モデルファイルなら開き、それ以外は形状ファイルとして読み込む（いくつか落とした
    /// ときは、落とした順に行う）
    pub fn drop_file(&mut self, path: PathBuf) {
        if is_model_file(&path) {
            self.request_file_action(FileAction::OpenPath(path));
        } else {
            self.request_file_action(FileAction::LoadPath(path));
        }
    }

    /// アプリを終えてよいことが決まったか（保存していない変更を確かめた後）
    pub fn exit_confirmed(&self) -> bool {
        self.exit_confirmed
    }

    /// 要求されたモデルファイルの操作を、要求の順に行う（ファイルを選ぶ画面を開いたら、閉じるまで残りを待たせる）
    pub(super) fn run_file_actions(&mut self) {
        if self.file_dialog_open() {
            return;
        }
        // 編集している名前を確定するまで、ファイルの操作を待たせる（最小化して描かない間は、待たずに今の名前で確定する）
        if self.operation_name_confirm_requested && self.is_operation_name_editing() {
            if self.window.is_minimized() != Some(true) {
                return;
            }
            self.operation_name_confirm_requested = false;
            self.confirm_operation_name_edit();
        }
        while !self.file_actions.is_empty() && !self.exit_confirmed && !self.file_dialog_open() {
            let action = self.file_actions.remove(0);
            self.run_file_action(action);
        }
    }

    fn run_file_action(&mut self, action: FileAction) {
        match action {
            FileAction::New => {
                if self.confirm_discard(&FileAction::New) {
                    let document = super::machining_conditions::with_default_names(
                        self.ui_locale,
                        SpaceModelDocument::new,
                    );
                    self.replace_document(document);
                    self.set_notice(Notice::ModelCreated);
                }
            }
            FileAction::Open => {
                if self.confirm_discard(&FileAction::Open) {
                    let picking = self.model_file_dialog().pick_file();
                    self.open_file_dialog(FileDialogPurpose::OpenModel, one_file(picking));
                }
            }
            FileAction::OpenPath(path) => {
                if self.confirm_discard(&FileAction::OpenPath(path.clone())) {
                    self.open_model_file(&path);
                }
            }
            FileAction::Save => {
                self.save_document(None);
            }
            FileAction::SaveAs => self.save_document_as(None),
            FileAction::Load => self.load_shape_file(),
            FileAction::LoadPath(path) => self.import_product_shape(&path),
            FileAction::Exit => {
                if self.confirm_discard(&FileAction::Exit) {
                    self.exit_confirmed = true;
                }
            }
        }
    }

    /// 保存していない変更があれば、保存するかを確かめる。続けてよければ `true`（保存した・保存しない・変更がない）。
    /// まだ名前のない文書を保存するときは、名前を付けて保存する画面を開いて `false` を返し、保存できたら操作 `then`
    /// をし直す
    fn confirm_discard(&mut self, then: &FileAction) -> bool {
        if !self.document.is_modified() {
            return true;
        }
        match self.ask_discard() {
            DiscardAnswer::Save => self.save_document(Some(then.clone())),
            DiscardAnswer::DontSave => true,
            DiscardAnswer::Cancel => false,
        }
    }

    fn ask_discard(&self) -> DiscardAnswer {
        let text = |key| resolve_menu_bar_text(self.ui_locale, key);
        let (save, dont_save, cancel) = (
            text("file.confirm.save"),
            text("file.confirm.dont_save"),
            text("file.confirm.cancel"),
        );
        let answer = rfd::MessageDialog::new()
            .set_parent(&*self.window)
            .set_level(rfd::MessageLevel::Warning)
            .set_title(crate::app::APP_TITLE)
            .set_description(text("file.confirm.unsaved"))
            .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
                save.clone(),
                dont_save.clone(),
                cancel,
            ))
            .show();
        match answer {
            rfd::MessageDialogResult::Yes => DiscardAnswer::Save,
            rfd::MessageDialogResult::No => DiscardAnswer::DontSave,
            rfd::MessageDialogResult::Custom(label) if label == save => DiscardAnswer::Save,
            rfd::MessageDialogResult::Custom(label) if label == dont_save => {
                DiscardAnswer::DontSave
            }
            _ => DiscardAnswer::Cancel,
        }
    }

    /// 開いたファイルに保存する。保存したら `true`。まだなければ、名前を付けて保存する画面を開いて `false` を返す
    /// （保存できたら、続けて操作 `then` を行う）
    fn save_document(&mut self, then: Option<FileAction>) -> bool {
        match self.document.path().map(Path::to_path_buf) {
            Some(path) => self.save_document_to(&path),
            None => {
                self.save_document_as(then);
                false
            }
        }
    }

    /// 名前を付けて保存する画面を開く（保存できたら、続けて操作 `then` を行う）
    fn save_document_as(&mut self, then: Option<FileAction>) {
        let mut dialog = self.model_file_dialog();
        dialog = match self.document.path() {
            Some(path) => match path.file_name() {
                Some(name) => dialog.set_file_name(name.to_string_lossy()),
                None => dialog,
            },
            None => dialog.set_file_name(format!(
                "{}.{MODEL_FILE_EXTENSION}",
                resolve_menu_bar_text(self.ui_locale, "file.untitled")
            )),
        };
        let picking = dialog.save_file();
        self.open_file_dialog(FileDialogPurpose::SaveModelAs { then }, one_file(picking));
    }

    /// 名前を付けて保存する画面で選んだ `path` に保存する（モデルファイルの拡張子がなければ足す）。保存したら `true`
    pub(super) fn save_document_to_new_file(&mut self, path: PathBuf) -> bool {
        self.save_document_to(&with_model_file_extension(path))
    }

    fn save_document_to(&mut self, path: &Path) -> bool {
        let file = file_name(path);
        let started = Instant::now();
        let saved = self.document.save_as(path, env!("CARGO_PKG_VERSION"));
        let elapsed_ms = millis_since(started);
        match saved {
            Ok(()) => {
                tracing::info!(
                    path = %path.display(),
                    elapsed_ms,
                    bytes = file_bytes(path),
                    "model file saved"
                );
                self.set_notice(Notice::ModelSaved { file });
                true
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), elapsed_ms, "model file save failed: {error}");
                self.set_notice(Notice::ModelSaveFailed {
                    file,
                    reason: viewmodel::failure_message_mapper::model_file_failure_message(&error),
                });
                false
            }
        }
    }

    pub(super) fn open_model_file(&mut self, path: &Path) {
        let file = file_name(path);
        let started = Instant::now();
        let opened = super::machining_conditions::with_default_names(self.ui_locale, |names| {
            SpaceModelDocument::open(path, names)
        });
        let elapsed_ms = millis_since(started);
        match opened {
            Ok(document) => {
                // 読む時間と、製品形状を表示する時間を分けて出す
                let shown = Instant::now();
                self.replace_document(document);
                tracing::info!(
                    path = %path.display(),
                    elapsed_ms,
                    display_ms = millis_since(shown),
                    bytes = file_bytes(path),
                    "model file opened"
                );
                self.set_notice(Notice::ModelOpened { file });
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), elapsed_ms, "model file open failed: {error}");
                self.set_notice(Notice::ModelOpenFailed {
                    file,
                    reason: viewmodel::failure_message_mapper::model_file_failure_message(&error),
                });
            }
        }
    }

    /// 空間モデルの文書を `document` に置き換え、その製品形状を表示する
    fn replace_document(&mut self, document: SpaceModelDocument) {
        self.document = document;
        self.cutting_simulation.forget_condition_checks();
        // 開いたモデルファイル・新規の空間モデルでは、加工手順を開かない（CAM の表示は、製品形状を描き直すときに放す）
        self.forget_machining_sequence_state();
        self.show_product_shapes();
    }

    /// モデルファイルを選ぶ画面（開く・保存）
    fn model_file_dialog(&self) -> rfd::AsyncFileDialog {
        let mut dialog = rfd::AsyncFileDialog::new()
            .set_parent(&*self.window)
            .add_filter(
                resolve_menu_bar_text(self.ui_locale, "file.model_file"),
                MODEL_FILE_EXTENSIONS,
            );
        if let Some(folder) = self.document.path().and_then(Path::parent) {
            dialog = dialog.set_directory(folder);
        }
        dialog
    }

    /// 形状ファイルを選ぶ画面を開く（選んだファイルは、ドラッグ＆ドロップと同じ経路で読み込む）
    fn load_shape_file(&mut self) {
        let supported: Vec<&str> = SHAPE_FILE_FORMATS
            .iter()
            .flat_map(|(_, extensions)| extensions.iter().copied())
            .collect();
        let mut dialog = rfd::AsyncFileDialog::new()
            .set_parent(&*self.window)
            .add_filter(
                resolve_menu_bar_text(self.ui_locale, "menu.file.load.supported"),
                &supported,
            );
        for (name, extensions) in SHAPE_FILE_FORMATS {
            dialog = dialog.add_filter(*name, extensions);
        }
        let picking = dialog.pick_file();
        self.open_file_dialog(FileDialogPurpose::LoadShape, one_file(picking));
    }

    /// ウィンドウのタイトルの先頭（モデルファイルの名前と、保存していない変更の印）
    pub(super) fn document_title(&self) -> String {
        let name = self.document.path().map_or_else(
            || resolve_menu_bar_text(self.ui_locale, "file.untitled"),
            file_name,
        );
        let modified = if self.document.is_modified() { "*" } else { "" };
        format!("{name}{modified}")
    }

    /// ウィンドウのタイトルを今の文書に合わせる（文書のタイトルが変わったときだけ出し直す。切削シミュレーションの時点を
    /// 出している間は、その後ろに時点を出す）
    pub fn refresh_window_title(&mut self) {
        let document_title = self.document_title();
        if document_title == self.window_title {
            return;
        }
        self.window_title = document_title;
        if self.cutting_animation.is_some() && !self.debug_view_shown() {
            self.refresh_snapshot_window_title();
        } else {
            self.window.set_title(&format!(
                "{} - {}",
                self.window_title,
                crate::app::APP_TITLE
            ));
        }
    }
}

/// モデルファイル（拡張子が `.rrm`）か
fn is_model_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(MODEL_FILE_EXTENSION))
}

/// パス `path` のファイルの名前（名前がなければパスそのもの）
pub(super) fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// `started` から今までのミリ秒（時間のかかる処理のログに出す）
pub(super) fn millis_since(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// ファイル `path` の大きさ（バイト。読めなければ `None`。時間のかかる処理のログに出す）
pub(super) fn file_bytes(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|metadata| metadata.len())
}

/// モデルファイルの拡張子でなければ、モデルファイルの拡張子を足す（名前の中の `.` は残す）
fn with_model_file_extension(path: PathBuf) -> PathBuf {
    if is_model_file(&path) {
        return path;
    }
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(MODEL_FILE_EXTENSION);
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_files_always_get_the_model_file_extension() {
        assert_eq!(
            with_model_file_extension(PathBuf::from("a/model")),
            PathBuf::from("a/model.rrm")
        );
        assert_eq!(
            with_model_file_extension(PathBuf::from("a/model.rrm")),
            PathBuf::from("a/model.rrm")
        );
        assert_eq!(
            with_model_file_extension(PathBuf::from("a/model.RRM")),
            PathBuf::from("a/model.RRM")
        );
        assert_eq!(
            with_model_file_extension(PathBuf::from("a/part.v2")),
            PathBuf::from("a/part.v2.rrm")
        );
    }
}
