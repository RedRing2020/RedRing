//! Circle2D のテスト

use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::abstract_types::geometry::foundation::{BasicContainment, BasicParametric};
use std::f64::consts::{PI, TAU};

/// 基本作成テスト
#[test]
fn test_circle2d_creation() {
    let center = Point2D::new(2.0, 3.0);
    let radius = 5.0;
    let circle = Circle2D::new(center, radius).unwrap();

    assert_eq!(circle.center(), center);
    assert_eq!(circle.radius(), radius);
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

    assert_eq!(circle.center(), Point2D::new(0.0, 0.0));
    assert_eq!(circle.radius(), 1.0);
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
    assert!((circle.center().x() - 1.5f64).abs() < 1e-10);
    assert!((circle.center().y() - 2.0f64).abs() < 1e-10);
    assert!((circle.radius() - 2.5f64).abs() < 1e-10);

    // 3点すべてが円上にあることを確認
    assert!(circle.contains_point_on_circle(&p1, 1e-10));
    assert!(circle.contains_point_on_circle(&p2, 1e-10));
    assert!(circle.contains_point_on_circle(&p3, 1e-10));
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
    assert!((circle.circumference() - expected_circumference).abs() < 1e-10);

    // 面積 = πr²
    let expected_area = PI * 4.0;
    assert!((circle.area() - expected_area).abs() < 1e-10);
}

/// 角度での点取得テスト
#[test]
fn test_point_at_angle() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    // 0度の点（右方向）
    let p0 = circle.point_at_angle(0.0);
    assert!((p0.x() - 3.0f64).abs() < 1e-10);
    assert!((p0.y() - 1.0f64).abs() < 1e-10);

    // 90度の点（上方向）
    let p90 = circle.point_at_angle(PI / 2.0);
    assert!((p90.x() - 1.0).abs() < 1e-10);
    assert!((p90.y() - 3.0).abs() < 1e-10);

    // 180度の点（左方向）
    let p180 = circle.point_at_angle(PI);
    assert!((p180.x() - (-1.0)).abs() < 1e-10);
    assert!((p180.y() - 1.0).abs() < 1e-10);
}

/// パラメータでの点取得テスト
#[test]
fn test_point_at_parameter() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // t=0（開始点）
    let p0 = circle.point_at_parameter(0.0);
    assert!((p0.x() - 1.0f64).abs() < 1e-10);
    assert!((p0.y() - 0.0f64).abs() < 1e-10);

    // t=0.25（90度）
    let p25 = circle.point_at_parameter(0.25);
    assert!((p25.x() - 0.0f64).abs() < 1e-10);
    assert!((p25.y() - 1.0f64).abs() < 1e-10);

    // t=0.5（180度）
    let p50 = circle.point_at_parameter(0.5);
    assert!((p50.x() - (-1.0f64)).abs() < 1e-10);
    assert!((p50.y() - 0.0f64).abs() < 1e-10);

    // t=1.0（360度、開始点と同じ）
    let p100 = circle.point_at_parameter(1.0);
    assert!((p100.x() - 1.0f64).abs() < 1e-10);
    assert!((p100.y() - 0.0f64).abs() < 1e-10);
}

/// 接線ベクトルテスト
#[test]
fn test_tangent_at_parameter() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // t=0での接線（上方向）
    let t0 = circle.tangent_at_parameter(0.0);
    assert!((t0.x() - 0.0f64).abs() < 1e-10);
    assert!((t0.y() - 1.0f64).abs() < 1e-10);

    // t=0.25での接線（左方向）
    let t25 = circle.tangent_at_parameter(0.25);
    assert!((t25.x() - (-1.0f64)).abs() < 1e-10);
    assert!((t25.y() - 0.0f64).abs() < 1e-10);

    // 接線ベクトルは単位ベクトル
    assert!((t0.length() - 1.0f64).abs() < 1e-10);
    assert!((t25.length() - 1.0f64).abs() < 1e-10);
}

/// 点の包含判定テスト
#[test]
fn test_point_containment() {
    let circle = Circle2D::new(Point2D::new(2.0, 3.0), 5.0).unwrap();

    // 中心点（内部）
    assert!(circle.contains_point_inside(&Point2D::new(2.0, 3.0)));

    // 円上の点
    let on_circle = Point2D::new(7.0, 3.0); // 右端
    assert!(circle.contains_point_on_circle(&on_circle, 1e-10));
    assert!(circle.contains_point_inside(&on_circle));

    // 外部の点
    let outside = Point2D::new(10.0, 3.0);
    assert!(!circle.contains_point_inside(&outside));
    assert!(!circle.contains_point_on_circle(&outside, 1e-10));
}

/// 距離計算テスト
#[test]
fn test_distance_calculations() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();

    // 中心からの距離（0）
    assert_eq!(circle.distance_to_point(&Point2D::new(0.0, 0.0)), 0.0);

    // 円上の点からの距離（0）
    assert_eq!(circle.distance_to_point(&Point2D::new(3.0, 0.0)), 0.0);

    // 外部の点からの距離
    let outside_point = Point2D::new(6.0, 0.0);
    assert_eq!(circle.distance_to_point(&outside_point), 3.0);

    // 内部の点からの距離（0）
    assert_eq!(circle.distance_to_point(&Point2D::new(1.0, 0.0)), 0.0);
}

/// 変形操作テスト
#[test]
fn test_transformations() {
    let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();

    // スケール
    let scaled = circle.scale(2.0).unwrap();
    assert_eq!(scaled.center(), Point2D::new(1.0, 2.0));
    assert_eq!(scaled.radius(), 6.0);

    // 無効なスケール
    assert!(circle.scale(-1.0).is_none());
    assert!(circle.scale(0.0).is_none());

    // 平行移動
    let offset = Vector2D::new(2.0, -1.0);
    let translated = circle.translate(offset);
    assert_eq!(translated.center(), Point2D::new(3.0, 1.0));
    assert_eq!(translated.radius(), 3.0);

    // 移動
    let new_center = Point2D::new(5.0, 5.0);
    let moved = circle.move_to(new_center);
    assert_eq!(moved.center(), new_center);
    assert_eq!(moved.radius(), 3.0);
}

/// 円同士の関係テスト
#[test]
fn test_circle_relationships() {
    let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();
    let circle2 = Circle2D::new(Point2D::new(4.0, 0.0), 2.0).unwrap(); // 交差
    let circle3 = Circle2D::new(Point2D::new(1.0, 0.0), 1.0).unwrap(); // 内包
    let circle4 = Circle2D::new(Point2D::new(10.0, 0.0), 1.0).unwrap(); // 離れている

    // 交差判定
    assert!(circle1.intersects_circle(&circle2)); // 交差する
    assert!(!circle1.intersects_circle(&circle3)); // 内包関係（交差しない）
    assert!(!circle1.intersects_circle(&circle4)); // 離れている

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

    assert_eq!(bbox.min(), Point2D::new(0.5, 1.5));
    assert_eq!(bbox.max(), Point2D::new(3.5, 4.5));
    assert_eq!(bbox.width(), 3.0);
    assert_eq!(bbox.height(), 3.0);
}

/// 3D変換テスト
#[test]
fn test_to_3d() {
    let circle2d = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();
    let circle3d = circle2d.to_3d();

    assert_eq!(circle3d.center().x(), 1.0);
    assert_eq!(circle3d.center().y(), 2.0);
    assert_eq!(circle3d.center().z(), 0.0);
    assert_eq!(circle3d.radius(), 3.0);

    // Z値指定での変換
    let circle3d_z = circle2d.to_3d_at_z(5.0);
    assert_eq!(circle3d_z.center().z(), 5.0);
}

/// Foundation trait - GeometryFoundationテスト
#[test]
fn test_geometry_foundation() {
    let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();

    // 境界ボックス取得
    let bbox = circle.bounding_box();
    assert_eq!(bbox.min(), Point2D::new(-2.0, -1.0));
    assert_eq!(bbox.max(), Point2D::new(4.0, 5.0));
}

/// Foundation trait - BasicMetricsテスト
#[test]
fn test_basic_metrics() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

    // 長さ（円周）
    use geo_foundation::abstract_types::geometry::foundation::BasicMetrics;
    let length = BasicMetrics::length(&circle).unwrap();
    assert!((length - TAU * 2.0).abs() < 1e-10);

    // 面積
    let area = BasicMetrics::area(&circle).unwrap();
    assert!((area - PI * 4.0).abs() < 1e-10);

    // 周長（円周と同じ）
    let perimeter = BasicMetrics::perimeter(&circle).unwrap();
    assert!((perimeter - TAU * 2.0).abs() < 1e-10);

    // 体積は定義されない
    assert!(BasicMetrics::volume(&circle).is_none());
}

/// Foundation trait - BasicContainmentテスト
#[test]
fn test_basic_containment() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    let inside = Point2D::new(1.0, 1.0); // 中心
    let on_boundary = Point2D::new(3.0, 1.0); // 円上
    let outside = Point2D::new(5.0, 1.0); // 外部

    // 包含判定
    assert!(circle.contains_point(&inside));
    assert!(circle.contains_point(&on_boundary));
    assert!(!circle.contains_point(&outside));

    // 境界判定
    assert!(!circle.on_boundary(&inside, 1e-10));
    assert!(circle.on_boundary(&on_boundary, 1e-10));
    assert!(!circle.on_boundary(&outside, 1e-10));

    // 距離計算
    assert_eq!(circle.distance_to_point(&inside), 0.0);
    assert_eq!(circle.distance_to_point(&on_boundary), 0.0);
    assert_eq!(circle.distance_to_point(&outside), 2.0);
}

/// Foundation trait - BasicParametricテスト
#[test]
fn test_basic_parametric() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // パラメータ範囲
    let (start, end) = circle.parameter_range();
    assert_eq!(start, 0.0);
    assert_eq!(end, 1.0);

    // パラメータでの点取得
    let p0 = circle.point_at_parameter(0.0);
    let p25 = circle.point_at_parameter(0.25);
    let p50 = circle.point_at_parameter(0.5);

    assert!((p0.x() - 1.0f64).abs() < 1e-10);
    assert!((p25.y() - 1.0f64).abs() < 1e-10);
    assert!((p50.x() - (-1.0f64)).abs() < 1e-10);

    // 接線ベクトル取得
    let t0 = circle.tangent_at_parameter(0.0);
    let t25 = circle.tangent_at_parameter(0.25);

    assert!((t0.length() - 1.0f64).abs() < 1e-10);
    assert!((t25.length() - 1.0f64).abs() < 1e-10);
}

/// f32での動作テスト
#[test]
fn test_circle2d_f32() {
    let circle: Circle2D<f32> = Circle2D::new(Point2D::new(1.0f32, 2.0f32), 3.0f32).unwrap();

    assert_eq!(circle.radius(), 3.0f32);
    assert_eq!(circle.diameter(), 6.0f32);

    let area = circle.area();
    assert!((area - (std::f32::consts::PI * 9.0f32)).abs() < 1e-6);
}
