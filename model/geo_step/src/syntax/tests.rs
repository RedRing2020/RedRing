use super::*;

/// 試験の STEP のファイル（AP214 のヘッダ、単純・複合のエンティティ、いろいろな引数）。エンティティの番号の
/// 印 `#` は `%` で書き、`step` で直して使う（ソースコードの検査で、別の番号と見分けられないため）
const TEMPLATE: &str = "ISO-10303-21;
HEADER;
/* 注釈は飛ばす */
FILE_DESCRIPTION(('a sample'),'2;1');
FILE_NAME('part.stp','2026-11-16T00:00:00',('author'),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 3 1 1 }'));
ENDSEC;
DATA;
%20=CARTESIAN_POINT('origin',(0.,-1.5,2.5E-3));
%3=PRODUCT('It''s a part','\\X2\\90E8\\X0\\\\X\\E9',$,(%20,*));
%10=(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
%11=MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),%10);
%12=BINARY_THING(\"0FF\",.T.,-7);
ENDSEC;
END-ISO-10303-21;
";

/// `%` を、エンティティの番号の印 `#` に直した STEP の文
fn step(text: &str) -> String {
    text.replace('%', "#")
}

fn sample() -> String {
    step(TEMPLATE)
}

fn parse_text(text: &str) -> Result<StepFile, SyntaxError> {
    parse(text.as_bytes(), &Limits::default())
}

fn record(file: &StepFile, id: u64) -> &Record {
    &file.entities[&id].records[0]
}

#[test]
fn a_file_becomes_an_entity_table_in_number_order() {
    let file = parse_text(&sample()).unwrap();

    assert_eq!(
        file.schemas,
        vec!["AUTOMOTIVE_DESIGN { 1 0 10303 214 3 1 1 }"]
    );
    assert_eq!(
        file.entities.keys().copied().collect::<Vec<_>>(),
        vec![3, 10, 11, 12, 20]
    );
    assert!(file.duplicates.is_empty());

    let point = record(&file, 20);
    assert_eq!(point.name, "CARTESIAN_POINT");
    assert_eq!(
        point.parameters,
        vec![
            Parameter::String("origin".to_string()),
            Parameter::List(vec![
                Parameter::Real(0.0),
                Parameter::Real(-1.5),
                Parameter::Real(2.5e-3),
            ]),
        ]
    );
}

#[test]
fn every_kind_of_parameter_is_read() {
    let file = parse_text(&sample()).unwrap();

    // 文字列の '' と、\X2\（UTF-16）・\X\（ISO 8859-1）のエスケープ、$・* と参照
    assert_eq!(
        record(&file, 3).parameters,
        vec![
            Parameter::String("It's a part".to_string()),
            Parameter::String("部é".to_string()),
            Parameter::Unset,
            Parameter::List(vec![Parameter::Reference(20), Parameter::Derived]),
        ]
    );
    // 型つきの値
    assert_eq!(
        record(&file, 11).parameters[0],
        Parameter::Typed {
            name: "LENGTH_MEASURE".to_string(),
            value: Box::new(Parameter::Real(25.4)),
        }
    );
    // バイナリ・列挙・整数
    assert_eq!(
        record(&file, 12).parameters,
        vec![
            Parameter::Binary("0FF".to_string()),
            Parameter::Enumeration("T".to_string()),
            Parameter::Integer(-7),
        ]
    );
}

#[test]
fn a_complex_entity_has_several_records() {
    let file = parse_text(&sample()).unwrap();
    let names: Vec<&str> = file.entities[&10]
        .records
        .iter()
        .map(|record| record.name.as_str())
        .collect();

    assert_eq!(names, vec!["LENGTH_UNIT", "NAMED_UNIT", "SI_UNIT"]);
    assert_eq!(
        file.entities[&10].records[2].parameters,
        vec![
            Parameter::Enumeration("MILLI".to_string()),
            Parameter::Enumeration("METRE".to_string()),
        ]
    );
}

#[test]
fn a_repeated_number_keeps_the_first_entity() {
    let text = sample().replace(&step("%12=BINARY_THING"), &step("%3=BINARY_THING"));
    let file = parse_text(&text).unwrap();

    assert_eq!(record(&file, 3).name, "PRODUCT");
    assert_eq!(file.duplicates, vec![3]);
}

#[test]
fn a_byte_order_mark_and_several_data_sections_are_accepted() {
    let text = sample().replace(
        "ENDSEC;\nEND-ISO-10303-21;",
        &step(
            "ENDSEC;\nANCHOR;\n<anything> = %3;\nENDSEC;\nDATA;\n%30=FACE_BOUND($,.F.);\nENDSEC;\nEND-ISO-10303-21;",
        ),
    );
    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend_from_slice(text.as_bytes());

    let file = parse(&bytes, &Limits::default()).unwrap();
    assert!(file.entities.contains_key(&3));
    assert!(file.entities.contains_key(&30));
}

#[test]
fn escapes_are_decoded_or_kept_as_written() {
    assert_eq!(text::decode(br"a\\b"), "a\\b");
    assert_eq!(text::decode(br"\S\a"), "\u{e1}");
    assert_eq!(text::decode(br"\X4\0001F600\X0\"), "\u{1F600}");
    assert_eq!(text::decode(br"\PA\x"), "x");
    // 閉じていない \X2\ と、知らないエスケープは書かれたとおり
    assert_eq!(text::decode(br"\X2\90E8"), "\\X2\\90E8");
    assert_eq!(text::decode(br"\Q"), "\\Q");
    // UTF-8 として読めないバイトだけを ISO 8859-1 として読み、読める部分はそのまま
    assert_eq!(text::decode(b"caf\xE9"), "caf\u{e9}");
    assert_eq!(
        text::decode(
            "部品"
                .as_bytes()
                .iter()
                .chain(b"\xFF")
                .copied()
                .collect::<Vec<_>>()
                .as_slice()
        ),
        "部品\u{ff}"
    );
    // \X2\・\X4\ の中の直せない組は、置き換え文字
    assert_eq!(text::decode(br"\X2\00E9ZZZZ0041\X0\"), "\u{e9}\u{fffd}A");
    assert_eq!(text::decode(br"\X2\00E\X0\"), "\u{fffd}");
}

#[test]
fn limits_stop_large_or_deep_input() {
    let limits = |change: fn(&mut Limits)| {
        let mut limits = Limits::default();
        change(&mut limits);
        limits
    };
    let kind = |limits: Limits| parse(sample().as_bytes(), &limits).unwrap_err().kind;

    assert_eq!(
        kind(limits(|l| l.max_bytes = 10)),
        SyntaxErrorKind::TooLarge
    );
    assert_eq!(
        kind(limits(|l| l.max_entities = 4)),
        SyntaxErrorKind::TooManyEntities
    );
    assert_eq!(
        kind(limits(|l| l.max_parameters = 10)),
        SyntaxErrorKind::TooManyParameters
    );
    assert_eq!(kind(limits(|l| l.max_depth = 1)), SyntaxErrorKind::TooDeep);
    assert_eq!(
        kind(limits(|l| l.max_records = 2)),
        SyntaxErrorKind::TooManyRecords
    );
}

#[test]
fn malformed_input_is_an_error_with_its_line() {
    let error = |text: &str| parse_text(text).unwrap_err();

    assert_eq!(
        error("HEADER;").kind,
        SyntaxErrorKind::Unexpected("ISO-10303-21")
    );
    assert_eq!(
        error("ISO-10303-21;\nHEADER;\n").kind,
        SyntaxErrorKind::UnexpectedEnd
    );
    assert_eq!(
        error(
            "ISO-10303-21;
HEADER;
FILE_NAME('part"
        )
        .kind,
        SyntaxErrorKind::Unterminated
    );
    assert_eq!(
        error(
            "ISO-10303-21;
HEADER;
/* comment"
        )
        .kind,
        SyntaxErrorKind::Unterminated
    );
    let too_big = sample().replace("2.5E-3", "1.0E999");
    let too_big = error(&too_big);
    assert_eq!(too_big.kind, SyntaxErrorKind::InvalidNumber);
    assert_eq!(too_big.line, 9);
    let overflow = sample().replace("-7", "99999999999999999999");
    assert_eq!(error(&overflow).kind, SyntaxErrorKind::InvalidNumber);
    let stray = sample().replace(&step("%11="), &step("%11=@"));
    assert_eq!(error(&stray).kind, SyntaxErrorKind::Unexpected("a token"));
}

#[test]
fn truncated_or_changed_input_never_panics() {
    let sample = sample();
    let bytes = sample.as_bytes();
    // どこで切っても、読めるか、エラーを返す
    for end in 0..bytes.len() {
        let _ = parse(&bytes[..end], &Limits::default());
    }
    // 1 バイトずつ、いろいろな値に替えても同じ
    for position in 0..bytes.len() {
        for replacement in [b'(', b')', b'\'', b'#', b';', b'\\', b'.', 0x00, 0xFF] {
            let mut changed = bytes.to_vec();
            changed[position] = replacement;
            let _ = parse(&changed, &Limits::default());
        }
    }
    // 深い入れ子も、上限で止まる
    let deep = format!(
        "ISO-10303-21;HEADER;ENDSEC;DATA;{}=A({}{});ENDSEC;END-ISO-10303-21;",
        step("%1"),
        "(".repeat(100_000),
        ")".repeat(100_000)
    );
    assert_eq!(
        parse_text(&deep).unwrap_err().kind,
        SyntaxErrorKind::TooDeep
    );
}

#[test]
fn line_breaks_inside_strings_and_binaries_are_dropped() {
    let text = sample()
        .replace("'origin'", "'ori\r\ngin'")
        .replace("\"0FF\"", "\"0F\nF\"");
    let file = parse_text(&text).unwrap();

    assert_eq!(
        record(&file, 20).parameters[0],
        Parameter::String("origin".to_string())
    );
    assert_eq!(
        record(&file, 12).parameters[0],
        Parameter::Binary("0FF".to_string())
    );
    // バイナリの中の 16 進でない文字は、置けない文字
    let wrong = sample().replace("\"0FF\"", "\"0FG\"");
    assert_eq!(
        parse_text(&wrong).unwrap_err().kind,
        SyntaxErrorKind::Unexpected("a binary digit")
    );
}

#[test]
fn a_skipped_section_ends_only_at_its_own_endsec() {
    // 文字列・参照先の名前の中の ENDSEC; は終わりではなく、ENDSEC と ; の間の空白・注釈は飛ばす
    let text = sample().replace(
        "ENDSEC;\nEND-ISO-10303-21;",
        &step(
            "ENDSEC;\nREFERENCE;\n%40 = <other.stp#ENDSEC;>;\n%41 = 'ENDSEC;';\nENDSEC /* end */ ;\nDATA;\n%30=FACE_BOUND($,.F.);\nENDSEC;\nEND-ISO-10303-21;",
        ),
    );
    let file = parse_text(&text).unwrap();

    assert!(file.entities.contains_key(&30));
    assert!(!file.entities.contains_key(&40));
}

#[test]
fn line_numbers_count_cr_only_line_breaks() {
    let text = sample().replace('\n', "\r").replace("2.5E-3", "1.0E999");
    assert_eq!(parse_text(&text).unwrap_err().line, 9);
}
