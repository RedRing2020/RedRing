//! モデルファイルの区画の中身のバイトの並び（版 1）
//!
//! 数値はすべてリトルエンディアン。文字列は長さ `u32` と UTF-8 のバイト列、有無は `u8`（0: なし、1: あり）、
//! ID と改訂の鍵は UUID の 16 バイト、時刻は UNIX 時刻の秒 `i64` とナノ秒 `u32` で表す。
//!
//! - `INFO`: 空間モデルの ID、保存したプログラムの版（文字列）、保存した時刻（UNIX 時刻のミリ秒 `u64`）
//! - `FACE`: 面の数 `u32` と、面ごとに
//!   - エンティティの ID・面の ID
//!   - 形状: 種類 `u8`（1: 三角形メッシュ、2: NURBS 曲面）と、`geo_io::binary_geometry` の形状
//!   - 表示の属性: 色 `f32`×4、線の種類 `u8`（1: 実線、2: 破線、3: 点線、4: 一点鎖線、5: 二点鎖線、6: 隠れ線）、
//!     線の太さ `f32`、レイヤー（文字列）、見えるか `u8`
//!   - 属性: 数 `u32` と、属性ごとにコード `i32`・値の種類 `u8`（1: 文字列、2: 整数 `i64`、3: 実数 `f64`、
//!     4: 真偽 `u8`、5: 色 `f32`×4、6: バイト列（長さ `u32` とバイト））と値。コードの小さい順に並べる
//!   - メタデータ: 名前（有無と文字列）、説明（有無と文字列）、タグ（数 `u32` と文字列）、作った時刻・変えた時刻、
//!     そのほかの項目（数 `u32` と、名前と値の文字列。名前の順に並べる）
//! - `MSEQ`: 加工手順の数 `u32` と、手順ごとに
//!   - 加工手順の ID、名前（文字列）、加工の対象（数 `u32` と面の ID）
//!   - 素材: 余裕（−X・+X・−Y・+Y・底・上の `f64`×6）、精度 `f64`
//!   - 加工工程の数 `u32` と、工程ごとに、加工工程の ID・改訂の番号 `u32`（1 以上）・改訂の鍵と中身
//!     - 名前（文字列）、加工の段階 `u8`（1: 荒、2: 中、3: 仕上げ）
//!     - 加工の種類 `u8`（1: 走査線）と条件: ピックフィード・送り `f64`、工具の移動範囲 `u8`（1: 工具の半径だけ
//!       広げる、2: 形状の範囲）、送り方 `u8`（1: ダウンカット、2: アップカット、3: ジグザグ）、走査線の角度 `f64`
//!     - 工具: 種類 `u8`（1: フラット、2: ラジアス、3: ボール）、ID（文字列）、直径・角の半径 `f64`、工具長（有無と
//!       `f64`）
//!     - 加工領域（有無と、長方形の最小・最大の XY の `f64`×4）、精度プロファイル `u8`（1: 大物プレス荒、
//!       2: 小物金型仕上げ）
//! - `CLDT`（必須でない）: 取り込んだ CL の数 `u32` と、CL ごとに、もとにした加工工程の ID・取り込んだときの改訂の番号
//!   `u32`（1 以上）・改訂の鍵・CL の編集の番号 `u32`（0 は編集していない）・CL（ツールパスの成果物のバイト列の長さ
//!   `u64` とバイト列）

use std::num::NonZeroU32;

use cam_core::{
    MachiningBoundary, MachiningStage, OperationSpec, RectangleBoundary, ScanlineCutDirection,
    ScanlineParams, ScanlineToolRange, StockMargins, ToleranceProfile, Tool, ToolType,
};
use chrono::{DateTime, Utc};
use geo_entity::{
    AttributeCode, AttributeValue, Attributes, DisplayAttributes, EntityId, FaceId, Metadata,
    StrokePattern,
};
use geo_io::binary_geometry::{
    BinaryGeometryError, read_nurbs_surface, read_triangle_mesh, write_nurbs_surface,
    write_triangle_mesh,
};
use uuid::Uuid;

use super::{SpaceModelFileError, SpaceModelSaveInfo};
use crate::imported_toolpath::ImportedToolpathRecord;
use crate::machining_sequence::{
    MachiningOperationContent, MachiningOperationId, MachiningOperationRecord,
    MachiningRevisionKey, MachiningSequenceId, MachiningSequenceRecord, MachiningStock,
};
use crate::product_shape::{ProductFaceGeometry, ProductFaceRecord};

/// 区画の中身の読み込みの誤り（項目の名前）
pub(super) type FieldError = &'static str;

/// 区画の中身を書く
#[derive(Default)]
pub(super) struct ByteWriter(Vec<u8>);

impl ByteWriter {
    pub(super) fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub(super) fn bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    fn u8(&mut self, value: u8) {
        self.0.push(value);
    }

    pub(super) fn u16(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    pub(super) fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    fn i32(&mut self, value: i32) {
        self.bytes(&value.to_le_bytes());
    }

    fn i64(&mut self, value: i64) {
        self.bytes(&value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) {
        self.bytes(&value.to_le_bytes());
    }

    fn f64(&mut self, value: f64) {
        self.bytes(&value.to_le_bytes());
    }

    fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    fn uuid(&mut self, value: Uuid) {
        self.bytes(value.as_bytes());
    }

    /// 個数を `u32` で書く
    pub(super) fn count(
        &mut self,
        count: usize,
        field: &'static str,
    ) -> Result<(), SpaceModelFileError> {
        let count = u32::try_from(count).map_err(|_| length_overflow(field))?;
        self.u32(count);
        Ok(())
    }

    fn string(&mut self, value: &str, field: &'static str) -> Result<(), SpaceModelFileError> {
        self.count(value.len(), field)?;
        self.bytes(value.as_bytes());
        Ok(())
    }

    fn optional_string(
        &mut self,
        value: Option<&str>,
        field: &'static str,
    ) -> Result<(), SpaceModelFileError> {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.string(value, field)?;
            }
        }
        Ok(())
    }

    fn time(&mut self, value: DateTime<Utc>) {
        self.i64(value.timestamp());
        self.u32(value.timestamp_subsec_nanos());
    }
}

fn length_overflow(field: &'static str) -> SpaceModelFileError {
    SpaceModelFileError::Format(cam_core::BinaryFormatError::LengthOverflow(field))
}

fn geometry_write_error(error: BinaryGeometryError) -> SpaceModelFileError {
    SpaceModelFileError::Format(match error {
        BinaryGeometryError::LengthOverflow(field) => {
            cam_core::BinaryFormatError::LengthOverflow(field)
        }
        BinaryGeometryError::InvalidValue(field) => {
            cam_core::BinaryFormatError::InvalidPayload(field)
        }
        BinaryGeometryError::UnexpectedEnd => {
            cam_core::BinaryFormatError::InvalidPayload("geometry")
        }
    })
}

/// 区画の中身を読む（信用できない入力として、残りのバイトの数を確かめながら読む）
pub(super) struct ByteReader<'a> {
    bytes: &'a [u8],
}

impl<'a> ByteReader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// `length` バイトを切り出す
    pub(super) fn take(&mut self, length: usize) -> Result<&'a [u8], FieldError> {
        if length > self.bytes.len() {
            return Err("unexpected end");
        }
        let (taken, rest) = self.bytes.split_at(length);
        self.bytes = rest;
        Ok(taken)
    }

    pub(super) fn array<const N: usize>(&mut self) -> Result<[u8; N], FieldError> {
        let (bytes, rest) = self
            .bytes
            .split_first_chunk::<N>()
            .ok_or("unexpected end")?;
        self.bytes = rest;
        Ok(*bytes)
    }

    fn u8(&mut self) -> Result<u8, FieldError> {
        Ok(self.array::<1>()?[0])
    }

    pub(super) fn u16(&mut self) -> Result<u16, FieldError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, FieldError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    pub(super) fn u64(&mut self) -> Result<u64, FieldError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn i32(&mut self) -> Result<i32, FieldError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn i64(&mut self) -> Result<i64, FieldError> {
        Ok(i64::from_le_bytes(self.array()?))
    }

    fn f32(&mut self) -> Result<f32, FieldError> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    fn f64(&mut self) -> Result<f64, FieldError> {
        Ok(f64::from_le_bytes(self.array()?))
    }

    fn bool(&mut self, field: FieldError) -> Result<bool, FieldError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(field),
        }
    }

    fn uuid(&mut self) -> Result<Uuid, FieldError> {
        Ok(Uuid::from_bytes(self.array()?))
    }

    /// 個数 `u32` を読み、1 つ `element_len` バイト以上の要素がその数だけ残りのバイトに収まることを確かめる
    pub(super) fn count(&mut self, element_len: usize) -> Result<usize, FieldError> {
        let count = self.u32()? as usize;
        match count.checked_mul(element_len) {
            Some(length) if length <= self.bytes.len() => Ok(count),
            _ => Err("count"),
        }
    }

    fn string(&mut self, field: FieldError) -> Result<String, FieldError> {
        let length = self.count(1).map_err(|_| field)?;
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| field)
    }

    fn optional_string(&mut self, field: FieldError) -> Result<Option<String>, FieldError> {
        if self.bool(field)? {
            self.string(field).map(Some)
        } else {
            Ok(None)
        }
    }

    fn time(&mut self, field: FieldError) -> Result<DateTime<Utc>, FieldError> {
        let seconds = self.i64()?;
        let nanos = self.u32()?;
        DateTime::from_timestamp(seconds, nanos).ok_or(field)
    }

    fn geometry<T>(
        &mut self,
        read: fn(&mut &[u8]) -> Result<T, BinaryGeometryError>,
    ) -> Result<T, FieldError> {
        let mut cursor = self.bytes;
        let value = read(&mut cursor).map_err(|error| match error {
            BinaryGeometryError::UnexpectedEnd => "unexpected end",
            BinaryGeometryError::LengthOverflow(field)
            | BinaryGeometryError::InvalidValue(field) => field,
        })?;
        self.bytes = cursor;
        Ok(value)
    }

    /// 読み終えたことを確かめる（余りがあれば誤り）
    pub(super) fn finish(&self) -> Result<(), FieldError> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err("trailing bytes")
        }
    }
}

// ---- INFO ----

pub(super) fn write_info(
    id: Uuid,
    info: &SpaceModelSaveInfo,
) -> Result<Vec<u8>, SpaceModelFileError> {
    let mut out = ByteWriter::default();
    out.uuid(id);
    out.string(&info.program_version, "info.program_version")?;
    out.u64(info.saved_at_ms);
    Ok(out.into_bytes())
}

pub(super) fn read_info(bytes: &[u8]) -> Result<(Uuid, SpaceModelSaveInfo), FieldError> {
    let mut cursor = ByteReader::new(bytes);
    let id = cursor.uuid()?;
    let program_version = cursor.string("info.program_version")?;
    let saved_at_ms = cursor.u64()?;
    cursor.finish()?;
    Ok((
        id,
        SpaceModelSaveInfo {
            program_version,
            saved_at_ms,
        },
    ))
}

// ---- FACE ----

const GEOMETRY_TRIANGLE_MESH: u8 = 1;
const GEOMETRY_NURBS_SURFACE: u8 = 2;

pub(super) fn write_faces(faces: &[ProductFaceRecord]) -> Result<Vec<u8>, SpaceModelFileError> {
    let mut out = ByteWriter::default();
    out.count(faces.len(), "faces")?;
    for face in faces {
        out.uuid(face.entity_id.as_uuid());
        out.uuid(face.face_id.as_uuid());
        match &face.geometry {
            ProductFaceGeometry::Mesh(mesh) => {
                out.u8(GEOMETRY_TRIANGLE_MESH);
                write_triangle_mesh(&mut out.0, mesh).map_err(geometry_write_error)?;
            }
            ProductFaceGeometry::Surface(surface) => {
                out.u8(GEOMETRY_NURBS_SURFACE);
                write_nurbs_surface(&mut out.0, surface).map_err(geometry_write_error)?;
            }
        }
        write_display(&mut out, &face.display)?;
        write_attributes(&mut out, &face.attributes)?;
        write_metadata(&mut out, &face.metadata)?;
    }
    Ok(out.into_bytes())
}

pub(super) fn read_faces(bytes: &[u8]) -> Result<Vec<ProductFaceRecord>, FieldError> {
    let mut cursor = ByteReader::new(bytes);
    // 面 1 つは、ID・形状の種類・表示の属性・属性の数・メタデータだけで 100 バイト以上
    let count = cursor.count(100).map_err(|_| "faces")?;
    let mut faces = Vec::with_capacity(count);
    for _ in 0..count {
        let entity_id = EntityId::from_uuid(cursor.uuid()?);
        let face_id = FaceId::from_uuid(cursor.uuid()?);
        let geometry = match cursor.u8()? {
            GEOMETRY_TRIANGLE_MESH => {
                ProductFaceGeometry::Mesh(cursor.geometry(read_triangle_mesh)?)
            }
            GEOMETRY_NURBS_SURFACE => {
                ProductFaceGeometry::Surface(cursor.geometry(read_nurbs_surface)?)
            }
            _ => return Err("face.geometry"),
        };
        faces.push(ProductFaceRecord {
            entity_id,
            face_id,
            geometry,
            display: read_display(&mut cursor)?,
            attributes: read_attributes(&mut cursor)?,
            metadata: read_metadata(&mut cursor)?,
        });
    }
    cursor.finish()?;
    Ok(faces)
}

fn write_display(
    out: &mut ByteWriter,
    display: &DisplayAttributes,
) -> Result<(), SpaceModelFileError> {
    for value in display.color {
        out.f32(value);
    }
    out.u8(match display.line_style {
        StrokePattern::Solid => 1,
        StrokePattern::Dashed => 2,
        StrokePattern::Dotted => 3,
        StrokePattern::DashDot => 4,
        StrokePattern::DashDotDot => 5,
        StrokePattern::Hidden => 6,
    });
    out.f32(display.line_width);
    out.string(&display.layer, "display.layer")?;
    out.bool(display.visible);
    Ok(())
}

fn read_display(cursor: &mut ByteReader<'_>) -> Result<DisplayAttributes, FieldError> {
    let color = [cursor.f32()?, cursor.f32()?, cursor.f32()?, cursor.f32()?];
    let line_style = match cursor.u8()? {
        1 => StrokePattern::Solid,
        2 => StrokePattern::Dashed,
        3 => StrokePattern::Dotted,
        4 => StrokePattern::DashDot,
        5 => StrokePattern::DashDotDot,
        6 => StrokePattern::Hidden,
        _ => return Err("display.line_style"),
    };
    Ok(DisplayAttributes {
        color,
        line_style,
        line_width: cursor.f32()?,
        layer: cursor.string("display.layer")?,
        visible: cursor.bool("display.visible")?,
    })
}

fn write_attributes(
    out: &mut ByteWriter,
    attributes: &Attributes,
) -> Result<(), SpaceModelFileError> {
    let mut entries: Vec<_> = attributes.iter().collect();
    entries.sort_by_key(|(code, _)| *code);
    out.count(entries.len(), "attributes")?;
    for (code, value) in entries {
        out.i32(code.raw());
        match value {
            AttributeValue::String(text) => {
                out.u8(1);
                out.string(text, "attribute.string")?;
            }
            AttributeValue::Integer(value) => {
                out.u8(2);
                out.i64(*value);
            }
            AttributeValue::Float(value) => {
                out.u8(3);
                out.f64(*value);
            }
            AttributeValue::Boolean(value) => {
                out.u8(4);
                out.bool(*value);
            }
            AttributeValue::Color(color) => {
                out.u8(5);
                for value in color {
                    out.f32(*value);
                }
            }
            AttributeValue::Binary(bytes) => {
                out.u8(6);
                out.count(bytes.len(), "attribute.binary")?;
                out.bytes(bytes);
            }
        }
    }
    Ok(())
}

fn read_attributes(cursor: &mut ByteReader<'_>) -> Result<Attributes, FieldError> {
    // 属性 1 つは、コードと値の種類と最も短い値（真偽）で 6 バイト以上
    let count = cursor.count(6).map_err(|_| "attributes")?;
    let mut attributes = Attributes::new();
    for _ in 0..count {
        let code = AttributeCode::try_new(cursor.i32()?).map_err(|_| "attribute.code")?;
        let value = match cursor.u8()? {
            1 => AttributeValue::String(cursor.string("attribute.string")?),
            2 => AttributeValue::Integer(cursor.i64()?),
            3 => AttributeValue::Float(cursor.f64()?),
            4 => AttributeValue::Boolean(cursor.bool("attribute.boolean")?),
            5 => {
                AttributeValue::Color([cursor.f32()?, cursor.f32()?, cursor.f32()?, cursor.f32()?])
            }
            6 => {
                let length = cursor.count(1).map_err(|_| "attribute.binary")?;
                AttributeValue::Binary(cursor.take(length)?.to_vec())
            }
            _ => return Err("attribute.kind"),
        };
        if attributes.contains(code) {
            return Err("attribute.duplicate");
        }
        // システムの属性は、定義にあるコードと値の種類だけを受け入れる
        attributes.set(code, value).map_err(|_| "attribute.value")?;
    }
    Ok(attributes)
}

fn write_metadata(out: &mut ByteWriter, metadata: &Metadata) -> Result<(), SpaceModelFileError> {
    out.optional_string(metadata.name.as_deref(), "metadata.name")?;
    out.optional_string(metadata.description.as_deref(), "metadata.description")?;
    out.count(metadata.tags.len(), "metadata.tags")?;
    for tag in &metadata.tags {
        out.string(tag, "metadata.tag")?;
    }
    out.time(metadata.created_at);
    out.time(metadata.modified_at);
    let mut custom: Vec<_> = metadata.custom.iter().collect();
    custom.sort();
    out.count(custom.len(), "metadata.custom")?;
    for (key, value) in custom {
        out.string(key, "metadata.custom.key")?;
        out.string(value, "metadata.custom.value")?;
    }
    Ok(())
}

fn read_metadata(cursor: &mut ByteReader<'_>) -> Result<Metadata, FieldError> {
    let name = cursor.optional_string("metadata.name")?;
    let description = cursor.optional_string("metadata.description")?;
    let tag_count = cursor.count(4).map_err(|_| "metadata.tags")?;
    let mut tags = Vec::with_capacity(tag_count);
    for _ in 0..tag_count {
        tags.push(cursor.string("metadata.tag")?);
    }
    let created_at = cursor.time("metadata.created_at")?;
    let modified_at = cursor.time("metadata.modified_at")?;
    let custom_count = cursor.count(8).map_err(|_| "metadata.custom")?;
    let mut custom = std::collections::HashMap::with_capacity(custom_count);
    for _ in 0..custom_count {
        let key = cursor.string("metadata.custom.key")?;
        let value = cursor.string("metadata.custom.value")?;
        if custom.insert(key, value).is_some() {
            return Err("metadata.custom.duplicate");
        }
    }
    Ok(Metadata {
        name,
        description,
        tags,
        created_at,
        modified_at,
        custom,
    })
}

// ---- MSEQ ----

pub(super) fn write_sequences(
    sequences: &[MachiningSequenceRecord],
) -> Result<Vec<u8>, SpaceModelFileError> {
    let mut out = ByteWriter::default();
    out.count(sequences.len(), "sequences")?;
    for sequence in sequences {
        out.uuid(sequence.id.as_uuid());
        out.string(&sequence.name, "sequence.name")?;
        out.count(sequence.target_faces.len(), "sequence.target_faces")?;
        for face in &sequence.target_faces {
            out.uuid(face.as_uuid());
        }
        let margins = sequence.stock.margins;
        for value in [
            margins.minus_x,
            margins.plus_x,
            margins.minus_y,
            margins.plus_y,
            margins.bottom,
            margins.top,
            sequence.stock.precision,
        ] {
            out.f64(value);
        }
        out.count(sequence.operations.len(), "sequence.operations")?;
        for operation in &sequence.operations {
            out.uuid(operation.id.as_uuid());
            out.u32(operation.revision.get());
            out.uuid(operation.revision_key.as_uuid());
            write_content(&mut out, &operation.content)?;
        }
    }
    Ok(out.into_bytes())
}

pub(super) fn read_sequences(bytes: &[u8]) -> Result<Vec<MachiningSequenceRecord>, FieldError> {
    let mut cursor = ByteReader::new(bytes);
    // 手順 1 つは、ID・名前の長さ・加工の対象の数・素材・工程の数だけで 84 バイト以上
    let count = cursor.count(84).map_err(|_| "sequences")?;
    let mut sequences = Vec::with_capacity(count);
    for _ in 0..count {
        let id = MachiningSequenceId::from_uuid(cursor.uuid()?);
        let name = cursor.string("sequence.name")?;
        let face_count = cursor.count(16).map_err(|_| "sequence.target_faces")?;
        let mut target_faces = Vec::with_capacity(face_count);
        for _ in 0..face_count {
            target_faces.push(FaceId::from_uuid(cursor.uuid()?));
        }
        let margins = StockMargins {
            minus_x: cursor.f64()?,
            plus_x: cursor.f64()?,
            minus_y: cursor.f64()?,
            plus_y: cursor.f64()?,
            bottom: cursor.f64()?,
            top: cursor.f64()?,
        };
        let stock = MachiningStock {
            margins,
            precision: cursor.f64()?,
        };
        // 工程 1 つは、ID・改訂の番号・改訂の鍵だけで 36 バイト以上
        let operation_count = cursor.count(36).map_err(|_| "sequence.operations")?;
        let mut operations = Vec::with_capacity(operation_count);
        for _ in 0..operation_count {
            let operation_id = MachiningOperationId::from_uuid(cursor.uuid()?);
            let revision = NonZeroU32::new(cursor.u32()?).ok_or("operation.revision")?;
            let revision_key = MachiningRevisionKey::from_uuid(cursor.uuid()?);
            operations.push(MachiningOperationRecord {
                id: operation_id,
                content: read_content(&mut cursor)?,
                revision,
                revision_key,
            });
        }
        sequences.push(MachiningSequenceRecord {
            id,
            name,
            target_faces,
            stock,
            operations,
        });
    }
    cursor.finish()?;
    Ok(sequences)
}

fn write_content(
    out: &mut ByteWriter,
    content: &MachiningOperationContent,
) -> Result<(), SpaceModelFileError> {
    out.string(&content.name, "operation.name")?;
    out.u8(match content.stage {
        MachiningStage::Rough => 1,
        MachiningStage::SemiFinish => 2,
        MachiningStage::Finish => 3,
    });
    let OperationSpec::Scanline(scanline) = content.spec;
    out.u8(1);
    out.f64(scanline.stepover);
    out.f64(scanline.feed_rate);
    out.u8(match scanline.tool_range {
        ScanlineToolRange::ExtendByToolRadius => 1,
        ScanlineToolRange::GeometryExtent => 2,
    });
    out.u8(match scanline.cut_direction {
        ScanlineCutDirection::DownCut => 1,
        ScanlineCutDirection::UpCut => 2,
        ScanlineCutDirection::ZigZag => 3,
    });
    out.f64(scanline.angle_degrees);
    let tool = &content.tool;
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
    match content.boundary {
        None => out.u8(0),
        Some(MachiningBoundary::Rectangle(rectangle)) => {
            out.u8(1);
            for value in [
                rectangle.rect_min[0],
                rectangle.rect_min[1],
                rectangle.rect_max[0],
                rectangle.rect_max[1],
            ] {
                out.f64(value);
            }
        }
    }
    out.u8(match content.tolerance_profile {
        ToleranceProfile::PressRough => 1,
        ToleranceProfile::MoldFinish => 2,
    });
    Ok(())
}

fn read_content(cursor: &mut ByteReader<'_>) -> Result<MachiningOperationContent, FieldError> {
    let name = cursor.string("operation.name")?;
    let stage = match cursor.u8()? {
        1 => MachiningStage::Rough,
        2 => MachiningStage::SemiFinish,
        3 => MachiningStage::Finish,
        _ => return Err("operation.stage"),
    };
    if cursor.u8()? != 1 {
        return Err("operation.kind");
    }
    let stepover = cursor.f64()?;
    let feed_rate = cursor.f64()?;
    let tool_range = match cursor.u8()? {
        1 => ScanlineToolRange::ExtendByToolRadius,
        2 => ScanlineToolRange::GeometryExtent,
        _ => return Err("operation.tool_range"),
    };
    let cut_direction = match cursor.u8()? {
        1 => ScanlineCutDirection::DownCut,
        2 => ScanlineCutDirection::UpCut,
        3 => ScanlineCutDirection::ZigZag,
        _ => return Err("operation.cut_direction"),
    };
    let angle_degrees = cursor.f64()?;
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
    let boundary = match cursor.u8()? {
        0 => None,
        1 => Some(MachiningBoundary::Rectangle(RectangleBoundary {
            rect_min: [cursor.f64()?, cursor.f64()?],
            rect_max: [cursor.f64()?, cursor.f64()?],
        })),
        _ => return Err("operation.boundary"),
    };
    let tolerance_profile = match cursor.u8()? {
        1 => ToleranceProfile::PressRough,
        2 => ToleranceProfile::MoldFinish,
        _ => return Err("operation.tolerance_profile"),
    };
    Ok(MachiningOperationContent {
        name,
        stage,
        spec: OperationSpec::Scanline(ScanlineParams {
            stepover,
            feed_rate,
            tool_range,
            cut_direction,
            angle_degrees,
        }),
        tool,
        boundary,
        tolerance_profile,
    })
}

// ---- CLDT ----

pub(super) fn write_toolpaths(
    toolpaths: &[ImportedToolpathRecord],
) -> Result<Vec<u8>, SpaceModelFileError> {
    let mut out = ByteWriter::default();
    out.count(toolpaths.len(), "toolpaths")?;
    for toolpath in toolpaths {
        out.uuid(toolpath.operation.as_uuid());
        out.u32(toolpath.revision.get());
        out.uuid(toolpath.revision_key.as_uuid());
        out.u32(toolpath.edit_number);
        out.u64(toolpath.artifact.len() as u64);
        out.bytes(&toolpath.artifact);
    }
    Ok(out.into_bytes())
}

pub(super) fn read_toolpaths(bytes: &[u8]) -> Result<Vec<ImportedToolpathRecord>, FieldError> {
    let mut cursor = ByteReader::new(bytes);
    // CL 1 つは、ID・改訂の番号・改訂の鍵・編集の番号・長さだけで 48 バイト
    let count = cursor.count(48).map_err(|_| "toolpaths")?;
    let mut toolpaths = Vec::with_capacity(count);
    for _ in 0..count {
        let operation = MachiningOperationId::from_uuid(cursor.uuid()?);
        let revision = NonZeroU32::new(cursor.u32()?).ok_or("toolpath.revision")?;
        let revision_key = MachiningRevisionKey::from_uuid(cursor.uuid()?);
        let edit_number = cursor.u32()?;
        let length = usize::try_from(cursor.u64()?).map_err(|_| "toolpath.artifact")?;
        let artifact = cursor.take(length)?.to_vec();
        toolpaths.push(ImportedToolpathRecord {
            operation,
            revision,
            revision_key,
            edit_number,
            artifact,
        });
    }
    cursor.finish()?;
    Ok(toolpaths)
}
