//! 加工工程の工具とツールライブラリの結び付き（設計: TOOL_LIBRARY_DESIGN.md 5.2）
//!
//! 工程は、選んだツールセットへの参照（属性）と、計算に使う工具の値の写し（計算の条件）を持つ。ライブラリを直しても
//! 工程は自動では替えず、比べた状態を出して、利用者がそろえる・選び替える・写しから登録する。

use std::fmt;

use super::catalog::ToolLibraryCatalog;
use super::shared::{ToolLibraryEdit, ToolLibraryEditError};
use super::{ToolLibrary, ToolLibrarySaveInfo};
use crate::machining_sequence::{
    MachiningOperationContent, MachiningSequenceError, MachiningSequenceId, MachiningSequenceStore,
    Tool, ToolLibraryId, ToolSetEntry, ToolSetId, ToolSetReference,
};

/// 加工工程の工具の、ツールライブラリと比べた状態（工程の計算の状態とは別。どの状態でも工程は計算できる）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationToolStatus {
    /// ライブラリのツールセットと、工具の値が同じ
    SameAsLibrary,
    /// ライブラリのツールセットと、工具の値が違う（ライブラリの値にそろえられる）
    DiffersFromLibrary,
    /// ライブラリはあるが、ツールセットがない（消された。写しから登録し直せる）
    NotInLibrary,
    /// ライブラリが、使うライブラリの一覧にない・読めたことがない
    LibraryNotFound,
    /// ライブラリから選んでいない（前の版のモデルファイル・ライブラリが空のときに作った工程）
    NotFromLibrary,
}

/// 加工工程の工具の操作の誤り
#[derive(Debug)]
pub enum OperationToolError {
    /// 保管していない加工手順・加工手順にない加工工程
    Sequence(MachiningSequenceError),
    /// 使えるライブラリに、そろえる・選ぶツールセットがない
    ToolSetNotFound,
    /// 工具のツールセットが、すでに使えるライブラリにある（写しから登録しない。そろえる・選び替える）
    AlreadyInLibrary,
    /// ライブラリに書けない（写しから登録するとき）
    Library(ToolLibraryEditError),
}

impl fmt::Display for OperationToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sequence(error) => write!(f, "{error}"),
            Self::ToolSetNotFound => write!(f, "the tool set is not in the tool libraries"),
            Self::AlreadyInLibrary => write!(f, "the tool set is already in a tool library"),
            Self::Library(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for OperationToolError {}

impl From<MachiningSequenceError> for OperationToolError {
    fn from(error: MachiningSequenceError) -> Self {
        Self::Sequence(error)
    }
}

impl From<ToolLibraryEditError> for OperationToolError {
    fn from(error: ToolLibraryEditError) -> Self {
        Self::Library(error)
    }
}

impl ToolLibraryCatalog {
    /// 一覧の使えるライブラリのうち、ID が `id` のもの（一覧の番号と内容。読めないものは前に読めた内容）
    pub fn library(&self, id: ToolLibraryId) -> Option<(usize, &ToolLibrary)> {
        self.entries()
            .iter()
            .enumerate()
            .find_map(|(index, entry)| Some((index, entry.library()?)).filter(|(_, l)| l.id == id))
    }

    /// 参照 `reference` のツールセット（使えるライブラリにないときは `None`）
    fn toolset(&self, reference: &ToolSetReference) -> Option<(&ToolLibrary, &ToolSetEntry)> {
        let (_, library) = self.library(reference.library)?;
        let entry = library
            .toolsets
            .iter()
            .find(|entry| entry.id == reference.toolset)?;
        Some((library, entry))
    }

    /// 加工工程の工具（選んだツールセットへの参照 `toolset` と工具の値の写し `tool`）の、ライブラリと比べた状態
    pub fn operation_tool_status(
        &self,
        toolset: Option<&ToolSetReference>,
        tool: &Tool<f64>,
    ) -> OperationToolStatus {
        let Some(reference) = toolset else {
            return OperationToolStatus::NotFromLibrary;
        };
        let Some((_, library)) = self.library(reference.library) else {
            return OperationToolStatus::LibraryNotFound;
        };
        match library
            .toolsets
            .iter()
            .find(|entry| entry.id == reference.toolset)
        {
            None => OperationToolStatus::NotInLibrary,
            Some(entry) if entry.tool == *tool => OperationToolStatus::SameAsLibrary,
            Some(_) => OperationToolStatus::DiffersFromLibrary,
        }
    }
}

/// 中身 `content` の工具を、ライブラリ `library` のツールセット `entry` の値と参照にした中身
fn with_toolset(
    content: &MachiningOperationContent,
    library: &ToolLibrary,
    entry: &ToolSetEntry,
) -> MachiningOperationContent {
    let mut content = content.clone();
    content.conditions.tool = entry.tool.clone();
    content.attributes.toolset = Some(ToolSetReference {
        library: library.id,
        toolset: entry.id,
        name: entry.name.clone(),
    });
    content
}

/// 中身 `content` の工具を、ライブラリ `library` のツールセット `toolset` に選び替えた中身（工具の値が今と同じなら、
/// 工程に入れても改訂は進まない）
///
/// # Errors
///
/// ライブラリにツールセットがない場合。
pub fn content_with_toolset(
    content: &MachiningOperationContent,
    library: &ToolLibrary,
    toolset: ToolSetId,
) -> Result<MachiningOperationContent, OperationToolError> {
    let entry = library
        .toolsets
        .iter()
        .find(|entry| entry.id == toolset)
        .ok_or(OperationToolError::ToolSetNotFound)?;
    Ok(with_toolset(content, library, entry))
}

/// 中身 `content` の工具を、参照しているツールセットの今の値と名前にそろえた中身（工具の値が変われば、工程に入れると
/// 改訂が進む）
///
/// # Errors
///
/// 参照しているツールセットが使えるライブラリにない場合（ライブラリから選んでいない工程を含む）。
pub fn aligned_operation_content(
    content: &MachiningOperationContent,
    catalog: &ToolLibraryCatalog,
) -> Result<MachiningOperationContent, OperationToolError> {
    let (library, entry) = content
        .attributes
        .toolset
        .as_ref()
        .and_then(|reference| catalog.toolset(reference))
        .ok_or(OperationToolError::ToolSetNotFound)?;
    Ok(with_toolset(content, library, entry))
}

/// 加工手順 `sequence` のすべての加工工程のうち、参照しているツールセットが使えるライブラリにあるものを、その今の
/// 値と名前にそろえ、工具の値の変わった工程の数を返す（ツールセットのない工程はそのまま）
///
/// # Errors
///
/// 保管していない加工手順の場合。
pub fn align_sequence_tools(
    sequences: &mut MachiningSequenceStore,
    sequence: MachiningSequenceId,
    catalog: &ToolLibraryCatalog,
) -> Result<usize, OperationToolError> {
    let mut changed = 0;
    for operation in sequences.operation_ids(sequence)? {
        let content = sequences.operation_content(sequence, operation)?;
        let Ok(aligned) = aligned_operation_content(content, catalog) else {
            continue;
        };
        if aligned.conditions != content.conditions {
            changed += 1;
        }
        sequences.set_operation_content(sequence, operation, aligned)?;
    }
    Ok(changed)
}

/// 中身 `content` の工具の写しを、一覧の `library_index` 番目のライブラリに名前 `name` で登録し、参照をそのツール
/// セットにした中身を返す（工具の値は変わらないため、工程に入れても改訂は進まない）
///
/// 参照を持てば同じツールセットの ID で、持たなければ新しい ID で登録する。ライブラリへは、書き手 `owner`・保存した
/// ときの情報 `info` で書く（[`ToolLibraryCatalog::edit`]）。
///
/// # Errors
///
/// ライブラリに書けない場合（名前が空・工具の値が妥当でない場合を含む）、工具のツールセットがすでに使えるライブラリに
/// ある場合（ライブラリの値を写しで上書きしない）。
pub fn register_operation_tool(
    content: &MachiningOperationContent,
    catalog: &mut ToolLibraryCatalog,
    library_index: usize,
    name: String,
    owner: &str,
    info: &ToolLibrarySaveInfo,
) -> Result<MachiningOperationContent, OperationToolError> {
    if matches!(
        catalog.operation_tool_status(
            content.attributes.toolset.as_ref(),
            &content.conditions.tool
        ),
        OperationToolStatus::SameAsLibrary | OperationToolStatus::DiffersFromLibrary
    ) {
        return Err(OperationToolError::AlreadyInLibrary);
    }
    let id = content
        .attributes
        .toolset
        .as_ref()
        .map_or_else(ToolSetId::new_random, |reference| reference.toolset);
    let entry = ToolSetEntry {
        id,
        name,
        tool: content.conditions.tool.clone(),
    };
    catalog.edit(
        library_index,
        ToolLibraryEdit::AddToolSet(entry.clone()),
        owner,
        info,
    )?;
    let library = catalog
        .entries()
        .get(library_index)
        .and_then(|entry| entry.library())
        .ok_or(OperationToolError::ToolSetNotFound)?;
    Ok(with_toolset(content, library, &entry))
}
