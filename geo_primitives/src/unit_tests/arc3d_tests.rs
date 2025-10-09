//! Arc3D implementation tests
//!
//! Arc3D型の機能テスト

use crate::geometry3d::{Arc, ArcKind, Circle, Direction3D, Point3D, Vector3D};
use geo_foundation::abstract_types::geometry::Direction;
use geo_foundation::constants::precision::GEOMETRIC_TOLERANCE;
use geo_foundation::constants::precision::PI;
use geo_foundation::Angle;

#[test]
fn test_arc_creation() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Vector3D::new(0.0, 0.0, 1.0);
    let u_axis = Vector3D::new(1.0, 0.0, 0.0);
    let normal_dir = Direction3D::from_vector(normal).unwrap();
    let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
    let circle = Circle::new(center, 5.0, normal_dir, u_axis_dir);
    let arc = Arc::from_radians(circle, 0.0, PI);

    assert_eq!(arc.center().x(), 0.0);
    assert_eq!(arc.center().y(), 0.0);
    assert_eq!(arc.center().z(), 0.0);
    assert_eq!(arc.radius(), 5.0);
    assert_eq!(arc.start_angle().to_radians(), 0.0);
    assert_eq!(arc.end_angle().to_radians(), PI);
}

#[test]
fn test_arc_length() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Vector3D::new(0.0, 0.0, 1.0);
    let u_axis = Vector3D::new(1.0, 0.0, 0.0);
    let normal_dir = Direction3D::from_vector(normal).unwrap();
    let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
    let circle = Circle::new(center, 3.0, normal_dir, u_axis_dir);
    let arc = Arc::from_radians(circle, 0.0, PI);

    let expected_length = 3.0 * PI;
    assert!((arc.arc_length() - expected_length).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_arc_kind() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Vector3D::new(0.0, 0.0, 1.0);
    let u_axis = Vector3D::new(1.0, 0.0, 0.0);
    let normal_dir = Direction3D::from_vector(normal).unwrap();
    let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
    let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);

    let minor_arc = Arc::from_radians(circle.clone(), 0.0, PI / 3.0);
    assert_eq!(minor_arc.arc_kind(), ArcKind::MinorArc);

    let major_arc = Arc::from_radians(circle.clone(), 0.0, 4.0 * PI / 3.0);
    assert_eq!(major_arc.arc_kind(), ArcKind::MajorArc);

    let semicircle = Arc::from_radians(circle.clone(), 0.0, PI);
    assert_eq!(semicircle.arc_kind(), ArcKind::Semicircle);

    let full_circle = Arc::from_radians(circle, 0.0, 2.0 * PI);
    assert_eq!(full_circle.arc_kind(), ArcKind::FullCircle);
}

#[test]
fn test_angle_contains() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Vector3D::new(0.0, 0.0, 1.0);
    let u_axis = Vector3D::new(1.0, 0.0, 0.0);
    let normal_dir = Direction3D::from_vector(normal).unwrap();
    let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
    let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
    let arc = Arc::from_radians(circle, 0.0, PI);

    assert!(arc.angle_contains(Angle::from_radians(PI / 4.0)));
    assert!(arc.angle_contains(Angle::from_radians(PI / 2.0)));
    assert!(!arc.angle_contains(Angle::from_radians(3.0 * PI / 2.0)));
}

#[test]
fn test_arc_reverse() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Vector3D::new(0.0, 0.0, 1.0);
    let u_axis = Vector3D::new(1.0, 0.0, 0.0);
    let normal_dir = Direction3D::from_vector(normal).unwrap();
    let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
    let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
    let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
    let reversed = arc.reverse();

    assert_eq!(reversed.start_angle(), arc.end_angle());
    assert_eq!(reversed.end_angle(), arc.start_angle());
}

#[test]
fn test_arc_midpoint() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Vector3D::new(0.0, 0.0, 1.0);
    let u_axis = Vector3D::new(1.0, 0.0, 0.0);
    let normal_dir = Direction3D::from_vector(normal).unwrap();
    let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
    let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
    let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
    let mid = arc.midpoint();

    let expected_angle = PI / 4.0;
    let expected_x = expected_angle.cos();
    let expected_y = expected_angle.sin();

    assert!((mid.x() - expected_x).abs() < GEOMETRIC_TOLERANCE);
    assert!((mid.y() - expected_y).abs() < GEOMETRIC_TOLERANCE);
    assert!((mid.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_approximate_with_points() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Vector3D::new(0.0, 0.0, 1.0);
    let u_axis = Vector3D::new(1.0, 0.0, 0.0);
    let normal_dir = Direction3D::from_vector(normal).unwrap();
    let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
    let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
    let arc = Arc::from_radians(circle, 0.0, PI / 2.0);

    let points = arc.approximate_with_points(4); // 4セグメントを生成
    assert_eq!(points.len(), 5); // 4セグメント = 5点

    // 最初と最後の点をチェック
    let first_point = points.first().unwrap();
    let last_point = points.last().unwrap();

    assert!((first_point.x() - 1.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((first_point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((first_point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((last_point.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((last_point.y() - 1.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((last_point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
}
