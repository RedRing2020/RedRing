use geo_algorithms::{Point3D, TriangleMesh3D};
use geo_contracts::{Triangle3DBoundaryAccess, TriangleMesh3DConstructor};

use super::InverseOffsetShape;
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

/// 形状の外も含む、細かい格子の点ごとの工具先端の高さ
fn heights(shape: &InverseOffsetShape, span: f64) -> Vec<Option<f64>> {
    let steps = (span * 4.0) as usize + 8;
    (0..=steps)
        .flat_map(|j| (0..=steps).map(move |i| (i, j)))
        .map(|(i, j)| {
            let (x, y) = (-1.0 + i as f64 * 0.27, -1.0 + j as f64 * 0.27);
            shape.tip_height_at(x, y)
        })
        .collect()
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
