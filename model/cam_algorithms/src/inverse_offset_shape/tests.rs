use geo_algorithms::{Point3D, TriangleMesh3D};
use geo_contracts::{
    Triangle3DBoundaryAccess, TriangleMesh3DConstructor, TriangleMesh3DProperties,
    TriangleMesh3DTriangleAccess,
};

use super::{InverseOffsetPart, InverseOffsetPartKind, InverseOffsetShape, in_range};
use crate::inverse_offset::CutterShape;

/// 格子の点の高さ `height` の地形（1 つおきの三角形は頂点の順を逆にして、面の向きをそろえない）
fn terrain(count: usize, height: impl Fn(f64, f64) -> f64, flip: bool) -> TriangleMesh3D<f64> {
    let mut vertices = Vec::new();
    for j in 0..=count {
        for i in 0..=count {
            let (x, y) = (i as f64, j as f64);
            vertices.push(Point3D::new(x, y, height(x, y)));
        }
    }
    let at = |i: usize, j: usize| j * (count + 1) + i;
    let mut triangles = Vec::new();
    for j in 0..count {
        for i in 0..count {
            for (k, triangle) in [
                [at(i, j), at(i + 1, j), at(i + 1, j + 1)],
                [at(i, j), at(i + 1, j + 1), at(i, j + 1)],
            ]
            .into_iter()
            .enumerate()
            {
                let reversed = flip && (i + j + k) % 2 == 0;
                triangles.push(if reversed {
                    [triangle[0], triangle[2], triangle[1]]
                } else {
                    triangle
                });
            }
        }
    }
    TriangleMesh3D::new(vertices, triangles).unwrap()
}

/// 尾根と谷のある地形
fn ridges(x: f64, y: f64) -> f64 {
    3.0 * (x / 2.0).sin() + 2.0 * (y / 3.0).cos()
}

/// x < 5 が高さ 4、x >= 5 が高さ 0 の段（x = 5 に鉛直な壁）
fn step() -> TriangleMesh3D<f64> {
    let vertices = vec![
        Point3D::new(0.0, 0.0, 4.0),
        Point3D::new(5.0, 0.0, 4.0),
        Point3D::new(5.0, 10.0, 4.0),
        Point3D::new(0.0, 10.0, 4.0),
        Point3D::new(5.0, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 0.0),
        Point3D::new(10.0, 10.0, 0.0),
        Point3D::new(5.0, 10.0, 0.0),
    ];
    let triangles = vec![
        [0, 1, 2],
        [0, 2, 3],
        // 鉛直な壁
        [1, 4, 7],
        [1, 7, 2],
        [4, 5, 6],
        [4, 6, 7],
    ];
    TriangleMesh3D::new(vertices, triangles).unwrap()
}

/// 形状の外も含む、細かい格子の点
fn grid(span: f64) -> impl Iterator<Item = (f64, f64)> {
    let steps = (span * 4.0) as usize + 8;
    (0..=steps)
        .flat_map(move |j| (0..=steps).map(move |i| (i, j)))
        .map(|(i, j)| (-1.0 + i as f64 * 0.27, -1.0 + j as f64 * 0.27))
}

/// 閉じた四面体（底面 z = 0、頂点 (2, 2, 5)）
fn tetrahedron() -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(0.0, 10.0, 0.0),
            Point3D::new(2.0, 2.0, 5.0),
        ],
        vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]],
    )
    .unwrap()
}

/// 格子の点ごとの工具先端の高さ
fn heights(shape: &InverseOffsetShape, span: f64) -> Vec<Option<f64>> {
    grid(span).map(|(x, y)| shape.tip_height_at(x, y)).collect()
}

fn assert_same(actual: &[Option<f64>], expected: &[Option<f64>]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (a, e)) in actual.iter().zip(expected).enumerate() {
        match (a, e) {
            (Some(a), Some(e)) => assert!((a - e).abs() < 1e-9, "{index}: {a} != {e}"),
            _ => assert_eq!(a, e, "{index}"),
        }
    }
}

const SHAPES: [CutterShape; 2] = [
    CutterShape::Ball { radius: 1.5 },
    CutterShape::Flat { radius: 1.5 },
];

#[test]
fn excluding_concave_elements_keeps_the_heights() {
    for shape in SHAPES {
        for (mesh, span) in [(terrain(12, ridges, true), 12.0), (step(), 10.0)] {
            let excluded = InverseOffsetShape::build(&mesh, shape, true).unwrap();
            let all = InverseOffsetShape::build(&mesh, shape, false).unwrap();
            assert_same(&heights(&excluded, span), &heights(&all, span));
        }
    }
}

#[test]
fn concave_and_flat_edges_and_their_vertices_are_excluded() {
    // 平らな面の内側の辺・頂点はすべて外し、縁の辺・頂点だけが残る
    let plane = terrain(4, |_, _| 2.0, true);
    let shape = InverseOffsetShape::build(&plane, CutterShape::Ball { radius: 1.0 }, true).unwrap();
    assert_eq!(shape.edges.len(), 16);
    assert_eq!(shape.vertices.len(), 16);
    // 尾根と谷のある地形では、谷の辺を外す
    let mesh = terrain(12, ridges, true);
    let excluded =
        InverseOffsetShape::build(&mesh, CutterShape::Ball { radius: 1.0 }, true).unwrap();
    let all = InverseOffsetShape::build(&mesh, CutterShape::Ball { radius: 1.0 }, false).unwrap();
    assert!(excluded.edges.len() < all.edges.len());
    assert!(excluded.vertices.len() < all.vertices.len());
    // 鉛直な壁の上の辺は凸として残し、下の辺は凹として外す（外を形から決めるため、壁でも判定できる）
    let wall = InverseOffsetShape::build(&step(), CutterShape::Ball { radius: 1.0 }, true).unwrap();
    let on_wall = |x: f64, z: f64| {
        wall.edges
            .iter()
            .any(|edge| edge.ends.iter().all(|end| end[0] == x && end[2] == z))
    };
    assert!(on_wall(5.0, 4.0));
    assert!(!on_wall(5.0, 0.0));
}

/// XZ の断面 `profile` を y = 0..10 に押し出した面（1 つおきの三角形は頂点の順を逆にする）
fn extruded(profile: &[(f64, f64)]) -> TriangleMesh3D<f64> {
    let mut vertices = Vec::new();
    for &(x, z) in profile {
        vertices.push(Point3D::new(x, 0.0, z));
        vertices.push(Point3D::new(x, 10.0, z));
    }
    let mut triangles = Vec::new();
    for k in 0..profile.len() - 1 {
        let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
        triangles.push([a, c, d]);
        triangles.push(if k % 2 == 0 { [a, b, d] } else { [a, d, b] });
    }
    TriangleMesh3D::new(vertices, triangles).unwrap()
}

/// 張り出しのある形: 上面 z = 4（x 0..10）、張り出しの前の面 x = 10、張り出しの下の面 z = 3（x 6..10、下向き）、
/// 奥の壁 x = 6、床 z = 0（x 6..14）
fn overhang() -> TriangleMesh3D<f64> {
    extruded(&[
        (0.0, 4.0),
        (10.0, 4.0),
        (10.0, 3.0),
        (6.0, 3.0),
        (6.0, 0.0),
        (14.0, 0.0),
    ])
}

#[test]
fn undercut_faces_are_offset_away_from_the_product_and_do_not_change_the_heights() {
    let radius = 1.0;
    let shape = InverseOffsetShape::new(&overhang(), CutterShape::Ball { radius }).unwrap();
    // 張り出しの下の面（z = 3、下向き）の要素は、下（材料のない側）へ工具の半径だけずれる
    let underside: Vec<_> = shape
        .faces
        .iter()
        .filter(|face| {
            [face.vertex_a(), face.vertex_b(), face.vertex_c()]
                .iter()
                .all(|p| p.x() >= 6.0 && p.x() <= 10.0 && (p.z() - (3.0 - radius)).abs() < 1e-12)
        })
        .collect();
    assert_eq!(underside.len(), 2);
    // 張り出しの下は上面で決まり、張り出しの外は床で決まる
    let tip = |x: f64| shape.tip_height_at(x, 5.0).unwrap();
    assert!((tip(8.0) - 4.0).abs() < 1e-12);
    assert!((tip(12.0) - 0.0).abs() < 1e-12);
    // 張り出しの角（10, 4）の上を工具の側面が転がる
    assert!((tip(10.5) - (4.0 + (1.0_f64 - 0.25).sqrt() - radius)).abs() < 1e-12);
    for shape in SHAPES {
        let excluded = InverseOffsetShape::build(&overhang(), shape, true).unwrap();
        let all = InverseOffsetShape::build(&overhang(), shape, false).unwrap();
        assert_same(&heights(&excluded, 14.0), &heights(&all, 14.0));
    }
}

#[test]
fn nearly_flat_ridges_are_kept() {
    // 尾根の高さ 1e-9 のほぼ平らな屋根（x = 5 の辺が尾根）。フラットの面の要素は面ごとに工具の半径だけ下りの向きに
    // ずれ、尾根の両側に間が開くため、尾根の辺を外すと上側の面に穴が開く
    let roof = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(5.0, 0.0, 1e-9),
            Point3D::new(5.0, 10.0, 1e-9),
            Point3D::new(0.0, 10.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
        ],
        vec![[0, 1, 2], [0, 2, 3], [1, 4, 5], [1, 5, 2]],
    )
    .unwrap();
    for shape in SHAPES {
        let excluded = InverseOffsetShape::build(&roof, shape, true).unwrap();
        let all = InverseOffsetShape::build(&roof, shape, false).unwrap();
        let excluded_heights = heights(&excluded, 10.0);
        assert_same(&excluded_heights, &heights(&all, 10.0));
        assert!(excluded.tip_height_at(5.0, 5.0).is_some());
    }
}

#[test]
fn the_order_of_triangle_vertices_does_not_change_the_heights() {
    for shape in SHAPES {
        let mixed = InverseOffsetShape::new(&terrain(12, ridges, true), shape).unwrap();
        let aligned = InverseOffsetShape::new(&terrain(12, ridges, false), shape).unwrap();
        assert_same(&heights(&mixed, 12.0), &heights(&aligned, 12.0));
    }
}

#[test]
fn nonpositive_or_nonfinite_radii_are_rejected() {
    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        for shape in [CutterShape::Ball { radius }, CutterShape::Flat { radius }] {
            assert!(
                InverseOffsetShape::new(&step(), shape).is_err(),
                "{shape:?}"
            );
        }
    }
}

#[test]
fn points_on_a_shared_edge_of_a_flat_face_have_a_height() {
    // 平らな四角形 [A, B, C]・[C, D, A] の共有辺 AC は外す。辺の上の点で、両方の三角形の要素が丸め誤差で外れない
    let face = TriangleMesh3D::new(
        [(0.1, 0.2), (9.8, 3.5), (6.5, 13.2), (-3.2, 9.9)]
            .map(|(x, y)| Point3D::new(x, y, 0.0))
            .to_vec(),
        vec![[0, 1, 2], [2, 3, 0]],
    )
    .unwrap();
    for shape in [
        CutterShape::Ball { radius: 0.1 },
        CutterShape::Flat { radius: 0.1 },
    ] {
        let shape = InverseOffsetShape::new(&face, shape).unwrap();
        let tip = shape.tip_height_at(0.9960000000000001, 2.0200000000000005);
        assert!(tip.is_some_and(|z| z.abs() < 1e-12), "{tip:?}");
    }
}

#[test]
fn flat_edge_heights_do_not_extend_beyond_the_edge() {
    // ほぼ鉛直な短い辺 (0, 0, 0)–(1e-8, 0, 100) の端のすぐ外で円が辺の延長に接する。交点は端点から許容誤差まで辺の
    // 外を含むため、辺に沿って延ばすと端点より高くなる
    let mesh = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1e-8, 0.0, 100.0),
            Point3D::new(0.0, -5.0, 0.0),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap();
    let shape = InverseOffsetShape::new(&mesh, CutterShape::Flat { radius: 1.0 }).unwrap();
    let tip = shape.tip_height_at(1e-8 + 5e-13, 1.0);
    assert!(tip.is_some_and(|z| z <= 100.0), "{tip:?}");
}

#[test]
fn unused_vertices_do_not_change_the_heights() {
    // どの三角形にも使われていない、極端に低い頂点
    let square = |extra: Option<Point3D<f64>>| {
        let mut vertices = vec![
            Point3D::new(0.0, 0.0, 10.0),
            Point3D::new(10.0, 0.0, 10.0),
            Point3D::new(10.0, 10.0, 10.0),
            Point3D::new(0.0, 10.0, 10.0),
        ];
        vertices.extend(extra);
        TriangleMesh3D::new(vertices, vec![[0, 1, 2], [0, 2, 3]]).unwrap()
    };
    let with_unused = square(Some(Point3D::new(0.0, 0.0, -1e20)));
    for shape in SHAPES {
        let expected = InverseOffsetShape::new(&square(None), shape).unwrap();
        let actual = InverseOffsetShape::new(&with_unused, shape).unwrap();
        assert_same(&heights(&actual, 10.0), &heights(&expected, 10.0));
        assert!((actual.tip_height_at(5.0, 5.0).unwrap() - 10.0).abs() < 1e-12);
    }
}

#[test]
fn edges_of_a_face_overlapping_its_reverse_side_are_kept() {
    // 同じ三角形が 2 枚重なる（向きをそろえると裏表になる）厚みのない板。辺は平らでなく板の縁のため残す
    let sheet = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(0.0, 10.0, 0.0),
        ],
        vec![[0, 1, 2], [0, 1, 2]],
    )
    .unwrap();
    for shape in SHAPES {
        let shape = InverseOffsetShape::new(&sheet, shape).unwrap();
        assert!(shape.tip_height_at(5.0, -0.25).is_some(), "{shape:?}");
    }
}

#[test]
fn downward_faces_of_open_shapes_exposed_from_above_have_heights() {
    // 張り出しの下の面 z = 3（外向きは下）が、上面 z = 4（x 0..10）より左（x -5..0）にはみ出し、上から見える
    let sheet = extruded(&[(0.0, 4.0), (10.0, 4.0), (10.0, 3.0), (-5.0, 3.0)]);
    for shape in SHAPES {
        let inverse = InverseOffsetShape::new(&sheet, shape).unwrap();
        let tip = inverse.tip_height_at(-3.0, 5.0);
        assert!(
            tip.is_some_and(|z| (z - 3.0).abs() < 1e-12),
            "{shape:?}: {tip:?}"
        );
        // 上面の下に隠れている部分は、上面で決まる
        assert!((inverse.tip_height_at(5.0, 5.0).unwrap() - 4.0).abs() < 1e-12);
        let all = InverseOffsetShape::build(&sheet, shape, false).unwrap();
        assert_same(&heights(&inverse, 10.0), &heights(&all, 10.0));
    }
}

#[test]
fn downward_faces_of_closed_shapes_are_not_indexed() {
    // 閉じた四面体の底面（外向きは下）は、上から届かない
    let tetrahedron = tetrahedron();
    for shape in SHAPES {
        let inverse = InverseOffsetShape::new(&tetrahedron, shape).unwrap();
        assert!(inverse.upper_sides.is_empty());
        let all = InverseOffsetShape::build(&tetrahedron, shape, false).unwrap();
        assert_same(&heights(&inverse, 10.0), &heights(&all, 10.0));
    }
}

#[test]
fn distant_low_triangles_do_not_change_the_heights() {
    // z = 10 の正方形と、XY で離れた z = -1e20 の三角形
    let square = |with_low: bool| {
        let mut vertices = vec![
            Point3D::new(0.0, 0.0, 10.0),
            Point3D::new(10.0, 0.0, 10.0),
            Point3D::new(10.0, 10.0, 10.0),
            Point3D::new(0.0, 10.0, 10.0),
        ];
        let mut triangles = vec![[0, 1, 2], [0, 2, 3]];
        if with_low {
            vertices.extend([
                Point3D::new(100.0, 100.0, -1e20),
                Point3D::new(110.0, 100.0, -1e20),
                Point3D::new(100.0, 110.0, -1e20),
            ]);
            triangles.push([4, 5, 6]);
        }
        TriangleMesh3D::new(vertices, triangles).unwrap()
    };
    for shape in SHAPES {
        let inverse = InverseOffsetShape::new(&square(true), shape).unwrap();
        let tip = inverse.tip_height_at(5.0, 5.0);
        assert!(
            tip.is_some_and(|z| (z - 10.0).abs() < 1e-12),
            "{shape:?}: {tip:?}"
        );
    }
}

#[test]
fn low_triangles_below_a_face_do_not_change_its_height() {
    // z = 10 の正方形の真下に z = -1e20 の三角形がある（同じ XY で両方の要素を調べる）
    let mesh = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 10.0),
            Point3D::new(10.0, 0.0, 10.0),
            Point3D::new(10.0, 10.0, 10.0),
            Point3D::new(0.0, 10.0, 10.0),
            Point3D::new(0.0, 0.0, -1e20),
            Point3D::new(10.0, 0.0, -1e20),
            Point3D::new(0.0, 10.0, -1e20),
        ],
        vec![[0, 1, 2], [0, 2, 3], [4, 5, 6]],
    )
    .unwrap();
    for shape in SHAPES {
        let inverse = InverseOffsetShape::new(&mesh, shape).unwrap();
        let tip = inverse.tip_height_at(5.0, 5.0);
        assert!(
            tip.is_some_and(|z| (z - 10.0).abs() < 1e-12),
            "{shape:?}: {tip:?}"
        );
    }
}

#[test]
fn steep_faces_do_not_extend_beyond_the_triangle() {
    // ほぼ鉛直な面。面の外の点を半直線の向きから見た距離で受け入れると、面を延ばした高さ（約 100.0025）が返る
    let mesh = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(2e-8, 0.0, 100.0),
            Point3D::new(0.0, -5.0, 0.0),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap();
    let inverse = InverseOffsetShape::new(&mesh, CutterShape::Flat { radius: 1.0 }).unwrap();
    let tip = inverse.tip_height_at(-1.0 + 2e-8 + 5e-13, 0.0);
    assert!(tip.is_some_and(|z| z <= 100.0), "{tip:?}");
}

/// フラットの工具先端の高さを、すべての三角形の、(x, y) を中心とする半径 `radius` の円の内側の部分の最も高い点から
/// 総当たりで求める（逆オフセット形状を使わない）
///
/// 円の内側の部分の高さは線形のため、最も高い点は、円の内側の頂点、辺と円の交点、面の上りの向きの円の縁の点、
/// （水平な面では）円の中心のどれか。
fn flat_reference_height(mesh: &TriangleMesh3D<f64>, radius: f64, x: f64, y: f64) -> Option<f64> {
    let mut highest: Option<f64> = None;
    let mut candidate = |z: f64| highest = Some(highest.map_or(z, |h| h.max(z)));
    for index in 0..mesh.triangle_count() {
        let triangle = mesh.triangle(index).unwrap();
        let corners = [
            triangle.vertex_a(),
            triangle.vertex_b(),
            triangle.vertex_c(),
        ]
        .map(|p| [p.x(), p.y(), p.z()]);
        for k in 0..3 {
            let (p, q) = (corners[k], corners[(k + 1) % 3]);
            if (p[0] - x).hypot(p[1] - y) <= radius {
                candidate(p[2]);
            }
            // 辺 p + t (q − p) と円の交点
            let d = [q[0] - p[0], q[1] - p[1]];
            let e = [p[0] - x, p[1] - y];
            let a = d[0] * d[0] + d[1] * d[1];
            if a == 0.0 {
                continue;
            }
            let b = d[0] * e[0] + d[1] * e[1];
            let discriminant = b * b - a * (e[0] * e[0] + e[1] * e[1] - radius * radius);
            if discriminant < 0.0 {
                continue;
            }
            for t in [
                (-b - discriminant.sqrt()) / a,
                (-b + discriminant.sqrt()) / a,
            ] {
                if (0.0..=1.0).contains(&t) {
                    candidate(p[2] + t * (q[2] - p[2]));
                }
            }
        }
        // 面を含む平面 z = z0 + g·(x − x0, y − y0) の、円の縁の上りの向きの点と円の中心
        let [a, b, c] = corners;
        let n = [
            (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]),
            (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]),
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]),
        ];
        if n[2] == 0.0 {
            continue;
        }
        let gradient = [-n[0] / n[2], -n[1] / n[2]];
        let slope = gradient[0].hypot(gradient[1]);
        let mut points = vec![[x, y]];
        if slope > 0.0 {
            points.push([
                x + radius * gradient[0] / slope,
                y + radius * gradient[1] / slope,
            ]);
        }
        for [px, py] in points {
            let inside = (0..3).all(|k| {
                let (p, q) = (corners[k], corners[(k + 1) % 3]);
                ((q[0] - p[0]) * (py - p[1]) - (q[1] - p[1]) * (px - p[0])) * n[2] >= 0.0
            });
            if inside {
                candidate(a[2] + gradient[0] * (px - a[0]) + gradient[1] * (py - a[1]));
            }
        }
    }
    highest
}

#[test]
fn flat_heights_match_the_highest_points_under_the_bottom_of_the_tool() {
    // 頂点・辺の要素を範囲（隣り合う面ベクトルの間）に限っても、工具の底の円の内側の最も高い点と同じ高さになる
    // 尾根の高さ `top`、尾根の両端の高さの差 `tilt` の屋根
    let roof = |top: f64, tilt: f64| {
        TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(5.0, 0.0, top),
                Point3D::new(5.0, 10.0, top + tilt),
                Point3D::new(0.0, 10.0, 0.0),
                Point3D::new(10.0, 0.0, 0.0),
                Point3D::new(10.0, 10.0, 0.0),
            ],
            vec![[0, 1, 2], [0, 2, 3], [1, 4, 5], [1, 5, 2]],
        )
        .unwrap()
    };
    let tetrahedron = tetrahedron();
    for (mesh, span) in [
        (terrain(12, ridges, true), 12.0),
        (
            terrain(8, |x, y| ridges(x, y) + 0.7 * x - 0.4 * y, false),
            8.0,
        ),
        (step(), 10.0),
        (overhang(), 14.0),
        (roof(3.0, 0.0), 10.0),
        (roof(3.0, 2.0), 10.0),
        (roof(1e-9, 0.0), 10.0),
        (roof(1e-9, 1e-9), 10.0),
        (tetrahedron, 10.0),
    ] {
        for radius in [0.6, 1.5] {
            let shape = InverseOffsetShape::new(&mesh, CutterShape::Flat { radius }).unwrap();
            let expected: Vec<Option<f64>> = grid(span)
                .map(|(x, y)| flat_reference_height(&mesh, radius, x, y))
                .collect();
            assert_same(&heights(&shape, span), &expected);
        }
    }
}

#[test]
fn flat_vertices_and_edges_are_limited_to_their_ranges() {
    // 尾根と谷のある地形では、工具の底が乗らない（つながる辺の先がより高い）頂点を外す
    let mesh = terrain(12, ridges, true);
    let flat = InverseOffsetShape::new(&mesh, CutterShape::Flat { radius: 1.0 }).unwrap();
    let ball = InverseOffsetShape::new(&mesh, CutterShape::Ball { radius: 1.0 }).unwrap();
    assert!(flat.vertices.len() < ball.vertices.len());
    // 斜めの辺は、辺の点から (x, y) への水平な向きのうち、上りの向きを範囲に持たない
    let directions = |edge: &super::EdgeElement| {
        (0..36)
            .map(|k| {
                let angle = std::f64::consts::TAU * k as f64 / 36.0;
                [angle.cos(), angle.sin(), 0.0]
            })
            .filter(|&h| in_range(h, &edge.range))
            .collect::<Vec<_>>()
    };
    for edge in &flat.edges {
        let [start, end] = edge.ends;
        let rise = end[2] - start[2];
        if rise.abs() < 1e-9 {
            continue;
        }
        let uphill = [rise * (end[0] - start[0]), rise * (end[1] - start[1]), 0.0];
        let length = super::dot(uphill, uphill).sqrt();
        // 辺に垂直な向きは、境目として丸め誤差の分だけ範囲に入りうる
        assert!(
            directions(edge)
                .iter()
                .all(|&h| super::dot(h, uphill) <= 1e-9 * length)
        );
    }
    // 水平な尾根の辺は、工具の底が辺に乗るため全周
    let ridge = InverseOffsetShape::new(&step(), CutterShape::Flat { radius: 1.0 }).unwrap();
    assert!(ridge.edges.iter().any(|edge| directions(edge).len() == 36));
}

#[test]
fn cutting_off_by_upper_bounds_keeps_the_heights_and_evaluates_fewer_elements() {
    // 上を向く面の要素は高さの上限が低い要素より先に評価され、隠れた要素の評価を打ち切る。外を決められない塊
    // （上の面に隠れた塊）・開いた形でも結果は変わらない
    let hidden = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 5.0),
            Point3D::new(10.0, 0.0, 5.0),
            Point3D::new(0.0, 10.0, 5.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(3.0, 1.0, 0.0),
            Point3D::new(1.0, 3.0, 0.0),
        ],
        vec![[0, 1, 2], [3, 5, 4]],
    )
    .unwrap();
    for shape in SHAPES {
        for (mesh, span) in [
            (terrain(12, ridges, true), 12.0),
            (step(), 10.0),
            (overhang(), 14.0),
            (tetrahedron(), 10.0),
            (hidden.clone(), 10.0),
        ] {
            let inverse = InverseOffsetShape::new(&mesh, shape).unwrap();
            let (mut cut, mut all) = (0, 0);
            for (x, y) in grid(span) {
                let (highest, evaluated) = inverse.highest_at(x, y, true);
                let (expected, candidates) = inverse.highest_at(x, y, false);
                assert_eq!(highest, expected, "{shape:?} ({x}, {y})");
                cut += evaluated;
                all += candidates;
            }
            assert!(cut < all, "{shape:?}: {cut} / {all}");
        }
    }
}

/// 表示用の三角形と鉛直線の交点のうち、最も高い点の高さ
fn display_height(parts: &[InverseOffsetPart], x: f64, y: f64) -> Option<f64> {
    use geo_algorithms::{CrossIntersection, IntersectionGeometry, Ray3D, Triangle3D, Vector3D};
    use geo_contracts::{Ray3DConstructor, Triangle3DConstructor};
    let ray = Ray3D::new(Point3D::new(x, y, -100.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    parts
        .iter()
        .flat_map(|part| &part.triangles)
        .filter_map(|corners| {
            let [a, b, c] = corners.map(|p| Point3D::new(p[0], p[1], p[2]));
            match Triangle3D::new(a, b, c)?.intersection(&ray, 1e-12).geometry {
                IntersectionGeometry::Point(point) => Some(point.z()),
                _ => None,
            }
        })
        .reduce(f64::max)
}

fn kinds(parts: &[InverseOffsetPart]) -> Vec<InverseOffsetPartKind> {
    parts.iter().map(|part| part.kind).collect()
}

/// 表示用の三角形が、上側の面より上に出ず、弦の誤差より下に下がらないことを確かめる
///
/// 三角形は要素の面の弦（範囲の外まで残す部分は、工具を動かした形の中にあり上側の面の下）。弦と面の隔たりは、
/// 面に垂直な向きに r × (1 − cos きざみ) 以下のため、鉛直な向きでは、上側の面の傾き s の点で √(1 + s²) 倍以下になる。
fn assert_display_follows(inverse: &InverseOffsetShape, step: f64, span: f64) {
    let parts = inverse.display_parts(step).unwrap();
    let shape = inverse.shape();
    let chord = shape.radius() * (1.0 - step.cos());
    let center = |x: f64, y: f64| {
        inverse
            .tip_height_at(x, y)
            .map(|z| z + shape.reference_offset())
    };
    let steps = (span * 4.0) as usize;
    for j in 0..=steps {
        for i in 0..=steps {
            let (x, y) = (i as f64 * 0.25, j as f64 * 0.25);
            let Some(expected) = center(x, y) else {
                continue;
            };
            // 傾きは、前後の差のうち急な方（上側の面の折れ目でも小さく見積もらない）
            let h = 1e-4;
            let slope = |dx: f64, dy: f64| {
                center(x + dx, y + dy).map_or(0.0, |z| ((z - expected) / h).abs())
            };
            let sx = slope(h, 0.0).max(slope(-h, 0.0));
            let sy = slope(0.0, h).max(slope(0.0, -h));
            let allowed = chord * (1.0 + sx * sx + sy * sy).sqrt() + 1e-9;
            let actual = display_height(&parts, x, y)
                .unwrap_or_else(|| panic!("{shape:?} ({x}, {y}): no display triangle"));
            assert!(
                actual <= expected + 1e-9 && actual >= expected - allowed,
                "{shape:?} ({x}, {y}): {actual}, {expected}, allowed {allowed}"
            );
        }
    }
}

#[test]
fn display_parts_follow_the_upper_surface() {
    let step = 5.0_f64.to_radians();
    for shape in SHAPES {
        let inverse = InverseOffsetShape::new(&terrain(4, ridges, true), shape).unwrap();
        assert_display_follows(&inverse, step, 4.0);
    }
}

#[test]
fn narrow_ranges_of_nearly_flat_convex_elements_are_displayed() {
    // 尾根の高さ 1e-3 のほぼ平らな屋根（尾根の辺の範囲と、尾根の端の頂点の範囲は、きざみ 10° より狭い）
    let roof = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(5.0, 0.0, 1e-3),
            Point3D::new(5.0, 10.0, 1e-3),
            Point3D::new(0.0, 10.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
        ],
        vec![[0, 1, 2], [0, 2, 3], [1, 4, 5], [1, 5, 2]],
    )
    .unwrap();
    for shape in SHAPES {
        let inverse = InverseOffsetShape::new(&roof, shape).unwrap();
        assert_display_follows(&inverse, 10.0_f64.to_radians(), 10.0);
    }
}

#[test]
fn display_triangles_of_ball_vertices_and_edges_lie_on_the_offset_surfaces() {
    let radius = 1.5;
    let mesh = terrain(3, ridges, true);
    let inverse = InverseOffsetShape::new(&mesh, CutterShape::Ball { radius }).unwrap();
    let parts = inverse.display_parts(10.0_f64.to_radians()).unwrap();
    let points: Vec<[f64; 3]> = mesh
        .vertices()
        .iter()
        .map(|p| [p.x(), p.y(), p.z()])
        .collect();
    let distance_to_segment = |p: [f64; 3], a: [f64; 3], b: [f64; 3]| {
        let d = [0, 1, 2].map(|k| b[k] - a[k]);
        let length_squared = d.iter().map(|v| v * v).sum::<f64>();
        let t = if length_squared == 0.0 {
            0.0
        } else {
            ([0, 1, 2].map(|k| (p[k] - a[k]) * d[k]).iter().sum::<f64>() / length_squared)
                .clamp(0.0, 1.0)
        };
        [0, 1, 2]
            .map(|k| p[k] - a[k] - t * d[k])
            .iter()
            .map(|v| v * v)
            .sum::<f64>()
            .sqrt()
    };
    // 頂点の三角形の頂点はどれかの頂点から、辺の三角形の頂点はどれかの辺から、ちょうど半径の距離にある（ほかの
    // 要素の方が近い位置もありうる）
    fn on_surface(mut distances: impl Iterator<Item = f64>, radius: f64) -> bool {
        distances.any(|distance| (distance - radius).abs() < 1e-9)
    }
    for part in &parts {
        for corner in part.triangles.iter().flatten() {
            let found = match part.kind {
                InverseOffsetPartKind::Vertex => on_surface(
                    points.iter().map(|&p| distance_to_segment(*corner, p, p)),
                    radius,
                ),
                InverseOffsetPartKind::Edge => on_surface(
                    mesh.indices()
                        .iter()
                        .flat_map(|t| [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]])
                        .map(|[a, b]| distance_to_segment(*corner, points[a], points[b])),
                    radius,
                ),
                _ => continue,
            };
            assert!(found, "{:?}: {corner:?}", part.kind);
        }
    }
}

#[test]
fn display_triangles_are_wound_to_face_outward() {
    // 三角形の頂点の順（右ねじ）の法線の z
    let normal_z = |[a, b, c]: [[f64; 3]; 3]| {
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        n[2] / (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
    };
    let shape = InverseOffsetShape::new(&overhang(), CutterShape::Ball { radius: 1.0 }).unwrap();
    let parts = shape.display_parts(10.0_f64.to_radians()).unwrap();
    let normals_z = |kind| {
        parts
            .iter()
            .find(|part| part.kind == kind)
            .unwrap()
            .triangles
            .iter()
            .map(|&triangle| normal_z(triangle))
            .collect::<Vec<_>>()
    };
    assert!(
        normals_z(InverseOffsetPartKind::IndexedFace)
            .iter()
            .all(|&z| z > 0.99)
    );
    // 張り出しの下の面は外向き（下）、その上側の要素は上を向く
    assert!(
        normals_z(InverseOffsetPartKind::UnindexedFace)
            .iter()
            .any(|&z| z < -0.99)
    );
    assert!(
        normals_z(InverseOffsetPartKind::UpperSide)
            .iter()
            .all(|&z| z > 0.99)
    );
}

#[test]
fn display_parts_are_classified_by_the_role_of_the_elements() {
    let step = 10.0_f64.to_radians();
    use InverseOffsetPartKind::*;
    // 開いた張り出し: 上を向く面・鉛直な面と下を向く面・下を向く面の上側の要素
    let open = InverseOffsetShape::new(&overhang(), CutterShape::Ball { radius: 1.0 }).unwrap();
    assert_eq!(
        kinds(&open.display_parts(step).unwrap()),
        [IndexedFace, UnindexedFace, UpperSide, Vertex, Edge]
    );
    // 閉じた四面体の底面は、上側の要素を持たない
    let tetrahedron = tetrahedron();
    let closed = InverseOffsetShape::new(&tetrahedron, CutterShape::Flat { radius: 1.0 }).unwrap();
    assert_eq!(
        kinds(&closed.display_parts(step).unwrap()),
        [IndexedFace, UnindexedFace, Vertex, Edge]
    );
    // 上の面に隠れた別の塊は、上から届く面がなく外を決められない
    let hidden = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 5.0),
            Point3D::new(10.0, 0.0, 5.0),
            Point3D::new(0.0, 10.0, 5.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(3.0, 1.0, 0.0),
            Point3D::new(1.0, 3.0, 0.0),
        ],
        vec![[0, 1, 2], [3, 5, 4]],
    )
    .unwrap();
    let parts = InverseOffsetShape::new(&hidden, CutterShape::Ball { radius: 1.0 })
        .unwrap()
        .display_parts(step)
        .unwrap();
    let unoriented = parts
        .iter()
        .find(|part| part.kind == UnorientedFace)
        .unwrap();
    assert_eq!(unoriented.triangles.len(), 1);
    // 面ごとに上向きにそろえ、工具の半径だけ上にずれる
    assert!(
        unoriented.triangles[0]
            .iter()
            .all(|p| (p[2] - 1.0).abs() < 1e-12)
    );
}

#[test]
fn display_angle_steps_out_of_range_are_rejected() {
    let inverse = InverseOffsetShape::new(&step(), CutterShape::Ball { radius: 1.0 }).unwrap();
    for angle_step in [0.0, -0.1, f64::NAN, 2.0] {
        assert!(inverse.display_parts(angle_step).is_err(), "{angle_step}");
    }
}

/// 区間を `count` に分けた点の、工具先端の高さが弦から上に出る量の最大（総当たり）
fn sampled_rise(
    inverse: &InverseOffsetShape,
    y: f64,
    [x0, z0]: [f64; 2],
    [x1, z1]: [f64; 2],
    count: usize,
) -> Option<f64> {
    (0..=count)
        .filter_map(|i| {
            let x = x0 + (x1 - x0) * i as f64 / count as f64;
            let chord = z0 + (z1 - z0) * (x - x0) / (x1 - x0);
            inverse.tip_height_at(x, y).map(|tip| tip - chord)
        })
        .reduce(f64::max)
}

/// 走査線の区間ごとに、弦から上に出る量の最大が、総当たりの最大以上で、返した x の工具先端の高さで実際に上に出る
fn assert_rise_above_chord_is_the_maximum(inverse: &InverseOffsetShape, span: f64) {
    let lines = (0..)
        .map(|j| -1.0 + j as f64 * 0.61)
        .take_while(|&y| y <= span + 1.0);
    for y in lines {
        let starts = (0..)
            .map(|i| -1.0 + i as f64 * 0.83)
            .take_while(|&x| x <= span + 1.0);
        for x0 in starts {
            let x1 = x0 + 1.3;
            let (Some(z0), Some(z1)) = (inverse.tip_height_at(x0, y), inverse.tip_height_at(x1, y))
            else {
                continue;
            };
            // CL の点を結ぶ弦と、傾けた弦
            for [d0, d1] in [[0.0, 0.0], [0.7, -0.4]] {
                let (start, end) = ([x0, z0 + d0], [x1, z1 + d1]);
                let sampled = sampled_rise(inverse, y, start, end, 400).unwrap();
                let (x, rise) = inverse
                    .rise_above_chord(y, end, start)
                    .unwrap_or_else(|| panic!("y = {y}, x = {x0}: no rise"));
                assert!(
                    rise >= sampled - 1e-9,
                    "y = {y}, x = {x0}: {rise} < sampled {sampled}"
                );
                assert!((x0 - 1e-9..=x1 + 1e-9).contains(&x), "y = {y}: x = {x}");
                let chord = start[1] + (end[1] - start[1]) * (x - x0) / (x1 - x0);
                let tip = inverse.tip_height_at(x, y).map(|tip| tip - chord);
                assert!(
                    tip.is_some_and(|tip| (tip - rise).abs() < 1e-9),
                    "y = {y}, x = {x}: {tip:?} != {rise}"
                );
            }
        }
    }
}

#[test]
fn rises_above_chords_are_the_maxima_over_the_intervals() {
    let meshes = [
        (terrain(12, ridges, true), 12.0),
        (step(), 10.0),
        (tetrahedron(), 10.0),
        (overhang(), 14.0),
    ];
    for shape in SHAPES {
        for (mesh, span) in &meshes {
            let inverse = InverseOffsetShape::new(mesh, shape).unwrap();
            assert_rise_above_chord_is_the_maximum(&inverse, *span);
        }
    }
}

#[test]
fn rises_above_chords_find_narrow_peaks_between_the_points() {
    // 区間の両端は床の上、間に幅 0.02 の尖った尾根（両端の点だけでは見えない）。ボールは尾根の真上、フラットは工具の
    // 底が尾根に乗る範囲で最も上に出る
    let peak = extruded(&[
        (0.0, 0.0),
        (4.99, 0.0),
        (5.0, 3.0),
        (5.01, 0.0),
        (10.0, 0.0),
    ]);
    for (shape, width) in [(SHAPES[0], 1e-9), (SHAPES[1], SHAPES[1].radius() + 1e-9)] {
        let inverse = InverseOffsetShape::new(&peak, shape).unwrap();
        let tip = |x: f64| inverse.tip_height_at(x, 5.0).unwrap();
        let (start, end) = ([0.5, tip(0.5)], [9.5, tip(9.5)]);
        let (x, rise) = inverse.rise_above_chord(5.0, start, end).unwrap();
        assert!((x - 5.0).abs() <= width, "{shape:?}: x = {x}");
        assert!((rise - 3.0).abs() < 1e-9, "{shape:?}: rise = {rise}");
    }
}

#[test]
fn rises_above_chords_are_none_without_contact_or_length() {
    let inverse = InverseOffsetShape::new(&step(), CutterShape::Ball { radius: 1.0 }).unwrap();
    assert!(
        inverse
            .rise_above_chord(5.0, [20.0, 0.0], [30.0, 0.0])
            .is_none()
    );
    assert!(
        inverse
            .rise_above_chord(5.0, [2.0, 4.0], [2.0, 4.0])
            .is_none()
    );
}

#[test]
fn rises_above_chords_keep_contacts_at_the_tool_radius_from_an_edge() {
    // 縁の辺（y = 0.3、z = 4）から、走査線がちょうど工具の半径だけ離れる（丸め誤差で、わずかに外れる）
    let plate = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.3, 4.0),
            Point3D::new(10.0, 0.3, 4.0),
            Point3D::new(10.0, 10.0, 4.0),
            Point3D::new(0.0, 10.0, 4.0),
        ],
        vec![[0, 1, 2], [0, 2, 3]],
    )
    .unwrap();
    for shape in SHAPES {
        let inverse = InverseOffsetShape::new(&plate, shape).unwrap();
        let radius = shape.radius();
        let mut y = 0.3 - radius;
        for _ in 0..8 {
            if let (Some(z0), Some(z1)) =
                (inverse.tip_height_at(2.0, y), inverse.tip_height_at(8.0, y))
            {
                let (_, rise) = inverse
                    .rise_above_chord(y, [2.0, z0], [8.0, z1])
                    .unwrap_or_else(|| panic!("{shape:?}: y = {y}: no rise"));
                assert!(rise.abs() < 1e-9, "{shape:?}: y = {y}: rise = {rise}");
            }
            y = y.next_down();
        }
    }
}

#[test]
fn rises_above_chords_on_planes_tangent_to_sloped_edges_stay_on_the_edges() {
    // 曲面を三角形に分けた形の三角形。傾いた辺 y = 5 から、走査線 y = 2 がちょうど工具の半径だけ離れる。円柱面を接する
    // 平面で切った形は 1 本の直線になり、辺の端の条件の直線との交点を求める 2 次式の係数がどれも丸め誤差の大きさになる
    // （もう一方の解が曲線の上になく、区間に入ると 14 mm 余り上に出るとしていた）。区間は辺から x の向きに工具の半径
    // より離れ、工具は接しない
    let triangle = TriangleMesh3D::new(
        vec![
            Point3D::new(35.0, 5.0, 80.95703125),
            Point3D::new(35.625, 5.0, 80.85235595703125),
            Point3D::new(35.625, 5.625, 80.94177722930908),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap();
    let inverse = InverseOffsetShape::new(&triangle, CutterShape::Ball { radius: 3.0 }).unwrap();
    let (start, end) = ([38.476679, 80.158529], [38.481679, 80.157989]);
    assert_eq!(inverse.tip_height_at(start[0], 2.0), None);
    assert_eq!(inverse.rise_above_chord(2.0, start, end), None);
}
