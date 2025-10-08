//! Circle3D implementation tests
//!
//! Circle3D型の機能テスト

use crate::geometry3d::{Circle, Point3D};
use geo_foundation::common::constants::GEOMETRIC_TOLERANCE;
use std::f64::consts::{PI, TAU};

#[test]
fn test_xy_plane_circle() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let circle = Circle::xy_plane_circle(center, 5.0);

    assert_eq!(circle.center(), center);
    assert_eq!(circle.radius(), 5.0);
    assert_eq!(circle.area(), PI * 25.0);
    assert_eq!(circle.circumference(), TAU * 5.0);

    // 法線がZ軸方向であることを確認
    let normal = circle.normal();
    assert!((normal.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((normal.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((normal.z() - 1.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_unit_circle() {
    let circle = Circle::unit_circle();

    assert_eq!(circle.center(), Point3D::origin());
    assert_eq!(circle.radius(), 1.0);
    assert_eq!(circle.area(), PI);
    assert_eq!(circle.circumference(), TAU);
}

#[test]
fn test_contains_point() {
    let circle = Circle::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 5.0);

    assert!(circle.contains_point(&Point3D::new(0.0, 0.0, 0.0))); // 中心
    assert!(circle.contains_point(&Point3D::new(3.0, 4.0, 0.0))); // 内部
    assert!(circle.contains_point(&Point3D::new(5.0, 0.0, 0.0))); // 円周上
    assert!(!circle.contains_point(&Point3D::new(6.0, 0.0, 0.0))); // 外部
    assert!(!circle.contains_point(&Point3D::new(0.0, 0.0, 1.0))); // 平面外
}

#[test]
fn test_point_at_angle() {
    let circle = Circle::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 2.0);

    let point = circle.point_at_angle(0.0);
    assert!((point.x() - 2.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);

    let point = circle.point_at_angle(PI / 2.0);
    assert!((point.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.y() - 2.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_bounding_box() {
    let circle = Circle::xy_plane_circle(Point3D::new(1.0, 2.0, 3.0), 4.0);
    let (min, max) = circle.bounding_box();

    assert!((min.x() - (-3.0)).abs() < GEOMETRIC_TOLERANCE);
    assert!((min.y() - (-2.0)).abs() < GEOMETRIC_TOLERANCE);
    assert!((min.z() - 3.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((max.x() - 5.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((max.y() - 6.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((max.z() - 3.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_to_2d() {
    let circle_3d = Circle::xy_plane_circle(Point3D::new(1.0, 2.0, 3.0), 4.0);
    let circle_2d = circle_3d.to_2d();

    // 基本的なプロパティの確認
    assert_eq!(circle_2d.radius(), 4.0);
    assert_eq!(circle_2d.area(), PI * 16.0);
    assert_eq!(circle_2d.circumference(), TAU * 4.0);
}
