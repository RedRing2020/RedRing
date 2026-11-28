//! 適合の試験（設計: BREP_AND_STEP_IMPORT_DESIGN.md 11 章）。形を式で決めた部品の STEP（`tests/fixtures/step`）を
//! 読み、三角形にした結果の体積・面積・外形の箱と、B-rep の面の数を、式の値と比べる

use std::f64::consts::PI;

use super::super::super::shell_mesh::triangulate_brep;
use super::*;

/// 三角形に分ける許容弦誤差（mm）
const CHORD_TOLERANCE: f64 = 0.01;
/// 体積・面積の、式の値との相対の差の上限（許容弦誤差で曲面を内側の三角形にした差を含む）
const RELATIVE_TOLERANCE: f64 = 2.0e-3;

/// 試験の STEP のファイル `name` を読む
pub(super) fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../tests/fixtures/step/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// 読み込んだ部品を三角形にして測った値
#[derive(Debug)]
pub(super) struct Measured {
    pub(super) report: StepImportReport,
    pub(super) faces: usize,
    /// 三角形の向き（面の外向き）から求めた、シェルの囲む符号つきの体積
    pub(super) volume: f64,
    pub(super) area: f64,
    pub(super) min: [f64; 3],
    pub(super) max: [f64; 3],
}

/// STEP のバイト列を読み、1 つのシェルを配置なしの位置で三角形にして測る
pub(super) fn measure(bytes: &[u8]) -> Measured {
    let (shells, report) = read_shells(bytes, "part.step").unwrap();
    let [shell] = shells.as_slice() else {
        panic!("one shell is read");
    };
    let mesh = triangulate_brep(&shell.brep, CHORD_TOLERANCE);
    // 原点から遠い部品の桁落ちを避けるため、体積は 1 つ目の点を基準に求める
    let reference = mesh.points[0];
    let offset = |index: usize| {
        let point = mesh.points[index];
        [
            point.x() - reference.x(),
            point.y() - reference.y(),
            point.z() - reference.z(),
        ]
    };
    let (mut volume, mut area) = (0.0, 0.0);
    for position in 0..mesh.faces.len() {
        let face = mesh.face_within_tolerance(position).unwrap();
        for triangle in &face.triangles {
            let [p, q, r] = triangle.map(|corner| offset(face.vertices[corner]));
            let normal = cross(sub(q, p), sub(r, p));
            area += dot(normal, normal).sqrt() / 2.0;
            volume += dot(p, cross(q, r)) / 6.0;
        }
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in &mesh.points {
        for (axis, value) in [point.x(), point.y(), point.z()].into_iter().enumerate() {
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }
    Measured {
        report,
        faces: shell.brep.face_count(),
        volume,
        area,
        min,
        max,
    }
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `actual` が `expected` に、相対の差 `RELATIVE_TOLERANCE` で近いことを確かめる
pub(super) fn assert_relative(what: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= expected.abs() * RELATIVE_TOLERANCE,
        "{what}: {actual}, expected {expected}"
    );
}

/// 部品の式の値
struct Expected {
    file: &'static str,
    faces: usize,
    volume: f64,
    area: f64,
    min: [f64; 3],
    max: [f64; 3],
}

fn expected_parts() -> Vec<Expected> {
    let inch = 25.4;
    let cone_inch_height = inch / 30.0_f64.to_radians().tan();
    let cylinder = |file, faces, origin: [f64; 3]| Expected {
        file,
        faces,
        volume: PI * 100.0 * 20.0,
        area: 2.0 * PI * 10.0 * 20.0 + 2.0 * PI * 100.0,
        min: [origin[0] - 10.0, origin[1] - 10.0, origin[2]],
        max: [origin[0] + 10.0, origin[1] + 10.0, origin[2] + 20.0],
    };
    let sphere = |file, faces| Expected {
        file,
        faces,
        volume: 4.0 / 3.0 * PI * 1000.0,
        area: 4.0 * PI * 100.0,
        min: [-10.0; 3],
        max: [10.0; 3],
    };
    let torus = |file, faces| Expected {
        file,
        faces,
        volume: 2.0 * PI * PI * 30.0 * 64.0,
        area: 4.0 * PI * PI * 30.0 * 8.0,
        min: [-38.0, -38.0, -8.0],
        max: [38.0, 38.0, 8.0],
    };
    // 大半径 −30・小半径 50 のトーラスの内側（レモン形）。軸から ρ(z) = √(50² − z²) − 30 の回転体で、
    // |z| ≤ c の部分の体積は π ∫ ρ² dz、側面の面積は 2π · 50 ∫ (1 − 30 / √(50² − z²)) dz
    let (lemon_major, lemon_minor) = (30.0_f64, 50.0_f64);
    let lemon_volume = |c: f64| {
        let root = (lemon_minor * lemon_minor - c * c).sqrt();
        PI * (2.0 * c * (lemon_minor * lemon_minor + lemon_major * lemon_major)
            - 2.0 * c * c * c / 3.0
            - 2.0 * lemon_major * (c * root + lemon_minor * lemon_minor * (c / lemon_minor).asin()))
    };
    let lemon_side_area =
        |c: f64| 2.0 * PI * lemon_minor * (2.0 * c - 2.0 * lemon_major * (c / lemon_minor).asin());
    vec![
        // 樽: 側面を半周ずつ 2 つの面にし、上と下を平面でふさぐ（端の円の半径 10）
        Expected {
            file: "barrel.step",
            faces: 4,
            volume: lemon_volume(30.0),
            area: lemon_side_area(30.0) + 2.0 * PI * 100.0,
            min: [-20.0, -20.0, -30.0],
            max: [20.0, 20.0, 30.0],
        },
        // 先まで: 先（軸の上の点）は高さ ±40
        Expected {
            file: "spindle.step",
            faces: 2,
            volume: lemon_volume(40.0),
            area: lemon_side_area(40.0),
            min: [-20.0, -20.0, -40.0],
            max: [20.0, 20.0, 40.0],
        },
        Expected {
            file: "block.step",
            faces: 6,
            volume: 40.0 * 30.0 * 20.0,
            area: 2.0 * (40.0 * 30.0 + 40.0 * 20.0 + 30.0 * 20.0),
            min: [0.0; 3],
            max: [40.0, 30.0, 20.0],
        },
        cylinder("cylinder.step", 3, [0.0; 3]),
        cylinder("cylinder_seam.step", 3, [0.0; 3]),
        cylinder("cylinder_far.step", 3, [1.0e6, -2.0e6, 5.0e5]),
        Expected {
            file: "cone.step",
            faces: 2,
            volume: PI * 100.0 * 15.0 / 3.0,
            area: PI * 100.0 + PI * 10.0 * 325.0_f64.sqrt(),
            min: [-10.0, -10.0, 0.0],
            max: [10.0, 10.0, 15.0],
        },
        Expected {
            file: "cone_inch.step",
            faces: 2,
            volume: PI * inch * inch * cone_inch_height / 3.0,
            area: PI * inch * inch + PI * inch * (2.0 * inch),
            min: [-inch, -inch, 0.0],
            max: [inch, inch, cone_inch_height],
        },
        sphere("sphere.step", 2),
        sphere("sphere_seam.step", 1),
        torus("torus.step", 2),
        torus("torus_seam.step", 1),
        Expected {
            file: "plate_with_holes.step",
            faces: 12,
            volume: 100.0 * 60.0 * 10.0 - 6.0 * PI * 36.0 * 10.0,
            area: 2.0 * (100.0 * 60.0 - 6.0 * PI * 36.0)
                + 2.0 * (100.0 + 60.0) * 10.0
                + 6.0 * 2.0 * PI * 6.0 * 10.0,
            min: [0.0; 3],
            max: [100.0, 60.0, 10.0],
        },
        // 空洞のシェルは、囲む体積が負になる向きで読む
        Expected {
            file: "block_with_void.step",
            faces: 8,
            volume: 40.0 * 40.0 * 40.0 - 4.0 / 3.0 * PI * 1000.0,
            area: 6.0 * 40.0 * 40.0 + 4.0 * PI * 100.0,
            min: [-20.0; 3],
            max: [20.0; 3],
        },
    ]
}

#[test]
fn parts_defined_by_formulas_match_their_volume_box_and_faces() {
    for expected in expected_parts() {
        let measured = measure(&fixture(expected.file));
        let file = expected.file;
        assert_eq!(measured.faces, expected.faces, "{file}: faces");
        // 正しく書いたファイルなので、外す要素も直す向きもない
        assert_eq!(measured.report.skipped_elements, 0, "{file}");
        assert_eq!(measured.report.reoriented, 0, "{file}");
        assert_relative(&format!("{file}: volume"), measured.volume, expected.volume);
        assert_relative(&format!("{file}: area"), measured.area, expected.area);
        // 三角形の頂点は曲面の上にあり、外形の箱は許容弦誤差より内側には縮まない
        for axis in 0..3 {
            for (actual, formula) in [
                (measured.min[axis], expected.min[axis]),
                (measured.max[axis], expected.max[axis]),
            ] {
                assert!(
                    (actual - formula).abs() <= CHORD_TOLERANCE,
                    "{file}: box {actual}, expected {formula}"
                );
            }
        }
    }
}

#[test]
fn faces_with_and_without_seams_give_the_same_volume_and_area() {
    for (seamless, seamed) in [
        ("cylinder.step", "cylinder_seam.step"),
        ("sphere.step", "sphere_seam.step"),
        ("torus.step", "torus_seam.step"),
    ] {
        let seamless_part = measure(&fixture(seamless));
        let seamed_part = measure(&fixture(seamed));
        assert_relative(
            &format!("{seamed}: volume"),
            seamed_part.volume,
            seamless_part.volume,
        );
        assert_relative(
            &format!("{seamed}: area"),
            seamed_part.area,
            seamless_part.area,
        );
    }
}

/// STEP の文 `text` で、面 `face` の `same_sense` を逆にする
fn reverse_face_flag(text: &str, face: &str) -> String {
    let start = text
        .find(&format!("ADVANCED_FACE('{face}'"))
        .unwrap_or_else(|| panic!("the face {face}"));
    let end = start + text[start..].find(';').unwrap();
    let written = &text[start..end];
    let reversed = if written.ends_with(".T.)") {
        written.replace(",.T.)", ",.F.)")
    } else {
        written.replace(",.F.)", ",.T.)")
    };
    format!("{}{reversed}{}", &text[..start], &text[end..])
}

#[test]
fn a_wrong_face_flag_is_corrected_by_its_loops_and_keeps_the_volume() {
    // 面の向きの旗（same_sense）を逆に書いても、ループの回りから向きを直し、体積は旗の正しいファイルと合う
    // （半球やトーラスの半分のように、ループのどちらの側も面になる面は、ループの回りでは向きを決められないので、
    // ここでは確かめない）
    for (file, face) in [
        ("block.step", "top"),
        ("cylinder.step", "side"),
        ("cone.step", "side"),
        ("plate_with_holes.step", "hole"),
    ] {
        let text = String::from_utf8(fixture(file)).unwrap();
        let correct = measure(text.as_bytes());
        let reversed = measure(reverse_face_flag(&text, face).as_bytes());
        assert!(reversed.report.reoriented > 0, "{file}: {face}");
        assert!(reversed.volume > 0.0, "{file}: {face}");
        assert_relative(
            &format!("{file}: volume with the face {face} reversed"),
            reversed.volume,
            correct.volume,
        );
        assert_relative(
            &format!("{file}: area with the face {face} reversed"),
            reversed.area,
            correct.area,
        );
    }
}
