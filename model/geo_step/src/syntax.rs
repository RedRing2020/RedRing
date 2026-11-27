//! STEP の構文（ISO 10303-21）: バイト列を、エンティティの表にする
//!
//! 幾何の計算をせず、形状の型も使わない（設計: BREP_AND_STEP_IMPORT_DESIGN.md 4.1・9.1）。ファイルから読んだ数で
//! 先に領域を取らず、上限（`Limits`）を超えたら型つきのエラーを返す。

#[cfg(test)]
mod tests;
mod text;

use std::collections::BTreeMap;

use crate::progress::{ENTITY_INTERVAL, Progress};

/// 構文の段の上限（設計: BREP_AND_STEP_IMPORT_DESIGN.md 4.3）
///
/// 呼び出し元が変えられるのは、ファイルの大きさだけ（ほかの上限は、構文の段の決まりとして固定する。入れ子の
/// 深さを大きくすると、読む処理のスタックがあふれるため）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// ファイルの大きさ（バイト）
    pub max_bytes: usize,
    /// エンティティの数（同じ番号の重なりも数える）
    pub(crate) max_entities: usize,
    /// 引数の数（入れ子の中の引数も数える）
    pub(crate) max_parameters: usize,
    /// 引数の入れ子の深さ
    pub(crate) max_depth: usize,
    /// 複合のエンティティの部分の数
    pub(crate) max_records: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 512 * 1024 * 1024,
            max_entities: 20_000_000,
            max_parameters: 20_000_000,
            max_depth: 64,
            max_records: 64,
        }
    }
}

/// 構文を読んだ結果
#[derive(Debug, Clone, PartialEq)]
pub struct StepFile {
    /// ヘッダの `FILE_SCHEMA` のスキーマの名前（AP の種類と版。例: `AUTOMOTIVE_DESIGN { 1 0 10303 214 3 1 1 }`）
    pub schemas: Vec<String>,
    /// エンティティの表（番号の順）
    pub entities: BTreeMap<u64, Entity>,
    /// 2 回以上出た番号（初めのものを表に入れ、後のものは捨てた）
    pub duplicates: Vec<u64>,
}

/// エンティティ（単純なエンティティは部分が 1 つ、複合のエンティティは部分が複数）
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub records: Vec<Record>,
}

/// エンティティの部分（名前と引数の並び。名前はファイルに書かれたとおり）
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub name: String,
    pub parameters: Vec<Parameter>,
}

/// 引数
#[derive(Debug, Clone, PartialEq)]
pub enum Parameter {
    Integer(i64),
    /// 実数（有限の値だけ）
    Real(f64),
    /// 文字列（エスケープを直したもの）
    String(String),
    /// 列挙（`.T.` の `T` など、点を除いた名前）
    Enumeration(String),
    /// ほかのエンティティの番号（番号の印 `#` に続く数）
    Reference(u64),
    List(Vec<Parameter>),
    /// 型つきの値（`LENGTH_MEASURE(1.5)` など）
    Typed {
        name: String,
        value: Box<Parameter>,
    },
    /// バイナリ（`"` の間の 16 進の文字）
    Binary(String),
    /// なし（`$`）
    Unset,
    /// 派生（`*`）
    Derived,
}

/// 構文を読めない理由
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxErrorKind {
    /// ファイルが大きさの上限を超える
    TooLarge,
    /// エンティティの数が上限を超える
    TooManyEntities,
    /// 引数の数が上限を超える
    TooManyParameters,
    /// 引数の入れ子が上限より深い
    TooDeep,
    /// 複合のエンティティの部分の数が上限を超える
    TooManyRecords,
    /// 途中で終わっている
    UnexpectedEnd,
    /// ここに置けない記号・語（期待していたもの）
    Unexpected(&'static str),
    /// 数を読めない（桁あふれ・有限でない実数を含む）
    InvalidNumber,
    /// 閉じていない文字列・注釈・バイナリ
    Unterminated,
    /// 進み具合の閉包に止めると言われて、途中で止めた
    Cancelled,
}

/// 構文を読めないエラー（行は 1 から数える）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub line: usize,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self.kind {
            SyntaxErrorKind::TooLarge => "the file is larger than the limit".to_string(),
            SyntaxErrorKind::TooManyEntities => "too many entities".to_string(),
            SyntaxErrorKind::TooManyParameters => "too many parameters".to_string(),
            SyntaxErrorKind::TooDeep => "parameters are nested too deeply".to_string(),
            SyntaxErrorKind::TooManyRecords => "too many parts in a complex entity".to_string(),
            SyntaxErrorKind::UnexpectedEnd => "unexpected end of file".to_string(),
            SyntaxErrorKind::Unexpected(expected) => format!("expected {expected}"),
            SyntaxErrorKind::InvalidNumber => "invalid number".to_string(),
            SyntaxErrorKind::Unterminated => "unterminated string, comment or binary".to_string(),
            SyntaxErrorKind::Cancelled => "cancelled".to_string(),
        };
        write!(f, "line {}: {text}", self.line)
    }
}

impl std::error::Error for SyntaxError {}

/// バイト列 `bytes` の構文を読み、エンティティの表にする
///
/// 先頭の BOM は飛ばす。`DATA` の節が複数あれば、まとめて 1 つの表にする。`ANCHOR`・`REFERENCE`・`SIGNATURE`
/// の節は飛ばす。
///
/// エンティティ 4,096 個ごとに、`progress` に進み具合（読んだバイトの数と、全体のバイトの数）を知らせる。
///
/// # Errors
///
/// 構文の誤り、上限 `limits` を超えるとき、または `progress` が止めると返したとき（[`SyntaxErrorKind::Cancelled`]）。
pub fn parse(
    bytes: &[u8],
    limits: &Limits,
    progress: &mut Progress<'_>,
) -> Result<StepFile, SyntaxError> {
    if bytes.len() > limits.max_bytes {
        return Err(SyntaxError {
            kind: SyntaxErrorKind::TooLarge,
            line: 1,
        });
    }
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    Parser {
        lexer: Lexer::new(bytes),
        limits,
        progress,
        parameters: 0,
        entities: 0,
    }
    .file()
}

/// 字句
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Keyword(String),
    Reference(u64),
    Integer(i64),
    Real(f64),
    String(String),
    Enumeration(String),
    Binary(String),
    Dollar,
    Star,
    Open,
    Close,
    Comma,
    Equals,
    Semicolon,
    End,
}

/// 字句を切り出す
struct Lexer<'a> {
    bytes: &'a [u8],
    position: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            line: 1,
        }
    }

    fn error(&self, kind: SyntaxErrorKind) -> SyntaxError {
        SyntaxError {
            kind,
            line: self.line,
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.position += 1;
        // LF・CR LF・CR だけの改行を、それぞれ 1 行に数える
        if byte == b'\n' || (byte == b'\r' && self.peek_byte() != Some(b'\n')) {
            self.line += 1;
        }
        Some(byte)
    }

    /// 改行（CR・LF）か（規格では、改行は字句の中身に含めない）
    fn is_line_break(byte: u8) -> bool {
        byte == b'\r' || byte == b'\n'
    }

    /// `start` から今の位置までの字句（ASCII だけの字句に使う）
    fn text_from(&self, start: usize) -> &'a str {
        self.bytes
            .get(start..self.position)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or_default()
    }

    /// 空白と注釈を飛ばす
    fn skip_blank(&mut self) -> Result<(), SyntaxError> {
        loop {
            match self.peek_byte() {
                Some(byte) if byte.is_ascii_whitespace() => {
                    self.bump();
                }
                Some(b'/') if self.bytes.get(self.position + 1) == Some(&b'*') => {
                    self.position += 2;
                    loop {
                        match self.bump() {
                            Some(b'*') if self.peek_byte() == Some(b'/') => {
                                self.position += 1;
                                break;
                            }
                            Some(_) => {}
                            None => return Err(self.error(SyntaxErrorKind::Unterminated)),
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    /// 次の字句
    fn next(&mut self) -> Result<Token, SyntaxError> {
        self.skip_blank()?;
        let Some(byte) = self.peek_byte() else {
            return Ok(Token::End);
        };
        let token = match byte {
            b'(' => self.single(Token::Open),
            b')' => self.single(Token::Close),
            b',' => self.single(Token::Comma),
            b'=' => self.single(Token::Equals),
            b';' => self.single(Token::Semicolon),
            b'$' => self.single(Token::Dollar),
            b'*' => self.single(Token::Star),
            b'#' => {
                self.bump();
                let start = self.position;
                while self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
                    self.bump();
                }
                let digits = self.text_from(start);
                if digits.is_empty() {
                    return Err(self.error(SyntaxErrorKind::Unexpected("an entity number")));
                }
                Token::Reference(
                    digits
                        .parse()
                        .map_err(|_| self.error(SyntaxErrorKind::InvalidNumber))?,
                )
            }
            b'\'' => Token::String(self.string()?),
            b'"' => Token::Binary(self.binary()?),
            b'.' => {
                self.bump();
                let name = self.take_while(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
                if name.is_empty() || self.bump() != Some(b'.') {
                    return Err(self.error(SyntaxErrorKind::Unexpected("an enumeration")));
                }
                Token::Enumeration(name)
            }
            b'+' | b'-' | b'0'..=b'9' => self.number()?,
            byte if byte.is_ascii_alphabetic() || byte == b'_' || byte == b'!' => {
                Token::Keyword(self.take_while(|byte| {
                    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' || byte == b'!'
                }))
            }
            _ => return Err(self.error(SyntaxErrorKind::Unexpected("a token"))),
        };
        Ok(token)
    }

    fn single(&mut self, token: Token) -> Token {
        self.bump();
        token
    }

    /// 条件に合うバイトを切り出す（ASCII だけの字句に使う）
    fn take_while(&mut self, accept: impl Fn(u8) -> bool) -> String {
        let start = self.position;
        while self.peek_byte().is_some_and(&accept) {
            self.bump();
        }
        String::from_utf8_lossy(self.bytes.get(start..self.position).unwrap_or_default())
            .into_owned()
    }

    /// 整数か実数（実数は小数点を含む）
    fn number(&mut self) -> Result<Token, SyntaxError> {
        let start = self.position;
        if matches!(self.peek_byte(), Some(b'+' | b'-')) {
            self.bump();
        }
        let digits_start = self.position;
        while self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
            self.bump();
        }
        if self.position == digits_start {
            return Err(self.error(SyntaxErrorKind::InvalidNumber));
        }
        let mut real = false;
        if self.peek_byte() == Some(b'.') {
            real = true;
            self.bump();
            while self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
                self.bump();
            }
            if matches!(self.peek_byte(), Some(b'E' | b'e')) {
                self.bump();
                if matches!(self.peek_byte(), Some(b'+' | b'-')) {
                    self.bump();
                }
                let exponent_start = self.position;
                while self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
                    self.bump();
                }
                if self.position == exponent_start {
                    return Err(self.error(SyntaxErrorKind::InvalidNumber));
                }
            }
        }
        let text = self.text_from(start);
        if real {
            match text.parse::<f64>() {
                Ok(value) if value.is_finite() => Ok(Token::Real(value)),
                _ => Err(self.error(SyntaxErrorKind::InvalidNumber)),
            }
        } else {
            text.parse::<i64>()
                .map(Token::Integer)
                .map_err(|_| self.error(SyntaxErrorKind::InvalidNumber))
        }
    }

    /// 文字列（`''` は `'` 1 つ。エスケープは `text::decode` で直す）
    fn string(&mut self) -> Result<String, SyntaxError> {
        self.bump();
        let mut raw = Vec::new();
        loop {
            match self.bump() {
                Some(b'\'') if self.peek_byte() == Some(b'\'') => {
                    self.bump();
                    raw.push(b'\'');
                }
                Some(b'\'') => return Ok(text::decode(&raw)),
                Some(byte) if Self::is_line_break(byte) => {}
                Some(byte) => raw.push(byte),
                None => return Err(self.error(SyntaxErrorKind::Unterminated)),
            }
        }
    }

    /// バイナリ（`"` の間の 16 進の文字。改行は含めない）
    fn binary(&mut self) -> Result<String, SyntaxError> {
        self.bump();
        let mut digits = String::new();
        loop {
            match self.bump() {
                Some(b'"') => return Ok(digits),
                Some(byte) if byte.is_ascii_hexdigit() => digits.push(char::from(byte)),
                Some(byte) if Self::is_line_break(byte) => {}
                Some(_) => return Err(self.error(SyntaxErrorKind::Unexpected("a binary digit"))),
                None => return Err(self.error(SyntaxErrorKind::Unterminated)),
            }
        }
    }

    /// 中身を構文として読まない節を、`ENDSEC` と `;` の後ろまで飛ばす
    ///
    /// 文字列・注釈・`<` と `>` の間（参照先の名前）の中の `ENDSEC` は、節の終わりとみなさない。`ENDSEC` と
    /// `;` の間の空白と注釈は飛ばす。
    fn skip_section(&mut self) -> Result<(), SyntaxError> {
        let marker = b"ENDSEC";
        loop {
            self.skip_blank()?;
            let Some(byte) = self.peek_byte() else {
                return Err(self.error(SyntaxErrorKind::UnexpectedEnd));
            };
            match byte {
                b'\'' => {
                    self.string()?;
                }
                b'<' => loop {
                    match self.bump() {
                        Some(b'>') => break,
                        Some(_) => {}
                        None => return Err(self.error(SyntaxErrorKind::Unterminated)),
                    }
                },
                byte if byte.is_ascii_alphabetic() || byte == b'_' || byte == b'!' => {
                    let start = self.position;
                    while self.peek_byte().is_some_and(|byte| {
                        byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' || byte == b'!'
                    }) {
                        self.bump();
                    }
                    if self.bytes.get(start..self.position) == Some(marker.as_slice()) {
                        self.skip_blank()?;
                        match self.bump() {
                            Some(b';') => return Ok(()),
                            Some(_) => return Err(self.error(SyntaxErrorKind::Unexpected("';'"))),
                            None => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                        }
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }
    }
}

/// 構文を読む
struct Parser<'a, 'l, 'p, 'f> {
    lexer: Lexer<'a>,
    limits: &'l Limits,
    progress: &'p mut Progress<'f>,
    parameters: usize,
    entities: usize,
}

impl Parser<'_, '_, '_, '_> {
    fn error(&self, kind: SyntaxErrorKind) -> SyntaxError {
        self.lexer.error(kind)
    }

    fn expect(&mut self, expected: Token, name: &'static str) -> Result<(), SyntaxError> {
        let token = self.lexer.next()?;
        if token == expected {
            Ok(())
        } else if token == Token::End {
            Err(self.error(SyntaxErrorKind::UnexpectedEnd))
        } else {
            Err(self.error(SyntaxErrorKind::Unexpected(name)))
        }
    }

    fn expect_keyword(&mut self, keyword: &str, name: &'static str) -> Result<(), SyntaxError> {
        self.expect(Token::Keyword(keyword.to_string()), name)
    }

    fn file(mut self) -> Result<StepFile, SyntaxError> {
        self.expect_keyword("ISO-10303-21", "ISO-10303-21")?;
        self.expect(Token::Semicolon, "';'")?;
        self.expect_keyword("HEADER", "HEADER")?;
        self.expect(Token::Semicolon, "';'")?;
        let schemas = self.header()?;
        let mut entities = BTreeMap::new();
        let mut duplicates = Vec::new();
        loop {
            match self.lexer.next()? {
                Token::Keyword(keyword) if keyword == "DATA" => {
                    // 第 3 版の DATA は、名前とスキーマの引数を持てる
                    match self.lexer.next()? {
                        Token::Semicolon => {}
                        Token::Open => {
                            self.list_rest(1)?;
                            self.expect(Token::Semicolon, "';'")?;
                        }
                        Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                        _ => return Err(self.error(SyntaxErrorKind::Unexpected("';'"))),
                    }
                    self.data(&mut entities, &mut duplicates)?;
                }
                Token::Keyword(keyword)
                    if matches!(keyword.as_str(), "ANCHOR" | "REFERENCE" | "SIGNATURE") =>
                {
                    self.lexer.skip_section()?;
                }
                Token::Keyword(keyword) if keyword == "END-ISO-10303-21" => {
                    self.expect(Token::Semicolon, "';'")?;
                    return Ok(StepFile {
                        schemas,
                        entities,
                        duplicates,
                    });
                }
                Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                _ => {
                    return Err(self.error(SyntaxErrorKind::Unexpected("DATA or END-ISO-10303-21")));
                }
            }
        }
    }

    /// ヘッダの節（`FILE_SCHEMA` のスキーマの名前を返す）
    fn header(&mut self) -> Result<Vec<String>, SyntaxError> {
        let mut schemas = Vec::new();
        loop {
            match self.lexer.next()? {
                Token::Keyword(keyword) if keyword == "ENDSEC" => {
                    self.expect(Token::Semicolon, "';'")?;
                    return Ok(schemas);
                }
                Token::Keyword(name) => {
                    let Record { name, parameters } = self.record(name)?;
                    self.expect(Token::Semicolon, "';'")?;
                    if name == "FILE_SCHEMA"
                        && let Some(Parameter::List(names)) = parameters.first()
                    {
                        schemas.extend(names.iter().filter_map(|name| match name {
                            Parameter::String(name) => Some(name.clone()),
                            _ => None,
                        }));
                    }
                }
                Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                _ => return Err(self.error(SyntaxErrorKind::Unexpected("a header entity"))),
            }
        }
    }

    /// データの節（`ENDSEC;` まで）
    fn data(
        &mut self,
        entities: &mut BTreeMap<u64, Entity>,
        duplicates: &mut Vec<u64>,
    ) -> Result<(), SyntaxError> {
        loop {
            let id = match self.lexer.next()? {
                Token::Keyword(keyword) if keyword == "ENDSEC" => {
                    return self.expect(Token::Semicolon, "';'");
                }
                Token::Reference(id) => id,
                Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                _ => return Err(self.error(SyntaxErrorKind::Unexpected("an entity instance"))),
            };
            self.entities += 1;
            if self.entities > self.limits.max_entities {
                return Err(self.error(SyntaxErrorKind::TooManyEntities));
            }
            if self.entities.is_multiple_of(ENTITY_INTERVAL)
                && (self.progress)(self.lexer.position, self.lexer.bytes.len()).is_break()
            {
                return Err(self.error(SyntaxErrorKind::Cancelled));
            }
            self.expect(Token::Equals, "'='")?;
            let records = match self.lexer.next()? {
                Token::Keyword(name) => vec![self.record(name)?],
                // 複合のエンティティ: (NAME(...) NAME(...) ...)
                Token::Open => {
                    let mut records = Vec::new();
                    loop {
                        match self.lexer.next()? {
                            Token::Close if !records.is_empty() => break,
                            Token::Keyword(name) => {
                                if records.len() >= self.limits.max_records {
                                    return Err(self.error(SyntaxErrorKind::TooManyRecords));
                                }
                                records.push(self.record(name)?);
                            }
                            Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                            _ => {
                                return Err(
                                    self.error(SyntaxErrorKind::Unexpected("an entity name"))
                                );
                            }
                        }
                    }
                    records
                }
                Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                _ => return Err(self.error(SyntaxErrorKind::Unexpected("an entity name"))),
            };
            self.expect(Token::Semicolon, "';'")?;
            match entities.entry(id) {
                std::collections::btree_map::Entry::Vacant(vacant) => {
                    vacant.insert(Entity { records });
                }
                std::collections::btree_map::Entry::Occupied(_) => duplicates.push(id),
            }
        }
    }

    /// 名前 `name` の後ろの、`(` から `)` までの引数の並びを読んだエンティティの部分
    fn record(&mut self, name: String) -> Result<Record, SyntaxError> {
        self.expect(Token::Open, "'('")?;
        let parameters = self.list_rest(1)?;
        Ok(Record { name, parameters })
    }

    /// `(` の後ろから `)` までの引数の並び（`depth` は入れ子の深さ）
    fn list_rest(&mut self, depth: usize) -> Result<Vec<Parameter>, SyntaxError> {
        if depth > self.limits.max_depth {
            return Err(self.error(SyntaxErrorKind::TooDeep));
        }
        let mut parameters = Vec::new();
        let mut token = self.lexer.next()?;
        if token == Token::Close {
            return Ok(parameters);
        }
        loop {
            parameters.push(self.parameter(token, depth)?);
            match self.lexer.next()? {
                Token::Comma => token = self.lexer.next()?,
                Token::Close => return Ok(parameters),
                Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
                _ => return Err(self.error(SyntaxErrorKind::Unexpected("',' or ')'"))),
            }
        }
    }

    /// 字句 `token` から始まる引数 1 つ
    fn parameter(&mut self, token: Token, depth: usize) -> Result<Parameter, SyntaxError> {
        self.parameters += 1;
        if self.parameters > self.limits.max_parameters {
            return Err(self.error(SyntaxErrorKind::TooManyParameters));
        }
        Ok(match token {
            Token::Integer(value) => Parameter::Integer(value),
            Token::Real(value) => Parameter::Real(value),
            Token::String(value) => Parameter::String(value),
            Token::Enumeration(value) => Parameter::Enumeration(value),
            Token::Reference(value) => Parameter::Reference(value),
            Token::Binary(value) => Parameter::Binary(value),
            Token::Dollar => Parameter::Unset,
            Token::Star => Parameter::Derived,
            Token::Open => Parameter::List(self.list_rest(depth + 1)?),
            Token::Keyword(name) => {
                if depth + 1 > self.limits.max_depth {
                    return Err(self.error(SyntaxErrorKind::TooDeep));
                }
                self.expect(Token::Open, "'('")?;
                let token = self.lexer.next()?;
                let value = self.parameter(token, depth + 1)?;
                self.expect(Token::Close, "')'")?;
                Parameter::Typed {
                    name,
                    value: Box::new(value),
                }
            }
            Token::End => return Err(self.error(SyntaxErrorKind::UnexpectedEnd)),
            _ => return Err(self.error(SyntaxErrorKind::Unexpected("a parameter"))),
        })
    }
}
