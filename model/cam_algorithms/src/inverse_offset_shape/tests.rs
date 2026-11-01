use geo_algorithms::{Point3D, TriangleMesh3D};
use geo_contracts::TriangleMesh3DConstructor;

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
    // 真上・真下を向く鉛直な壁の辺は、壁のどちらの側が材料かが決まらないため、上の辺も下の辺も残す
    let wall = InverseOffsetShape::build(&step(), CutterShape::Ball { radius: 1.0 }, true).unwrap();
    let on_wall = |x: f64, z: f64| {
        wall.edges
            .iter()
            .any(|edge| edge.ends.iter().all(|end| end[0] == x && end[2] == z))
    };
    assert!(on_wall(5.0, 4.0));
    assert!(on_wall(5.0, 0.0));
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
