//! AppState のファイルを選ぶ画面（別のスレッドで開き、選んだ結果をイベントの処理が終わった後に受け取る）
//!
//! ファイルを選ぶ画面を、閉じるまで待つ形でイベントの処理の中で開くと、Windows では、閉じるまでの間アプリの
//! ウィンドウの描き直しの依頼が切れ目なく続き、タイマーの合図が届かなくなる。ファイルを選ぶ画面は、時間のかかる
//! フォルダ（PC・ダウンロードなど）の中身をタイマーの合図で並べるため、止まってしまう。そのため、画面は別の
//! スレッドで開き、アプリの画面は動かし続ける。

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use super::AppState;
use super::model_file::FileAction;

/// 選んだファイル（選ばずに閉じたときは `None`）
type Picked = Option<Vec<PathBuf>>;

/// ファイルを選ぶ画面で選ぶもの（選んだ後に行うこと）
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FileDialogPurpose {
    /// モデルファイルを開く
    OpenModel,
    /// 名前を付けて保存する（保存できたら、続けてファイルの操作 `then` を行う）
    SaveModelAs { then: Option<FileAction> },
    /// 形状ファイルを今の空間モデルに加える
    LoadShape,
    /// ツールライブラリを新しく作る（`source` があれば、その番号のライブラリを複写する）
    NewToolLibrary { source: Option<usize> },
    /// ツールライブラリのファイルを一覧に足す
    OpenToolLibraries,
}

/// 開いているファイルを選ぶ画面
pub(super) struct PendingFileDialog {
    purpose: FileDialogPurpose,
    picked: Pin<Box<dyn Future<Output = Picked>>>,
}

/// 1 つのファイルを選ぶ画面 `picking` の結果を、選んだファイルの一覧にする
pub(super) fn one_file(
    picking: impl Future<Output = Option<rfd::FileHandle>> + 'static,
) -> Pin<Box<dyn Future<Output = Picked>>> {
    Box::pin(async move { picking.await.map(|file| vec![file.path().to_path_buf()]) })
}

/// いくつかのファイルを選ぶ画面 `picking` の結果を、選んだファイルの一覧にする
pub(super) fn many_files(
    picking: impl Future<Output = Option<Vec<rfd::FileHandle>>> + 'static,
) -> Pin<Box<dyn Future<Output = Picked>>> {
    Box::pin(async move {
        picking
            .await
            .map(|files| files.iter().map(|file| file.path().to_path_buf()).collect())
    })
}

impl AppState {
    /// ファイルを選ぶ画面を開いているか（開いている間は、ファイルの操作・ツールライブラリの操作を待たせる）
    pub(super) fn file_dialog_open(&self) -> bool {
        self.file_dialog.is_some()
    }

    /// ファイルを選ぶ画面 `picked` を、選ぶもの `purpose` で開く
    pub(super) fn open_file_dialog(
        &mut self,
        purpose: FileDialogPurpose,
        picked: Pin<Box<dyn Future<Output = Picked>>>,
    ) {
        self.file_dialog = Some(PendingFileDialog { purpose, picked });
        // 画面は、初めて結果を問い合わせたときに開く
        self.receive_file_dialog_result();
    }

    /// ファイルを選ぶ画面が閉じていれば、選んだ結果を受け取り、選んだ後のことを行う
    pub(super) fn receive_file_dialog_result(&mut self) {
        let Some(dialog) = self.file_dialog.as_mut() else {
            return;
        };
        let mut context = Context::from_waker(Waker::noop());
        let Poll::Ready(picked) = dialog.picked.as_mut().poll(&mut context) else {
            return;
        };
        let purpose = dialog.purpose.clone();
        self.file_dialog = None;
        let paths = picked.unwrap_or_default();
        match purpose {
            FileDialogPurpose::OpenModel => {
                if let Some(path) = paths.first() {
                    self.open_model_file(path);
                }
            }
            FileDialogPurpose::SaveModelAs { then } => {
                let saved = paths
                    .first()
                    .is_some_and(|path| self.save_document_to_new_file(path.clone()));
                // 保存できたら続けて行う（ほかに要求された操作より先に）
                if saved && let Some(action) = then {
                    self.file_actions.insert(0, action);
                }
            }
            // 選んだファイルは、落としたファイルと同じく、要求として積む（実行中のコマンドが許さなければ待つ）
            FileDialogPurpose::LoadShape => {
                if let Some(path) = paths.first() {
                    self.request_file_action(FileAction::LoadPath(path.clone()));
                }
            }
            FileDialogPurpose::NewToolLibrary { source } => {
                if let Some(path) = paths.first() {
                    self.create_tool_library_file(path.clone(), source);
                }
            }
            FileDialogPurpose::OpenToolLibraries => self.add_tool_library_files(paths),
        }
    }
}
