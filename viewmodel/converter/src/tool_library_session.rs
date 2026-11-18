//! ツールライブラリの画面の状態（設計: TOOL_LIBRARY_DESIGN.md 4・6.1）
//!
//! 使うライブラリのファイルの一覧（`ToolLibraryCatalog`）を持ち、画面に出す行と、画面の操作（ライブラリを作る・
//! 一覧に足す・外す・名前を替える、ツールセットを足す・替える・消す・動かす）を、失敗の理由の文言のキーとともに返す。

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use application::machining_sequence::{MachiningOperationContent, Tool, ToolSetEntry};
pub use application::machining_sequence::{ToolSetId, ToolSetReference};
use application::tool_library::OperationToolError;
pub use application::tool_library::OperationToolStatus;
use application::tool_library::{
    ToolLibraryCatalog, ToolLibraryEdit, ToolLibraryEditError, ToolLibraryFileError,
    ToolLibrarySaveInfo, ToolLibraryStatus, aligned_operation_content, content_with_toolset,
    register_operation_tool, remove_tool_library_lock, tool_library_lock,
};
use i18n_foundation::{UiMessage, UiMessageArg};

use crate::cutting_simulation_session::CuttingToolKind;
use crate::failure_message_mapper::{
    tool_library_edit_failure_message, tool_library_file_failure_message,
};

/// 一覧の 1 つのライブラリの状態
#[derive(Debug, Clone, PartialEq)]
pub enum ToolLibraryState {
    /// 読めて、書ける
    Writable,
    /// 読めるが、書けない（読むだけの権限・読むだけの共有）
    ReadOnly,
    /// 読めない（理由。前に読めた内容があれば、読むだけで使う）
    Unreadable(UiMessage),
    /// 一覧の前のファイル `first` と同じライブラリ（このファイルは使わない）
    Duplicate { first: PathBuf },
}

/// 一覧の 1 つのライブラリの行
#[derive(Debug, Clone, PartialEq)]
pub struct ToolLibraryRow {
    /// ライブラリの名前（読めたことがない・使わないファイルは `None`）
    pub name: Option<String>,
    /// ファイルの場所
    pub path: PathBuf,
    pub state: ToolLibraryState,
}

impl ToolLibraryRow {
    /// ツールセットを編集できるか
    pub fn editable(&self) -> bool {
        self.state == ToolLibraryState::Writable
    }
}

/// ライブラリの 1 つのツールセットの行
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSetRow {
    pub id: ToolSetId,
    pub name: String,
    /// 工具の種類（ボール・フラットエンドミルでないときは `None`）
    pub kind: Option<CuttingToolKind>,
    /// 径（mm）
    pub diameter: f64,
    /// コーナー R（mm）
    pub corner_radius: f64,
    /// 工具長（mm。長さに制限のない工具は `None`）
    pub cutting_length: Option<f64>,
}

/// 画面で編集するツールセットの値
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSetDraft {
    pub name: String,
    pub kind: CuttingToolKind,
    /// 径（mm）
    pub diameter: f64,
    /// 工具長（mm。`None` は長さに制限のない工具）
    pub cutting_length: Option<f64>,
}

/// 編集するツールセットの値の誤り
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSetDraftIssue {
    /// 名前が空
    EmptyName,
    /// 径が正の値でない
    DiameterNotPositive,
    /// 工具長が正の値でない
    CuttingLengthNotPositive,
}

impl ToolSetDraft {
    /// 行 `row` の値（ボール・フラットエンドミルでない工具は、ボールエンドミルとして編集する）
    pub fn of(row: &ToolSetRow) -> Self {
        Self {
            name: row.name.clone(),
            kind: row.kind.unwrap_or_default(),
            diameter: row.diameter,
            cutting_length: row.cutting_length,
        }
    }

    /// 値の誤り（なければ空）
    pub fn issues(&self) -> Vec<ToolSetDraftIssue> {
        let positive = |value: f64| value.is_finite() && value > 0.0;
        let mut issues = Vec::new();
        if self.name.trim().is_empty() {
            issues.push(ToolSetDraftIssue::EmptyName);
        }
        if !positive(self.diameter) {
            issues.push(ToolSetDraftIssue::DiameterNotPositive);
        }
        if self.cutting_length.is_some_and(|length| !positive(length)) {
            issues.push(ToolSetDraftIssue::CuttingLengthNotPositive);
        }
        issues
    }

    fn tool(&self) -> Tool<f64> {
        self.kind.tool(self.diameter, self.cutting_length)
    }

    fn entry(&self, id: ToolSetId) -> ToolSetEntry {
        ToolSetEntry {
            id,
            name: self.name.trim().to_string(),
            tool: self.tool(),
        }
    }
}

/// ツールライブラリの画面の操作の失敗
#[derive(Debug, Clone, PartialEq)]
pub enum ToolLibraryFailure {
    /// ほかの PC・利用者が書いている（書き手と、古いロックか。古いロックは、確かめてから消して書ける）
    Locked { owner: String, stale: bool },
    /// そのほかの失敗（理由）
    Failed(UiMessage),
}

impl ToolLibraryFailure {
    fn from_edit(error: &ToolLibraryEditError) -> Self {
        match error {
            ToolLibraryEditError::Locked(lock) => Self::Locked {
                owner: lock.owner.clone(),
                stale: lock.is_stale(now_ms()),
            },
            _ => Self::Failed(tool_library_edit_failure_message(error)),
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

/// ツールライブラリの画面の状態
#[derive(Debug)]
pub struct ToolLibrarySession {
    catalog: ToolLibraryCatalog,
    /// ロックのファイルに書く書き手（PC と利用者）
    owner: String,
    /// 保存するプログラムの版
    program_version: String,
}

impl ToolLibrarySession {
    /// ファイル `paths` の一覧を読む（書き手 `owner`、保存するプログラムの版 `program_version`）
    pub fn open(paths: Vec<PathBuf>, owner: String, program_version: String) -> Self {
        Self {
            catalog: ToolLibraryCatalog::open(paths),
            owner,
            program_version,
        }
    }

    fn save_info(&self) -> ToolLibrarySaveInfo {
        ToolLibrarySaveInfo {
            program_version: self.program_version.clone(),
            saved_at_ms: now_ms(),
        }
    }

    /// 一覧のファイルの場所（一覧の順。アプリの設定に保存する）
    pub fn paths(&self) -> Vec<PathBuf> {
        self.catalog.paths()
    }

    /// 変わったファイルを読み直す（画面を開くときに呼ぶ）
    pub fn refresh(&mut self) {
        self.catalog.refresh();
    }

    /// 一覧のライブラリの行（一覧の順）
    pub fn libraries(&self) -> Vec<ToolLibraryRow> {
        self.catalog
            .entries()
            .iter()
            .map(|entry| {
                let state = match entry.status() {
                    ToolLibraryStatus::Available { writable: true } => ToolLibraryState::Writable,
                    ToolLibraryStatus::Available { writable: false } => ToolLibraryState::ReadOnly,
                    ToolLibraryStatus::Unreadable(error) => {
                        ToolLibraryState::Unreadable(tool_library_file_failure_message(error))
                    }
                    ToolLibraryStatus::DuplicateId { first } => ToolLibraryState::Duplicate {
                        first: first.clone(),
                    },
                };
                ToolLibraryRow {
                    name: entry.library().map(|library| library.name.clone()),
                    path: entry.path().to_path_buf(),
                    state,
                }
            })
            .collect()
    }

    /// 一覧の `index` 番目のライブラリのツールセットの行（使えないライブラリは空）
    pub fn toolsets(&self, index: usize) -> Vec<ToolSetRow> {
        let Some(library) = self
            .catalog
            .entries()
            .get(index)
            .and_then(|entry| entry.library())
        else {
            return Vec::new();
        };
        library
            .toolsets
            .iter()
            .map(|entry| ToolSetRow {
                id: entry.id,
                name: entry.name.clone(),
                kind: CuttingToolKind::of(&entry.tool),
                diameter: entry.tool.diameter(),
                corner_radius: entry.tool.corner_radius(),
                cutting_length: entry.tool.cutting_length,
            })
            .collect()
    }

    /// 名前 `name` の新しいライブラリのファイルを `path` に作って一覧に足し、その番号を返す
    ///
    /// # Errors
    ///
    /// 名前が空・ファイルがすでにある・書けない場合（理由）。
    pub fn create_library(&mut self, path: PathBuf, name: &str) -> Result<usize, UiMessage> {
        let info = self.save_info();
        self.catalog
            .create(path, name.trim().to_string(), &info)
            .map_err(|error| tool_library_file_failure_message(&error))
    }

    /// ファイル `path` を一覧に足して読み、その番号を返す（すでにあれば、その番号）
    pub fn add_library(&mut self, path: PathBuf) -> usize {
        self.catalog.add(path)
    }

    /// 一覧の `index` 番目のライブラリを一覧から外す（ファイルは消さない）
    pub fn remove_library(&mut self, index: usize) {
        self.catalog.remove(index);
    }

    fn edit(&mut self, index: usize, edit: ToolLibraryEdit) -> Result<(), ToolLibraryFailure> {
        let info = self.save_info();
        let owner = self.owner.clone();
        self.catalog
            .edit(index, edit, &owner, &info)
            .map_err(|error| ToolLibraryFailure::from_edit(&error))
    }

    /// 一覧の `index` 番目のライブラリの名前を `name` に替える
    ///
    /// # Errors
    ///
    /// 書けない・ほかの PC が書いている・名前が空の場合。
    pub fn rename_library(&mut self, index: usize, name: &str) -> Result<(), ToolLibraryFailure> {
        self.edit(index, ToolLibraryEdit::Rename(name.trim().to_string()))
    }

    /// 一覧の `index` 番目のライブラリの最後に、値 `draft` の新しいツールセットを足し、その ID を返す
    ///
    /// # Errors
    ///
    /// 書けない・ほかの PC が書いている・値に誤りがある場合。
    pub fn add_toolset(
        &mut self,
        index: usize,
        draft: &ToolSetDraft,
    ) -> Result<ToolSetId, ToolLibraryFailure> {
        let id = ToolSetId::new_random();
        self.edit(index, ToolLibraryEdit::AddToolSet(draft.entry(id)))?;
        Ok(id)
    }

    /// 一覧の `index` 番目のライブラリのツールセット `id` を、値 `draft` に替える
    ///
    /// # Errors
    ///
    /// 書けない・ほかの PC が書いている・ほかの PC が消した・値に誤りがある場合。
    pub fn update_toolset(
        &mut self,
        index: usize,
        id: ToolSetId,
        draft: &ToolSetDraft,
    ) -> Result<(), ToolLibraryFailure> {
        self.edit(index, ToolLibraryEdit::UpdateToolSet(draft.entry(id)))
    }

    /// 一覧の `index` 番目のライブラリのツールセット `id` を消す
    ///
    /// # Errors
    ///
    /// 書けない・ほかの PC が書いている・ほかの PC が消した場合。
    pub fn remove_toolset(
        &mut self,
        index: usize,
        id: ToolSetId,
    ) -> Result<(), ToolLibraryFailure> {
        self.edit(index, ToolLibraryEdit::RemoveToolSet(id))
    }

    /// 一覧の `index` 番目のライブラリのツールセット `id` を、位置 `to` に動かす
    ///
    /// # Errors
    ///
    /// 書けない・ほかの PC が書いている・ほかの PC が消した場合。
    pub fn move_toolset(
        &mut self,
        index: usize,
        id: ToolSetId,
        to: usize,
    ) -> Result<(), ToolLibraryFailure> {
        self.edit(index, ToolLibraryEdit::MoveToolSet { id, index: to })
    }

    /// 使うライブラリの一覧（application の操作に渡す）
    pub(crate) fn catalog(&self) -> &ToolLibraryCatalog {
        &self.catalog
    }

    /// 工程の工具（選んだツールセットへの参照 `toolset` と工具の値の写し `tool`）の、ライブラリと比べた状態
    pub fn operation_tool_status(
        &self,
        toolset: Option<&ToolSetReference>,
        tool: &Tool<f64>,
    ) -> OperationToolStatus {
        self.catalog.operation_tool_status(toolset, tool)
    }

    /// 工程の参照 `toolset` のライブラリの一覧の番号（使えるライブラリにないときは `None`）
    pub fn library_index_of(&self, toolset: &ToolSetReference) -> Option<usize> {
        self.catalog
            .library(toolset.library)
            .map(|(index, _)| index)
    }

    /// 工程の中身 `content` の工具を、一覧の `index` 番目のライブラリのツールセット `toolset` に選び替えた中身
    /// （ライブラリ・ツールセットがないときは `None`）
    pub fn content_with_toolset(
        &self,
        content: &MachiningOperationContent,
        index: usize,
        toolset: ToolSetId,
    ) -> Option<MachiningOperationContent> {
        let library = self.catalog.entries().get(index)?.library()?;
        content_with_toolset(content, library, toolset).ok()
    }

    /// 工程の中身 `content` の工具を、参照しているツールセットの今の値と名前にそろえた中身（ないときは `None`）
    pub fn aligned_content(
        &self,
        content: &MachiningOperationContent,
    ) -> Option<MachiningOperationContent> {
        aligned_operation_content(content, &self.catalog).ok()
    }

    /// 工程の中身 `content` の工具の写しを、一覧の `index` 番目のライブラリに名前 `name` で登録し、参照をその
    /// ツールセットにした中身を返す
    ///
    /// # Errors
    ///
    /// 書けない・ほかの PC が書いている・名前が空・すでにライブラリにある工具の場合。
    pub fn register_operation_tool(
        &mut self,
        content: &MachiningOperationContent,
        index: usize,
        name: &str,
    ) -> Result<MachiningOperationContent, ToolLibraryFailure> {
        let info = self.save_info();
        let owner = self.owner.clone();
        register_operation_tool(
            content,
            &mut self.catalog,
            index,
            name.trim().to_string(),
            &owner,
            &info,
        )
        .map_err(|error| match error {
            OperationToolError::Library(error) => ToolLibraryFailure::from_edit(&error),
            OperationToolError::AlreadyInLibrary => {
                ToolLibraryFailure::Failed(message("failure.tool_library.already_registered"))
            }
            OperationToolError::ToolSetNotFound | OperationToolError::Sequence(_) => {
                ToolLibraryFailure::Failed(message("failure.tool_library.unavailable"))
            }
        })
    }

    /// 一覧の `index` 番目のライブラリの、古いロックのファイルを消す（利用者に確かめてから呼ぶ）
    ///
    /// # Errors
    ///
    /// 消せない場合（理由）。ロックがない・古くない場合は、何もしない。
    pub fn remove_stale_lock(&mut self, index: usize) -> Result<(), UiMessage> {
        let Some(path) = self
            .catalog
            .entries()
            .get(index)
            .map(|entry| entry.path().to_path_buf())
        else {
            return Ok(());
        };
        let stale = tool_library_lock(&path).is_some_and(|lock| lock.is_stale(now_ms()));
        if !stale {
            return Ok(());
        }
        remove_tool_library_lock(&path)
            .map_err(|error| tool_library_file_failure_message(&ToolLibraryFileError::Io(error)))
    }
}

fn message(key: &str) -> UiMessage {
    UiMessage {
        key: key.to_string(),
        args: Vec::new(),
    }
}

/// ファイル `path` の名前（画面に出す。名前のないパスはパス全体）
pub fn file_label(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// ほかの PC・利用者が書いていることの文言の引数
pub fn locked_message(owner: &str) -> UiMessage {
    UiMessage {
        key: "failure.tool_library.locked".to_string(),
        args: vec![UiMessageArg {
            name: "owner".to_string(),
            value: owner.to_string(),
        }],
    }
}
