//! Point3D のテスト

use crate::Point3D;

#[test]
fn test_point3d_creation() {
    let point = Point3D::new(1.0, 2.0, 3.0);
    assert_eq!(point.x(), 1.0);
    assert_eq!(point.y(), 2.0);
    assert_eq!(point.z(), 3.0);
}

#[test]
fn test_point3d_origin() {
    let origin = Point3D::<f64>::origin();
    assert_eq!(origin.x(), 0.0);
    assert_eq!(origin.y(), 0.0);
    assert_eq!(origin.z(), 0.0);
}

#[test]
fn test_point3d_distance() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(3.0, 4.0, 0.0);
    assert_eq!(p1.distance_to(&p2), 5.0); // 3-4-5直角三角形
}

#[test]
fn test_point3d_equality() {
    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let p2 = Point3D::new(1.0, 2.0, 3.0);
    let p3 = Point3D::new(1.0, 2.0, 4.0);

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
}

#[test]
fn test_point3d_f32_compatibility() {
    let point_f32 = Point3D::new(1.0f32, 2.0f32, 3.0f32);
    let point_f64 = Point3D::new(1.0f64, 2.0f64, 3.0f64);

    assert_eq!(point_f32.x(), 1.0f32);
    assert_eq!(point_f64.x(), 1.0f64);
}

#[test]
fn test_basic_operations() {
    let point = Point3D::new(1.0, 2.0, 3.0);
    let same_point = Point3D::new(1.0, 2.0, 3.0);
    let different_point = Point3D::new(2.0, 3.0, 4.0);

    // 点は自分自身を含む
    assert!(point.contains_point(&same_point));
    assert!(!point.contains_point(&different_point));

    // 境界判定（許容誤差付き）

    // 距離計算
    assert_eq!(point.distance_to(&same_point), 0.0);
    assert_eq!(point.distance_to(&Point3D::new(1.0, 2.0, 6.0)), 3.0);
}

#[test]
fn test_coordinate_arithmetic() {
    let point = Point3D::new(1.0, 2.0, 3.0);

    // 座標演算による移動テスト（実装済み機能）
    let translated_x = Point3D::new(point.x() + 1.0, point.y(), point.z());
    assert_eq!(translated_x, Point3D::new(2.0, 2.0, 3.0));

    let translated_y = Point3D::new(point.x(), point.y() + 1.0, point.z());
    assert_eq!(translated_y, Point3D::new(1.0, 3.0, 3.0));

    let translated_xy = Point3D::new(point.x() + 1.0, point.y() + 1.0, point.z());
    assert_eq!(translated_xy, Point3D::new(2.0, 3.0, 3.0));
}

#[test]
fn test_lerp_interpolation() {
    let point1 = Point3D::new(0.0, 0.0, 0.0);
    let point2 = Point3D::new(4.0, 6.0, 8.0);

    // 線形補間テスト（実装済み機能）
    let midpoint = point1.lerp(&point2, 0.5);
    assert_eq!(midpoint.x(), 2.0);
    assert_eq!(midpoint.y(), 3.0);
    assert_eq!(midpoint.z(), 4.0);

    let quarter = point1.lerp(&point2, 0.25);
    assert_eq!(quarter.x(), 1.0);
    assert_eq!(quarter.y(), 1.5);
    assert_eq!(quarter.z(), 2.0);
}

#[test]
fn test_new() {
    let p = Point3D::new(1.0, 2.0, 3.0);
    assert_eq!(p.x(), 1.0);
    assert_eq!(p.y(), 2.0);
    assert_eq!(p.z(), 3.0);
}

#[test]
fn test_origin() {
    let origin = Point3D::<f64>::origin();
    assert_eq!(origin.x(), 0.0);
    assert_eq!(origin.y(), 0.0);
    assert_eq!(origin.z(), 0.0);
}

#[test]
fn test_from_tuple() {
    let p = Point3D::from_tuple((1.0, 2.0, 3.0));
    assert_eq!(p.x(), 1.0);
    assert_eq!(p.y(), 2.0);
    assert_eq!(p.z(), 3.0);
}

#[test]
fn test_distance_to() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(3.0, 4.0, 0.0);
    assert_eq!(p1.distance_to(&p2), 5.0);
}

#[test]
fn test_distance_squared_to() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(3.0, 4.0, 0.0);
    assert_eq!(p1.distance_squared_to(&p2), 25.0);
}

#[test]
fn test_norm() {
    let p = Point3D::new(3.0, 4.0, 0.0);
    assert_eq!(p.norm(), 5.0);
}

#[test]
fn test_norm_squared() {
    let p = Point3D::new(3.0, 4.0, 0.0);
    assert_eq!(p.norm_squared(), 25.0);
}

#[test]
fn test_midpoint() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(2.0, 4.0, 6.0);
    let mid = p1.midpoint(&p2);
    assert_eq!(mid.x(), 1.0);
    assert_eq!(mid.y(), 2.0);
    assert_eq!(mid.z(), 3.0);
}

#[test]
fn test_lerp() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(10.0, 10.0, 10.0);

    let lerp0 = p1.lerp(&p2, 0.0);
    assert_eq!(lerp0.x(), 0.0);

    let lerp_half = p1.lerp(&p2, 0.5);
    assert_eq!(lerp_half.x(), 5.0);
    assert_eq!(lerp_half.y(), 5.0);
    assert_eq!(lerp_half.z(), 5.0);

    let lerp1 = p1.lerp(&p2, 1.0);
    assert_eq!(lerp1.x(), 10.0);
}

#[test]
fn test_from_trait() {
    let p: Point3D<f64> = (1.0, 2.0, 3.0).into();
    assert_eq!(p.x(), 1.0);
    assert_eq!(p.y(), 2.0);
    assert_eq!(p.z(), 3.0);
}

#[test]
fn test_equality() {
    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let p2 = Point3D::new(1.0, 2.0, 3.0);
    let p3 = Point3D::new(1.0, 2.0, 4.0);

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
}

#[test]
fn test_clone() {
    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let p2 = p1;
    assert_eq!(p1, p2);
}
