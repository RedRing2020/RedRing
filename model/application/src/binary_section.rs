//! 区画を並べたファイル（モデルファイル・ツールライブラリのファイル）の、バイトの読み書きと区画の並び
//!
//! 数値はすべてリトルエンディアン。文字列は長さ `u32` と UTF-8 のバイト列、有無は `u8`（0: なし、1: あり）、
//! UUID は 16 バイト、時刻は UNIX 時刻の秒 `i64` とナノ秒 `u32` で表す。
//!
//! 区画は、種類 `[u8; 4]`・版 `u16`・印 `u16`（bit0: 必須）・長さ `u64` と中身を持つ。区画の並びは、区画の数 `u32` と
//! 区画を並べたもの。読み込みは信用できない入力として扱う（個数・長さは残りのバイトの数で確かめてから確保する）。

use cam_core::BinaryFormatError;
use geo_io::binary_geometry::BinaryGeometryError;
use jiff::Timestamp;
use uuid::Uuid;

/// 区画の印: 必須（読む側が知らなければ開かない）
pub(crate) const SECTION_REQUIRED: u16 = 1;

/// 区画の中身の読み込みの誤り（項目の名前）
pub(crate) type FieldError = &'static str;

/// 区画の中身を書く
#[derive(Default)]
pub(crate) struct ByteWriter(pub(crate) Vec<u8>);

impl ByteWriter {
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    pub(crate) fn u8(&mut self, value: u8) {
        self.0.push(value);
    }

    pub(crate) fn u16(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    pub(crate) fn i32(&mut self, value: i32) {
        self.bytes(&value.to_le_bytes());
    }

    pub(crate) fn i64(&mut self, value: i64) {
        self.bytes(&value.to_le_bytes());
    }

    pub(crate) fn f32(&mut self, value: f32) {
        self.bytes(&value.to_le_bytes());
    }

    pub(crate) fn f64(&mut self, value: f64) {
        self.bytes(&value.to_le_bytes());
    }

    pub(crate) fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    pub(crate) fn uuid(&mut self, value: Uuid) {
        self.bytes(value.as_bytes());
    }

    /// 個数を `u32` で書く
    pub(crate) fn count(
        &mut self,
        count: usize,
        field: &'static str,
    ) -> Result<(), BinaryFormatError> {
        let count = u32::try_from(count).map_err(|_| BinaryFormatError::LengthOverflow(field))?;
        self.u32(count);
        Ok(())
    }

    pub(crate) fn string(
        &mut self,
        value: &str,
        field: &'static str,
    ) -> Result<(), BinaryFormatError> {
        self.count(value.len(), field)?;
        self.bytes(value.as_bytes());
        Ok(())
    }

    pub(crate) fn optional_string(
        &mut self,
        value: Option<&str>,
        field: &'static str,
    ) -> Result<(), BinaryFormatError> {
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
    pub(crate) fn time(&mut self, value: Timestamp) {
        const NANOS_PER_SECOND: i128 = 1_000_000_000;
        let nanos = value.as_nanosecond();
        // 時刻の範囲（1 万年より内）では、秒は i64 に、秒からのナノ秒は u32 に収まる
        self.i64(nanos.div_euclid(NANOS_PER_SECOND) as i64);
        self.u32(nanos.rem_euclid(NANOS_PER_SECOND) as u32);
    }
}

/// 区画の中身を読む（信用できない入力として、残りのバイトの数を確かめながら読む）
pub(crate) struct ByteReader<'a> {
    bytes: &'a [u8],
}

impl<'a> ByteReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// `length` バイトを切り出す
    pub(crate) fn take(&mut self, length: usize) -> Result<&'a [u8], FieldError> {
        if length > self.bytes.len() {
            return Err("unexpected end");
        }
        let (taken, rest) = self.bytes.split_at(length);
        self.bytes = rest;
        Ok(taken)
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], FieldError> {
        let (bytes, rest) = self
            .bytes
            .split_first_chunk::<N>()
            .ok_or("unexpected end")?;
        self.bytes = rest;
        Ok(*bytes)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, FieldError> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16, FieldError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, FieldError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, FieldError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    pub(crate) fn i32(&mut self) -> Result<i32, FieldError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    pub(crate) fn i64(&mut self) -> Result<i64, FieldError> {
        Ok(i64::from_le_bytes(self.array()?))
    }

    pub(crate) fn f32(&mut self) -> Result<f32, FieldError> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    pub(crate) fn f64(&mut self) -> Result<f64, FieldError> {
        Ok(f64::from_le_bytes(self.array()?))
    }

    pub(crate) fn bool(&mut self, field: FieldError) -> Result<bool, FieldError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(field),
        }
    }

    pub(crate) fn uuid(&mut self) -> Result<Uuid, FieldError> {
        Ok(Uuid::from_bytes(self.array()?))
    }

    /// 個数 `u32` を読み、1 つ `element_len` バイト以上の要素がその数だけ残りのバイトに収まることを確かめる
    pub(crate) fn count(&mut self, element_len: usize) -> Result<usize, FieldError> {
        let count = self.u32()? as usize;
        match count.checked_mul(element_len) {
            Some(length) if length <= self.bytes.len() => Ok(count),
            _ => Err("count"),
        }
    }

    pub(crate) fn string(&mut self, field: FieldError) -> Result<String, FieldError> {
        let length = self.count(1).map_err(|_| field)?;
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| field)
    }

    pub(crate) fn optional_string(
        &mut self,
        field: FieldError,
    ) -> Result<Option<String>, FieldError> {
        if self.bool(field)? {
            self.string(field).map(Some)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn time(&mut self, field: FieldError) -> Result<Timestamp, FieldError> {
        let seconds = self.i64()?;
        let nanos = i32::try_from(self.u32()?).map_err(|_| field)?;
        Timestamp::new(seconds, nanos).map_err(|_| field)
    }

    pub(crate) fn geometry<T>(
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
    pub(crate) fn finish(&self) -> Result<(), FieldError> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err("trailing bytes")
        }
    }
}

/// 区画を 1 つ書く
pub(crate) fn write_section(
    out: &mut ByteWriter,
    kind: [u8; 4],
    version: u16,
    flags: u16,
    body: &[u8],
) {
    out.bytes(&kind);
    out.u16(version);
    out.u16(flags);
    out.u64(body.len() as u64);
    out.bytes(body);
}

/// 読んだ区画（中身は読んだバイト列を指す）
pub(crate) struct Section<'a> {
    pub(crate) kind: [u8; 4],
    pub(crate) version: u16,
    pub(crate) flags: u16,
    pub(crate) body: &'a [u8],
}

impl Section<'_> {
    /// 必須の区画か
    pub(crate) fn required(&self) -> bool {
        self.flags & SECTION_REQUIRED != 0
    }
}

/// 区画の並び `payload` を、区画の数が `max_sections` 以下で、余りなく読む（誤りは項目の名前）
pub(crate) fn read_sections(
    payload: &[u8],
    max_sections: usize,
) -> Result<Vec<Section<'_>>, FieldError> {
    let mut cursor = ByteReader::new(payload);
    // 区画 1 つは、種類・版・印・長さだけで 16 バイト
    let count = cursor.count(16).map_err(|_| "sections")?;
    if count > max_sections {
        return Err("sections");
    }
    let mut sections = Vec::with_capacity(count);
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
            return Err("section header");
        };
        let body = usize::try_from(length)
            .ok()
            .and_then(|length| cursor.take(length).ok())
            .ok_or("section length")?;
        sections.push(Section {
            kind,
            version,
            flags,
            body,
        });
    }
    cursor.finish().map_err(|_| "trailing bytes")?;
    Ok(sections)
}
