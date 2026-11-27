use std::collections::HashMap;
use std::f64::consts::PI;

use super::*;
use crate::product_shape::test_fixtures::{
    BUMP_HEIGHT, bump_brep, cone_brep, cube_brep, cylinder_brep, plate_with_hole_brep,
    seamed_sphere_brep, sphere_brep, torus_brep,
};

/// B-rep の三角形が閉じていて（どの辺も、2 つの三角形が逆向きに使う）、外を向き（頂点の法線と三角形の法線が同じ
/// 側）、体積が `volume` に近いことを確かめる
fn assert_closed(mesh: &ShellMesh, volume: f64) {
    let mut directed: HashMap<(usize, usize), usize> = HashMap::new();
    let mut total = 0.0;
    let center = mesh
        .points
        .iter()
        .fold(Vector3D::new(0.0, 0.0, 0.0), |sum, point| {
            sum + Vector3D::new(point.x(), point.y(), point.z())
        })
        * (1.0 / mesh.points.len() as f64);
    for face in &mesh.faces {
        let face = face.as_ref().unwrap();
        for &triangle in &face.triangles {
            let [a, b, c] = triangle.map(|corner| face.vertices[corner]);
            for (from, to) in [(a, b), (b, c), (c, a)] {
                *directed.entry((from, to)).or_default() += 1;
            }
            let [p, q, r] = [a, b, c].map(|index| mesh.points[index]);
            let normal = Vector3D::from_points(p, q).cross(Vector3D::from_points(p, r));
            for corner in triangle {
                assert!(
                    normal.dot(face.normals[corner]) > 0.0,
                    "{triangle:?} faces against its vertex normals"
                );
            }
            let offset =
                |point: Point3D<f64>| Vector3D::new(point.x(), point.y(), point.z()) - center;
            total += offset(p).dot(offset(q).cross(offset(r))) / 6.0;
        }
    }
    for (&(from, to), &count) in &directed {
        assert_eq!(
            count, 1,
            "the edge {from}-{to} is used twice in one direction"
        );
        assert!(
            directed.contains_key(&(to, from)),
            "the edge {from}-{to} is used by one triangle only"
        );
    }
    assert!(
        (total - volume).abs() < volume * 0.02,
        "volume {total}, expected {volume}"
    );
}

#[test]
fn closed_shells_are_triangulated_without_gaps() {
    for (brep, volume) in [
        (cube_brep(), 1000.0),
        (cylinder_brep(), PI * 100.0 * 20.0),
        (sphere_brep(), 4.0 / 3.0 * PI * 1000.0),
        (torus_brep(), 2.0 * PI * PI * 20.0 * 25.0),
        (
            plate_with_hole_brep(),
            40.0 * 30.0 * 10.0 - PI * 25.0 * 10.0,
        ),
        // 継ぎ目の稜線が極を通る面（行きと帰りの使用が、極の線の 1 周だけ離れる）
        (seamed_sphere_brep(), 4.0 / 3.0 * PI * 1000.0),
        (cone_brep(), PI * 1000.0 / 3.0),
    ] {
        let mesh = triangulate_brep(&brep, 0.05);
        assert_closed(&mesh, volume);
    }
}

/// 三角形ごとに頂点・辺・内側の 28 点（重心座標が (i, j, k) / 6 の点）をとり、曲面への最短距離 `distance` が
/// 許容弦誤差以下であることを確かめる
fn assert_within_tolerance(
    mesh: &ShellMesh,
    distance: impl Fn(Point3D<f64>) -> f64,
    chord_tolerance: f64,
) {
    for face in &mesh.faces {
        let face = face.as_ref().unwrap();
        for triangle in &face.triangles {
            let [a, b, c] = triangle.map(|corner| mesh.points[face.vertices[corner]]);
            for i in 0..=6 {
                for j in 0..=6 - i {
                    let (s, t) = (i as f64 / 6.0, j as f64 / 6.0);
                    let sample = Point3D::new(
                        a.x() * (1.0 - s - t) + b.x() * s + c.x() * t,
                        a.y() * (1.0 - s - t) + b.y() * s + c.y() * t,
                        a.z() * (1.0 - s - t) + b.z() * s + c.z() * t,
                    );
                    let deviation = distance(sample);
                    assert!(
                        deviation <= chord_tolerance * (1.0 + 1.0e-9),
                        "{deviation} in {triangle:?}"
                    );
                }
            }
        }
    }
}

/// 試験の B-rep の球（半径 10）とトーラス（中心の円の半径 20・管の半径 5）を、許容弦誤差 `chord_tolerance` で
/// 三角形に分け、閉じていて、三角形の内側でも許容弦誤差を守ることを確かめる
fn assert_sphere_and_torus(chord_tolerance: f64) {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let sphere = triangulate_brep(&sphere_brep(), chord_tolerance);
    assert_closed(&sphere, 4.0 / 3.0 * PI * 1000.0);
    assert_within_tolerance(
        &sphere,
        |point| (point.distance_to(origin) - 10.0).abs(),
        chord_tolerance,
    );
    let torus = triangulate_brep(&torus_brep(), chord_tolerance);
    assert_closed(&torus, 2.0 * PI * PI * 20.0 * 25.0);
    assert_within_tolerance(
        &torus,
        |point| ((point.x().hypot(point.y()) - 20.0).hypot(point.z()) - 5.0).abs(),
        chord_tolerance,
    );
}

#[test]
fn sphere_and_torus_shells_meet_the_machining_tolerance() {
    assert_sphere_and_torus(0.001);
}

#[test]
#[ignore = "許容弦誤差 0.0001 で、トーラスの面が 122 万点ほどになる（確かめも含めて、リリースのビルドで 15 秒ほど）。cargo test --release -- --ignored で確かめる"]
fn sphere_and_torus_shells_meet_the_finest_machining_tolerance() {
    assert_sphere_and_torus(0.0001);
}

/// 試験の B-rep の NURBS 曲面の山 (10 u, 10 v, 9 h u (1 − u) v (1 − v)) への、点 `point` からの最短距離（底の平面
/// z = 0 の点は 0）
fn bump_distance(point: Point3D<f64>) -> f64 {
    if point.z() == 0.0 {
        return 0.0;
    }
    let scale = 9.0 * BUMP_HEIGHT;
    let height = |u: f64, v: f64| scale * u * (1.0 - u) * v * (1.0 - v);
    // 曲面の上の点と点 `point` の差が、u・v の向きの接線に垂直になる (u, v) を、ガウス・ニュートン法で求める
    let (mut u, mut v) = (point.x() / 10.0, point.y() / 10.0);
    for _ in 0..32 {
        let difference = [
            10.0 * u - point.x(),
            10.0 * v - point.y(),
            height(u, v) - point.z(),
        ];
        let tangent_u = [10.0, 0.0, scale * (1.0 - 2.0 * u) * v * (1.0 - v)];
        let tangent_v = [0.0, 10.0, scale * u * (1.0 - u) * (1.0 - 2.0 * v)];
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let (a, b, c) = (
            dot(tangent_u, tangent_u),
            dot(tangent_u, tangent_v),
            dot(tangent_v, tangent_v),
        );
        let (p, q) = (-dot(difference, tangent_u), -dot(difference, tangent_v));
        let determinant = a * c - b * b;
        // 曲面の範囲の外（式を延ばした曲面）に出ないよう、範囲に収める
        u = (u + (c * p - b * q) / determinant).clamp(0.0, 1.0);
        v = (v + (a * q - b * p) / determinant).clamp(0.0, 1.0);
    }
    point.distance_to(Point3D::new(10.0 * u, 10.0 * v, height(u, v)))
}

#[test]
fn nurbs_surface_face_meets_the_chord_tolerance() {
    for chord_tolerance in [0.05, 0.001] {
        let mesh = triangulate_brep(&bump_brep(), chord_tolerance);
        assert_closed(&mesh, 25.0 * BUMP_HEIGHT);
        assert_within_tolerance(&mesh, bump_distance, chord_tolerance);
    }
}

/// 円錐の一部を表す 3 次の NURBS 曲面（u = 0 で半径 20 の 4 分の 1 の円に近い線、u の終わり `u_end` で頂点の
/// 1 点につぶれる。u の向きの線はまっすぐ）を、原点から離れた所（座標の丸め誤差の大きい所）に置いたもの
fn collapsed_side_surface(u_end: f64) -> SurfaceRef<f64> {
    use geo_contracts::NurbsSurface3DConstructor;
    // 4 分の 1 の円を 3 次のベジエ曲線で近似する制御点の、中の 2 点の位置
    let handle = 20.0 * 0.552_284_749_8;
    let arc = [(20.0, 0.0), (20.0, handle), (handle, 20.0), (0.0, 20.0)];
    let control_points: Vec<Vec<_>> = (0..4)
        .map(|i| {
            let ratio = 1.0 - i as f64 / 3.0;
            arc.iter()
                .map(|&(x, y)| {
                    geo_topology::Point3D::new(
                        200.0 + ratio * x,
                        100.0 + ratio * y,
                        -5000.0 + (1.0 - ratio) * 10.0,
                    )
                })
                .collect()
        })
        .collect();
    let u_knots = vec![0.0, 0.0, 0.0, 0.0, u_end, u_end, u_end, u_end];
    let v_knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    SurfaceRef::Nurbs(
        geo_topology::TopoNurbsSurface3D::new(3, 3, u_knots, v_knots, &control_points, None)
            .unwrap(),
    )
}

#[test]
fn a_nurbs_side_collapsed_to_a_point_does_not_make_the_grid_fine() {
    let max_angle = Angle::from_degrees(MAX_CHORD_ANGLE_DEGREES);
    let u_end = 0.005_154_953_853_047_42;
    let surface = collapsed_side_surface(u_end);
    // 面の (u, v) の範囲（u は 0〜`u_high`、v は 0〜1）を囲むループ
    let loops = |u_high: f64| {
        vec![
            [(0.0, 0.0), (u_high, 0.0), (u_high, 1.0), (0.0, 1.0)]
                .iter()
                .enumerate()
                .map(|(vertex, &(u, v))| LoopPoint {
                    vertex,
                    uv: Point2D::new(u, v),
                })
                .collect::<Vec<_>>(),
        ]
    };
    for chord_tolerance in [0.05, 0.001] {
        // 範囲がつぶれた辺のちょうどまでなら、その辺の上の点は同じ点になり、曲がりを測らない
        let (_, expected) =
            FaceSurface::new(&surface, &loops(u_end)).steps(chord_tolerance, max_angle);
        let expected = expected.unwrap();
        // 範囲がつぶれた辺のわずかに手前まで（ループの点の丸め）でも、その辺の上の点の、丸め誤差で決まる曲がりを
        // 刻みに使わない
        // 範囲がつぶれた辺の丸めほど手前までなら、その辺の上の点の曲がりの半径から求めた刻みは、実際の曲がりの刻み
        // よりずっと小さい（試験が、丸め誤差で決まる曲がりを含むことを確かめる）
        let face = FaceSurface::new(&surface, &loops(u_end * (1.0 - 1.0e-15)));
        let noisy = face.nurbs.as_ref().unwrap().bends.1.iter().any(|bend| {
            bend.length <= 2.0 * chord_tolerance
                && chord_angle_step(bend.radius, chord_tolerance, max_angle) * bend.radius
                    / bend.speed
                    < 0.1 * expected
        });
        assert!(noisy, "a bend on the collapsed side is measured");
        for u_high in [u_end * (1.0 - 1.0e-15), u_end * (1.0 - 1.0e-12)] {
            let face = FaceSurface::new(&surface, &loops(u_high));
            let (step_u, step_v) = face.steps(chord_tolerance, max_angle);
            assert_eq!(step_u, Some(u_high), "the lines along u are straight");
            let step_v = step_v.unwrap();
            assert!(
                (step_v - expected).abs() <= expected * 1.0e-9,
                "step {step_v}, expected {expected}"
            );
        }
    }
}
