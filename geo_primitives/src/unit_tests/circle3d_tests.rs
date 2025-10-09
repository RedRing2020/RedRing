//! Circle3D implementation tests
//!
//! Circle3D型の機能テスト

use crate::geometry3d::{Circle, Point3D};
use geo_foundation::{GEOMETRIC_TOLERANCE, PI, TAU};

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
    let circle: Circle<f64> = Circle::unit_circle();

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
    assert!((point.x() - 2.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.y() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.z() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);

    let point = circle.point_at_angle(PI / 2.0);
    assert!((point.x() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.y() - 2.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((point.z() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_bounding_box() {
    let circle = Circle::xy_plane_circle(Point3D::new(1.0, 2.0, 3.0), 4.0);
    let (min, max) = circle.bounding_box();

    // XY平面の円の境界ボックス計算
    // u_axis = (1,0,0), v_axis = (0,1,0) なので、Z軸方向には範囲が広がらない
    assert!((min.x() - (-3.0_f64)).abs() < GEOMETRIC_TOLERANCE); // center.x - radius = 1 - 4 = -3
    assert!((min.y() - (-2.0_f64)).abs() < GEOMETRIC_TOLERANCE); // center.y - radius = 2 - 4 = -2
    assert!((min.z() - 3.0_f64).abs() < GEOMETRIC_TOLERANCE); // center.z (変化なし) = 3
    assert!((max.x() - 5.0_f64).abs() < GEOMETRIC_TOLERANCE); // center.x + radius = 1 + 4 = 5
    assert!((max.y() - 6.0_f64).abs() < GEOMETRIC_TOLERANCE); // center.y + radius = 2 + 4 = 6
    assert!((max.z() - 3.0_f64).abs() < GEOMETRIC_TOLERANCE); // center.z (変化なし) = 3
}

// TODO: to_2dメソッドは現在実装されていないため、テストを無効化
// #[test]
// fn test_to_2d() {
//     let circle_3d = Circle::xy_plane_circle(Point3D::new(1.0, 2.0, 3.0), 4.0);
//     let circle_2d = circle_3d.to_2d();

//     // 基本的なプロパティの確認
//     assert_eq!(circle_2d.radius(), 4.0);
//     assert_eq!(circle_2d.area(), PI * 16.0);
//     assert_eq!(circle_2d.circumference(), TAU * 4.0);
// }
