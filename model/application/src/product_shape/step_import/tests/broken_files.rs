//! 壊れたファイルの試験（設計: BREP_AND_STEP_IMPORT_DESIGN.md 4.3・11 章）。適合の試験の STEP を壊した入力で、
//! 読み込みと三角形への分割がエラーを返すか、読める要素だけを読み、パニックしないことを確かめる

use super::super::super::shell_mesh::triangulate_brep;
use super::conformance::fixture;
use super::*;

/// 壊れた入力を三角形に分ける許容弦誤差（mm。パニックしないことだけを確かめるので粗くする）
const CHORD_TOLERANCE: f64 = 0.1;

/// 壊す元のファイル（平面・円柱・円錐・球・トーラス・穴・空洞・インチを含む）
const FILES: [&str; 6] = [
    "block_with_void.step",
    "cylinder_seam.step",
    "cone_inch.step",
    "sphere_seam.step",
    "torus.step",
    "plate_with_holes.step",
];

/// `bytes` を読み、読めたシェルをすべて三角形に分ける（パニックすれば試験が落ちる）
fn read_and_triangulate(bytes: &[u8]) -> Result<StepImportReport, StepImportError> {
    let (shells, report) = read_shells(bytes, "broken.step")?;
    assert_eq!(report.shells, shells.len());
    for shell in &shells {
        let mesh = triangulate_brep(&shell.brep, CHORD_TOLERANCE);
        assert_eq!(mesh.faces.len(), shell.brep.face_count());
    }
    Ok(report)
}

fn text(name: &str) -> String {
    String::from_utf8(fixture(name)).unwrap()
}

/// 種 `seed` から決まる疑似乱数（線形合同法）
struct Sequence(u64);

impl Sequence {
    fn next_below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % bound as u64) as usize
    }
}

#[test]
fn a_file_cut_anywhere_is_refused() {
    for name in FILES {
        let bytes = fixture(name);
        let end = text(name).find("END-ISO-10303-21;").unwrap() + "END-ISO-10303-21;".len();
        // 終わりの印の前で切ったファイルは、構文の誤り
        for cut in (0..end).step_by(3) {
            assert!(
                read_and_triangulate(&bytes[..cut]).is_err(),
                "{name} cut at {cut}"
            );
        }
    }
}

#[test]
fn changed_bytes_never_panic() {
    // 数・区切り・参照・文字列・列挙の印と、ASCII でないバイト
    const REPLACEMENTS: [u8; 14] = [
        b'0', b'9', b'-', b'.', b'E', b',', b'(', b')', b'#', b'\'', b'$', b'*', 0x00, 0xFF,
    ];
    let mut sequence = Sequence(0x5EED);
    for name in FILES {
        let bytes = fixture(name);
        let data = text(name).find("DATA;").unwrap();
        for _ in 0..200 {
            let mut changed = bytes.clone();
            let position = data + sequence.next_below(bytes.len() - data);
            changed[position] = REPLACEMENTS[sequence.next_below(REPLACEMENTS.len())];
            let _ = read_and_triangulate(&changed);
        }
    }
}

/// エンティティの番号の印 `#` を `%` で書いた文を直す（ソースコードの検査で、別の番号と見分けられないため）
fn entity(text: &str) -> String {
    text.replace('%', "#")
}

/// STEP の文 `text` で、`from` を `to` に替える（`from` があることを確かめる。`#` は `%` で書く）
fn replace(text: &str, from: &str, to: &str) -> Vec<u8> {
    let (from, to) = (entity(from), entity(to));
    assert!(text.contains(&from), "{from}");
    text.replace(&from, &to).into_bytes()
}

#[test]
fn large_or_non_finite_numbers_never_panic() {
    let cylinder = text("cylinder_seam.step");
    let cone = text("cone_inch.step");
    let torus = text("torus.step");
    let cases = [
        // 座標の上限を超える値と、f64 の最大に近い値（差が無限大になる）
        replace(&cylinder, "(10.,0.,0.)", "(1.E10,0.,0.)"),
        replace(&cylinder, "(10.,0.,0.)", "(1.7E308,0.,0.)"),
        replace(&cylinder, "(10.,0.,20.)", "(-1.7E308,1.7E308,20.)"),
        // 大きな半径・小さな半径・負の半径・0 の半径
        replace(&cylinder, ",10.)", ",1.E300)"),
        replace(&cylinder, "10.)", "1.E-300)"),
        replace(&cylinder, "10.)", "-10.)"),
        replace(&cylinder, "10.)", "0.)"),
        // 長さが 0 の向き（正規化すると NaN になる）と、大きな成分の向き
        replace(
            &cylinder,
            "DIRECTION('',(0.,0.,1.))",
            "DIRECTION('',(0.,0.,0.))",
        ),
        replace(
            &cylinder,
            "DIRECTION('',(0.,0.,1.))",
            "DIRECTION('',(1.E308,1.E308,1.E308))",
        ),
        // 円錐の半頂角が 0・直角・負（正接が 0・無限大・負）
        replace(&cone, ",30.)", ",0.)"),
        replace(&cone, ",30.)", ",90.)"),
        replace(&cone, ",30.)", ",-30.)"),
        // 単位の倍率が 0・大きな値・負
        replace(&cone, "LENGTH_MEASURE(25.4)", "LENGTH_MEASURE(0.)"),
        replace(&cone, "LENGTH_MEASURE(25.4)", "LENGTH_MEASURE(1.E300)"),
        replace(&cone, "LENGTH_MEASURE(25.4)", "LENGTH_MEASURE(-25.4)"),
        // 管の半径が中心の円の半径より大きいトーラス
        replace(&torus, ",30.,8.)", ",8.,30.)"),
        // 負の大半径（管が軸と交わる内側・大きさが小半径以上で面がない）
        replace(&torus, ",30.,8.)", ",-8.,30.)"),
        replace(&torus, ",30.,8.)", ",-30.,8.)"),
    ];
    // 読めない要素は、エラーにするか、外して結果に書く
    for bytes in &cases {
        assert_reported(read_and_triangulate(bytes));
    }
    // ファイルの許容誤差の大きな値・0（B-rep の許容誤差には使わないので、読めてよい）と、大きなエンティティの番号
    for bytes in [
        replace(&torus, "LENGTH_MEASURE(1.E-06)", "LENGTH_MEASURE(1.E300)"),
        replace(&torus, "LENGTH_MEASURE(1.E-06)", "LENGTH_MEASURE(0.)"),
        replace(&cylinder, "\n%1=", "\n%18446744073709551615="),
    ] {
        let _ = read_and_triangulate(&bytes);
    }
    // 書けない大きさの数・NaN の文字は、構文の誤り
    for bytes in [
        replace(&cylinder, "\n%1=", "\n%99999999999999999999="),
        replace(&cylinder, "(10.,0.,0.)", "(1.E999,0.,0.)"),
        replace(&cylinder, "(10.,0.,0.)", "(NaN,0.,0.)"),
    ] {
        assert!(matches!(
            read_and_triangulate(&bytes),
            Err(StepImportError::Syntax(_))
        ));
    }
}

/// STEP の文 `text` の DATA の節の、エンティティの番号と、引数の中の参照の位置（バイトの範囲）
fn references(text: &str) -> Vec<(String, Vec<std::ops::Range<usize>>)> {
    let data = text.find("DATA;").unwrap();
    let mut entities = Vec::new();
    let mut offset = data;
    for line in text[data..].lines() {
        if let Some((id, rest)) = line.split_once('=')
            && id.starts_with('#')
        {
            let start = offset + id.len() + 1;
            let bytes = rest.as_bytes();
            let mut ranges = Vec::new();
            let mut position = 0;
            while position < bytes.len() {
                if bytes[position] == b'#' {
                    let end = position
                        + 1
                        + bytes[position + 1..]
                            .iter()
                            .take_while(|byte| byte.is_ascii_digit())
                            .count();
                    ranges.push(start + position..start + end);
                    position = end;
                } else {
                    position += 1;
                }
            }
            entities.push((id.to_string(), ranges));
        }
        offset += line.len() + 1;
    }
    entities
}

#[test]
fn broken_and_cyclic_references_never_panic() {
    for name in ["block_with_void.step", "cylinder_seam.step", "torus.step"] {
        let text = text(name);
        let entities = references(&text);
        for (index, (id, ranges)) in entities.iter().enumerate() {
            let next = &entities[(index + 1) % entities.len()].0;
            for range in ranges {
                // ない番号・自分自身（循環）・次のエンティティ（型の違う要素や、2 つの間の循環）を指す
                for target in [entity("%999999").as_str(), id.as_str(), next.as_str()] {
                    let mut changed = text.clone();
                    changed.replace_range(range.clone(), target);
                    let _ = read_and_triangulate(changed.as_bytes());
                }
            }
        }
    }
}

/// 読めない要素を、エラーにするか、外して結果に書いたことを確かめる
fn assert_reported(result: Result<StepImportReport, StepImportError>) {
    assert!(
        result.is_err()
            || result.as_ref().is_ok_and(|report| {
                report.skipped_elements + report.skipped_occurrences + report.unplaced_occurrences
                    > 0
            }),
        "{result:?}"
    );
}

#[test]
fn deep_nesting_never_panics() {
    let block = text("block_with_void.step");
    let tail = "ENDSEC;\nEND-ISO-10303-21;\n";
    let body = &block[..block.rfind(tail).unwrap()];
    // ORIENTED_FACE を、上限（4 段）を超えて重ねた面を、シェルの 1 つ目の面にする
    let shell = block.find("CLOSED_SHELL('',(").unwrap() + "CLOSED_SHELL('',(".len();
    let first_face = &block[shell..shell + block[shell..].find(',').unwrap()];
    let mut nested = String::new();
    let mut inner = first_face.to_string();
    for depth in 0..10_000 {
        let id = format!("#{}", 1_000_000 + depth);
        nested.push_str(&format!("{id}=ORIENTED_FACE('',*,{inner},.T.);\n"));
        inner = id;
    }
    let shell_entry = format!("({first_face},");
    assert!(body.contains(&shell_entry));
    let deep_faces = body.replacen(&shell_entry, &format!("({inner},"), 1) + &nested + tail;
    assert_reported(read_and_triangulate(deep_faces.as_bytes()));
    // ORIENTED_EDGE を、ORIENTED_EDGE に重ねる
    let mut chain = String::new();
    let first_edge = references(&block)
        .into_iter()
        .find(|(id, _)| block.contains(&format!("{id}=ORIENTED_EDGE(")))
        .unwrap()
        .0;
    let mut inner = first_edge.clone();
    for depth in 0..10_000 {
        let id = format!("#{}", 2_000_000 + depth);
        chain.push_str(&format!("{id}=ORIENTED_EDGE('',*,*,{inner},.T.);\n"));
        inner = id;
    }
    let loop_entry = format!("({first_edge},");
    assert!(body.contains(&loop_entry));
    let deep_edges = body.replacen(&loop_entry, &format!("({inner},"), 1) + &chain + tail;
    assert_reported(read_and_triangulate(deep_edges.as_bytes()));
    // 引数の入れ子が、構文の上限（64 段）を超える
    let deep_list = format!(
        "{body}{}=CARTESIAN_POINT('',{}0.{});\n{tail}",
        entity("%3000000"),
        "(".repeat(100_000),
        ")".repeat(100_000)
    );
    assert_eq!(
        read_and_triangulate(deep_list.as_bytes()).err(),
        Some(StepImportError::TooLarge)
    );
}
