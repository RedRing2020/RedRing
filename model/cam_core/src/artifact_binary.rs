//! CAM batch artifact binary format (v1).
//!
//! Issue #300 の初回実装として、共通ヘッダの Reader/Writer と
//! 互換性チェックを提供する。

use std::fmt;
use std::io::{Read, Write};

/// toolpath artifact magic: `RRTP`
pub const TOOLPATH_MAGIC: [u8; 4] = *b"RRTP";

/// interference artifact magic: `RRIN`
pub const INTERFERENCE_MAGIC: [u8; 4] = *b"RRIN";

pub const FORMAT_VERSION_MAJOR_V1: u16 = 1;
pub const FORMAT_VERSION_MINOR_V1: u16 = 0;

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
    UnsupportedMajorVersion { found: u16, supported: u16 },
    UnsupportedUnit(u8),
    UnsupportedCoordinateFrame(u8),
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
            Self::UnsupportedUnit(value) => write!(f, "unsupported length unit: {value}"),
            Self::UnsupportedCoordinateFrame(value) => {
                write!(f, "unsupported coordinate frame: {value}")
            }
        }
    }
}

impl std::error::Error for BinaryFormatError {}

impl From<std::io::Error> for BinaryFormatError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// Shared header for all CAM binary artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactHeaderV1 {
    pub kind: ArtifactKind,
    pub version_major: u16,
    pub version_minor: u16,
    pub unit: LengthUnit,
    pub frame: CoordinateFrame,
    pub reserved: [u8; 4],
    pub payload_len: u64,
}

impl ArtifactHeaderV1 {
    pub const BYTE_LEN: usize = 4 + 2 + 2 + 1 + 1 + 4 + 8;

    pub fn new(kind: ArtifactKind, payload_len: u64) -> Self {
        Self {
            kind,
            version_major: FORMAT_VERSION_MAJOR_V1,
            version_minor: FORMAT_VERSION_MINOR_V1,
            unit: LengthUnit::Millimeter,
            frame: CoordinateFrame::WorldRightHandedZUp,
            reserved: [0; 4],
            payload_len,
        }
    }

    pub fn is_supported_major(&self) -> bool {
        self.version_major == FORMAT_VERSION_MAJOR_V1
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

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), BinaryFormatError> {
        writer.write_all(&self.kind.magic())?;
        writer.write_all(&self.version_major.to_le_bytes())?;
        writer.write_all(&self.version_minor.to_le_bytes())?;
        writer.write_all(&[self.unit as u8])?;
        writer.write_all(&[self.frame as u8])?;
        writer.write_all(&self.reserved)?;
        writer.write_all(&self.payload_len.to_le_bytes())?;
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

        let mut reserved = [0_u8; 4];
        reader.read_exact(&mut reserved)?;

        let payload_len = read_u64_le(reader)?;

        Ok(Self {
            kind,
            version_major,
            version_minor,
            unit,
            frame,
            reserved,
            payload_len,
        })
    }
}

fn read_u16_le<R: Read>(reader: &mut R) -> Result<u16, BinaryFormatError> {
    let mut buf = [0_u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, BinaryFormatError> {
    let mut buf = [0_u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_round_trip_toolpath() {
        let header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 4096);

        let mut bytes = Vec::new();
        header.write_to(&mut bytes).unwrap();

        assert_eq!(bytes.len(), ArtifactHeaderV1::BYTE_LEN);

        let decoded = ArtifactHeaderV1::read_from(&mut bytes.as_slice()).unwrap();

        assert_eq!(decoded.kind, ArtifactKind::ToolPath);
        assert_eq!(decoded.version_major, FORMAT_VERSION_MAJOR_V1);
        assert_eq!(decoded.version_minor, FORMAT_VERSION_MINOR_V1);
        assert_eq!(decoded.payload_len, 4096);
        assert_eq!(decoded.unit, LengthUnit::Millimeter);
        assert_eq!(decoded.frame, CoordinateFrame::WorldRightHandedZUp);
    }

    #[test]
    fn unsupported_major_version_is_rejected() {
        let mut header = ArtifactHeaderV1::new(ArtifactKind::Interference, 10);
        header.version_major = 2;

        let result = header.ensure_supported_major();
        assert!(matches!(
            result,
            Err(BinaryFormatError::UnsupportedMajorVersion {
                found: 2,
                supported: 1
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
}
