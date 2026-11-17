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
//! - `SHEL`（シェルがなければ書かない）: B-rep の表の数 `u32` と表、シェルの数 `u32` とシェル
//!   - B-rep の表: 全体の許容誤差 `f64`、頂点の数 `u32` と頂点ごとに点 `f64`×3・許容誤差 `f64`、稜線の数 `u32` と
//!     稜線ごとに曲線の種類 `u8`（1: 直線、2: 円、3: 楕円、4: NURBS 曲線）と `geo_io::binary_geometry` の曲線・区間
//!     `f64`×2・始点と終点の頂点の番号 `u32`×2・許容誤差 `f64`、面の数 `u32` と面ごとに曲面の種類 `u8`（1: 平面、
//!     2: 円柱、3: 円錐、4: 球、5: トーラス、6: NURBS 曲面）と曲面・向き `u8`（1: 曲面の法線と同じ）・ループの数
//!     `u32` とループごとに稜線の使用の数 `u32` と使用（稜線の番号 `u32`・稜線の向きに沿うか `u8`）、シェルの数
//!     `u32` とシェルごとに面の数 `u32` と面の番号 `u32`、立体の数 `u32` と立体ごとにシェルの数 `u32` とシェルの
//!     番号 `u32`。番号は表の中の位置
//!   - シェル: エンティティの ID、B-rep の表の番号 `u32`（同じ部品の出現は同じ表を指す）、配置（4×4 の行列の
//!     `f64`×16。行の順）、面の数 `u32` と面の ID（B-rep の面の順）、元の要素（有無と、ファイルの名前（文字列）・
//!     出現の数 `u32` と出現のエンティティの番号 `u64`・面の数 `u32` と元の面のエンティティの番号 `u64`）、
//!     表示の属性・属性・メタデータ（`FACE` と同じ）
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
//! - `NSET`（必須でない）: 計算の条件を設定していない加工工程の数 `u32` と、その加工工程の ID（なければ、すべての
//!   工程を設定した工程とする）
//! - `CLDT`（必須でない）: 取り込んだ CL の数 `u32` と、CL ごとに、もとにした加工工程の ID・取り込んだときの改訂の番号
//!   `u32`（1 以上）・改訂の鍵・CL の編集の番号 `u32`（0 は編集していない）・CL（ツールパスの成果物のバイト列の長さ
//!   `u64` とバイト列）

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use analysis::linalg::matrix::Matrix4x4;

use cam_core::{
    MachiningBoundary, MachiningStage, OperationSpec, RectangleBoundary, ScanlineCutDirection,
    ScanlineParams, ScanlineToolRange, StockMargins, ToleranceProfile, Tool, ToolType,
};
use geo_entity::{
    AttributeCode, AttributeValue, Attributes, DisplayAttributes, EntityId, FaceId, Metadata,
    StrokePattern,
};
use geo_io::binary_geometry::{
    BinaryGeometryError, read_circle, read_cone, read_cylinder, read_ellipse, read_line,
    read_nurbs_curve, read_nurbs_surface, read_plane, read_sphere, read_torus, read_triangle_mesh,
    write_circle, write_cone, write_cylinder, write_ellipse, write_line, write_nurbs_curve,
    write_nurbs_surface, write_plane, write_sphere, write_torus, write_triangle_mesh,
};
use geo_topology::{
    BrepRecords, CurveRef, EdgeRecord, FaceRecord, Point3D, SurfaceRef, VertexRecord,
};
use jiff::Timestamp;
use uuid::Uuid;

use super::{SpaceModelFileError, SpaceModelSaveInfo};
use crate::imported_toolpath::ImportedToolpathRecord;
use crate::machining_sequence::{
    MachiningOperationAttributes, MachiningOperationConditions, MachiningOperationContent,
    MachiningOperationId, MachiningOperationRecord, MachiningRevisionKey, MachiningSequenceId,
    MachiningSequenceRecord, MachiningStock,
};
use crate::product_shape::{
    Brep, ProductFaceGeometry, ProductFaceRecord, ProductShellRecord, ShellSource,
    SimilarityTransform3D,
};

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

    /// 時刻を、秒（切り捨て）と、その秒からのナノ秒（0 以上）で書く
    fn time(&mut self, value: Timestamp) {
        const NANOS_PER_SECOND: i128 = 1_000_000_000;
        let nanos = value.as_nanosecond();
        // 時刻の範囲（1 万年より内）では、秒は i64 に、秒からのナノ秒は u32 に収まる
        self.i64(nanos.div_euclid(NANOS_PER_SECOND) as i64);
        self.u32(nanos.rem_euclid(NANOS_PER_SECOND) as u32);
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

    fn time(&mut self, field: FieldError) -> Result<Timestamp, FieldError> {
        let seconds = self.i64()?;
        let nanos = i32::try_from(self.u32()?).map_err(|_| field)?;
        Timestamp::new(seconds, nanos).map_err(|_| field)
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

// ---- NSET ----

/// 計算の条件を設定していない加工工程の ID（`NSET`）を書く
pub(super) fn write_operations_not_set(
    operations: &[MachiningOperationId],
) -> Result<Vec<u8>, SpaceModelFileError> {
    let mut out = ByteWriter::default();
    out.count(operations.len(), "operations_not_set")?;
    for operation in operations {
        out.uuid(operation.as_uuid());
    }
    Ok(out.into_bytes())
}

/// 計算の条件を設定していない加工工程の ID（`NSET`）を読む
pub(super) fn read_operations_not_set(
    bytes: &[u8],
) -> Result<Vec<MachiningOperationId>, FieldError> {
    let mut cursor = ByteReader::new(bytes);
    let count = cursor.count(16).map_err(|_| "operations_not_set")?;
    let mut operations = Vec::with_capacity(count);
    for _ in 0..count {
        operations.push(MachiningOperationId::from_uuid(cursor.uuid()?));
    }
    cursor.finish()?;
    Ok(operations)
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
            // 計算の条件を設定していない工程は、別の区画（`NSET`）で表す。なければ設定した工程
            operations.push(MachiningOperationRecord {
                id: operation_id,
                content: read_content(&mut cursor)?,
                revision,
                revision_key,
                conditions_set: true,
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
    // 保存の形は、属性と計算の条件に分ける前と同じ（名前・加工の段階・加工の種類と条件・工具・加工領域・精度の順）。
    // 組の中まで分解し、項目を足したときに書き忘れをコンパイルで気づけるようにする
    let MachiningOperationContent {
        attributes: MachiningOperationAttributes { name, stage },
        conditions:
            MachiningOperationConditions {
                spec,
                tool,
                boundary,
                tolerance_profile,
            },
    } = content;
    out.string(name, "operation.name")?;
    out.u8(match stage {
        MachiningStage::Rough => 1,
        MachiningStage::SemiFinish => 2,
        MachiningStage::Finish => 3,
    });
    let OperationSpec::Scanline(scanline) = spec;
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
    match boundary {
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
    out.u8(match tolerance_profile {
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
        attributes: MachiningOperationAttributes { name, stage },
        conditions: MachiningOperationConditions {
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
        },
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

// ---- SHEL ----

const CURVE_LINE: u8 = 1;
const CURVE_CIRCLE: u8 = 2;
const CURVE_ELLIPSE: u8 = 3;
const CURVE_NURBS: u8 = 4;
const SURFACE_PLANE: u8 = 1;
const SURFACE_CYLINDER: u8 = 2;
const SURFACE_CONE: u8 = 3;
const SURFACE_SPHERE: u8 = 4;
const SURFACE_TORUS: u8 = 5;
const SURFACE_NURBS: u8 = 6;

/// シェルの記録を書く（同じ B-rep の表を共有するシェルは、表を 1 回だけ書いて番号で指す）
pub(super) fn write_shells(shells: &[ProductShellRecord]) -> Result<Vec<u8>, SpaceModelFileError> {
    let mut breps: Vec<&Arc<Brep<f64>>> = Vec::new();
    let mut index_of: HashMap<*const Brep<f64>, usize> = HashMap::new();
    let mut brep_of = Vec::with_capacity(shells.len());
    for shell in shells {
        let index = *index_of.entry(Arc::as_ptr(&shell.brep)).or_insert_with(|| {
            breps.push(&shell.brep);
            breps.len() - 1
        });
        brep_of.push(index);
    }
    let mut out = ByteWriter::default();
    out.count(breps.len(), "breps")?;
    for brep in breps {
        write_brep(&mut out, &brep.records())?;
    }
    out.count(shells.len(), "shells")?;
    for (shell, brep) in shells.iter().zip(brep_of) {
        out.uuid(shell.entity_id.as_uuid());
        out.count(brep, "shell.brep")?;
        for row in shell.placement.to_matrix().rows() {
            for value in row {
                out.f64(value);
            }
        }
        out.count(shell.face_ids.len(), "shell.faces")?;
        for face in &shell.face_ids {
            out.uuid(face.as_uuid());
        }
        match &shell.source {
            None => out.u8(0),
            Some(source) => {
                out.u8(1);
                out.string(&source.file_name, "shell.source.file_name")?;
                out.count(source.occurrences.len(), "shell.source.occurrences")?;
                for &occurrence in &source.occurrences {
                    out.u64(occurrence);
                }
                out.count(source.faces.len(), "shell.source.faces")?;
                for &face in &source.faces {
                    out.u64(face);
                }
            }
        }
        write_display(&mut out, &shell.display)?;
        write_attributes(&mut out, &shell.attributes)?;
        write_metadata(&mut out, &shell.metadata)?;
    }
    Ok(out.into_bytes())
}

pub(super) fn write_brep(
    out: &mut ByteWriter,
    brep: &BrepRecords<f64>,
) -> Result<(), SpaceModelFileError> {
    out.f64(brep.tolerance);
    out.count(brep.vertices.len(), "brep.vertices")?;
    for vertex in &brep.vertices {
        out.f64(vertex.point.x());
        out.f64(vertex.point.y());
        out.f64(vertex.point.z());
        out.f64(vertex.tolerance);
    }
    out.count(brep.edges.len(), "brep.edges")?;
    for edge in &brep.edges {
        match &edge.curve {
            CurveRef::Line(line) => {
                out.u8(CURVE_LINE);
                write_line(&mut out.0, line);
            }
            CurveRef::Circle(circle) => {
                out.u8(CURVE_CIRCLE);
                write_circle(&mut out.0, circle);
            }
            CurveRef::Ellipse(ellipse) => {
                out.u8(CURVE_ELLIPSE);
                write_ellipse(&mut out.0, ellipse);
            }
            CurveRef::Nurbs(nurbs) => {
                out.u8(CURVE_NURBS);
                write_nurbs_curve(&mut out.0, nurbs).map_err(geometry_write_error)?;
            }
        }
        out.f64(edge.parameter_range.0);
        out.f64(edge.parameter_range.1);
        out.count(edge.start, "brep.edge.start")?;
        out.count(edge.end, "brep.edge.end")?;
        out.f64(edge.tolerance);
    }
    out.count(brep.faces.len(), "brep.faces")?;
    for face in &brep.faces {
        match &face.surface {
            SurfaceRef::Plane(plane) => {
                out.u8(SURFACE_PLANE);
                write_plane(&mut out.0, plane);
            }
            SurfaceRef::Cylinder(cylinder) => {
                out.u8(SURFACE_CYLINDER);
                write_cylinder(&mut out.0, cylinder);
            }
            SurfaceRef::Cone(cone) => {
                out.u8(SURFACE_CONE);
                write_cone(&mut out.0, cone);
            }
            SurfaceRef::Sphere(sphere) => {
                out.u8(SURFACE_SPHERE);
                write_sphere(&mut out.0, sphere);
            }
            SurfaceRef::Torus(torus) => {
                out.u8(SURFACE_TORUS);
                write_torus(&mut out.0, torus);
            }
            SurfaceRef::Nurbs(nurbs) => {
                out.u8(SURFACE_NURBS);
                write_nurbs_surface(&mut out.0, nurbs).map_err(geometry_write_error)?;
            }
        }
        out.bool(face.same_sense);
        out.count(face.loops.len(), "brep.face.loops")?;
        for uses in &face.loops {
            out.count(uses.len(), "brep.loop.uses")?;
            for &(edge, forward) in uses {
                out.count(edge, "brep.use.edge")?;
                out.bool(forward);
            }
        }
    }
    for (groups, field) in [(&brep.shells, "brep.shells"), (&brep.solids, "brep.solids")] {
        out.count(groups.len(), field)?;
        for members in groups {
            out.count(members.len(), field)?;
            for &member in members {
                out.count(member, field)?;
            }
        }
    }
    Ok(())
}

pub(super) fn read_shells(bytes: &[u8]) -> Result<Vec<ProductShellRecord>, FieldError> {
    let mut cursor = ByteReader::new(bytes);
    // B-rep の表 1 つは、許容誤差と要素の数だけで 28 バイト以上
    let brep_count = cursor.count(28).map_err(|_| "breps")?;
    let mut breps = Vec::with_capacity(brep_count);
    for _ in 0..brep_count {
        let records = read_brep(&mut cursor)?;
        let brep = Brep::from_records(records).map_err(|_| "brep")?;
        // 保存する表なら必ず満たす性質（稜線の端と頂点の一致・ループのつながり）だけを確かめる。面の向きなどは、
        // 取り込んだ表で合わないことがあるため求めない
        let report = brep.validate();
        if !report.edges_off_vertices.is_empty() || !report.disconnected_loops.is_empty() {
            return Err("brep");
        }
        breps.push(Arc::new(brep));
    }
    // シェル 1 つは、ID・表の番号・配置・面の数・元の要素の有無・表示の属性などだけで 200 バイト以上
    let count = cursor.count(200).map_err(|_| "shells")?;
    let mut referenced = vec![false; breps.len()];
    let mut shells = Vec::with_capacity(count);
    for _ in 0..count {
        let entity_id = EntityId::from_uuid(cursor.uuid()?);
        let index = cursor.u32()? as usize;
        let brep = breps.get(index).cloned().ok_or("shell.brep")?;
        if let Some(flag) = referenced.get_mut(index) {
            *flag = true;
        }
        let mut matrix = Matrix4x4::zeros();
        for row in 0..4 {
            let values = [cursor.f64()?, cursor.f64()?, cursor.f64()?, cursor.f64()?];
            if values.iter().any(|value| !value.is_finite()) {
                return Err("shell.placement");
            }
            matrix.set_row(row, values);
        }
        let placement =
            SimilarityTransform3D::from_matrix(matrix).map_err(|_| "shell.placement")?;
        let face_count = cursor.count(16).map_err(|_| "shell.faces")?;
        let mut face_ids = Vec::with_capacity(face_count);
        for _ in 0..face_count {
            face_ids.push(FaceId::from_uuid(cursor.uuid()?));
        }
        let source = if cursor.bool("shell.source")? {
            let file_name = cursor.string("shell.source.file_name")?;
            let occurrences = read_u64_list(&mut cursor, "shell.source.occurrences")?;
            let faces = read_u64_list(&mut cursor, "shell.source.faces")?;
            Some(ShellSource {
                file_name,
                occurrences,
                faces,
            })
        } else {
            None
        };
        shells.push(ProductShellRecord {
            entity_id,
            brep,
            placement,
            face_ids,
            source,
            display: read_display(&mut cursor)?,
            attributes: read_attributes(&mut cursor)?,
            metadata: read_metadata(&mut cursor)?,
        });
    }
    cursor.finish()?;
    // どのシェルも指さない表は書かないので、あれば壊れたファイル
    if referenced.contains(&false) {
        return Err("breps");
    }
    Ok(shells)
}

fn read_u64_list(cursor: &mut ByteReader<'_>, field: FieldError) -> Result<Vec<u64>, FieldError> {
    let count = cursor.count(8).map_err(|_| field)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(cursor.u64()?);
    }
    Ok(values)
}

fn read_brep(cursor: &mut ByteReader<'_>) -> Result<BrepRecords<f64>, FieldError> {
    let tolerance = cursor.f64()?;
    let count = cursor.count(32).map_err(|_| "brep.vertices")?;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..count {
        vertices.push(VertexRecord {
            point: Point3D::new(cursor.f64()?, cursor.f64()?, cursor.f64()?),
            tolerance: cursor.f64()?,
        });
    }
    // 稜線 1 つは、曲線の種類・区間・始点と終点・許容誤差だけで 33 バイト以上
    let count = cursor.count(33).map_err(|_| "brep.edges")?;
    let mut edges = Vec::with_capacity(count);
    for _ in 0..count {
        let curve = match cursor.u8()? {
            CURVE_LINE => CurveRef::Line(cursor.geometry(read_line)?),
            CURVE_CIRCLE => CurveRef::Circle(cursor.geometry(read_circle)?),
            CURVE_ELLIPSE => CurveRef::Ellipse(cursor.geometry(read_ellipse)?),
            CURVE_NURBS => CurveRef::Nurbs(cursor.geometry(read_nurbs_curve)?),
            _ => return Err("brep.edge.curve"),
        };
        edges.push(EdgeRecord {
            curve,
            parameter_range: (cursor.f64()?, cursor.f64()?),
            start: cursor.u32()? as usize,
            end: cursor.u32()? as usize,
            tolerance: cursor.f64()?,
        });
    }
    // 面 1 つは、曲面の種類・向き・ループの数だけで 6 バイト以上
    let count = cursor.count(6).map_err(|_| "brep.faces")?;
    let mut faces = Vec::with_capacity(count);
    for _ in 0..count {
        let surface = match cursor.u8()? {
            SURFACE_PLANE => SurfaceRef::Plane(cursor.geometry(read_plane)?),
            SURFACE_CYLINDER => SurfaceRef::Cylinder(cursor.geometry(read_cylinder)?),
            SURFACE_CONE => SurfaceRef::Cone(cursor.geometry(read_cone)?),
            SURFACE_SPHERE => SurfaceRef::Sphere(cursor.geometry(read_sphere)?),
            SURFACE_TORUS => SurfaceRef::Torus(cursor.geometry(read_torus)?),
            SURFACE_NURBS => SurfaceRef::Nurbs(cursor.geometry(read_nurbs_surface)?),
            _ => return Err("brep.face.surface"),
        };
        let same_sense = cursor.bool("brep.face.same_sense")?;
        let loop_count = cursor.count(4).map_err(|_| "brep.face.loops")?;
        let mut loops = Vec::with_capacity(loop_count);
        for _ in 0..loop_count {
            let use_count = cursor.count(5).map_err(|_| "brep.loop.uses")?;
            let mut uses = Vec::with_capacity(use_count);
            for _ in 0..use_count {
                uses.push((cursor.u32()? as usize, cursor.bool("brep.use.forward")?));
            }
            loops.push(uses);
        }
        faces.push(FaceRecord {
            surface,
            same_sense,
            loops,
        });
    }
    let mut groups = Vec::with_capacity(2);
    for field in ["brep.shells", "brep.solids"] {
        let count = cursor.count(4).map_err(|_| field)?;
        let mut members_of = Vec::with_capacity(count);
        for _ in 0..count {
            let member_count = cursor.count(4).map_err(|_| field)?;
            let mut members = Vec::with_capacity(member_count);
            for _ in 0..member_count {
                members.push(cursor.u32()? as usize);
            }
            members_of.push(members);
        }
        groups.push(members_of);
    }
    let solids = groups.pop().unwrap_or_default();
    let shells = groups.pop().unwrap_or_default();
    Ok(BrepRecords {
        tolerance,
        vertices,
        edges,
        faces,
        shells,
        solids,
    })
}
