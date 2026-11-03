//! EllipsoidalSurface3D のテスト

use super::*;
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};
use std::f64::consts::FRAC_PI_2;

#[test]
fn test_ellipsoidal_surface_aabb_available() {
    let surface =
        EllipsoidalSurface3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 2.0, 3.0, 4.0).unwrap();

    assert_eq!(surface.primitive_kind(), PrimitiveKind::EllipsoidalSurface);

    let bbox = surface.aabb().unwrap();
    assert!(bbox.max().x() > bbox.min().x());
    assert!(bbox.max().y() > bbox.min().y());
    assert!(bbox.max().z() > bbox.min().z());
}

#[test]
fn test_ellipsoidal_surface_aabb_rotated() {
    let surface = EllipsoidalSurface3D::new(
        Point3D::origin(),
        Direction3D::new(0.0, 1.0, 1.0).unwrap(),
        Direction3D::new(1.0, 0.0, 0.0).unwrap(),
        2.0,
        1.0,
        3.0,
    )
    .unwrap();

    let bbox = surface.aabb().unwrap();
    assert!(!bbox.is_empty());
}

#[test]
fn test_distance_to_point() {
    let surface = EllipsoidalSurface3D::new_standard(Point3D::origin(), 2.0, 3.0, 4.0).unwrap();

    // 中心からは最も短い半軸の長さ、面上の点は 0、軸上の外部の点は半軸の端までの距離
    assert!((surface.distance_to_point(Point3D::origin()) - 2.0).abs() < TOLERANCE_F64);
    assert!(surface.distance_to_point(Point3D::new(0.0, 3.0, 0.0)) < TOLERANCE_F64);
    assert!((surface.distance_to_point(Point3D::new(0.0, 0.0, 6.0)) - 2.0).abs() < TOLERANCE_F64);
}

#[test]
fn test_normal_at_is_gradient_direction() {
    let surface = EllipsoidalSurface3D::new_standard(Point3D::origin(), 2.0, 3.0, 4.0).unwrap();

    // 主軸の端では主軸の方向
    let cases = [
        (0.0, 0.0, Vector3D::new(1.0, 0.0, 0.0)),
        (FRAC_PI_2, 0.0, Vector3D::new(0.0, 1.0, 0.0)),
        (0.0, FRAC_PI_2, Vector3D::new(0.0, 0.0, 1.0)),
    ];
    for (u, v, expected) in cases {
        assert!((surface.normal_at(u, v).as_vector() - expected).length() < TOLERANCE_F64);
    }

    // 一般の点では陰関数の勾配 (x/a², y/b², z/c²) の向き
    let (u, v) = (0.7, 0.4);
    let point = surface.point_at_uv(u, v);
    let gradient = Vector3D::new(point.x() / 4.0, point.y() / 9.0, point.z() / 16.0);
    let expected = gradient / gradient.length();
    assert!((surface.normal_at(u, v).as_vector() - expected).length() < TOLERANCE_F64);
}
