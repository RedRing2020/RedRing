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
