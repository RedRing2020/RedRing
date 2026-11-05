//! CAM batch artifact binary format (v0.3).
//!
//! 共通ヘッダの Reader/Writer と互換性チェックを提供する
//! （設計: ARTIFACT_BINARY_IO_CONTRACT_DESIGN.md）。
//!
//! # Multi-axis 拡張ポリシー
//!
//! - 本モジュールは現時点で `version_major=0, version_minor=3` の
//!   3軸相当 payload を安定対象として扱う。v0.2 は v0.1 と同じ wire layout で、
//!   ToolPath の切削方向の値に `Mixed`（2）を足したもの。v0.3 は切削シミュレーションの中身を
//!   差分のキーフレームの節の列にしたもので、ToolPath・Interference の wire layout は変えていない。
//!   ToolPath・Interference は v0.1 も読み込める（切削シミュレーションの中身の版は cam_sim が検査する）。
//! - multi-axis 向け payload は将来の version 追加で導入し、
//!   `v0.1` wire layout は改変しない。
//! - API名に含まれる `*_v1` は API 世代名であり、wire format の実値は
//!   ヘッダの `version_major`/`version_minor` で判定する。

#[cfg(test)]
mod tests;

use std::fmt;
use std::io::{Read, Write};

use geo_algorithms::Point3D;

use crate::toolpath::{
    ArcDirection, ContourLevelPath, CuttingDirection, PathGeometry, PathSegment, SegmentType,
    ToolPath, ToolPathKinematicMeta,
};

/// toolpath artifact magic: `RRTP`
pub const TOOLPATH_MAGIC: [u8; 4] = *b"RRTP";

/// interference artifact magic: `RRIN`
pub const INTERFERENCE_MAGIC: [u8; 4] = *b"RRIN";

/// cutting simulation artifact magic: `RRCS`
pub const CUTTING_SIMULATION_MAGIC: [u8; 4] = *b"RRCS";

pub const FORMAT_VERSION_MAJOR_V1: u16 = 0;
pub const FORMAT_VERSION_MINOR_V1: u16 = 3;

/// v0.x 系で受理可能な minor version 一覧。
pub const KNOWN_MINOR_VERSIONS_V0: &[u16] = &[1, 2, FORMAT_VERSION_MINOR_V1];

/// 読み込みで、ファイルに書かれた個数から先に確保する要素の数の上限
///
/// 個数はファイルに書かれた値のため、信用せずに上限で抑え、それを超える分は読んだ要素の数に応じて増やす。
/// 細工したファイル（個数に大きな値を書いたもの）で、巨大な確保をさせないため。
const MAX_PREALLOCATED_ELEMENTS: usize = 4096;

/// 読み込みで、本体を読み終える前に確保するバイトの数の上限（[`MAX_PREALLOCATED_ELEMENTS`] と同じ理由）
const MAX_PREALLOCATED_BYTES: usize = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityDecision {
    Accept,
    Reject,
    Convert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    ToolPath,
    Interference,
    CuttingSimulation,
}

impl ArtifactKind {
    pub fn magic(self) -> [u8; 4] {
        match self {
            Self::ToolPath => TOOLPATH_MAGIC,
            Self::Interference => INTERFERENCE_MAGIC,
            Self::CuttingSimulation => CUTTING_SIMULATION_MAGIC,
        }
    }

    pub fn from_magic(magic: [u8; 4]) -> Option<Self> {
        if magic == TOOLPATH_MAGIC {
            Some(Self::ToolPath)
        } else if magic == INTERFERENCE_MAGIC {
            Some(Self::Interference)
        } else if magic == CUTTING_SIMULATION_MAGIC {
            Some(Self::CuttingSimulation)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LengthUnit {
    Millimeter = 1,
}

impl LengthUnit {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Millimeter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CoordinateFrame {
    WorldRightHandedZUp = 1,
}

impl CoordinateFrame {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::WorldRightHandedZUp),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum BinaryFormatError {
    Io(std::io::Error),
    UnknownMagic([u8; 4]),
    ArtifactKindMismatch {
        expected: ArtifactKind,
        found: ArtifactKind,
    },
    UnsupportedMajorVersion {
        found: u16,
        supported: u16,
    },
    UnsupportedMinorVersion {
        major: u16,
        found_minor: u16,
        supported_minors: &'static [u16],
    },
    ConverterRequired {
        found_major: u16,
        found_minor: u16,
    },
    UnsupportedUnit(u8),
    UnsupportedCoordinateFrame(u8),
    ReservedExtAttributeTag(i16),
    UnknownCuttingDirection(u8),
    UnknownSegmentType(u8),
    UnknownGeometryType(u8),
    UnknownInterferenceKind(u8),
    LengthOverflow(&'static str),
    InvalidPayload(&'static str),
}

impl fmt::Display for BinaryFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::UnknownMagic(magic) => write!(f, "unknown artifact magic: {magic:?}"),
            Self::ArtifactKindMismatch { expected, found } => {
                write!(
                    f,
                    "artifact kind mismatch: expected={expected:?}, found={found:?}"
                )
            }
            Self::UnsupportedMajorVersion { found, supported } => {
                write!(
                    f,
                    "unsupported major version: found={found}, supported={supported}"
                )
            }
            Self::UnsupportedMinorVersion {
                major,
                found_minor,
                supported_minors,
            } => {
                write!(
                    f,
                    "unsupported minor version: major={major}, found_minor={found_minor}, supported_minors={supported_minors:?}"
                )
            }
            Self::ConverterRequired {
                found_major,
                found_minor,
            } => {
                write!(
                    f,
                    "converter required for version: major={found_major}, minor={found_minor}"
                )
            }
            Self::UnsupportedUnit(value) => write!(f, "unsupported length unit: {value}"),
            Self::UnsupportedCoordinateFrame(value) => {
                write!(f, "unsupported coordinate frame: {value}")
            }
            Self::ReservedExtAttributeTag(tag) => {
                write!(f, "reserved ext attribute tag is not allowed: {tag}")
            }
            Self::UnknownCuttingDirection(value) => {
                write!(f, "unknown cutting direction: {value}")
            }
            Self::UnknownSegmentType(value) => write!(f, "unknown segment type: {value}"),
            Self::UnknownGeometryType(value) => write!(f, "unknown geometry type: {value}"),
            Self::UnknownInterferenceKind(value) => {
                write!(f, "unknown interference kind: {value}")
            }
            Self::LengthOverflow(field) => write!(f, "length overflow: field={field}"),
            Self::InvalidPayload(field) => write!(f, "invalid payload: field={field}"),
        }
    }
}

impl std::error::Error for BinaryFormatError {}

impl From<std::io::Error> for BinaryFormatError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// TLV 形式の拡張属性エントリ。
/// wire layout: tag(u16 LE) + data_len(u16 LE) + data([u8; data_len])
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtAttribute {
    pub tag: i16,
    pub data: Vec<u8>,
}

impl ExtAttribute {
    pub fn new(tag: i16, data: impl Into<Vec<u8>>) -> Self {
        Self {
            tag,
            data: data.into(),
        }
    }

    /// RedRing ルール: 負のタグはシステム属性。
    pub fn is_system_attribute(&self) -> bool {
        self.tag < 0
    }

    /// RedRing ルール: 正のタグはユーザー属性。
    pub fn is_user_attribute(&self) -> bool {
        self.tag > 0
    }

    /// RedRing ルール: 0 は予約タグ。
    pub fn is_reserved_tag(&self) -> bool {
        self.tag == 0
    }

    fn wire_len(&self) -> usize {
        2 + 2 + self.data.len()
    }
}

/// Shared header for all CAM binary artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactHeaderV1 {
    pub kind: ArtifactKind,
    pub version_major: u16,
    pub version_minor: u16,
    pub unit: LengthUnit,
    pub frame: CoordinateFrame,
    /// ヘッダ直後に続く TLV 拡張属性。空のとき wire 上は 0 バイト。
    pub ext_attributes: Vec<ExtAttribute>,
    pub payload_len: u64,
}

impl ArtifactHeaderV1 {
    /// 固定ヘッダ部のバイト長（拡張属性ブロックは含まない）。
    pub const FIXED_HEADER_LEN: usize = 4 + 2 + 2 + 1 + 1 + 4 + 8;

    pub fn new(kind: ArtifactKind, payload_len: u64) -> Self {
        Self {
            kind,
            version_major: FORMAT_VERSION_MAJOR_V1,
            version_minor: FORMAT_VERSION_MINOR_V1,
            unit: LengthUnit::Millimeter,
            frame: CoordinateFrame::WorldRightHandedZUp,
            ext_attributes: Vec::new(),
            payload_len,
        }
    }

    pub fn is_supported_major(&self) -> bool {
        self.version_major == FORMAT_VERSION_MAJOR_V1
    }

    pub fn compatibility_decision(&self) -> CompatibilityDecision {
        if self.version_major == FORMAT_VERSION_MAJOR_V1 {
            if KNOWN_MINOR_VERSIONS_V0.contains(&self.version_minor) {
                CompatibilityDecision::Accept
            } else {
                CompatibilityDecision::Reject
            }
        } else {
            CompatibilityDecision::Convert
        }
    }

    fn ext_block_len(&self) -> u32 {
        self.ext_attributes
            .iter()
            .map(|a| a.wire_len())
            .sum::<usize>() as u32
    }

    pub fn ensure_supported_major(&self) -> Result<(), BinaryFormatError> {
        if self.is_supported_major() {
            Ok(())
        } else {
            Err(BinaryFormatError::UnsupportedMajorVersion {
                found: self.version_major,
                supported: FORMAT_VERSION_MAJOR_V1,
            })
        }
    }

    pub fn ensure_acceptable_version(&self) -> Result<(), BinaryFormatError> {
        match self.compatibility_decision() {
            CompatibilityDecision::Accept => Ok(()),
            CompatibilityDecision::Reject => Err(BinaryFormatError::UnsupportedMinorVersion {
                major: self.version_major,
                found_minor: self.version_minor,
                supported_minors: KNOWN_MINOR_VERSIONS_V0,
            }),
            CompatibilityDecision::Convert => Err(BinaryFormatError::ConverterRequired {
                found_major: self.version_major,
                found_minor: self.version_minor,
            }),
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), BinaryFormatError> {
        writer.write_all(&self.kind.magic())?;
        writer.write_all(&self.version_major.to_le_bytes())?;
        writer.write_all(&self.version_minor.to_le_bytes())?;
        writer.write_all(&[self.unit as u8])?;
        writer.write_all(&[self.frame as u8])?;
        writer.write_all(&self.ext_block_len().to_le_bytes())?;
        writer.write_all(&self.payload_len.to_le_bytes())?;
        for attr in &self.ext_attributes {
            if attr.is_reserved_tag() {
                return Err(BinaryFormatError::ReservedExtAttributeTag(attr.tag));
            }
            writer.write_all(&attr.tag.to_le_bytes())?;
            writer.write_all(&(attr.data.len() as u16).to_le_bytes())?;
            writer.write_all(&attr.data)?;
        }
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, BinaryFormatError> {
        let mut magic = [0_u8; 4];
        reader.read_exact(&mut magic)?;

        let kind = ArtifactKind::from_magic(magic).ok_or(BinaryFormatError::UnknownMagic(magic))?;

        let version_major = read_u16_le(reader)?;
        let version_minor = read_u16_le(reader)?;

        let mut unit_buf = [0_u8; 1];
        reader.read_exact(&mut unit_buf)?;
        let unit = LengthUnit::from_u8(unit_buf[0])
            .ok_or(BinaryFormatError::UnsupportedUnit(unit_buf[0]))?;

        let mut frame_buf = [0_u8; 1];
        reader.read_exact(&mut frame_buf)?;
        let frame = CoordinateFrame::from_u8(frame_buf[0])
            .ok_or(BinaryFormatError::UnsupportedCoordinateFrame(frame_buf[0]))?;

        let ext_block_len = read_u32_le(reader)?;
        let payload_len = read_u64_le(reader)?;
        let ext_attributes = read_ext_attributes(reader, ext_block_len)?;

        Ok(Self {
            kind,
            version_major,
            version_minor,
            unit,
            frame,
            ext_attributes,
            payload_len,
        })
    }
}

pub fn write_toolpath_payload_v1<W: Write>(
    writer: &mut W,
    toolpath: &ToolPath<f64>,
) -> Result<(), BinaryFormatError> {
    write_string_u16(writer, &toolpath.tool_id)?;
    write_u8(
        writer,
        cutting_direction_to_wire(toolpath.cutting_direction),
    )?;

    write_ext_attributes(writer, &toolpath.ext_attributes)?;
    write_segments(writer, &toolpath.approach_segments)?;

    write_u32(
        writer,
        usize_to_u32(toolpath.contour_levels.len(), "contour_levels")?,
    )?;
    for contour in &toolpath.contour_levels {
        write_u32(writer, usize_to_u32(contour.level_index, "level_index")?)?;
        write_f64(writer, contour.z_level)?;
        write_segments(writer, &contour.segments)?;
    }

    write_segments(writer, &toolpath.retract_segments)?;
    Ok(())
}

pub fn read_toolpath_payload_v1<R: Read>(
    reader: &mut R,
) -> Result<ToolPath<f64>, BinaryFormatError> {
    let tool_id = read_string_u16(reader)?;
    let cutting_direction = cutting_direction_from_wire(read_u8(reader)?)?;

    let ext_attributes = read_ext_attributes_list(reader)?;
    let approach_segments = read_segments(reader)?;

    let contour_count = read_u32_le(reader)? as usize;
    let mut contour_levels = Vec::with_capacity(preallocated(contour_count));
    for _ in 0..contour_count {
        let level_index = read_u32_le(reader)? as usize;
        let z_level = read_f64_le(reader)?;
        let segments = read_segments(reader)?;
        contour_levels.push(ContourLevelPath {
            level_index,
            z_level,
            segments,
        });
    }

    let retract_segments = read_segments(reader)?;

    Ok(ToolPath {
        tool_id,
        cutting_direction,
        approach_segments,
        contour_levels,
        retract_segments,
        ext_attributes,
        kinematic_meta: ToolPathKinematicMeta::three_axis_position_only(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterferenceKind {
    Tool = 1,
    Holder = 2,
    Shank = 3,
}

impl InterferenceKind {
    fn from_u8(value: u8) -> Result<Self, BinaryFormatError> {
        match value {
            1 => Ok(Self::Tool),
            2 => Ok(Self::Holder),
            3 => Ok(Self::Shank),
            other => Err(BinaryFormatError::UnknownInterferenceKind(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterferenceEvent {
    pub sample_index: u32,
    pub tool_id: String,
    pub position: Point3D<f64>,
    pub normal: Point3D<f64>,
    pub penetration_depth: f64,
    pub kind: InterferenceKind,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InterferencePayload {
    pub events: Vec<InterferenceEvent>,
}

pub fn write_interference_payload_v1<W: Write>(
    writer: &mut W,
    payload: &InterferencePayload,
) -> Result<(), BinaryFormatError> {
    write_u32(
        writer,
        usize_to_u32(payload.events.len(), "interference_events")?,
    )?;
    for event in &payload.events {
        write_u32(writer, event.sample_index)?;
        write_string_u16(writer, &event.tool_id)?;
        write_point3_f64(writer, event.position)?;
        write_point3_f64(writer, event.normal)?;
        write_f64(writer, event.penetration_depth)?;
        write_u8(writer, event.kind as u8)?;
    }
    Ok(())
}

pub fn read_interference_payload_v1<R: Read>(
    reader: &mut R,
) -> Result<InterferencePayload, BinaryFormatError> {
    let event_count = read_u32_le(reader)? as usize;
    let mut events = Vec::with_capacity(preallocated(event_count));

    for _ in 0..event_count {
        let sample_index = read_u32_le(reader)?;
        let tool_id = read_string_u16(reader)?;
        let position = read_point3_f64(reader)?;
        let normal = read_point3_f64(reader)?;
        let penetration_depth = read_f64_le(reader)?;
        let kind = InterferenceKind::from_u8(read_u8(reader)?)?;
        events.push(InterferenceEvent {
            sample_index,
            tool_id,
            position,
            normal,
            penetration_depth,
            kind,
        });
    }

    Ok(InterferencePayload { events })
}

/// [`read_artifact_v1`] が返す統合ペイロード型。
#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactPayload {
    ToolPath(ToolPath<f64>),
    Interference(InterferencePayload),
    /// 本モジュールでは中身を解釈しないバイト列
    CuttingSimulation(Vec<u8>),
}

/// ヘッダ + ペイロードを一括読み込みするエントリーポイント。
///
/// `ensure_acceptable_version()` を内部で必ず呼ぶため、
/// バージョン非対応のバイト列はこの関数が `Err` を返す。
/// 個別の `read_toolpath_payload_v1` / `read_interference_payload_v1` を
/// 直接呼ぶ場合はバージョン確認が呼び出し側の責任となる。
pub fn read_artifact_v1<R: Read>(
    reader: &mut R,
) -> Result<(ArtifactHeaderV1, ArtifactPayload), BinaryFormatError> {
    let header = ArtifactHeaderV1::read_from(reader)?;
    header.ensure_acceptable_version()?;
    let payload = match header.kind {
        ArtifactKind::ToolPath => {
            let mut payload = reader.by_ref().take(header.payload_len);
            let toolpath = read_toolpath_payload_v1(&mut payload)?;
            ensure_payload_consumed(&payload)?;
            ArtifactPayload::ToolPath(toolpath)
        }
        ArtifactKind::Interference => {
            let mut payload = reader.by_ref().take(header.payload_len);
            let interference = read_interference_payload_v1(&mut payload)?;
            ensure_payload_consumed(&payload)?;
            ArtifactPayload::Interference(interference)
        }
        ArtifactKind::CuttingSimulation => {
            let length = usize::try_from(header.payload_len)
                .map_err(|_| BinaryFormatError::LengthOverflow("payload_len"))?;
            // 長さはファイルに書かれた値のため、読んだバイトの数に応じて確保する
            let mut payload = Vec::with_capacity(length.min(MAX_PREALLOCATED_BYTES));
            reader.take(header.payload_len).read_to_end(&mut payload)?;
            if payload.len() != length {
                return Err(BinaryFormatError::Io(std::io::Error::from(
                    std::io::ErrorKind::UnexpectedEof,
                )));
            }
            ArtifactPayload::CuttingSimulation(payload)
        }
    };
    Ok((header, payload))
}

pub fn read_toolpath_artifact_v1<R: Read>(
    reader: &mut R,
) -> Result<(ArtifactHeaderV1, ToolPath<f64>), BinaryFormatError> {
    let (header, payload) = read_artifact_v1(reader)?;
    match payload {
        ArtifactPayload::ToolPath(toolpath) => Ok((header, toolpath)),
        ArtifactPayload::Interference(_) | ArtifactPayload::CuttingSimulation(_) => {
            Err(BinaryFormatError::ArtifactKindMismatch {
                expected: ArtifactKind::ToolPath,
                found: header.kind,
            })
        }
    }
}

pub fn read_interference_artifact_v1<R: Read>(
    reader: &mut R,
) -> Result<(ArtifactHeaderV1, InterferencePayload), BinaryFormatError> {
    let (header, payload) = read_artifact_v1(reader)?;
    match payload {
        ArtifactPayload::Interference(interference) => Ok((header, interference)),
        ArtifactPayload::ToolPath(_) | ArtifactPayload::CuttingSimulation(_) => {
            Err(BinaryFormatError::ArtifactKindMismatch {
                expected: ArtifactKind::Interference,
                found: header.kind,
            })
        }
    }
}

/// ヘッダの本体の長さの範囲で読んだ本体 `payload` を、ちょうど読み終えたことを確かめる（本体が余れば誤り）
fn ensure_payload_consumed<R>(payload: &std::io::Take<R>) -> Result<(), BinaryFormatError> {
    if payload.limit() == 0 {
        Ok(())
    } else {
        Err(BinaryFormatError::InvalidPayload("payload_len"))
    }
}

/// ファイルに書かれた個数 `count` から、先に確保する要素の数（上限で抑える）
fn preallocated(count: usize) -> usize {
    count.min(MAX_PREALLOCATED_ELEMENTS)
}

fn write_segments<W: Write>(
    writer: &mut W,
    segments: &[PathSegment<f64>],
) -> Result<(), BinaryFormatError> {
    write_u32(writer, usize_to_u32(segments.len(), "segments")?)?;
    for segment in segments {
        write_segment(writer, segment)?;
    }
    Ok(())
}

fn read_segments<R: Read>(reader: &mut R) -> Result<Vec<PathSegment<f64>>, BinaryFormatError> {
    let segment_count = read_u32_le(reader)? as usize;
    let mut segments = Vec::with_capacity(preallocated(segment_count));
    for _ in 0..segment_count {
        segments.push(read_segment(reader)?);
    }
    Ok(segments)
}

fn write_segment<W: Write>(
    writer: &mut W,
    segment: &PathSegment<f64>,
) -> Result<(), BinaryFormatError> {
    let (segment_type, feed_rate) = segment_type_to_wire(segment.segment_type);
    write_u8(writer, segment_type)?;
    write_f64(writer, feed_rate)?;

    write_ext_attributes(writer, &segment.ext_attributes)?;

    match &segment.geometry {
        PathGeometry::Line { end } => {
            write_u8(writer, 0)?;
            write_point3_f64(writer, segment.start)?;
            write_point3_f64(writer, *end)?;
        }
        PathGeometry::Arc {
            end,
            center,
            direction,
        } => {
            write_u8(writer, 1)?;
            write_point3_f64(writer, segment.start)?;
            write_point3_f64(writer, *end)?;
            write_point3_f64(writer, *center)?;
            write_u8(writer, arc_direction_to_wire(*direction))?;
        }
    }
    Ok(())
}

fn read_segment<R: Read>(reader: &mut R) -> Result<PathSegment<f64>, BinaryFormatError> {
    let segment_type_wire = read_u8(reader)?;
    let feed_rate = read_f64_le(reader)?;
    let ext_attributes = read_ext_attributes_list(reader)?;
    let geometry_type = read_u8(reader)?;

    let start = read_point3_f64(reader)?;
    let geometry = match geometry_type {
        0 => {
            let end = read_point3_f64(reader)?;
            PathGeometry::Line { end }
        }
        1 => {
            let end = read_point3_f64(reader)?;
            let center = read_point3_f64(reader)?;
            let direction = arc_direction_from_wire(read_u8(reader)?)?;
            PathGeometry::Arc {
                end,
                center,
                direction,
            }
        }
        other => return Err(BinaryFormatError::UnknownGeometryType(other)),
    };

    let segment_type = segment_type_from_wire(segment_type_wire, feed_rate)?;
    Ok(PathSegment {
        start,
        geometry,
        segment_type,
        ext_attributes,
    })
}

fn cutting_direction_to_wire(value: CuttingDirection) -> u8 {
    match value {
        CuttingDirection::Down => 0,
        CuttingDirection::Up => 1,
        CuttingDirection::Mixed => 2,
    }
}

fn cutting_direction_from_wire(value: u8) -> Result<CuttingDirection, BinaryFormatError> {
    match value {
        0 => Ok(CuttingDirection::Down),
        1 => Ok(CuttingDirection::Up),
        2 => Ok(CuttingDirection::Mixed),
        other => Err(BinaryFormatError::UnknownCuttingDirection(other)),
    }
}

fn segment_type_to_wire(value: SegmentType<f64>) -> (u8, f64) {
    match value {
        SegmentType::Cutting { feed_rate } => (0, feed_rate),
        SegmentType::Rapid => (1, 0.0),
        SegmentType::Approach { feed_rate } => (2, feed_rate),
        SegmentType::Retract { feed_rate } => (3, feed_rate),
        SegmentType::PassRetract { feed_rate } => (4, feed_rate),
    }
}

fn segment_type_from_wire(
    value: u8,
    feed_rate: f64,
) -> Result<SegmentType<f64>, BinaryFormatError> {
    match value {
        0 => Ok(SegmentType::Cutting { feed_rate }),
        1 => Ok(SegmentType::Rapid),
        2 => Ok(SegmentType::Approach { feed_rate }),
        3 => Ok(SegmentType::Retract { feed_rate }),
        4 => Ok(SegmentType::PassRetract { feed_rate }),
        other => Err(BinaryFormatError::UnknownSegmentType(other)),
    }
}

fn arc_direction_to_wire(value: ArcDirection) -> u8 {
    match value {
        ArcDirection::Clockwise => 0,
        ArcDirection::CounterClockwise => 1,
    }
}

fn arc_direction_from_wire(value: u8) -> Result<ArcDirection, BinaryFormatError> {
    match value {
        0 => Ok(ArcDirection::Clockwise),
        1 => Ok(ArcDirection::CounterClockwise),
        other => Err(BinaryFormatError::UnknownGeometryType(other)),
    }
}

fn write_string_u16<W: Write>(writer: &mut W, value: &str) -> Result<(), BinaryFormatError> {
    let bytes = value.as_bytes();
    let len = usize_to_u16(bytes.len(), "tool_id")?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

fn read_string_u16<R: Read>(reader: &mut R) -> Result<String, BinaryFormatError> {
    let len = read_u16_le(reader)? as usize;
    let mut bytes = vec![0_u8; len];
    reader.read_exact(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn write_point3_f64<W: Write>(
    writer: &mut W,
    point: Point3D<f64>,
) -> Result<(), BinaryFormatError> {
    write_f64(writer, point.x())?;
    write_f64(writer, point.y())?;
    write_f64(writer, point.z())?;
    Ok(())
}

fn read_point3_f64<R: Read>(reader: &mut R) -> Result<Point3D<f64>, BinaryFormatError> {
    let x = read_f64_le(reader)?;
    let y = read_f64_le(reader)?;
    let z = read_f64_le(reader)?;
    Ok(Point3D::new(x, y, z))
}

fn write_f64<W: Write>(writer: &mut W, value: f64) -> Result<(), BinaryFormatError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn read_f64_le<R: Read>(reader: &mut R) -> Result<f64, BinaryFormatError> {
    let mut buf = [0_u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<(), BinaryFormatError> {
    writer.write_all(&[value])?;
    Ok(())
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8, BinaryFormatError> {
    let mut buf = [0_u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<(), BinaryFormatError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn usize_to_u16(value: usize, field: &'static str) -> Result<u16, BinaryFormatError> {
    value
        .try_into()
        .map_err(|_| BinaryFormatError::LengthOverflow(field))
}

fn usize_to_u32(value: usize, field: &'static str) -> Result<u32, BinaryFormatError> {
    value
        .try_into()
        .map_err(|_| BinaryFormatError::LengthOverflow(field))
}

fn read_u16_le<R: Read>(reader: &mut R) -> Result<u16, BinaryFormatError> {
    let mut buf = [0_u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_i16_le<R: Read>(reader: &mut R) -> Result<i16, BinaryFormatError> {
    let mut buf = [0_u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, BinaryFormatError> {
    let mut buf = [0_u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, BinaryFormatError> {
    let mut buf = [0_u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_ext_attributes<R: Read>(
    reader: &mut R,
    ext_block_len: u32,
) -> Result<Vec<ExtAttribute>, BinaryFormatError> {
    let mut bytes_read: usize = 0;
    let total = ext_block_len as usize;
    let mut attrs = Vec::new();
    while bytes_read < total {
        let tag = read_i16_le(reader)?;
        if tag == 0 {
            return Err(BinaryFormatError::ReservedExtAttributeTag(tag));
        }
        let data_len = read_u16_le(reader)? as usize;
        // 属性は、拡張の区画の長さに収まること
        if bytes_read + 4 + data_len > total {
            return Err(BinaryFormatError::InvalidPayload("ext_block_len"));
        }
        let mut data = vec![0_u8; data_len];
        reader.read_exact(&mut data)?;
        bytes_read += 4 + data_len;
        attrs.push(ExtAttribute { tag, data });
    }
    Ok(attrs)
}

fn write_ext_attributes<W: Write>(
    writer: &mut W,
    attrs: &[ExtAttribute],
) -> Result<(), BinaryFormatError> {
    write_u32(writer, usize_to_u32(attrs.len(), "ext_attributes")?)?;
    for attr in attrs {
        if attr.is_reserved_tag() {
            return Err(BinaryFormatError::ReservedExtAttributeTag(attr.tag));
        }
        write_u16(writer, attr.tag as u16)?;
        write_u16(writer, usize_to_u16(attr.data.len(), "attr_data")?)?;
        writer.write_all(&attr.data)?;
    }
    Ok(())
}

fn read_ext_attributes_list<R: Read>(
    reader: &mut R,
) -> Result<Vec<ExtAttribute>, BinaryFormatError> {
    let count = read_u32_le(reader)? as usize;
    let mut attrs = Vec::with_capacity(preallocated(count));
    for _ in 0..count {
        let tag = read_i16_le(reader)?;
        if tag == 0 {
            return Err(BinaryFormatError::ReservedExtAttributeTag(tag));
        }
        let data_len = read_u16_le(reader)? as usize;
        let mut data = vec![0_u8; data_len];
        reader.read_exact(&mut data)?;
        attrs.push(ExtAttribute { tag, data });
    }
    Ok(attrs)
}

fn write_u16<W: Write>(writer: &mut W, value: u16) -> Result<(), BinaryFormatError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}
