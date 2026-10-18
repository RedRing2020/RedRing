//! Ray3D テストモジュール

use super::*;
use crate::{Point3D, Ray3D, Vector3D};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata, Scalar, TolerantEq};

// BasicTransformの実装を有効にするため
#[allow(unused_imports)]
// use crate::ray_3d_transform; // 削除済み
#[test]
fn test_ray_3d_creation() {
    let origin = Point3D::new(1.0, 2.0, 3.0);
    let direction = Vector3D::new(1.0, 0.0, 0.0);

    let ray = Ray3D::new(origin, direction).unwrap();

    assert_eq!(ray.origin(), origin);
    assert_eq!(ray.direction.as_vector(), Vector3D::new(1.0, 0.0, 0.0));
}

#[test]
fn test_ray_3d_creation_with_normalization() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Vector3D::new(3.0, 4.0, 0.0); // 長さ5のベクトル

    let ray = Ray3D::new(origin, direction).unwrap();

    assert_eq!(ray.origin(), origin);
    assert!((ray.direction.length() - 1.0).abs() < TOLERANCE_F64);
    assert!((ray.direction.x() - 0.6).abs() < TOLERANCE_F64);
    assert!((ray.direction.y() - 0.8).abs() < TOLERANCE_F64);
}

#[test]
fn test_ray_3d_creation_from_zero_vector() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Vector3D::zero();

    let ray = Ray3D::new(origin, direction);

    assert!(ray.is_none());
}

#[test]
fn test_ray_3d_from_points() {
    let start = Point3D::new(1.0, 2.0, 3.0);
    let through = Point3D::new(4.0, 6.0, 3.0);

    let ray = Ray3D::from_points(start, through).unwrap();

    assert_eq!(ray.origin(), start);
    assert!((ray.direction.length() - 1.0).abs() < TOLERANCE_F64);
}

#[test]
fn test_point_at_parameter() {
    let origin = Point3D::new(1.0, 2.0, 3.0);
    let direction = Vector3D::new(1.0, 0.0, 0.0);
    let ray = Ray3D::new(origin, direction).unwrap();

    let point_at_0 = ray.point_at_parameter(0.0);
    let point_at_5 = ray.point_at_parameter(5.0);

    assert_eq!(point_at_0, Point3D::new(1.0, 2.0, 3.0));
    assert_eq!(point_at_5, Point3D::new(6.0, 2.0, 3.0));
}

#[test]
fn test_parameter_for_point() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Vector3D::new(1.0, 0.0, 0.0);
    let ray = Ray3D::new(origin, direction).unwrap();

    let point1 = Point3D::new(3.0, 0.0, 0.0);
    let point2 = Point3D::new(-2.0, 0.0, 0.0);

    assert_eq!(ray.parameter_for_point(point1), 3.0);
    assert_eq!(ray.parameter_for_point(point2), -2.0);
}

#[test]
fn test_contains_point() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Vector3D::new(1.0, 0.0, 0.0);
    let ray = Ray3D::new(origin, direction).unwrap();

    let point_on_ray = Point3D::new(5.0, 0.0, 0.0);
    let point_behind = Point3D::new(-2.0, 0.0, 0.0);
    let point_off_ray = Point3D::new(3.0, 1.0, 0.0);

    assert!(ray.distance_to_point(point_on_ray) <= TOLERANCE_F64);
    assert!(ray.distance_to_point(point_behind) > TOLERANCE_F64);
    assert!(ray.distance_to_point(point_off_ray) > TOLERANCE_F64);
}

#[test]
fn test_distance_to_point() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Vector3D::new(1.0, 0.0, 0.0);
    let ray = Ray3D::new(origin, direction).unwrap();

    // 点が Ray 上にある場合
    let point_on_ray = Point3D::new(3.0, 0.0, 0.0);
    assert!((ray.distance_to_point(point_on_ray) - 0.0).abs() < TOLERANCE_F64);

    // 点が Ray の横にある場合
    let point_sideways = Point3D::new(3.0, 4.0, 0.0);
    assert!((ray.distance_to_point(point_sideways) - 4.0).abs() < TOLERANCE_F64);

    // 点が Ray の後ろにある場合
    let point_behind = Point3D::new(-3.0, 4.0, 0.0);
    let expected_distance = (3.0_f64.powi(2) + 4.0_f64.powi(2)).sqrt();
    assert!((ray.distance_to_point(point_behind) - expected_distance).abs() < TOLERANCE_F64);
}

#[test]
fn test_closest_point_on_ray() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Vector3D::new(1.0, 0.0, 0.0);
    let ray = Ray3D::new(origin, direction).unwrap();

    // 点が Ray の正の方向にある場合
    let point1 = Point3D::new(3.0, 4.0, 0.0);
    let closest1 = ray.closest_point(point1);
    assert_eq!(closest1, Point3D::new(3.0, 0.0, 0.0));

    // 点が Ray の後ろにある場合
    let point2 = Point3D::new(-2.0, 3.0, 0.0);
    let closest2 = ray.closest_point(point2);
    assert_eq!(closest2, Point3D::new(0.0, 0.0, 0.0)); // 起点が最も近い
}

#[test]
fn test_3d_diagonal_ray() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Vector3D::new(1.0, 1.0, 1.0);
    let ray = Ray3D::new(origin, direction).unwrap();

    // 正規化確認
    let normalized_length = 1.0 / (3.0_f64).sqrt();
    assert!((ray.direction.x() - normalized_length).abs() < TOLERANCE_F64);
    assert!((ray.direction.y() - normalized_length).abs() < TOLERANCE_F64);
    assert!((ray.direction.z() - normalized_length).abs() < TOLERANCE_F64);

    // 対角線上の点をテスト
    let point = ray.point_at_parameter((3.0_f64).sqrt());
    assert!((point.x() - 1.0).abs() < TOLERANCE_F64);
    assert!((point.y() - 1.0).abs() < TOLERANCE_F64);
    assert!((point.z() - 1.0).abs() < TOLERANCE_F64);
}

#[test]
fn test_contains_point_behind_origin_uses_tolerance() {
    let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let tolerance = geo_contracts::default_distance_tolerance::<f64>();

    // 起点の後方も、起点までの距離が許容誤差以内なら Ray 上とする
    assert!(ray.distance_to_point(Point3D::new(-0.5 * tolerance, 0.0, 0.0)) <= tolerance);
    assert!(ray.distance_to_point(Point3D::new(-2.0 * tolerance, 0.0, 0.0)) > tolerance);
    assert!(ray.distance_to_point(Point3D::new(5.0, 0.5 * tolerance, 0.0)) <= tolerance);
}

#[test]
fn test_ray_metadata() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let ray = Ray3D::new(origin, direction.as_vector()).unwrap();

    assert_eq!(ray.primitive_kind(), PrimitiveKind::Ray);
}

#[test]
fn test_ray_tolerant_eq() {
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

    let ray1 = Ray3D::new(origin, direction.as_vector()).unwrap();
    let ray2 = Ray3D::new(origin, direction.as_vector()).unwrap();
    let ray3 = Ray3D::new(Point3D::new(1.0, 0.0, 0.0), direction.as_vector()).unwrap();

    assert!(ray1.tolerant_eq(&ray2, 0.01));
    assert!(!ray1.tolerant_eq(&ray3, 0.01));
}
