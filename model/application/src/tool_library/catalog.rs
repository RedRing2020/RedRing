//! 使うツールライブラリのファイルの一覧（設計: TOOL_LIBRARY_DESIGN.md 4.1・4.4）
//!
//! 一覧のファイルを読み、更新の時刻と大きさが変わったファイルだけを読み直す。読めないファイルは、前に読めた内容が
//! あればそれを読むだけで使う。同じライブラリの ID のファイルが 2 つあれば、後のものを使わない。

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::shared::{ToolLibraryEdit, ToolLibraryEditError, edit_tool_library, writable};
use super::{ToolLibrary, ToolLibraryFileError, ToolLibrarySaveInfo, load_tool_library};

/// 読んだときのファイルの更新の時刻と大きさ（同じなら読み直さない）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(FileStamp {
        modified: metadata.modified().ok(),
        len: metadata.len(),
    })
}

/// 一覧の 1 つのファイルの状態
#[derive(Debug)]
pub enum ToolLibraryStatus {
    /// 読めた（書けないファイルは `writable` が偽）
    Available { writable: bool },
    /// 読めない（前に読めた内容があれば、[`ToolLibraryCatalogEntry::library`] で読むだけで使う）
    Unreadable(ToolLibraryFileError),
    /// 一覧の前のファイル `first` と、同じライブラリの ID（このファイルは使わない）
    DuplicateId { first: PathBuf },
}

/// 一覧の 1 つのファイル
#[derive(Debug)]
pub struct ToolLibraryCatalogEntry {
    path: PathBuf,
    status: ToolLibraryStatus,
    /// 最後に読めた内容
    library: Option<ToolLibrary>,
    /// `library` を読んだときのファイルの更新の時刻と大きさ
    stamp: Option<FileStamp>,
}

impl ToolLibraryCatalogEntry {
    /// ファイルの場所
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn status(&self) -> &ToolLibraryStatus {
        &self.status
    }

    /// 使えるライブラリの内容（読めないときは前に読めた内容。読めたことがない・ID が重なるときは `None`）
    pub fn library(&self) -> Option<&ToolLibrary> {
        match self.status {
            ToolLibraryStatus::DuplicateId { .. } => None,
            _ => self.library.as_ref(),
        }
    }

    /// ファイルが前に読んだときから変わっていれば（読めなかったものは毎回）読み直す
    fn refresh(&mut self) {
        let stamp = file_stamp(&self.path);
        let unchanged = matches!(self.status, ToolLibraryStatus::Available { .. })
            && stamp.is_some()
            && stamp == self.stamp;
        if unchanged {
            self.status = ToolLibraryStatus::Available {
                writable: writable(&self.path).is_ok(),
            };
            return;
        }
        match load_tool_library(&self.path) {
            Ok((library, _)) => {
                self.library = Some(library);
                self.stamp = stamp;
                self.status = ToolLibraryStatus::Available {
                    writable: writable(&self.path).is_ok(),
                };
            }
            Err(error) => self.status = ToolLibraryStatus::Unreadable(error),
        }
    }
}

/// 使うツールライブラリのファイルの一覧（一覧の順に持つ）
#[derive(Debug, Default)]
pub struct ToolLibraryCatalog {
    entries: Vec<ToolLibraryCatalogEntry>,
}

impl ToolLibraryCatalog {
    /// ファイル `paths` の一覧を作って読む
    pub fn open(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut catalog = Self {
            entries: paths
                .into_iter()
                .map(|path| ToolLibraryCatalogEntry {
                    path,
                    status: ToolLibraryStatus::Unreadable(ToolLibraryFileError::Io(
                        std::io::ErrorKind::NotFound.into(),
                    )),
                    library: None,
                    stamp: None,
                })
                .collect(),
        };
        catalog.refresh();
        catalog
    }

    /// 一覧のファイル（一覧の順）
    pub fn entries(&self) -> &[ToolLibraryCatalogEntry] {
        &self.entries
    }

    /// 変わったファイルを読み直し、ライブラリの ID の重なりを確かめ直す
    pub fn refresh(&mut self) {
        for entry in &mut self.entries {
            if matches!(entry.status, ToolLibraryStatus::DuplicateId { .. }) {
                // 重なりは読み直した内容で確かめ直す
                entry.status = ToolLibraryStatus::Unreadable(ToolLibraryFileError::Io(
                    std::io::ErrorKind::NotFound.into(),
                ));
                entry.stamp = None;
            }
            entry.refresh();
        }
        self.mark_duplicates();
    }

    /// 一覧の前のファイルと同じライブラリの ID のファイルを、使わないものにする
    fn mark_duplicates(&mut self) {
        for at in 0..self.entries.len() {
            let Some(id) = self.entries[at].library().map(|library| library.id) else {
                continue;
            };
            let first = self.entries[..at]
                .iter()
                .find(|entry| entry.library().is_some_and(|library| library.id == id));
            if let Some(first) = first {
                let first = first.path.clone();
                self.entries[at].status = ToolLibraryStatus::DuplicateId { first };
            }
        }
    }

    /// 一覧の `index` 番目のファイルに操作 `edit` を当てて書き（書き手 `owner`、保存したときの情報 `info`）、書いた
    /// 内容を一覧に映す
    ///
    /// # Errors
    ///
    /// 一覧にない番号・読めない・ID が重なるファイルの場合（[`ToolLibraryEditError::Unavailable`]）、書けない
    /// ファイルの場合（[`ToolLibraryEditError::ReadOnly`]）と、[`edit_tool_library`] の誤り。
    pub fn edit(
        &mut self,
        index: usize,
        edit: ToolLibraryEdit,
        owner: &str,
        info: &ToolLibrarySaveInfo,
    ) -> Result<(), ToolLibraryEditError> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or(ToolLibraryEditError::Unavailable)?;
        match entry.status {
            ToolLibraryStatus::Available { writable: true } => {}
            ToolLibraryStatus::Available { writable: false } => {
                return Err(ToolLibraryEditError::ReadOnly(
                    std::io::ErrorKind::PermissionDenied.into(),
                ));
            }
            ToolLibraryStatus::Unreadable(_) | ToolLibraryStatus::DuplicateId { .. } => {
                return Err(ToolLibraryEditError::Unavailable);
            }
        }
        let result = edit_tool_library(&entry.path, edit, owner, info);
        match result {
            Ok(library) => {
                entry.library = Some(library);
                entry.stamp = file_stamp(&entry.path);
                Ok(())
            }
            // 書けなかったときも、ほかの PC が書いた内容を映す
            Err(error) => {
                entry.refresh();
                Err(error)
            }
        }
    }
}
