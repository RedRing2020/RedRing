//! EllipsoidalSurface3D のテスト

use super::*;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};

#[test]
fn test_ellipsoidal_surface_aabb_available() {
    let surface = EllipsoidalSurface3D::new_standard((1.0, 2.0, 3.0), 2.0, 3.0, 4.0).unwrap();

    assert_eq!(surface.primitive_kind(), PrimitiveKind::EllipsoidalSurface);
    assert!(surface.surface_area() > 0.0);

    let bbox = surface.aabb().unwrap();
    assert!(bbox.max().x() > bbox.min().x());
    assert!(bbox.max().y() > bbox.min().y());
    assert!(bbox.max().z() > bbox.min().z());
}

#[test]
fn test_ellipsoidal_surface_aabb_rotated() {
    let surface = EllipsoidalSurface3D::new(
        Point3D::origin(),
        Vector3D::new(0.0, 1.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
        2.0,
        1.0,
        3.0,
    )
    .unwrap();

    let bbox = surface.aabb().unwrap();
    assert!(!bbox.is_empty());
}

#[test]
fn test_distance_to_surface_at_center() {
    let surface = EllipsoidalSurface3D::new_at_origin(2.0, 3.0, 4.0).unwrap();
    let center = Point3D::origin();
    let distance = surface.distance_to_surface(&center);
    assert!(distance > 0.0);
}

#[test]
fn test_contains_surface_point() {
    let surface = EllipsoidalSurface3D::new_at_origin(2.0, 2.0, 2.0).unwrap();
    let point_on_surface = Point3D::new(2.0, 0.0, 0.0);
    assert!(surface.contains_point(&point_on_surface, 1e-8));
}
