//! 枠（種類 1 バイト・長さ 4 バイトのリトルエンディアン・中身。設計: JOB_IPC_DESIGN.md 4.1）

use std::io::{self, Read, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// 要求の JSON の大きさの上限
pub const MAX_REQUEST_JSON_BYTES: usize = 64 * 1024;
/// 応答の JSON の大きさの上限（ジョブの一覧を 1 つの応答で返すため、要求より大きくする）
pub const MAX_RESPONSE_JSON_BYTES: usize = 16 * 1024 * 1024;
/// 送る中身の小分けの大きさの上限
pub const MAX_CHUNK_BYTES: usize = 1024 * 1024;
/// 投入の入力の大きさの合計の上限（受け手は、入力をすべて受け取ってから受付のフォルダに書く）
pub const MAX_INPUT_BYTES: u64 = 1 << 30;
/// ジョブのファイル 1 つの大きさの上限
pub const MAX_FILE_BYTES: u64 = 4 << 30;

/// 枠の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    /// 要求・応答（UTF-8 の JSON）
    Json,
    /// 送る中身の小分け
    Data,
    /// その中身を送り終えた
    DataEnd,
}

impl FrameKind {
    fn byte(self) -> u8 {
        match self {
            Self::Json => 0,
            Self::Data => 1,
            Self::DataEnd => 2,
        }
    }

    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Json),
            1 => Some(Self::Data),
            2 => Some(Self::DataEnd),
            _ => None,
        }
    }

    /// この種類の中身の大きさの上限（JSON は `json_limit`）
    fn limit(self, json_limit: usize) -> usize {
        match self {
            Self::Json => json_limit,
            Self::Data => MAX_CHUNK_BYTES,
            Self::DataEnd => 0,
        }
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// 枠を書く
///
/// # Errors
///
/// 中身が種類の上限を超える・書けない場合。
pub fn write_frame(writer: &mut impl Write, kind: FrameKind, bytes: &[u8]) -> io::Result<()> {
    if bytes.len() > kind.limit(MAX_RESPONSE_JSON_BYTES) {
        return Err(invalid(format!("a {kind:?} frame is too large")));
    }
    let mut header = [0u8; 5];
    header[0] = kind.byte();
    header[1..].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
    writer.write_all(&header)?;
    writer.write_all(bytes)
}

/// 枠を読む（JSON の大きさの上限は `json_limit`。枠の始まりで接続が終わっていれば `None`）
///
/// # Errors
///
/// 知らない種類・種類の上限を超える長さ・途中で終わった・読めない場合。
pub fn read_frame(
    reader: &mut impl Read,
    json_limit: usize,
) -> io::Result<Option<(FrameKind, Vec<u8>)>> {
    let mut header = [0u8; 5];
    let mut read = 0;
    while read < header.len() {
        match reader.read(&mut header[read..]) {
            Ok(0) if read == 0 => return Ok(None),
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(n) => read += n,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    let kind = FrameKind::from_byte(header[0]).ok_or_else(|| invalid("unknown frame kind"))?;
    let length = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    if length > kind.limit(json_limit) {
        return Err(invalid(format!("a {kind:?} frame is too large")));
    }
    let mut bytes = vec![0u8; length];
    reader.read_exact(&mut bytes)?;
    Ok(Some((kind, bytes)))
}

/// 値 `value` を JSON の枠で書く
///
/// # Errors
///
/// JSON が上限を超える・書けない場合。
pub fn write_json(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    write_frame(writer, FrameKind::Json, &bytes)
}

/// JSON の枠を読む（大きさの上限は `json_limit`。枠の始まりで接続が終わっていれば `None`）
///
/// # Errors
///
/// JSON の枠でない・上限を超える・形が違う・読めない場合。
pub fn read_json<T: DeserializeOwned>(
    reader: &mut impl Read,
    json_limit: usize,
) -> io::Result<Option<T>> {
    match read_frame(reader, json_limit)? {
        None => Ok(None),
        Some((FrameKind::Json, bytes)) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| invalid(error.to_string())),
        Some((kind, _)) => Err(invalid(format!("expected a JSON frame, got {kind:?}"))),
    }
}

/// 大きさ `size` の中身を `source` から読み、小分けの枠と、終わりの枠で書く
///
/// # Errors
///
/// `source` が `size` より短い・読めない・書けない場合。
pub fn write_data(writer: &mut impl Write, source: &mut impl Read, size: u64) -> io::Result<()> {
    let mut buffer = vec![0u8; size.min(MAX_CHUNK_BYTES as u64) as usize];
    let mut remaining = size;
    while remaining > 0 {
        let length = remaining.min(MAX_CHUNK_BYTES as u64) as usize;
        source.read_exact(&mut buffer[..length])?;
        write_frame(writer, FrameKind::Data, &buffer[..length])?;
        remaining -= length as u64;
    }
    write_frame(writer, FrameKind::DataEnd, &[])
}

/// 大きさ `size`（上限 `limit`）の中身を、小分けの枠と終わりの枠から読む
///
/// # Errors
///
/// `size` が `limit` を超える・受け取った長さが `size` と違う・ほかの枠が来た・読めない場合。
pub fn read_data(reader: &mut impl Read, size: u64, limit: u64) -> io::Result<Vec<u8>> {
    if size > limit {
        return Err(invalid("the data is too large"));
    }
    let mut bytes = Vec::with_capacity(size.min(MAX_CHUNK_BYTES as u64 * 16) as usize);
    loop {
        // 中身の間に JSON は来ないため、JSON の上限は要求のものでよい
        match read_frame(reader, MAX_REQUEST_JSON_BYTES)? {
            Some((FrameKind::Data, chunk)) => {
                if bytes.len() as u64 + chunk.len() as u64 > size {
                    return Err(invalid("more data than its size"));
                }
                bytes.extend_from_slice(&chunk);
            }
            Some((FrameKind::DataEnd, _)) if bytes.len() as u64 == size => return Ok(bytes),
            Some((FrameKind::DataEnd, _)) => return Err(invalid("less data than its size")),
            Some((FrameKind::Json, _)) => return Err(invalid("expected data, got a JSON frame")),
            None => return Err(io::ErrorKind::UnexpectedEof.into()),
        }
    }
}
