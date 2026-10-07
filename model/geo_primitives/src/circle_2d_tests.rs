//! Circle2D のテスト

use crate::{Circle2D, Point2D, Vector2D};
use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};
use std::f64::consts::{PI, TAU};

/// 基本作成テスト
#[test]
fn test_circle2d_creation() {
    let center = Point2D::new(2.0, 3.0);
    let radius = 5.0;
    let circle = Circle2D::new(center, radius).unwrap();

    assert_eq!(circle.center_internal(), center);
    assert_eq!(circle.radius_internal(), radius);
    assert_eq!(circle.diameter(), 10.0);
}

/// 無効な円の作成テスト
#[test]
fn test_circle2d_invalid_creation() {
    let center = Point2D::new(0.0, 0.0);

    // 負の半径
    assert!(Circle2D::new(center, -1.0).is_none());

    // ゼロ半径
    assert!(Circle2D::new(center, 0.0).is_none());
}

/// 単位円テスト
#[test]
fn test_unit_circle() {
    let circle = Circle2D::<f64>::unit_circle();

    assert_eq!(circle.center_internal(), Point2D::new(0.0, 0.0));
    assert_eq!(circle.radius_internal(), 1.0);
    assert_eq!(circle.diameter(), 2.0);
}

/// 3点からの外接円テスト
#[test]
fn test_from_three_points() {
    // 直角三角形の3点
    let p1 = Point2D::new(0.0, 0.0);
    let p2 = Point2D::new(3.0, 0.0);
    let p3 = Point2D::new(0.0, 4.0);

    let circle = Circle2D::from_three_points(p1, p2, p3).unwrap();

    // 外接円の中心は(1.5, 2.0)、半径は2.5
    assert!((circle.center_internal().x() - 1.5f64).abs() < TOLERANCE_F64);
    assert!((circle.center_internal().y() - 2.0f64).abs() < TOLERANCE_F64);
    assert!((circle.radius_internal() - 2.5f64).abs() < TOLERANCE_F64);

    // 3点すべてが円上にあることを確認
    assert!(circle.point_on_circumference(p1));
    assert!(circle.point_on_circumference(p2));
    assert!(circle.point_on_circumference(p3));
}

/// 共線点からの外接円テスト（失敗ケース）
#[test]
fn test_from_collinear_points() {
    let p1 = Point2D::new(0.0, 0.0);
    let p2 = Point2D::new(1.0, 1.0);
    let p3 = Point2D::new(2.0, 2.0);

    // 共線点からは円を作れない
    assert!(Circle2D::from_three_points(p1, p2, p3).is_none());
}

/// 円周・面積計算テスト
#[test]
fn test_circle2d_metrics() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

    // 円周 = 2πr
    let expected_circumference = TAU * 2.0;
    assert!((circle.circumference() - expected_circumference).abs() < TOLERANCE_F64);

    // 面積 = πr²
    let expected_area = PI * 4.0;
    assert!((circle.area() - expected_area).abs() < TOLERANCE_F64);
}

/// 角度での点取得テスト
#[test]
fn test_point_at_angle() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    // 0度の点（右方向）
    let p0 = circle.point_at_angle(0.0);
    assert!((p0.x() - 3.0f64).abs() < TOLERANCE_F64);
    assert!((p0.y() - 1.0f64).abs() < TOLERANCE_F64);

    // 90度の点（上方向）
    let p90 = circle.point_at_angle(PI / 2.0);
    assert!((p90.x() - 1.0).abs() < TOLERANCE_F64);
    assert!((p90.y() - 3.0).abs() < TOLERANCE_F64);

    // 180度の点（左方向）
    let p180 = circle.point_at_angle(PI);
    assert!((p180.x() - (-1.0)).abs() < TOLERANCE_F64);
    assert!((p180.y() - 1.0).abs() < TOLERANCE_F64);
}

/// パラメータでの点取得テスト
#[test]
fn test_point_at_parameter() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // Circle core parameter は local angle domain (0..=2π)
    // t=0（開始点）
    let p0 = circle.point_at_parameter(0.0);
    assert!((p0.x() - 1.0f64).abs() < TOLERANCE_F64);
    assert!((p0.y() - 0.0f64).abs() < TOLERANCE_F64);

    // t=π/2（90度）
    let p25 = circle.point_at_parameter(PI / 2.0);
    assert!((p25.x() - 0.0f64).abs() < TOLERANCE_F64);
    assert!((p25.y() - 1.0f64).abs() < TOLERANCE_F64);

    // t=π（180度）
    let p50 = circle.point_at_parameter(PI);
    assert!((p50.x() - (-1.0f64)).abs() < TOLERANCE_F64);
    assert!((p50.y() - 0.0f64).abs() < TOLERANCE_F64);

    // t=2π（360度、開始点と同じ）
    let p100 = circle.point_at_parameter(TAU);
    assert!((p100.x() - 1.0f64).abs() < TOLERANCE_F64);
    assert!((p100.y() - 0.0f64).abs() < TOLERANCE_F64);
}

/// 点の包含判定テスト
#[test]
fn test_point_containment() {
    let circle = Circle2D::new(Point2D::new(2.0, 3.0), 5.0).unwrap();

    // 中心点（内部）
    assert!(circle.contains_point(Point2D::new(2.0, 3.0)));

    // 円上の点
    let on_circle = Point2D::new(7.0, 3.0); // 右端
    assert!(circle.point_on_circumference(on_circle));
    assert!(circle.contains_point(on_circle));

    // 外部の点
    let outside = Point2D::new(10.0, 3.0);
    assert!(!circle.contains_point(outside));
    assert!(!circle.point_on_circumference(outside));
}

/// 距離計算テスト
#[test]
fn test_distance_calculations() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();

    // 円周までの距離のため、中心からは半径（3）
    assert_eq!(circle.distance_to_point(Point2D::new(0.0, 0.0)), 3.0);

    // 円上の点からの距離（0）
    assert_eq!(circle.distance_to_point(Point2D::new(3.0, 0.0)), 0.0);

    // 外部の点からの距離
    let outside_point = Point2D::new(6.0, 0.0);
    assert_eq!(circle.distance_to_point(outside_point), 3.0);

    // 内部の点からも円周までの距離（3 - 1 = 2）
    assert_eq!(circle.distance_to_point(Point2D::new(1.0, 0.0)), 2.0);
}

/// 変形操作テスト
#[test]
fn test_transformations() {
    let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();

    // スケール
    let scaled = circle.scale(2.0).unwrap();
    assert_eq!(scaled.center_internal(), Point2D::new(1.0, 2.0));
    assert_eq!(scaled.radius_internal(), 6.0);

    // 無効なスケール
    assert!(circle.scale(-1.0).is_none());
    assert!(circle.scale(0.0).is_none());

    // 平行移動
    let offset = Vector2D::new(2.0, -1.0);
    let translated = circle.translate(offset);
    assert_eq!(translated.center_internal(), Point2D::new(3.0, 1.0));
    assert_eq!(translated.radius_internal(), 3.0);

    // 移動
    let new_center = Point2D::new(5.0, 5.0);
    let moved = circle.move_to(new_center);
    assert_eq!(moved.center_internal(), new_center);
    assert_eq!(moved.radius_internal(), 3.0);
}

/// 円同士の関係テスト
#[test]
fn test_circle_relationships() {
    let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();
    let circle2 = Circle2D::new(Point2D::new(4.0, 0.0), 2.0).unwrap(); // 交差
    let circle3 = Circle2D::new(Point2D::new(1.0, 0.0), 1.0).unwrap(); // 内包
    let circle4 = Circle2D::new(Point2D::new(10.0, 0.0), 1.0).unwrap(); // 離れている

    // 包含判定
    assert!(circle1.contains_circle(&circle3)); // circle1がcircle3を包含
    assert!(!circle1.contains_circle(&circle2)); // 交差関係
    assert!(!circle1.contains_circle(&circle4)); // 離れている
}

/// 境界ボックステスト
#[test]
fn test_bounding_box() {
    let circle = Circle2D::new(Point2D::new(2.0, 3.0), 1.5).unwrap();
    let bbox = circle.bounding_box();

    assert_eq!(bbox.0, Point2D::new(0.5, 1.5));
    assert_eq!(bbox.1, Point2D::new(3.5, 4.5));
    assert_eq!((bbox.1.x() - bbox.0.x()), 3.0);
    assert_eq!((bbox.1.y() - bbox.0.y()), 3.0);
}

/// 3D変換テスト
#[test]
fn test_to_3d() {
    let circle2d = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();
    let circle3d = circle2d.to_3d();

    assert_eq!(circle3d.center_internal().x(), 1.0);
    assert_eq!(circle3d.center_internal().y(), 2.0);
    assert_eq!(circle3d.center_internal().z(), 0.0);
    assert_eq!(circle3d.radius_internal(), 3.0);

    // Z値指定での変換
    let circle3d_z = circle2d.to_3d_at_z(5.0);
    assert_eq!(circle3d_z.center_internal().z(), 5.0);
}

/// Foundation trait - CoreFoundationテスト
#[test]
fn test_geometry_foundation() {
    let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();

    // 境界ボックス取得
    let bbox = circle.bounding_box();
    assert_eq!(bbox.0, Point2D::new(-2.0, -1.0));
    assert_eq!(bbox.1, Point2D::new(4.0, 5.0));
}

/// Foundation trait - BasicMetricsテスト
#[test]
fn test_basic_metrics() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

    // 閉曲線の主語彙は circumference
    let circumference = circle.circumference();
    assert!((circumference - TAU * 2.0).abs() < TOLERANCE_F64);

    // 面積
    let area = circle.area();
    assert!((area - PI * 4.0).abs() < TOLERANCE_F64);

    // perimeter は導入せず、closed curve は circumference で読む
    let circumference_again = circle.circumference();
    assert!((circumference_again - TAU * 2.0).abs() < TOLERANCE_F64);
}

/// Foundation trait - BasicContainmentテスト
#[test]
fn test_basic_containment() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    let inside = Point2D::new(1.0, 1.0); // 中心
    let on_boundary = Point2D::new(3.0, 1.0); // 円上
    let outside = Point2D::new(5.0, 1.0); // 外部

    // 包含判定
    assert!(circle.contains_point(inside));
    assert!(circle.contains_point(on_boundary));
    assert!(!circle.contains_point(outside));

    // 境界判定
    assert!(!circle.point_on_circumference(inside));
    assert!(circle.point_on_circumference(on_boundary));
    assert!(!circle.point_on_circumference(outside));

    // 距離計算（円周までの距離のため、中心からは半径）
    assert_eq!(circle.distance_to_point(inside), 2.0);
    assert_eq!(circle.distance_to_point(on_boundary), 0.0);
    assert_eq!(circle.distance_to_point(outside), 2.0);
}

/// Foundation trait - BasicParametricテスト
#[test]
fn test_basic_parametric() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // local angle parameter での点取得
    let p0 = circle.point_at_parameter(0.0);
    let p25 = circle.point_at_parameter(PI / 2.0);
    let p50 = circle.point_at_parameter(PI);

    assert!((p0.x() - 1.0f64).abs() < TOLERANCE_F64);
    assert!((p25.y() - 1.0f64).abs() < TOLERANCE_F64);
    assert!((p50.x() - (-1.0f64)).abs() < TOLERANCE_F64);
}

/// f32での動作テスト
#[test]
fn test_circle2d_f32() {
    let circle: Circle2D<f32> = Circle2D::new(Point2D::new(1.0f32, 2.0f32), 3.0f32).unwrap();

    assert_eq!(circle.radius_internal(), 3.0f32);
    assert_eq!(circle.diameter(), 6.0f32);

    let area = circle.area();
    assert!((area - (std::f32::consts::PI * 9.0f32)).abs() < TOLERANCE_F32);
}
