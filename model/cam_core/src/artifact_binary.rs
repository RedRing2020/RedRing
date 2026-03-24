//! CAM batch artifact binary format (v0.1).
//!
//! Issue #300 の初回実装として、共通ヘッダの Reader/Writer と
//! 互換性チェックを提供する。

use std::fmt;
use std::io::{Read, Write};

use geo_algorithms::Point3D;

use crate::toolpath::{
    ArcDirection, ContourLevelPath, CuttingDirection, PathGeometry, PathSegment, SegmentType,
    ToolPath,
};

/// toolpath artifact magic: `RRTP`
pub const TOOLPATH_MAGIC: [u8; 4] = *b"RRTP";

/// interference artifact magic: `RRIN`
pub const INTERFERENCE_MAGIC: [u8; 4] = *b"RRIN";

pub const FORMAT_VERSION_MAJOR_V1: u16 = 0;
pub const FORMAT_VERSION_MINOR_V1: u16 = 1;

/// v0.x 系で受理可能な minor version 一覧。
pub const KNOWN_MINOR_VERSIONS_V0: &[u16] = &[FORMAT_VERSION_MINOR_V1];

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
}

impl ArtifactKind {
    pub fn magic(self) -> [u8; 4] {
        match self {
            Self::ToolPath => TOOLPATH_MAGIC,
            Self::Interference => INTERFERENCE_MAGIC,
        }
    }

    pub fn from_magic(magic: [u8; 4]) -> Option<Self> {
        if magic == TOOLPATH_MAGIC {
            Some(Self::ToolPath)
        } else if magic == INTERFERENCE_MAGIC {
            Some(Self::Interference)
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
}

impl fmt::Display for BinaryFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::UnknownMagic(magic) => write!(f, "unknown artifact magic: {magic:?}"),
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
    let mut contour_levels = Vec::with_capacity(contour_count);
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
    let mut events = Vec::with_capacity(event_count);

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
        ArtifactKind::ToolPath => ArtifactPayload::ToolPath(read_toolpath_payload_v1(reader)?),
        ArtifactKind::Interference => {
            ArtifactPayload::Interference(read_interference_payload_v1(reader)?)
        }
    };
    Ok((header, payload))
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
    let mut segments = Vec::with_capacity(segment_count);
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
    }
}

fn cutting_direction_from_wire(value: u8) -> Result<CuttingDirection, BinaryFormatError> {
    match value {
        0 => Ok(CuttingDirection::Down),
        1 => Ok(CuttingDirection::Up),
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
    let mut attrs = Vec::with_capacity(count);
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

/// ゴールデンバイト: toolpath 最小ペイロード (tool_id="t", Down, 1 contour, 1 line segment)
///
/// Wire layout を一度確定させるための回帰テスト用定数。
/// この定数を変更する場合は必ず `FORMAT_VERSION_MINOR_V1` を +1 してください。
#[cfg(test)]
const GOLDEN_TOOLPATH_MINIMAL: &[u8] = &[
    // tool_id: len=1
    0x01, 0x00, // tool_id: "t"
    0x74, // cutting_direction: Down=0
    0x00, // ext_attributes: count=0
    0x00, 0x00, 0x00, 0x00, // approach_segments: count=0
    0x00, 0x00, 0x00, 0x00, // contour_levels: count=1
    0x01, 0x00, 0x00, 0x00, // contour level_index=0
    0x00, 0x00, 0x00, 0x00, // z_level=0.0
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // segments: count=1
    0x01, 0x00, 0x00, 0x00, // segment_type=Cutting=0
    0x00, // feed_rate=1.0 (f64 LE)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // ext_attributes: count=0
    0x00, 0x00, 0x00, 0x00, // geometry_type=Line=0
    0x00, // start: (0.0, 0.0, 0.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // end: (1.0, 0.0, 0.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // retract_segments: count=0
    0x00, 0x00, 0x00, 0x00,
];

/// ゴールデンバイト: interference 最小ペイロード (1 event, Tool kind)
///
/// Wire layout を一度確定させるための回帰テスト用定数。
/// この定数を変更する場合は必ず `FORMAT_VERSION_MINOR_V1` を +1 してください。
#[cfg(test)]
const GOLDEN_INTERFERENCE_MINIMAL: &[u8] = &[
    // event_count=1
    0x01, 0x00, 0x00, 0x00, // sample_index=0
    0x00, 0x00, 0x00, 0x00, // tool_id: len=1
    0x01, 0x00, // tool_id: "t"
    0x74, // position: (0.0, 0.0, 0.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // normal: (0.0, 0.0, 1.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // penetration_depth=0.5 (f64 LE)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x3F, // kind=Tool=1
    0x01,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolpath::TOOLPATH_SCHEMA_VERSION_V0_1;

    #[test]
    fn header_round_trip_toolpath() {
        let header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 4096);

        let mut bytes = Vec::new();
        header.write_to(&mut bytes).unwrap();

        assert_eq!(bytes.len(), ArtifactHeaderV1::FIXED_HEADER_LEN);

        let decoded = ArtifactHeaderV1::read_from(&mut bytes.as_slice()).unwrap();

        assert_eq!(decoded.kind, ArtifactKind::ToolPath);
        assert_eq!(decoded.version_major, FORMAT_VERSION_MAJOR_V1);
        assert_eq!(decoded.version_minor, FORMAT_VERSION_MINOR_V1);
        assert_eq!(decoded.payload_len, 4096);
        assert_eq!(decoded.unit, LengthUnit::Millimeter);
        assert_eq!(decoded.frame, CoordinateFrame::WorldRightHandedZUp);
    }

    #[test]
    fn ext_attribute_round_trip() {
        let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
        header
            .ext_attributes
            .push(ExtAttribute::new(-1_i16, b"sys-note".to_vec()));
        header
            .ext_attributes
            .push(ExtAttribute::new(2_i16, vec![0xDE, 0xAD]));

        let mut bytes = Vec::new();
        header.write_to(&mut bytes).unwrap();

        // fixed(22) + attr1(tag2+len2+data8=12) + attr2(tag2+len2+data2=6)
        assert_eq!(bytes.len(), ArtifactHeaderV1::FIXED_HEADER_LEN + 12 + 6);

        let decoded = ArtifactHeaderV1::read_from(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded.ext_attributes.len(), 2);
        assert_eq!(decoded.ext_attributes[0].tag, -1);
        assert_eq!(decoded.ext_attributes[0].data, b"sys-note");
        assert_eq!(decoded.ext_attributes[1].tag, 2);
        assert_eq!(decoded.ext_attributes[1].data, vec![0xDE, 0xAD]);
    }

    #[test]
    fn unsupported_major_version_is_rejected() {
        let mut header = ArtifactHeaderV1::new(ArtifactKind::Interference, 10);
        header.version_major = 1;

        let result = header.ensure_supported_major();
        assert!(matches!(
            result,
            Err(BinaryFormatError::UnsupportedMajorVersion {
                found: 1,
                supported: 0
            })
        ));
    }

    #[test]
    fn version_sync_with_toolpath_schema_constant() {
        assert_eq!(
            TOOLPATH_SCHEMA_VERSION_V0_1,
            (FORMAT_VERSION_MAJOR_V1, FORMAT_VERSION_MINOR_V1)
        );
    }

    /// wire layout バージョンのリテラルピンテスト。
    ///
    /// wire layout を変更して GOLDEN_* を更新する際は以下の 3 か所をセットで更新してください:
    ///   1. `FORMAT_VERSION_MINOR_V1` を +1
    ///   2. `GOLDEN_TOOLPATH_MINIMAL` / `GOLDEN_INTERFERENCE_MINIMAL` を新 layout に合わせて更新
    ///   3. このテストの `PINNED_MINOR` を `FORMAT_VERSION_MINOR_V1` の新しい値に更新
    ///      (= `KNOWN_MINOR_VERSIONS_V0` への追記も同時に必要)
    #[test]
    fn version_pin_matches_golden_generation() {
        /// golden 定数が生成された時点の FORMAT_VERSION_MINOR_V1。
        /// wire layout を変更して golden を更新する場合はここも同時に変更してください。
        const PINNED_MINOR: u16 = 1;
        assert_eq!(
            FORMAT_VERSION_MINOR_V1, PINNED_MINOR,
            "FORMAT_VERSION_MINOR_V1 が変わりました。\
             GOLDEN_TOOLPATH_MINIMAL / GOLDEN_INTERFERENCE_MINIMAL を新 layout に合わせ、\
             KNOWN_MINOR_VERSIONS_V0 に新しい minor を追加し、\
             この PINNED_MINOR も更新してください。"
        );
        assert!(
            KNOWN_MINOR_VERSIONS_V0.contains(&PINNED_MINOR),
            "PINNED_MINOR={PINNED_MINOR} が KNOWN_MINOR_VERSIONS_V0 に含まれていません。\
             バージョンを +1 した場合は KNOWN_MINOR_VERSIONS_V0 に追記してください。"
        );
    }

    #[test]
    fn known_minor_is_accepted() {
        let header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
        assert_eq!(
            header.compatibility_decision(),
            CompatibilityDecision::Accept
        );
        assert!(header.ensure_acceptable_version().is_ok());
    }

    #[test]
    fn unknown_minor_is_rejected() {
        let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
        header.version_minor = 99;

        assert_eq!(
            header.compatibility_decision(),
            CompatibilityDecision::Reject
        );
        assert!(matches!(
            header.ensure_acceptable_version(),
            Err(BinaryFormatError::UnsupportedMinorVersion {
                major: 0,
                found_minor: 99,
                ..
            })
        ));
    }

    #[test]
    fn different_major_is_convert_decision() {
        let mut header = ArtifactHeaderV1::new(ArtifactKind::Interference, 0);
        header.version_major = 2;
        header.version_minor = 0;

        assert_eq!(
            header.compatibility_decision(),
            CompatibilityDecision::Convert
        );
        assert!(matches!(
            header.ensure_acceptable_version(),
            Err(BinaryFormatError::ConverterRequired {
                found_major: 2,
                found_minor: 0
            })
        ));
    }

    #[test]
    fn unknown_magic_is_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"ABCD");
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.push(1_u8);
        bytes.push(1_u8);
        bytes.extend_from_slice(&[0_u8; 4]);
        bytes.extend_from_slice(&0_u64.to_le_bytes());

        let result = ArtifactHeaderV1::read_from(&mut bytes.as_slice());
        assert!(matches!(result, Err(BinaryFormatError::UnknownMagic(_))));
    }

    #[test]
    fn reserved_tag_zero_is_rejected_on_write() {
        let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
        header
            .ext_attributes
            .push(ExtAttribute::new(0, b"x".to_vec()));

        let mut bytes = Vec::new();
        let result = header.write_to(&mut bytes);
        assert!(matches!(
            result,
            Err(BinaryFormatError::ReservedExtAttributeTag(0))
        ));
    }

    #[test]
    fn reserved_tag_zero_is_rejected_on_read() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&TOOLPATH_MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION_MAJOR_V1.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_VERSION_MINOR_V1.to_le_bytes());
        bytes.push(LengthUnit::Millimeter as u8);
        bytes.push(CoordinateFrame::WorldRightHandedZUp as u8);
        bytes.extend_from_slice(&(5_u32).to_le_bytes());
        bytes.extend_from_slice(&(0_u64).to_le_bytes());
        bytes.extend_from_slice(&(0_i16).to_le_bytes());
        bytes.extend_from_slice(&(1_u16).to_le_bytes());
        bytes.push(0xAA);

        let result = ArtifactHeaderV1::read_from(&mut bytes.as_slice());
        assert!(matches!(
            result,
            Err(BinaryFormatError::ReservedExtAttributeTag(0))
        ));
    }

    #[test]
    fn toolpath_payload_round_trip_minimal_line() {
        let toolpath = ToolPath::new(
            "tool-1".to_string(),
            CuttingDirection::Down,
            vec![PathSegment::new_line(
                Point3D::new(0.0, 0.0, 5.0),
                Point3D::new(0.0, 0.0, 0.0),
                SegmentType::Approach { feed_rate: 300.0 },
            )],
            vec![ContourLevelPath::new(
                0,
                -1.0,
                vec![PathSegment::new_line(
                    Point3D::new(0.0, 0.0, -1.0),
                    Point3D::new(10.0, 0.0, -1.0),
                    SegmentType::Cutting { feed_rate: 600.0 },
                )],
            )],
            vec![PathSegment::new_line(
                Point3D::new(10.0, 0.0, -1.0),
                Point3D::new(10.0, 0.0, 5.0),
                SegmentType::Retract { feed_rate: 500.0 },
            )],
        );

        let mut bytes = Vec::new();
        write_toolpath_payload_v1(&mut bytes, &toolpath).unwrap();
        let decoded = read_toolpath_payload_v1(&mut bytes.as_slice()).unwrap();

        assert_eq!(decoded, toolpath);
    }

    #[test]
    fn interference_payload_round_trip_minimal() {
        let payload = InterferencePayload {
            events: vec![InterferenceEvent {
                sample_index: 10,
                tool_id: "tool-1".to_string(),
                position: Point3D::new(1.0, 2.0, 3.0),
                normal: Point3D::new(0.0, 0.0, 1.0),
                penetration_depth: 0.25,
                kind: InterferenceKind::Holder,
            }],
        };

        let mut bytes = Vec::new();
        write_interference_payload_v1(&mut bytes, &payload).unwrap();
        let decoded = read_interference_payload_v1(&mut bytes.as_slice()).unwrap();

        assert_eq!(decoded, payload);
    }

    #[test]
    fn interference_payload_empty_events_round_trip() {
        let payload = InterferencePayload { events: vec![] };

        let mut bytes = Vec::new();
        write_interference_payload_v1(&mut bytes, &payload).unwrap();
        let decoded = read_interference_payload_v1(&mut bytes.as_slice()).unwrap();

        assert_eq!(decoded.events.len(), 0);
    }

    #[test]
    fn unknown_interference_kind_is_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&6_u16.to_le_bytes());
        bytes.extend_from_slice(b"tool-1");
        bytes.extend_from_slice(&1.0_f64.to_le_bytes());
        bytes.extend_from_slice(&2.0_f64.to_le_bytes());
        bytes.extend_from_slice(&3.0_f64.to_le_bytes());
        bytes.extend_from_slice(&0.0_f64.to_le_bytes());
        bytes.extend_from_slice(&0.0_f64.to_le_bytes());
        bytes.extend_from_slice(&1.0_f64.to_le_bytes());
        bytes.extend_from_slice(&0.1_f64.to_le_bytes());
        bytes.push(9_u8);

        let result = read_interference_payload_v1(&mut bytes.as_slice());
        assert!(matches!(
            result,
            Err(BinaryFormatError::UnknownInterferenceKind(9))
        ));
    }

    // ----- wire layout regression tests -----
    // これらのテストは Write/Read どちらか一方だけが変更された場合に失敗します。
    // - toolpath_wire_write_regression: Writer だけが変わると失敗
    // - toolpath_wire_read_regression:  Reader だけが変わると失敗
    // - interference_wire_write_regression / interference_wire_read_regression: 同上
    // ゴールデン定数 (GOLDEN_TOOLPATH_MINIMAL / GOLDEN_INTERFERENCE_MINIMAL) を更新する場合は
    // 必ず FORMAT_VERSION_MINOR_V1 を +1 してください。

    fn make_minimal_toolpath() -> ToolPath<f64> {
        ToolPath::new(
            "t".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(
                0,
                0.0,
                vec![PathSegment::new_line(
                    Point3D::new(0.0, 0.0, 0.0),
                    Point3D::new(1.0, 0.0, 0.0),
                    SegmentType::Cutting { feed_rate: 1.0 },
                )],
            )],
            vec![],
        )
    }

    fn make_minimal_interference() -> InterferencePayload {
        InterferencePayload {
            events: vec![InterferenceEvent {
                sample_index: 0,
                tool_id: "t".to_string(),
                position: Point3D::new(0.0, 0.0, 0.0),
                normal: Point3D::new(0.0, 0.0, 1.0),
                penetration_depth: 0.5,
                kind: InterferenceKind::Tool,
            }],
        }
    }

    #[test]
    fn toolpath_wire_write_regression() {
        let mut bytes = Vec::new();
        write_toolpath_payload_v1(&mut bytes, &make_minimal_toolpath()).unwrap();
        assert_eq!(
            bytes, GOLDEN_TOOLPATH_MINIMAL,
            "toolpath の wire layout が変わっています。\
             FORMAT_VERSION_MINOR_V1 を +1 して GOLDEN_TOOLPATH_MINIMAL を更新してください。"
        );
    }

    #[test]
    fn toolpath_wire_read_regression() {
        let mut bytes = GOLDEN_TOOLPATH_MINIMAL;
        let decoded = read_toolpath_payload_v1(&mut bytes).unwrap();
        assert_eq!(
            decoded,
            make_minimal_toolpath(),
            "toolpath Reader がゴールデンバイトを正しく解釈できません。\
             Reader の変更が Writer と整合しているか確認してください。"
        );
    }

    #[test]
    fn interference_wire_write_regression() {
        let mut bytes = Vec::new();
        write_interference_payload_v1(&mut bytes, &make_minimal_interference()).unwrap();
        assert_eq!(
            bytes, GOLDEN_INTERFERENCE_MINIMAL,
            "interference の wire layout が変わっています。\
             FORMAT_VERSION_MINOR_V1 を +1 して GOLDEN_INTERFERENCE_MINIMAL を更新してください。"
        );
    }

    #[test]
    fn interference_wire_read_regression() {
        let mut bytes = GOLDEN_INTERFERENCE_MINIMAL;
        let decoded = read_interference_payload_v1(&mut bytes).unwrap();
        assert_eq!(
            decoded,
            make_minimal_interference(),
            "interference Reader がゴールデンバイトを正しく解釈できません。\
             Reader の変更が Writer と整合しているか確認してください。"
        );
    }

    // ----- read_artifact_v1 統合エントリーポイントのテスト -----

    fn make_toolpath_artifact_bytes(minor: u16) -> Vec<u8> {
        let toolpath = make_minimal_toolpath();
        let mut payload_bytes = Vec::new();
        write_toolpath_payload_v1(&mut payload_bytes, &toolpath).unwrap();

        let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload_bytes.len() as u64);
        header.version_minor = minor;
        let mut bytes = Vec::new();
        header.write_to(&mut bytes).unwrap();
        bytes.extend_from_slice(&payload_bytes);
        bytes
    }

    #[test]
    fn read_artifact_v1_returns_toolpath_payload() {
        let bytes = make_toolpath_artifact_bytes(FORMAT_VERSION_MINOR_V1);
        let (header, payload) = read_artifact_v1(&mut bytes.as_slice()).unwrap();

        assert_eq!(header.kind, ArtifactKind::ToolPath);
        assert!(matches!(payload, ArtifactPayload::ToolPath(_)));
        if let ArtifactPayload::ToolPath(tp) = payload {
            assert_eq!(tp, make_minimal_toolpath());
        }
    }

    #[test]
    fn read_artifact_v1_rejects_unsupported_minor() {
        let bytes = make_toolpath_artifact_bytes(99);
        let result = read_artifact_v1(&mut bytes.as_slice());
        assert!(
            matches!(
                result,
                Err(BinaryFormatError::UnsupportedMinorVersion {
                    found_minor: 99,
                    ..
                })
            ),
            "バージョン不一致は payload に達する前に Err を返す必要があります"
        );
    }
}
