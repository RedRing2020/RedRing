//! ツールライブラリのファイル（`.rrtl`）の読み書き（設計: TOOL_LIBRARY_DESIGN.md）
//!
//! 成果物と同じ共通のヘッダ（種類 `RRTL`）の後ろに、モデルファイルと同じ形の区画を並べる。読む側が知らない必須の
//! 区画があれば開かず、知らない必須でない区画はそのままのバイト列で持って、保存のときに書き戻す。
//!
//! 区画の中身（版 1）:
//!
//! - `LINF`: ツールライブラリの ID、名前（文字列）、保存したプログラムの版（文字列）、保存した時刻（UNIX 時刻の
//!   ミリ秒 `u64`）
//! - `TSET`: ツールセットの数 `u32` と、ツールセットごとに ID・名前（文字列）・工具（[`write_tool`] の並び）
//!
//! 複数の PC から共有するファイルへの書き込み（ロックのファイルと書く前の読み直し）は `shared`、使うファイルの
//! 一覧（変わったファイルの読み直し・ID の重なり・読むだけの扱い）は `catalog` に置く。
//!
//! 読み込みは信用できない入力として扱い、ライブラリの名前とツールセットの名前が空でないこと、ツールセットの ID が
//! 重なっていないこと、工具の値が妥当であることを確かめる。

mod catalog;
mod shared;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ArtifactPayload, BinaryFormatError, ExtAttribute, Tool,
    ToolType, read_artifact_v1,
};
use uuid::Uuid;

use crate::binary_section::{
    ByteReader, ByteWriter, FieldError, SECTION_REQUIRED, read_sections, write_section,
};
use crate::machining_sequence::{ToolLibraryId, ToolSetEntry, ToolSetId};
pub use catalog::{ToolLibraryCatalog, ToolLibraryCatalogEntry, ToolLibraryStatus};
pub use shared::{
    ToolLibraryEdit, ToolLibraryEditError, ToolLibraryLock, edit_tool_library,
    remove_tool_library_lock, tool_library_lock,
};

/// 読み込むツールライブラリのファイルの大きさの上限（バイト）
const MAX_TOOL_LIBRARY_FILE_BYTES: u64 = 64 << 20;

/// 区画の数の上限
const MAX_SECTIONS: usize = 64;

/// ツールライブラリの情報
const LINF: [u8; 4] = *b"LINF";
/// ツールセットの項目の列
const TSET: [u8; 4] = *b"TSET";
/// 本モジュールが読み書きする区画の版（これより新しい版の区画は、新しい版のアプリで保存したもの）
const SECTION_VERSION: u16 = 1;

/// 読む側が知らない、必須でない区画（そのまま書き戻す）
#[derive(Debug, Clone, PartialEq, Eq)]
struct PreservedSection {
    kind: [u8; 4],
    version: u16,
    flags: u16,
    body: Vec<u8>,
}

/// ツールライブラリ（工作機械 1 台分のツールセットの集まり）
#[derive(Debug, Clone, PartialEq)]
pub struct ToolLibrary {
    /// ツールライブラリの ID（ファイルを移しても・名前を替えても変わらない）
    pub id: ToolLibraryId,
    /// 利用者が付ける名前（空にしない）
    pub name: String,
    /// ツールセットの項目（利用者が並べた順）
    pub toolsets: Vec<ToolSetEntry>,
    /// 読んだファイルにあった、知らない必須でない区画（読んだ順）
    preserved: Vec<PreservedSection>,
    /// 読んだファイルの共通のヘッダの拡張属性（そのまま書き戻す）
    ext_attributes: Vec<ExtAttribute>,
}

/// 保存したときの情報（区画 `LINF` の、ライブラリの ID・名前のほか）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolLibrarySaveInfo {
    /// 保存したプログラムの版
    pub program_version: String,
    /// 保存した時刻（UNIX 時刻のミリ秒）
    pub saved_at_ms: u64,
}

/// ツールライブラリのファイルの読み書きの誤り
#[derive(Debug)]
pub enum ToolLibraryFileError {
    /// ファイルを読めない・書けない
    Io(std::io::Error),
    /// ファイルが大きさの上限を超える
    TooLarge,
    /// 共通のヘッダ・形式の誤り（種類の違い・知らない版を含む）
    Format(BinaryFormatError),
    /// 読む側が知らない必須の区画
    UnknownRequiredSection([u8; 4]),
    /// 読めない版の区画
    UnsupportedSectionVersion { kind: [u8; 4], version: u16 },
    /// 必須の区画がない
    MissingSection([u8; 4]),
    /// 同じ区画が 2 つある
    DuplicateSection([u8; 4]),
    /// 区画の並び（区画の数・区画の種類と長さ・余り）が正しくない（項目の名前）
    InvalidSectionTable(&'static str),
    /// 区画の中身が正しくない（区画と項目の名前。空の名前・重なった ID・妥当でない工具を含む）
    InvalidSection { kind: [u8; 4], field: &'static str },
}

impl fmt::Display for ToolLibraryFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = |kind: &[u8; 4]| String::from_utf8_lossy(kind).into_owned();
        match self {
            Self::Io(error) => write!(f, "tool library file I/O error: {error}"),
            Self::TooLarge => write!(
                f,
                "the tool library file is larger than {MAX_TOOL_LIBRARY_FILE_BYTES} bytes"
            ),
            Self::Format(error) => write!(f, "invalid tool library file: {error}"),
            Self::UnknownRequiredSection(k) => write!(f, "unknown required section: {}", kind(k)),
            Self::UnsupportedSectionVersion { kind: k, version } => {
                write!(f, "unsupported version {version} of section {}", kind(k))
            }
            Self::MissingSection(k) => write!(f, "missing section: {}", kind(k)),
            Self::DuplicateSection(k) => write!(f, "duplicate section: {}", kind(k)),
            Self::InvalidSectionTable(field) => write!(f, "invalid section table: {field}"),
            Self::InvalidSection { kind: k, field } => {
                write!(f, "invalid section {}: {field}", kind(k))
            }
        }
    }
}

impl std::error::Error for ToolLibraryFileError {}

impl From<std::io::Error> for ToolLibraryFileError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<BinaryFormatError> for ToolLibraryFileError {
    fn from(error: BinaryFormatError) -> Self {
        Self::Format(error)
    }
}

/// 工具 `tool` を書く: 種類 `u8`（1: フラット、2: ラジアス、3: ボール）、ID（文字列）、直径・角の半径 `f64`、
/// 工具長（有無と `f64`）
///
/// モデルファイルの区画 `MSEQ` の工具も、同じ並びで書く。
pub(crate) fn write_tool(out: &mut ByteWriter, tool: &Tool<f64>) -> Result<(), BinaryFormatError> {
    out.u8(match tool.tool_type {
        ToolType::FlatEndMill => 1,
        ToolType::RadiusEndMill => 2,
        ToolType::BallEndMill => 3,
    });
    out.string(&tool.id, "tool.id")?;
    out.f64(tool.radius() * 2.0);
    out.f64(tool.corner_radius());
    match tool.cutting_length {
        None => out.u8(0),
        Some(length) => {
            out.u8(1);
            out.f64(length);
        }
    }
    Ok(())
}

/// [`write_tool`] の並びの工具を読む（値の妥当さは確かめない）
pub(crate) fn read_tool(cursor: &mut ByteReader<'_>) -> Result<Tool<f64>, FieldError> {
    let tool_type = match cursor.u8()? {
        1 => ToolType::FlatEndMill,
        2 => ToolType::RadiusEndMill,
        3 => ToolType::BallEndMill,
        _ => return Err("tool.type"),
    };
    let tool_id = cursor.string("tool.id")?;
    let diameter = cursor.f64()?;
    let corner_radius = cursor.f64()?;
    let cutting_length = match cursor.u8()? {
        0 => None,
        1 => Some(cursor.f64()?),
        _ => return Err("tool.cutting_length"),
    };
    let mut tool = Tool::new(tool_id, tool_type, diameter, corner_radius, 0.0);
    tool.cutting_length = cutting_length;
    Ok(tool)
}

/// ツールライブラリに登録できる工具か（径・工具長が正の有限の値で、種類と角の半径が合う）
fn valid_tool(tool: &Tool<f64>) -> bool {
    let positive = |value: f64| value.is_finite() && value > 0.0;
    positive(tool.radius())
        && tool.cutting_length.is_none_or(positive)
        && tool.validate_parameters()
}

fn write_info(
    library: &ToolLibrary,
    info: &ToolLibrarySaveInfo,
) -> Result<Vec<u8>, BinaryFormatError> {
    let mut out = ByteWriter::default();
    out.uuid(library.id.as_uuid());
    out.string(&library.name, "library.name")?;
    out.string(&info.program_version, "info.program_version")?;
    out.u64(info.saved_at_ms);
    Ok(out.into_bytes())
}

fn read_info(bytes: &[u8]) -> Result<(ToolLibraryId, String, ToolLibrarySaveInfo), FieldError> {
    let mut cursor = ByteReader::new(bytes);
    let id = ToolLibraryId::from_uuid(cursor.uuid()?);
    let name = cursor.string("library.name")?;
    if name.is_empty() {
        return Err("library.name");
    }
    let program_version = cursor.string("info.program_version")?;
    let saved_at_ms = cursor.u64()?;
    cursor.finish()?;
    Ok((
        id,
        name,
        ToolLibrarySaveInfo {
            program_version,
            saved_at_ms,
        },
    ))
}

fn write_toolsets(toolsets: &[ToolSetEntry]) -> Result<Vec<u8>, BinaryFormatError> {
    let mut out = ByteWriter::default();
    out.count(toolsets.len(), "toolsets")?;
    for ToolSetEntry { id, name, tool } in toolsets {
        out.uuid(id.as_uuid());
        out.string(name, "toolset.name")?;
        write_tool(&mut out, tool)?;
    }
    Ok(out.into_bytes())
}

fn read_toolsets(bytes: &[u8]) -> Result<Vec<ToolSetEntry>, FieldError> {
    let mut cursor = ByteReader::new(bytes);
    // ツールセット 1 つは、ID・名前の長さ・工具だけで 50 バイト以上
    let count = cursor.count(50).map_err(|_| "toolsets")?;
    let mut toolsets = Vec::with_capacity(count);
    let mut ids = HashSet::with_capacity(count);
    for _ in 0..count {
        let id = ToolSetId::from_uuid(cursor.uuid()?);
        if !ids.insert(id) {
            return Err("toolset.id");
        }
        let name = cursor.string("toolset.name")?;
        if name.is_empty() {
            return Err("toolset.name");
        }
        let tool = read_tool(&mut cursor)?;
        if !valid_tool(&tool) {
            return Err("toolset.tool");
        }
        toolsets.push(ToolSetEntry { id, name, tool });
    }
    cursor.finish()?;
    Ok(toolsets)
}

/// ツールライブラリ `library` を、保存したときの情報 `info` とともにツールライブラリのファイルの形式で書く
///
/// # Errors
///
/// 書き込みの失敗、個数・長さが形式に収まらない場合。
pub(crate) fn write_tool_library<W: Write>(
    writer: &mut W,
    library: &ToolLibrary,
    info: &ToolLibrarySaveInfo,
) -> Result<(), ToolLibraryFileError> {
    let sections = [
        (LINF, write_info(library, info)?),
        (TSET, write_toolsets(&library.toolsets)?),
    ];
    let mut payload = ByteWriter::default();
    payload.count(sections.len() + library.preserved.len(), "sections")?;
    for (kind, body) in &sections {
        write_section(&mut payload, *kind, SECTION_VERSION, SECTION_REQUIRED, body);
    }
    for section in &library.preserved {
        write_section(
            &mut payload,
            section.kind,
            section.version,
            section.flags,
            &section.body,
        );
    }
    let payload = payload.into_bytes();
    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolLibrary, payload.len() as u64);
    header.ext_attributes = library.ext_attributes.clone();
    header.write_to(writer)?;
    writer.write_all(&payload)?;
    Ok(())
}

/// ツールライブラリのファイルの形式のバイト列から、ツールライブラリと保存したときの情報を読む
///
/// # Errors
///
/// 読み込みの失敗、形式の誤り（本体の後ろに余りがある場合を含む）、知らない必須の区画・読めない版の区画がある場合、
/// 必須の区画がない場合、名前が空・ツールセットの ID が重なる・工具の値が妥当でない場合。
pub(crate) fn read_tool_library<R: Read>(
    reader: &mut R,
) -> Result<(ToolLibrary, ToolLibrarySaveInfo), ToolLibraryFileError> {
    let (header, payload) = read_artifact_v1(reader)?;
    if reader.read(&mut [0_u8; 1])? != 0 {
        return Err(ToolLibraryFileError::InvalidSectionTable("trailing bytes"));
    }
    let ArtifactPayload::ToolLibrary(payload) = payload else {
        return Err(BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::ToolLibrary,
            found: header.kind,
        }
        .into());
    };
    let sections =
        read_sections(&payload, MAX_SECTIONS).map_err(ToolLibraryFileError::InvalidSectionTable)?;
    let mut info = None;
    let mut toolsets = None;
    let mut preserved = Vec::new();
    for section in sections {
        let slot = match section.kind {
            LINF => &mut info,
            TSET => &mut toolsets,
            kind => {
                if section.required() {
                    return Err(ToolLibraryFileError::UnknownRequiredSection(kind));
                }
                preserved.push(PreservedSection {
                    kind,
                    version: section.version,
                    flags: section.flags,
                    body: section.body.to_vec(),
                });
                continue;
            }
        };
        if slot.is_some() {
            return Err(ToolLibraryFileError::DuplicateSection(section.kind));
        }
        if section.version != SECTION_VERSION {
            return Err(ToolLibraryFileError::UnsupportedSectionVersion {
                kind: section.kind,
                version: section.version,
            });
        }
        *slot = Some(section.body);
    }
    let section_error = |kind| move |field| ToolLibraryFileError::InvalidSection { kind, field };
    let (id, name, save_info) = read_info(info.ok_or(ToolLibraryFileError::MissingSection(LINF))?)
        .map_err(section_error(LINF))?;
    let toolsets = read_toolsets(toolsets.ok_or(ToolLibraryFileError::MissingSection(TSET))?)
        .map_err(section_error(TSET))?;
    let library = ToolLibrary {
        id,
        name,
        toolsets,
        preserved,
        ext_attributes: header.ext_attributes,
    };
    Ok((library, save_info))
}

/// ツールライブラリ `library` をファイル `path` に保存する（一時の名前で書き終えてから名前を変え、書きかけのファイル
/// を残さない）
///
/// # Errors
///
/// 書き込み・名前の変更の失敗、個数・長さが形式に収まらない場合。
pub(crate) fn save_tool_library(
    path: &Path,
    library: &ToolLibrary,
    info: &ToolLibrarySaveInfo,
) -> Result<(), ToolLibraryFileError> {
    platform_foundation::file::replace_file(path, &temporary_path(path), |file| {
        let mut writer = std::io::BufWriter::new(file);
        write_tool_library(&mut writer, library, info)?;
        writer.flush()?;
        Ok(())
    })
}

/// ツールライブラリのファイル `path` を読む
///
/// # Errors
///
/// ファイルを読めない場合、大きさの上限を超える場合、`read_tool_library` の誤り。
pub(crate) fn load_tool_library(
    path: &Path,
) -> Result<(ToolLibrary, ToolLibrarySaveInfo), ToolLibraryFileError> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > MAX_TOOL_LIBRARY_FILE_BYTES {
        return Err(ToolLibraryFileError::TooLarge);
    }
    // 開いた後に大きくなっても、上限より多くは読まない（上限を超えた分は、本体の後ろの余りの誤りになる）
    let mut reader = std::io::BufReader::new(file.take(MAX_TOOL_LIBRARY_FILE_BYTES + 1));
    read_tool_library(&mut reader)
}

/// 保存の一時のファイルの名前（同じフォルダに置き、名前の変更で置き換える。保存ごとに違う名前にする）
fn temporary_path(path: &Path) -> PathBuf {
    let mut name = std::ffi::OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(format!(".{}.tmp", Uuid::new_v4().simple()));
    path.with_file_name(name)
}
