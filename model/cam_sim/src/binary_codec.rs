//! 成果物・入力のバイト列の、リトルエンディアンの数値の読み書き

use std::io::Read;

use cam_core::BinaryFormatError;

pub(crate) fn read_u8(reader: &mut &[u8]) -> Result<u8, BinaryFormatError> {
    let mut bytes = [0_u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

pub(crate) fn read_u16(reader: &mut &[u8]) -> Result<u16, BinaryFormatError> {
    let mut bytes = [0_u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

pub(crate) fn read_u32(reader: &mut &[u8]) -> Result<u32, BinaryFormatError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

pub(crate) fn read_u64(reader: &mut &[u8]) -> Result<u64, BinaryFormatError> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

pub(crate) fn read_f64(reader: &mut &[u8]) -> Result<f64, BinaryFormatError> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_le_bytes(bytes))
}

/// 個数を `u32` で書く（`u32` に収まらなければ `field` の長さの誤り）
pub(crate) fn write_count(
    payload: &mut Vec<u8>,
    count: usize,
    field: &'static str,
) -> Result<(), BinaryFormatError> {
    let count = u32::try_from(count).map_err(|_| BinaryFormatError::LengthOverflow(field))?;
    payload.extend_from_slice(&count.to_le_bytes());
    Ok(())
}
