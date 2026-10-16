//! EllipsoidalSolid3D のテスト

use super::*;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};

#[test]
fn test_new_ellipsoidal_solid() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let axis = Direction3D::positive_z();
    let ref_dir = Direction3D::positive_x();

    let ellipsoid = EllipsoidalSolid3D::new(center, axis, ref_dir, 2.0, 3.0, 4.0);
    assert!(ellipsoid.is_some());

    let e = ellipsoid.unwrap();
    assert_eq!(e.a_radius, 2.0);
    assert_eq!(e.b_radius, 3.0);
    assert_eq!(e.c_radius, 4.0);
}

#[test]
fn test_new_standard_ellipsoidal_solid() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let ellipsoid = EllipsoidalSolid3D::new_standard(center, 2.0, 3.0, 4.0);

    assert!(ellipsoid.is_some());
    let e = ellipsoid.unwrap();
    assert_eq!(e.center, center);
}

#[test]
fn test_contains_point() {
    let ellipsoid = EllipsoidalSolid3D::new_standard(Point3D::origin(), 2.0, 3.0, 4.0).unwrap();

    // 中心点は内部
    assert!(ellipsoid.contains_point(Point3D::new(0.0, 0.0, 0.0)));

    // X軸上の点（境界内）
    assert!(ellipsoid.contains_point(Point3D::new(1.0, 0.0, 0.0)));

    // Y軸上の点（境界内）
    assert!(ellipsoid.contains_point(Point3D::new(0.0, 2.0, 0.0)));

    // Z軸上の点（境界内）
    assert!(ellipsoid.contains_point(Point3D::new(0.0, 0.0, 3.0)));

    // 外部の点
    assert!(!ellipsoid.contains_point(Point3D::new(3.0, 0.0, 0.0)));
    assert!(!ellipsoid.contains_point(Point3D::new(0.0, 4.0, 0.0)));
    assert!(!ellipsoid.contains_point(Point3D::new(0.0, 0.0, 5.0)));
}

#[test]
fn test_invalid_radii() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let axis = Direction3D::positive_z();
    let ref_dir = Direction3D::positive_x();

    // 負の半径
    assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, -1.0, 2.0, 3.0).is_none());
    assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, 1.0, -2.0, 3.0).is_none());
    assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, 1.0, 2.0, -3.0).is_none());

    // ゼロ半径
    assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, 0.0, 2.0, 3.0).is_none());
}

#[test]
fn test_ellipsoidal_solid_foundation() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let solid = EllipsoidalSolid3D::new_standard(center, 2.0, 3.0, 4.0).unwrap();

    // Foundation トレイトのテスト
    assert_eq!(solid.primitive_kind(), PrimitiveKind::EllipsoidalSolid);

    let bbox = solid.aabb().expect("should have aabb");
    // 標準方向なので、中心 ± 各半径
    assert_eq!(bbox.min(), Point3D::new(-1.0, -1.0, -1.0));
    assert_eq!(bbox.max(), Point3D::new(3.0, 5.0, 7.0));
}

#[test]
fn test_ellipsoidal_solid_bbox_rotated() {
    // 45度回転した楕円体
    let center = Point3D::new(0.0, 0.0, 0.0);
    let axis = Direction3D::new(1.0, 1.0, 0.0).unwrap(); // 45度傾き
    let ref_dir = Direction3D::new(1.0, -1.0, 0.0).unwrap();
    let solid = EllipsoidalSolid3D::new(center, axis, ref_dir, 2.0, 1.0, 1.0).unwrap();

    let bbox = solid.aabb().expect("should have aabb");

    // 回転しているので境界ボックスは単純な ±radius ではない
    assert!(bbox.min().x() < 0.0);
    assert!(bbox.min().y() < 0.0);
    assert!(bbox.max().x() > 0.0);
    assert!(bbox.max().y() > 0.0);
}

#[test]
fn test_bbox_rotated_is_tight() {
    // 長半径 2 の軸を XY 平面の 45° 方向に向けた楕円体
    let solid = EllipsoidalSolid3D::new(
        Point3D::origin(),
        Direction3D::positive_z(),
        Direction3D::new(1.0, 1.0, 0.0).unwrap(),
        2.0,
        1.0,
        1.0,
    )
    .unwrap();

    // 各座標軸方向の広がりは √((2 cos45°)² + (1 sin45°)²) = √2.5
    let bbox = solid.aabb().unwrap();
    let extent = 2.5_f64.sqrt();
    assert!((bbox.max().x() - extent).abs() < 1e-12);
    assert!((bbox.max().y() - extent).abs() < 1e-12);
    assert!((bbox.max().z() - 1.0).abs() < 1e-12);
}
