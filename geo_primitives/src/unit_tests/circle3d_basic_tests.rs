use crate::geometry3d::circle::Circle3D;
/// Circle3D 単体テスト
///
/// Circle3D実装の機能テスト
use crate::geometry3d::{Point3D, Vector};
use geo_foundation::constants::precision::{PI, TAU};
use geo_foundation::Scalar;

#[test]
fn test_circle3d_basic_functionality() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let circle = Circle3D::xy_plane_circle(center, 5.0);

    assert_eq!(circle.center(), center);
    assert_eq!(circle.radius(), 5.0);
    assert_eq!(circle.area(), PI * 25.0);
    assert_eq!(circle.circumference(), TAU * 5.0);

    // 法線がZ軸方向であることを確認
    let normal = circle.normal();
    assert!((normal.x() - 0.0).abs() < f64::TOLERANCE);
    assert!((normal.y() - 0.0).abs() < f64::TOLERANCE);
    assert!((normal.z() - 1.0).abs() < f64::TOLERANCE);
}

#[test]
fn test_circle3d_unit_circle() {
    let circle: Circle3D<f64> = Circle3D::unit_circle();

    assert_eq!(circle.center(), Point3D::origin());
    assert_eq!(circle.radius(), 1.0);
    assert_eq!(circle.area(), PI);
    assert_eq!(circle.circumference(), TAU);
}

#[test]
fn test_circle3d_contains_point() {
    let circle = Circle3D::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 5.0);

    assert!(circle.contains_point(&Point3D::new(0.0, 0.0, 0.0))); // 中心
    assert!(circle.contains_point(&Point3D::new(3.0, 4.0, 0.0))); // 内部
    assert!(circle.contains_point(&Point3D::new(5.0, 0.0, 0.0))); // 円周上
    assert!(!circle.contains_point(&Point3D::new(6.0, 0.0, 0.0))); // 外部
    assert!(!circle.contains_point(&Point3D::new(0.0, 0.0, 1.0))); // 平面外
}

#[test]
fn test_circle3d_point_at_angle() {
    let circle = Circle3D::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 2.0);

    // 0度での点（+X軸方向）
    let point_0 = circle.point_at_angle(0.0);
    assert!((point_0.x() - 2.0).abs() < f64::TOLERANCE);
    assert!(point_0.y().abs() < f64::TOLERANCE);
    assert!(point_0.z().abs() < f64::TOLERANCE);

    // 90度での点（+Y軸方向）
    let point_90 = circle.point_at_angle(PI / 2.0);
    assert!(point_90.x().abs() < f64::TOLERANCE);
    assert!((point_90.y() - 2.0).abs() < f64::TOLERANCE);
    assert!(point_90.z().abs() < f64::TOLERANCE);
}

#[test]
fn test_circle3d_transformations() {
    let original = Circle3D::xy_plane_circle(Point3D::new(2.0, 3.0, 1.0), 4.0);

    // スケーリング
    let scaled = original.scale(2.0);
    assert_eq!(scaled.center(), Point3D::new(2.0, 3.0, 1.0));
    assert_eq!(scaled.radius(), 8.0);

    // 平行移動
    let translation_vector = Vector::new(1.0, -1.0, 2.0);
    let translated = original.translate(&translation_vector);
    assert_eq!(translated.center(), Point3D::new(3.0, 2.0, 3.0));
    assert_eq!(translated.radius(), 4.0);
}

#[test]
fn test_circle3d_degenerate_cases() {
    let zero_circle = Circle3D::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 0.0);
    assert!(zero_circle.is_degenerate());

    let normal_circle = Circle3D::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 1.0);
    assert!(!normal_circle.is_degenerate());
}

#[test]
fn test_circle3d_different_planes() {
    let xy_circle = Circle3D::xy_plane_circle(Point3D::origin(), 1.0);
    let xz_circle = Circle3D::xz_plane_circle(Point3D::origin(), 1.0);
    let yz_circle = Circle3D::yz_plane_circle(Point3D::origin(), 1.0);

    // 各平面の法線ベクトルをチェック
    let xy_normal = xy_circle.normal();
    assert!((xy_normal.z() - 1.0).abs() < f64::TOLERANCE);

    let xz_normal = xz_circle.normal();
    assert!((xz_normal.y() - 1.0).abs() < f64::TOLERANCE);

    let yz_normal = yz_circle.normal();
    assert!((yz_normal.x() - 1.0).abs() < f64::TOLERANCE);
}
