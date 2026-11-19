use std::collections::HashMap;
use std::f64::consts::PI;

use super::*;
use crate::product_shape::test_fixtures::{
    cone_brep, cube_brep, cylinder_brep, plate_with_hole_brep, seamed_sphere_brep, sphere_brep,
    torus_brep,
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
