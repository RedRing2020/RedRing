//! `SphericalSurface3D` のテスト

use super::*;
use crate::Point3D;
use geo_contracts::{
    Bounded, PrimitiveKind, PrimitiveMetadata, SphericalSurface3DConstructor,
    SphericalSurface3DEvaluation,
};
use std::f64::consts::FRAC_PI_2;

#[test]
fn test_spherical_surface_foundation() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let surface = SphericalSurface3D::new_standard(center, 2.0).unwrap();

    // Foundation トレイトのテスト
    assert_eq!(surface.primitive_kind(), PrimitiveKind::SphericalSurface);

    let bbox = surface.aabb().expect("should have aabb");
    assert_eq!(bbox.min(), Point3D::new(-1.0, 0.0, 1.0));
    assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 5.0));
}

#[test]
fn test_point_at_uv_is_longitude_and_latitude() {
    let surface = SphericalSurface3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 2.0).unwrap();

    // u は経度、v は緯度
    let cases = [
        (0.0, 0.0, Point3D::new(3.0, 2.0, 3.0)),
        (FRAC_PI_2, 0.0, Point3D::new(1.0, 4.0, 3.0)),
        (0.0, FRAC_PI_2, Point3D::new(1.0, 2.0, 5.0)),
        (0.0, -FRAC_PI_2, Point3D::new(1.0, 2.0, 1.0)),
    ];
    for (u, v, expected) in cases {
        assert!(surface.point_at_uv(u, v).distance_to(expected) < 1e-12);
        let normal = surface.normal_at(u, v).as_vector();
        assert!((normal - (expected - Point3D::new(1.0, 2.0, 3.0)) / 2.0).length() < 1e-12);
    }
}
