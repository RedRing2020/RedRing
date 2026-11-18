//! 使うツールライブラリのファイルの一覧（設計: TOOL_LIBRARY_DESIGN.md 4.1・4.4）
//!
//! 一覧のファイルを読み、更新の時刻と大きさが変わったファイルだけを読み直す。読めないファイルは、前に読めた内容が
//! あればそれを読むだけで使う。同じライブラリの ID のファイルが 2 つあれば、後のものを使わない。

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::shared::{ToolLibraryEdit, ToolLibraryEditError, edit_tool_library, writable};
use crate::machining_sequence::{ToolSetEntry, ToolSetId};

use super::{
    ToolLibrary, ToolLibraryFileError, ToolLibrarySaveInfo, load_tool_library, save_tool_library,
};

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
            // 中身は読み直さないが、権限の変更などで読めなくなっていないかは確かめる
            self.status = match std::fs::File::open(&self.path) {
                Ok(_) => ToolLibraryStatus::Available {
                    writable: writable(&self.path).is_ok(),
                },
                Err(error) => ToolLibraryStatus::Unreadable(ToolLibraryFileError::Io(error)),
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

/// まだ読んでいない、一覧の 1 つのファイル
fn unread_entry(path: PathBuf) -> ToolLibraryCatalogEntry {
    ToolLibraryCatalogEntry {
        path,
        status: ToolLibraryStatus::Unreadable(ToolLibraryFileError::Io(
            std::io::ErrorKind::NotFound.into(),
        )),
        library: None,
        stamp: None,
    }
}

/// 使うツールライブラリのファイルの一覧（一覧の順に持つ）
#[derive(Debug, Default)]
pub struct ToolLibraryCatalog {
    entries: Vec<ToolLibraryCatalogEntry>,
}

impl ToolLibraryCatalog {
    /// ファイル `paths` の一覧を作って読む（同じ場所は 1 回だけ持つ）
    pub fn open(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut catalog = Self::default();
        for path in paths {
            if !catalog.contains(&path) {
                catalog.entries.push(unread_entry(path));
            }
        }
        catalog.refresh();
        catalog
    }

    /// 一覧のファイル（一覧の順）
    pub fn entries(&self) -> &[ToolLibraryCatalogEntry] {
        &self.entries
    }

    /// 一覧のファイルの場所（一覧の順。アプリの設定に保存する）
    pub fn paths(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .map(|entry| entry.path.clone())
            .collect()
    }

    fn contains(&self, path: &Path) -> bool {
        self.entries.iter().any(|entry| entry.path == path)
    }

    /// ファイル `path` を一覧の最後に足して読み、その番号を返す（同じ場所がすでにあれば、足さずにその番号を返す）
    pub fn add(&mut self, path: PathBuf) -> usize {
        if let Some(index) = self.entries.iter().position(|entry| entry.path == path) {
            return index;
        }
        let mut entry = unread_entry(path);
        entry.refresh();
        self.entries.push(entry);
        self.mark_duplicates();
        self.entries.len() - 1
    }

    /// 一覧の `index` 番目のファイルを一覧から外す（ファイルは消さない。一覧にない番号は何もしない）
    pub fn remove(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.refresh();
        }
    }

    /// 名前 `name` の、ツールセットのない新しいライブラリのファイルを `path` に作り（保存したときの情報 `info`）、
    /// 一覧の最後に足して、その番号を返す
    ///
    /// # Errors
    ///
    /// 名前が空の場合、`path` にすでにファイルがある場合（ほかのライブラリを上書きしない）、書けない場合。
    pub fn create(
        &mut self,
        path: PathBuf,
        name: String,
        info: &ToolLibrarySaveInfo,
    ) -> Result<usize, ToolLibraryFileError> {
        self.save_new(path, &ToolLibrary::new(name), info)
    }

    /// 一覧の `index` 番目のライブラリ（読めないときは前に読めた内容）のツールセットを写した、名前 `name` の別の
    /// ライブラリのファイルを `path` に作り（保存したときの情報 `info`）、一覧の最後に足して、その番号を返す
    ///
    /// ライブラリの ID とツールセットの ID は作り直す（写したライブラリを、元と同じライブラリとして扱わないため）。
    ///
    /// # Errors
    ///
    /// 一覧にない・使えないライブラリの場合（[`ToolLibraryFileError::Io`] の `NotFound`）、名前が空の場合、`path`
    /// にすでにファイルがある場合、書けない場合。
    pub fn duplicate(
        &mut self,
        index: usize,
        path: PathBuf,
        name: String,
        info: &ToolLibrarySaveInfo,
    ) -> Result<usize, ToolLibraryFileError> {
        let source = self
            .entries
            .get(index)
            .and_then(ToolLibraryCatalogEntry::library)
            .ok_or_else(|| ToolLibraryFileError::Io(std::io::ErrorKind::NotFound.into()))?;
        let mut library = ToolLibrary::new(name);
        library.toolsets = source
            .toolsets
            .iter()
            .map(|entry| ToolSetEntry {
                id: ToolSetId::new_random(),
                ..entry.clone()
            })
            .collect();
        self.save_new(path, &library, info)
    }

    /// 新しいライブラリ `library` のファイルを `path` に書き、一覧の最後に足して、その番号を返す（名前が空・ファイルが
    /// すでにあれば書かない）
    fn save_new(
        &mut self,
        path: PathBuf,
        library: &ToolLibrary,
        info: &ToolLibrarySaveInfo,
    ) -> Result<usize, ToolLibraryFileError> {
        if library.name.is_empty() {
            return Err(ToolLibraryFileError::InvalidSection {
                kind: super::LINF,
                field: "library.name",
            });
        }
        if path.exists() {
            return Err(ToolLibraryFileError::Io(
                std::io::ErrorKind::AlreadyExists.into(),
            ));
        }
        save_tool_library(&path, library, info)?;
        Ok(self.add(path))
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
