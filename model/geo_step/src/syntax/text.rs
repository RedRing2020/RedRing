//! STEP の文字列のエスケープを直す（ISO 10303-21 の 6.4.3）
//!
//! - `\\` は `\`
//! - `\X\hh` は ISO 8859-1 の文字 `hh`
//! - `\X2\hhhh...\X0\` は UTF-16 の並び、`\X4\hhhhhhhh...\X0\` は UCS-4 の並び
//! - `\S\c` は文字 `c` の値に 128 を足した ISO 8859-1 の文字
//! - `\P?\`（ISO 8859 の部の切り替え）は飛ばす
//!
//! エスケープの外のバイトは、UTF-8 として読める部分はそのまま、読めないバイトだけを ISO 8859-1 として読む（規格に
//! 沿わないファイルのため）。閉じていない・知らないエスケープは書かれたとおりに残し、`\X2\`・`\X4\` の中の
//! 直せない文字は置き換え文字にする。どの入力でも、長さに比例する時間で終わる。

/// 文字列の中身 `raw`（前後の `'` と、`''` の重なりを除いたもの）のエスケープを直す
pub(super) fn decode(raw: &[u8]) -> String {
    let mut text = String::new();
    let mut plain_start = 0;
    let mut position = 0;
    while position < raw.len() {
        if raw.get(position) != Some(&b'\\') {
            position += 1;
            continue;
        }
        push_plain(
            &mut text,
            raw.get(plain_start..position).unwrap_or_default(),
        );
        position = escape(&mut text, raw, position);
        plain_start = position;
    }
    push_plain(&mut text, raw.get(plain_start..).unwrap_or_default());
    text
}

/// エスケープの外のバイト（UTF-8 として読めないバイトだけ ISO 8859-1）
fn push_plain(text: &mut String, bytes: &[u8]) {
    for chunk in bytes.utf8_chunks() {
        text.push_str(chunk.valid());
        text.extend(chunk.invalid().iter().map(|&byte| char::from(byte)));
    }
}

/// `position` の `\` から始まるエスケープを直し、次に読む位置を返す
fn escape(text: &mut String, raw: &[u8], position: usize) -> usize {
    let rest = raw.get(position..).unwrap_or_default();
    if rest.starts_with(b"\\\\") {
        text.push('\\');
        return position + 2;
    }
    if rest.starts_with(b"\\X2\\") {
        return wide(text, raw, position + 4, 4);
    }
    if rest.starts_with(b"\\X4\\") {
        return wide(text, raw, position + 4, 8);
    }
    if rest.starts_with(b"\\X\\")
        && let Some(value) = rest.get(3..5).and_then(hex)
    {
        text.push(char::from(value as u8));
        return position + 5;
    }
    if rest.starts_with(b"\\S\\")
        && let Some(&byte) = rest.get(3)
    {
        text.push(char::from(byte.wrapping_add(128)));
        return position + 4;
    }
    if rest.len() >= 4
        && rest.get(1) == Some(&b'P')
        && rest.get(3) == Some(&b'\\')
        && rest.get(2).is_some_and(|byte| byte.is_ascii_uppercase())
    {
        return position + 4;
    }
    text.push('\\');
    position + 1
}

/// `\X2\` か `\X4\` の後ろ（`start`）から `\X0\` まで、`digits` 桁ずつの 16 進の文字の並びを直す
///
/// `\X0\` で閉じていなければ、書かれたとおりに残す。
fn wide(text: &mut String, raw: &[u8], start: usize, digits: usize) -> usize {
    let Some(length) = raw
        .get(start..)
        .and_then(|rest| rest.windows(4).position(|window| window == b"\\X0\\"))
    else {
        push_plain(text, raw.get(start - 4..).unwrap_or_default());
        return raw.len();
    };
    let body = raw.get(start..start + length).unwrap_or_default();
    if digits == 4 {
        // 16 進でない・桁の足りない組は、置き換え文字にする（UTF-16 の並びとしても読めない値にする）
        let units: Vec<u16> = body
            .chunks(4)
            .map(|chunk| {
                hex(chunk)
                    .filter(|_| chunk.len() == 4)
                    .map_or(0xDC00, |value| value as u16)
            })
            .collect();
        text.extend(
            char::decode_utf16(units).map(|unit| unit.unwrap_or(char::REPLACEMENT_CHARACTER)),
        );
    } else {
        text.extend(body.chunks(8).map(|chunk| {
            hex(chunk)
                .filter(|_| chunk.len() == 8)
                .and_then(char::from_u32)
                .unwrap_or(char::REPLACEMENT_CHARACTER)
        }));
    }
    start + length + 4
}

/// 16 進の文字の並び（8 桁まで）の値
fn hex(digits: &[u8]) -> Option<u32> {
    if digits.is_empty() || digits.len() > 8 || !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()
}
