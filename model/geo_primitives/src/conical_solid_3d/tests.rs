//! ConicalSolid3D のテスト

use super::*;
use approx::assert_relative_eq;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata, TolerantEq};

#[test]
fn test_tolerant_eq() {
    let solid1 = ConicalSolid3D::new_standard(Point3D::origin(), 5.0, 10.0).unwrap();
    let solid2 = ConicalSolid3D::new_standard(Point3D::origin(), 5.0, 10.0).unwrap();
    let solid3 = ConicalSolid3D::new_standard(Point3D::origin(), 3.0, 10.0).unwrap();

    assert!(solid1.tolerant_eq(&solid2, 0.01));
    assert!(!solid1.tolerant_eq(&solid3, 0.01));
}

#[test]
fn test_new_uses_base_center_and_axis_toward_apex() {
    // 底面の中心と、底面から頂点への軸で決まる
    let base_center = Point3D::new(1.0, 2.0, 3.0);
    let axis = Direction3D::new(0.0, 1.0, 0.0).unwrap();
    let cone = ConicalSolid3D::new(base_center, axis, Direction3D::positive_x(), 2.0, 4.0).unwrap();

    assert_eq!(cone.base_center(), base_center);
    assert!(cone.apex().distance_to(Point3D::new(1.0, 6.0, 3.0)) < 1e-12);
    assert!(cone.contains_point(Point3D::new(1.0, 5.9, 3.0)));
    assert!(!cone.contains_point(Point3D::new(1.0, 2.0 - 0.1, 3.0)));
}

#[test]
fn test_metadata_and_bounds_capabilities() {
    let conical_solid =
        ConicalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0, 10.0).unwrap();

    assert_eq!(conical_solid.primitive_kind(), PrimitiveKind::ConicalSolid);

    // 底面: 中心 ± 半径、頂点: 中心 + 軸 × 高さ
    let bbox = conical_solid.aabb().expect("should have aabb");
    assert_relative_eq!(bbox.min().x(), -4.0, epsilon = 1e-10);
    assert_relative_eq!(bbox.max().x(), 6.0, epsilon = 1e-10);
    assert_relative_eq!(bbox.min().y(), -3.0, epsilon = 1e-10);
    assert_relative_eq!(bbox.max().y(), 7.0, epsilon = 1e-10);
    assert_relative_eq!(bbox.min().z(), 3.0, epsilon = 1e-10);
    assert_relative_eq!(bbox.max().z(), 13.0, epsilon = 1e-10);
}

#[test]
fn test_bounds_tilted_axis() {
    // 軸を斜め 45° に向けた円錐：底面の円の広がりは r √(1 - (軸の成分)²)
    let axis = Direction3D::new(1.0, 0.0, 1.0).unwrap();
    let cone = ConicalSolid3D::new(
        Point3D::origin(),
        axis,
        Direction3D::new(0.0, 1.0, 0.0).unwrap(),
        2.0,
        4.0,
    )
    .unwrap();

    let bbox = cone.aabb().unwrap();
    let half_extent = 2.0 * 0.5_f64.sqrt();
    let apex = 4.0 * 0.5_f64.sqrt();
    assert_relative_eq!(bbox.min().x(), -half_extent, epsilon = 1e-12);
    assert_relative_eq!(bbox.max().x(), apex, epsilon = 1e-12);
    assert_relative_eq!(bbox.min().y(), -2.0, epsilon = 1e-12);
    assert_relative_eq!(bbox.max().y(), 2.0, epsilon = 1e-12);
    assert_relative_eq!(bbox.max().z(), apex, epsilon = 1e-12);
}

#[test]
fn test_conical_solid_degenerate() {
    let origin = Point3D::origin();
    assert!(ConicalSolid3D::new_standard(origin, 0.0, 5.0).is_none());
    assert!(ConicalSolid3D::new_standard(origin, 5.0, 0.0).is_none());
    assert!(ConicalSolid3D::new_standard(origin, -1.0, 5.0).is_none());
    assert!(ConicalSolid3D::new_standard(origin, 5.0, -1.0).is_none());
}
