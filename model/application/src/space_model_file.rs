//! 空間モデル（モデルファイル `.rrm`）の読み書き（設計: SPACE_MODEL_FILE_DESIGN.md）
//!
//! 成果物と同じ共通のヘッダ（種類 `RRMF`）の後ろに、区画を並べる。区画は、種類 `[u8; 4]`・版 `u16`・印 `u16`
//! （bit0: 必須）・長さ `u64` と中身を持つ。読む側が知らない必須の区画があれば開かず、知らない必須でない区画は
//! そのままのバイト列で持って、保存のときに書き戻す。
//!
//! 区画の中身のバイトの並びは `codec` に書く。読み込みは信用できない入力として扱う（個数・長さは残りのバイトの数で
//! 確かめてから確保する。区画は長さちょうどを読み終える）。

mod codec;
#[cfg(test)]
mod tests;

use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use cam_core::{ArtifactHeaderV1, ArtifactKind, ArtifactPayload, ExtAttribute, read_artifact_v1};
/// 共通のヘッダ・形式の誤り（[`SpaceModelFileError::Format`] の中身）と、読める共通のヘッダの大きい版
pub use cam_core::{BinaryFormatError, FORMAT_VERSION_MAJOR_V1};
use uuid::Uuid;

use crate::imported_toolpath::{ImportedToolpathRestoreError, ImportedToolpathStore};
use crate::machining_sequence::{MachiningSequenceRestoreError, MachiningSequenceStore};
use crate::product_shape::{ProductShapeRestoreError, ProductShapeStore};
use codec::{ByteReader, ByteWriter};

/// 読み込むモデルファイルの大きさの上限（バイト）
pub const MAX_SPACE_MODEL_FILE_BYTES: u64 = 2 << 30;

/// 区画の数の上限
const MAX_SECTIONS: usize = 64;

/// 区画の印: 必須（読む側が知らなければ開かない）
const SECTION_REQUIRED: u16 = 1;

const INFO: [u8; 4] = *b"INFO";
const FACE: [u8; 4] = *b"FACE";
const MSEQ: [u8; 4] = *b"MSEQ";
/// 取り込んだ CL（必須でない区画。古い版のアプリは、そのまま書き戻す）
const CLDT: [u8; 4] = *b"CLDT";
/// 本モジュールが読み書きする区画の版（これより新しい版の区画は、新しい版のアプリで保存したもの）
pub const SECTION_VERSION: u16 = 1;

/// 空間モデルの ID（全体で一意で、保存・読み込みで変えない）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceModelId(Uuid);

impl SpaceModelId {
    /// ランダムな ID を作る
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for SpaceModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 読む側が知らない、必須でない区画（そのまま書き戻す）
#[derive(Debug, Clone, PartialEq, Eq)]
struct PreservedSection {
    kind: [u8; 4],
    version: u16,
    flags: u16,
    body: Vec<u8>,
}

/// 空間モデル（製品形状と加工手順・加工工程、取り込んだ CL）
#[derive(Debug)]
pub struct SpaceModel {
    pub id: SpaceModelId,
    pub shapes: ProductShapeStore,
    pub sequences: MachiningSequenceStore,
    pub toolpaths: ImportedToolpathStore,
    /// 読んだファイルにあった、知らない必須でない区画（読んだ順）
    preserved: Vec<PreservedSection>,
    /// 読んだファイルの共通のヘッダの拡張属性（そのまま書き戻す）
    ext_attributes: Vec<ExtAttribute>,
}

impl SpaceModel {
    /// 新しい ID で、製品形状 `shapes` と加工手順 `sequences` を持ち、CL を取り込んでいない空間モデルを作る
    pub fn new(shapes: ProductShapeStore, sequences: MachiningSequenceStore) -> Self {
        Self {
            id: SpaceModelId::new_random(),
            shapes,
            sequences,
            toolpaths: ImportedToolpathStore::new(),
            preserved: Vec::new(),
            ext_attributes: Vec::new(),
        }
    }
}

/// 保存したときの情報（区画 `INFO`）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceModelSaveInfo {
    /// 保存したプログラムの版
    pub program_version: String,
    /// 保存した時刻（UNIX 時刻のミリ秒）
    pub saved_at_ms: u64,
}

/// モデルファイルの読み書きの誤り
#[derive(Debug)]
pub enum SpaceModelFileError {
    /// ファイルを読めない・書けない
    Io(std::io::Error),
    /// ファイルが大きさの上限を超える
    TooLarge,
    /// 共通のヘッダ・形式の誤り（種類の違い・知らない版を含む）
    Format(cam_core::BinaryFormatError),
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
    /// 区画の中身が正しくない（区画と項目の名前）
    InvalidSection { kind: [u8; 4], field: &'static str },
    /// 製品形状の検証の誤り（ID の重なり）
    ProductShape(ProductShapeRestoreError),
    /// 加工手順の検証の誤り（ID の重なり）
    MachiningSequence(MachiningSequenceRestoreError),
    /// 取り込んだ CL の検証の誤り（工程の重なり・読めない CL）
    ImportedToolpath(ImportedToolpathRestoreError),
}

impl fmt::Display for SpaceModelFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = |kind: &[u8; 4]| String::from_utf8_lossy(kind).into_owned();
        match self {
            Self::Io(error) => write!(f, "model file I/O error: {error}"),
            Self::TooLarge => write!(
                f,
                "the model file is larger than {MAX_SPACE_MODEL_FILE_BYTES} bytes"
            ),
            Self::Format(error) => write!(f, "invalid model file: {error}"),
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
            Self::ProductShape(error) => write!(f, "invalid model file: {error}"),
            Self::MachiningSequence(error) => write!(f, "invalid model file: {error}"),
            Self::ImportedToolpath(error) => write!(f, "invalid model file: {error}"),
        }
    }
}

impl std::error::Error for SpaceModelFileError {}

impl From<std::io::Error> for SpaceModelFileError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<cam_core::BinaryFormatError> for SpaceModelFileError {
    fn from(error: cam_core::BinaryFormatError) -> Self {
        Self::Format(error)
    }
}

/// 空間モデル `model` を、保存したときの情報 `info` とともにモデルファイルの形式で書く
///
/// # Errors
///
/// 書き込みの失敗、個数・長さが形式に収まらない場合。
pub fn write_space_model<W: Write>(
    writer: &mut W,
    model: &SpaceModel,
    info: &SpaceModelSaveInfo,
) -> Result<(), SpaceModelFileError> {
    let mut sections = vec![
        (INFO, SECTION_REQUIRED, codec::write_info(model.id.0, info)?),
        (
            FACE,
            SECTION_REQUIRED,
            codec::write_faces(&model.shapes.records())?,
        ),
        (
            MSEQ,
            SECTION_REQUIRED,
            codec::write_sequences(&model.sequences.records())?,
        ),
    ];
    // 取り込んだ CL がなければ、区画を書かない
    if !model.toolpaths.records().is_empty() {
        sections.push((CLDT, 0, codec::write_toolpaths(model.toolpaths.records())?));
    }
    let mut payload = ByteWriter::default();
    // 書き戻す区画は、今の版で書いた区画と同じ種類（読めない版の CL と、その後で取り込んだ CL）でも、すべて残す
    payload.count(sections.len() + model.preserved.len(), "sections")?;
    for (kind, flags, body) in &sections {
        write_section(&mut payload, *kind, SECTION_VERSION, *flags, body);
    }
    for section in &model.preserved {
        write_section(
            &mut payload,
            section.kind,
            section.version,
            section.flags,
            &section.body,
        );
    }
    let payload = payload.into_bytes();
    let mut header = ArtifactHeaderV1::new(ArtifactKind::SpaceModel, payload.len() as u64);
    header.ext_attributes = model.ext_attributes.clone();
    header.write_to(writer)?;
    writer.write_all(&payload)?;
    Ok(())
}

fn write_section(out: &mut ByteWriter, kind: [u8; 4], version: u16, flags: u16, body: &[u8]) {
    out.bytes(&kind);
    out.u16(version);
    out.u16(flags);
    out.u64(body.len() as u64);
    out.bytes(body);
}

/// モデルファイルの形式のバイト列から、空間モデルと保存したときの情報を読む
///
/// # Errors
///
/// 読み込みの失敗、形式の誤り（本体の後ろに余りがある場合を含む）、知らない必須の区画・読めない版の区画がある場合、
/// 必須の区画がない場合、ID が重なっている場合。
pub fn read_space_model<R: Read>(
    reader: &mut R,
) -> Result<(SpaceModel, SpaceModelSaveInfo), SpaceModelFileError> {
    let (header, payload) = read_artifact_v1(reader)?;
    if reader.read(&mut [0_u8; 1])? != 0 {
        return Err(SpaceModelFileError::InvalidSectionTable("trailing bytes"));
    }
    let ArtifactPayload::SpaceModel(payload) = payload else {
        return Err(cam_core::BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::SpaceModel,
            found: header.kind,
        }
        .into());
    };
    let mut cursor = ByteReader::new(&payload);
    let invalid = SpaceModelFileError::InvalidSectionTable;
    // 区画 1 つは、種類・版・印・長さだけで 16 バイト
    let count = cursor.count(16).map_err(|_| invalid("sections"))?;
    if count > MAX_SECTIONS {
        return Err(invalid("sections"));
    }
    let mut info = None;
    let mut faces = None;
    let mut sequences = None;
    let mut toolpaths = None;
    let mut preserved = Vec::new();
    for _ in 0..count {
        let header = (|| {
            Some((
                cursor.array::<4>().ok()?,
                cursor.u16().ok()?,
                cursor.u16().ok()?,
                cursor.u64().ok()?,
            ))
        })();
        let Some((kind, version, flags, length)) = header else {
            return Err(invalid("section header"));
        };
        let body = usize::try_from(length)
            .ok()
            .and_then(|length| cursor.take(length).ok())
            .ok_or(invalid("section length"))?;
        // 必須でない区画は、読めない版なら知らない区画と同じに、そのまま書き戻す
        let readable_optional = flags & SECTION_REQUIRED != 0 || version == SECTION_VERSION;
        let slot = match kind {
            INFO => Some(&mut info),
            FACE => Some(&mut faces),
            MSEQ => Some(&mut sequences),
            CLDT if readable_optional => Some(&mut toolpaths),
            _ => None,
        };
        let Some(slot) = slot else {
            if flags & SECTION_REQUIRED != 0 {
                return Err(SpaceModelFileError::UnknownRequiredSection(kind));
            }
            preserved.push(PreservedSection {
                kind,
                version,
                flags,
                body: body.to_vec(),
            });
            continue;
        };
        if slot.is_some() {
            return Err(SpaceModelFileError::DuplicateSection(kind));
        }
        if version != SECTION_VERSION {
            return Err(SpaceModelFileError::UnsupportedSectionVersion { kind, version });
        }
        *slot = Some(body);
    }
    cursor.finish().map_err(|_| invalid("trailing bytes"))?;

    let section_error = |kind| move |field| SpaceModelFileError::InvalidSection { kind, field };
    let (id, save_info) = codec::read_info(info.ok_or(SpaceModelFileError::MissingSection(INFO))?)
        .map_err(section_error(INFO))?;
    let faces = codec::read_faces(faces.ok_or(SpaceModelFileError::MissingSection(FACE))?)
        .map_err(section_error(FACE))?;
    let sequences =
        codec::read_sequences(sequences.ok_or(SpaceModelFileError::MissingSection(MSEQ))?)
            .map_err(section_error(MSEQ))?;
    let toolpaths = match toolpaths {
        Some(bytes) => codec::read_toolpaths(bytes).map_err(section_error(CLDT))?,
        None => Vec::new(),
    };
    let model = SpaceModel {
        id: SpaceModelId(id),
        shapes: ProductShapeStore::from_records(faces)
            .map_err(SpaceModelFileError::ProductShape)?,
        sequences: MachiningSequenceStore::from_records(sequences)
            .map_err(SpaceModelFileError::MachiningSequence)?,
        toolpaths: ImportedToolpathStore::from_records(toolpaths)
            .map_err(SpaceModelFileError::ImportedToolpath)?,
        preserved,
        ext_attributes: header.ext_attributes,
    };
    Ok((model, save_info))
}

/// 空間モデル `model` をモデルファイル `path` に保存する（一時の名前で書き終えてから名前を変え、書きかけのファイルを
/// 残さない）
///
/// # Errors
///
/// 書き込み・名前の変更の失敗、個数・長さが形式に収まらない場合。
pub fn save_space_model(
    path: &Path,
    model: &SpaceModel,
    info: &SpaceModelSaveInfo,
) -> Result<(), SpaceModelFileError> {
    let temporary = temporary_path(path);
    let result = (|| {
        let mut file = fs::File::create(&temporary)?;
        let mut writer = std::io::BufWriter::new(&mut file);
        write_space_model(&mut writer, model, info)?;
        writer.flush()?;
        drop(writer);
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// モデルファイル `path` を読む
///
/// # Errors
///
/// ファイルを読めない場合、大きさの上限を超える場合、[`read_space_model`] の誤り。
pub fn load_space_model(
    path: &Path,
) -> Result<(SpaceModel, SpaceModelSaveInfo), SpaceModelFileError> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > MAX_SPACE_MODEL_FILE_BYTES {
        return Err(SpaceModelFileError::TooLarge);
    }
    // 開いた後に大きくなっても、上限より多くは読まない（上限を超えた分は、本体の後ろの余りの誤りになる）
    let mut reader = std::io::BufReader::new(file.take(MAX_SPACE_MODEL_FILE_BYTES + 1));
    read_space_model(&mut reader)
}

/// 保存の一時のファイルの名前（同じフォルダに置き、名前の変更で置き換える。保存ごとに違う名前にし、同時の保存・
/// ほかのファイルと重ならないようにする）
fn temporary_path(path: &Path) -> PathBuf {
    let mut name = std::ffi::OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(format!(".{}.tmp", Uuid::new_v4().simple()));
    path.with_file_name(name)
}
