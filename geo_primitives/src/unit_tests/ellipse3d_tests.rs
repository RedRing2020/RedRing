//! Ellipse3D implementation tests
//!
//! Ellipse3D型の機能テスト

use crate::geometry3d::{Circle, Direction3D, Ellipse, Point3D, Vector3D};
use geo_foundation::constants::precision::GEOMETRIC_TOLERANCE;
use std::f64::consts::PI;

#[test]
fn test_ellipse_creation() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

    assert_eq!(ellipse.center(), center);
    assert_eq!(ellipse.major_radius(), 3.0);
    assert_eq!(ellipse.minor_radius(), 2.0);
}

#[test]
fn test_ellipse_invalid_parameters() {
    let center = Point3D::new(0.0, 0.0, 0.0);

    // 負の半径
    assert!(Ellipse::xy_plane(center, -1.0, 2.0).is_err());
    assert!(Ellipse::xy_plane(center, 2.0, -1.0).is_err());

    // 短軸が長軸より長い
    assert!(Ellipse::xy_plane(center, 2.0, 3.0).is_err());
}

#[test]
fn test_ellipse_area() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

    let expected_area = PI * 3.0 * 2.0;
    assert!((ellipse.area() - expected_area).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_ellipse_eccentricity() {
    let center = Point3D::new(0.0, 0.0, 0.0);

    // 円の場合
    let circle = Ellipse::xy_plane(center, 2.0, 2.0).unwrap();
    assert!((circle.eccentricity() - 0.0).abs() < GEOMETRIC_TOLERANCE);

    // 楕円の場合
    let ellipse = Ellipse::xy_plane(center, 5.0, 3.0).unwrap();
    let expected_eccentricity = (1.0f64 - (3.0 * 3.0) / (5.0 * 5.0)).sqrt();
    assert!((ellipse.eccentricity() - expected_eccentricity).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_ellipse_contains_point() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

    // 中心点
    assert!(ellipse.contains_point(&center));

    // 楕円内部の点
    assert!(ellipse.contains_point(&Point3D::new(1.0, 1.0, 0.0)));

    // 楕円外部の点
    assert!(!ellipse.contains_point(&Point3D::new(4.0, 0.0, 0.0)));
    assert!(!ellipse.contains_point(&Point3D::new(0.0, 3.0, 0.0)));
}

#[test]
fn test_ellipse_on_boundary() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

    // 長軸の端点
    assert!(ellipse.on_boundary(&Point3D::new(3.0, 0.0, 0.0)));
    assert!(ellipse.on_boundary(&Point3D::new(-3.0, 0.0, 0.0)));

    // 短軸の端点
    assert!(ellipse.on_boundary(&Point3D::new(0.0, 2.0, 0.0)));
    assert!(ellipse.on_boundary(&Point3D::new(0.0, -2.0, 0.0)));
}

#[test]
fn test_ellipse_scale() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();
    let scaled = ellipse.scale(2.0);

    assert_eq!(scaled.major_radius(), 6.0);
    assert_eq!(scaled.minor_radius(), 4.0);
    assert_eq!(scaled.center(), center);
}

#[test]
fn test_ellipse_translate() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();
    let vector = Vector3D::new(2.0, 3.0, 1.0);
    let translated = ellipse.translate(&vector);

    assert_eq!(translated.center(), Point3D::new(2.0, 3.0, 1.0));
    assert_eq!(translated.major_radius(), 3.0);
    assert_eq!(translated.minor_radius(), 2.0);
}

#[test]
fn test_ellipse_from_circle() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    let u_axis = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let circle = Circle::new(center, 5.0, normal, u_axis);
    let ellipse = Ellipse::from_circle(&circle);

    assert_eq!(ellipse.center(), center);
    assert_eq!(ellipse.major_radius(), 5.0);
    assert_eq!(ellipse.minor_radius(), 5.0);
    assert!(ellipse.is_circle());
}

#[test]
fn test_ellipse_foci() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 5.0, 3.0).unwrap();
    let (f1, f2) = ellipse.foci();

    let focal_distance = ellipse.focal_distance();
    assert_eq!(f1, Point3D::new(focal_distance, 0.0, 0.0));
    assert_eq!(f2, Point3D::new(-focal_distance, 0.0, 0.0));
}

#[test]
fn test_ellipse_is_circle() {
    let center = Point3D::new(0.0, 0.0, 0.0);

    // 円
    let circle = Ellipse::xy_plane(center, 2.0, 2.0).unwrap();
    assert!(circle.is_circle());

    // 楕円
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();
    assert!(!ellipse.is_circle());
}

#[test]
fn test_ellipse_point_at_angle() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

    // 0度の点（長軸上）
    let point_0 = ellipse.point_at_angle(0.0);
    assert!((point_0.x() - 3.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point_0.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point_0.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);

    // 90度の点（短軸上）
    let point_90 = ellipse.point_at_angle(PI / 2.0);
    assert!((point_90.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point_90.y() - 2.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((point_90.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
}
