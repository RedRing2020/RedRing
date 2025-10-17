//! InfiniteLine2D のテスト

use crate::{InfiniteLine2D, Point2D, Vector2D};
use geo_foundation::core_foundation::{
    BasicContainment, BasicDirectional, BasicParametric,
};
use std::f64::consts::FRAC_PI_4;

/// 基本作成テスト
#[test]
fn test_infinite_line2d_creation() {
    let point = Point2D::new(1.0, 2.0);
    let direction = Vector2D::new(3.0, 4.0);
    let line = InfiniteLine2D::new(point, direction).unwrap();

    assert_eq!(line.point(), point);
    // 方向ベクトルは正規化される
    assert!((line.direction().length() - 1.0f64).abs() < 1e-10);
    assert_eq!(line.direction().x(), 0.6);
    assert_eq!(line.direction().y(), 0.8);
}

/// 無効な直線作成テスト
#[test]
fn test_infinite_line2d_invalid_creation() {
    let point = Point2D::new(1.0, 2.0);
    let zero_direction = Vector2D::zero();

    // ゼロベクトルでは直線を作成できない
    assert!(InfiniteLine2D::new(point, zero_direction).is_none());
}

/// 2点からの作成テスト
#[test]
fn test_from_two_points() {
    let p1 = Point2D::new(1.0f64, 1.0f64);
    let p2 = Point2D::new(4.0f64, 5.0f64);
    let line = InfiniteLine2D::from_two_points(p1, p2).unwrap();

    assert_eq!(line.point(), p1);

    // 方向ベクトルは正規化される
    let expected_direction = Vector2D::new(3.0f64, 4.0f64).normalize();
    assert!((line.direction().x() - expected_direction.x()).abs() < 1e-10f64);
    assert!((line.direction().y() - expected_direction.y()).abs() < 1e-10f64);
}

/// 同一点での作成テスト（失敗ケース）
#[test]
fn test_from_identical_points() {
    let p = Point2D::new(1.0, 2.0);
    assert!(InfiniteLine2D::from_two_points(p, p).is_none());
}

/// 軸平行直線テスト
#[test]
fn test_axis_aligned_lines() {
    // 水平線
    let horizontal = InfiniteLine2D::horizontal(3.0);
    assert_eq!(horizontal.point(), Point2D::new(0.0, 3.0));
    assert_eq!(horizontal.direction(), Vector2D::unit_x());

    // 垂直線
    let vertical = InfiniteLine2D::vertical(2.0);
    assert_eq!(vertical.point(), Point2D::new(2.0, 0.0));
    assert_eq!(vertical.direction(), Vector2D::unit_y());
}

/// 傾きと切片からの作成テスト
#[test]
fn test_from_slope_intercept() {
    // y = 2x + 3
    let line = InfiniteLine2D::from_slope_intercept(2.0f64, 3.0f64);

    assert_eq!(line.point(), Point2D::new(0.0, 3.0));

    // 傾き2の直線の方向ベクトルは(1, 2)を正規化したもの
    let expected_direction = Vector2D::new(1.0f64, 2.0f64).normalize();
    assert!((line.direction().x() - expected_direction.x()).abs() < 1e-10f64);
    assert!((line.direction().y() - expected_direction.y()).abs() < 1e-10f64);
}

/// 法線ベクトルテスト
#[test]
fn test_normal() {
    let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();

    let normal = line.normal();
    assert_eq!(normal, Vector2D::new(0.0, -1.0)); // 右回り90度回転
}

/// パラメータでの点取得テスト
#[test]
fn test_point_at_parameter() {
    let line = InfiniteLine2D::new(Point2D::new(1.0, 2.0), Vector2D::new(1.0, 0.0)).unwrap();

    // t=0での点（基準点）
    assert_eq!(line.point_at_parameter(0.0), Point2D::new(1.0, 2.0));

    // t=3での点
    assert_eq!(line.point_at_parameter(3.0), Point2D::new(4.0, 2.0));

    // t=-2での点
    assert_eq!(line.point_at_parameter(-2.0), Point2D::new(-1.0, 2.0));
}

/// 距離計算テスト
#[test]
fn test_distance_calculations() {
    // X軸に平行な直線（y = 2）
    let line = InfiniteLine2D::horizontal(2.0);

    // 直線上の点
    assert_eq!(line.distance_to_point(&Point2D::new(5.0, 2.0)), 0.0);

    // 直線から上に3離れた点
    assert_eq!(line.distance_to_point(&Point2D::new(1.0, 5.0)), 3.0);

    // 直線から下に1離れた点
    assert_eq!(line.distance_to_point(&Point2D::new(-2.0, 1.0)), 1.0);
}

/// 点の包含判定テスト
#[test]
fn test_contains_point() {
    let line = InfiniteLine2D::from_slope_intercept(1.0, 0.0); // y = x

    // 直線上の点
    assert!(line.contains_point(&Point2D::new(1.0, 1.0), 1e-10));
    assert!(line.contains_point(&Point2D::new(-2.0, -2.0), 1e-10));

    // 直線外の点
    assert!(!line.contains_point(&Point2D::new(1.0, 2.0), 1e-10));
    assert!(!line.contains_point(&Point2D::new(0.0, 1.0), 1e-10));
}

/// 点の投影テスト
#[test]
fn test_point_projection() {
    let line = InfiniteLine2D::horizontal(3.0); // y = 3

    let point = Point2D::new(5.0, 7.0);
    let projected = line.project_point(&point);

    assert_eq!(projected, Point2D::new(5.0, 3.0));
}

/// パラメータ取得テスト
#[test]
fn test_parameter_for_point() {
    let line = InfiniteLine2D::new(Point2D::new(2.0, 1.0), Vector2D::new(1.0, 0.0)).unwrap();

    // 基準点のパラメータは0
    assert_eq!(line.parameter_for_point(&Point2D::new(2.0, 1.0)), 0.0);

    // 方向に沿って3進んだ点のパラメータは3
    assert_eq!(line.parameter_for_point(&Point2D::new(5.0, 1.0)), 3.0);

    // 逆方向に2進んだ点のパラメータは-2
    assert_eq!(line.parameter_for_point(&Point2D::new(0.0, 1.0)), -2.0);
}

/// 直線の交点テスト
#[test]
fn test_line_intersection() {
    let line1 = InfiniteLine2D::horizontal(2.0); // y = 2
    let line2 = InfiniteLine2D::vertical(3.0); // x = 3

    let intersection = line1.intersection(&line2).unwrap();
    assert_eq!(intersection, Point2D::new(3.0, 2.0));
}

/// 平行線の交点テスト（交点なし）
#[test]
fn test_parallel_lines_intersection() {
    let line1 = InfiniteLine2D::horizontal(1.0);
    let line2 = InfiniteLine2D::horizontal(2.0);

    assert!(line1.intersection(&line2).is_none());
}

/// 斜め直線の交点テスト
#[test]
fn test_diagonal_intersection() {
    let line1 = InfiniteLine2D::from_slope_intercept(1.0, 0.0); // y = x
    let line2 = InfiniteLine2D::from_slope_intercept(-1.0, 4.0); // y = -x + 4

    let intersection = line1.intersection(&line2).unwrap();
    assert_eq!(intersection, Point2D::new(2.0, 2.0));
}

/// 直線関係の判定テスト
#[test]
fn test_line_relationships() {
    let line1 = InfiniteLine2D::horizontal(1.0);
    let line2 = InfiniteLine2D::horizontal(2.0);
    let line3 = InfiniteLine2D::vertical(1.0);
    let line4 = InfiniteLine2D::horizontal(1.0); // line1と同一

    // 平行判定
    assert!(line1.is_parallel(&line2));
    assert!(!line1.is_parallel(&line3));

    // 垂直判定
    assert!(line1.is_perpendicular(&line3));
    assert!(!line1.is_perpendicular(&line2));

    // 同一判定
    assert!(line1.is_coincident(&line4));
    assert!(!line1.is_coincident(&line2));
}

/// 最近点取得テスト
#[test]
fn test_closest_point() {
    let line = InfiniteLine2D::from_slope_intercept(0.0, 2.0); // y = 2
    let point = Point2D::new(3.0, 5.0);

    let closest = line.closest_point(&point);
    assert_eq!(closest, Point2D::new(3.0, 2.0));
}

/// 変形操作テスト
#[test]
fn test_transformations() {
    let line = InfiniteLine2D::new(Point2D::new(1.0, 1.0), Vector2D::new(1.0, 0.0)).unwrap();

    // 平行移動
    let offset = Vector2D::new(2.0, 3.0);
    let translated = line.translate(offset);
    assert_eq!(translated.point(), Point2D::new(3.0, 4.0));
    assert_eq!(translated.direction(), line.direction());

    // 方向反転
    let reversed = line.reverse();
    assert_eq!(reversed.point(), line.point());
    assert_eq!(reversed.direction(), -line.direction());
}

/// 角度・傾き取得テスト
#[test]
fn test_angle_and_slope() {
    // 45度の直線
    let line_45 = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 1.0)).unwrap();

    assert!((line_45.angle() - FRAC_PI_4).abs() < 1e-10);
    assert!((line_45.slope().unwrap() - 1.0).abs() < 1e-10);

    // 垂直線
    let vertical = InfiniteLine2D::vertical(0.0);
    assert_eq!(vertical.slope(), None);

    // 水平線
    let horizontal = InfiniteLine2D::horizontal(0.0);
    assert_eq!(horizontal.slope(), Some(0.0));
}

/// 切片取得テスト
#[test]
fn test_intercepts() {
    // y = 2x + 3
    let line = InfiniteLine2D::from_slope_intercept(2.0, 3.0);

    // Y切片
    assert_eq!(line.y_intercept(), Some(3.0));

    // X切片（y = 0のときのx）: 0 = 2x + 3 => x = -1.5
    assert!((line.x_intercept().unwrap() - (-1.5f64)).abs() < 1e-10);

    // 垂直線のY切片はNone
    let vertical = InfiniteLine2D::vertical(5.0);
    assert_eq!(vertical.y_intercept(), None);

    // 水平線のX切片はNone
    let horizontal = InfiniteLine2D::horizontal(2.0);
    assert_eq!(horizontal.x_intercept(), None);
}

/// 3D変換テスト
#[test]
fn test_to_3d() {
    let line2d = InfiniteLine2D::new(Point2D::new(1.0, 2.0), Vector2D::new(3.0, 4.0)).unwrap();

    let line3d = line2d.to_3d();
    assert_eq!(line3d.point().x(), 1.0);
    assert_eq!(line3d.point().y(), 2.0);
    assert_eq!(line3d.point().z(), 0.0);

    // Z値指定での変換
    let line3d_z = line2d.to_3d_at_z(5.0);
    assert_eq!(line3d_z.point().z(), 5.0);
}

/// Foundation trait - CoreFoundationテスト
#[test]
fn test_geometry_foundation() {
    let line = InfiniteLine2D::horizontal(1.0);

    // 境界ボックス取得（実用的な大きな値）
    let bbox = line.bounding_box();
    let large_value = 1e6;
    assert_eq!(bbox.min().x(), -large_value);
    assert_eq!(bbox.max().x(), large_value);
}

/// Foundation trait - BasicContainmentテスト
#[test]
fn test_basic_containment() {
    let line = InfiniteLine2D::from_slope_intercept(1.0, 0.0); // y = x

    let on_line = Point2D::new(2.0, 2.0);
    let off_line = Point2D::new(2.0, 3.0);

    // 包含判定
    assert!(line.contains_point(&on_line, 1e-10));
    assert!(!line.contains_point(&off_line, 1e-10));

    // 境界判定
    assert!(line.on_boundary(&on_line, 1e-10));
    assert!(!line.on_boundary(&off_line, 0.5));
    assert!(line.on_boundary(&off_line, 1.5)); // 許容誤差内

    // 距離計算
    assert_eq!(line.distance_to_point(&on_line), 0.0);
    // y = x からの点(2, 3)の距離は |2 - 3| / sqrt(1^2 + (-1)^2) = 1/sqrt(2)
    let expected_distance = 1.0 / 2.0f64.sqrt();
    assert!((line.distance_to_point(&off_line) - expected_distance).abs() < 1e-10);
}

/// Foundation trait - BasicDirectionalテスト
#[test]
fn test_basic_directional() {
    let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(3.0, 4.0)).unwrap();

    // 方向取得
    let direction = line.direction();
    assert!((direction.length() - 1.0f64).abs() < 1e-10);
    assert_eq!(direction.x(), 0.6);
    assert_eq!(direction.y(), 0.8);

    // 方向反転
    let reversed = line.reverse_direction();
    assert_eq!(reversed.direction(), -direction);
}

/// Foundation trait - BasicParametricテスト
#[test]
fn test_basic_parametric() {
    let line = InfiniteLine2D::horizontal(2.0);

    // パラメータ範囲（実用的な大きな値）
    let (start, end) = line.parameter_range();
    let large_value = 1e6;
    assert_eq!(start, -large_value);
    assert_eq!(end, large_value);

    // パラメータでの点取得
    let p0 = line.point_at_parameter(0.0);
    let p5 = line.point_at_parameter(5.0);

    assert_eq!(p0, Point2D::new(0.0, 2.0));
    assert_eq!(p5, Point2D::new(5.0, 2.0));

    // 接線ベクトル取得（直線では方向ベクトルと同じ）
    let tangent = line.tangent_at_parameter(123.0); // パラメータに関係なく一定
    assert_eq!(tangent, line.direction());
}

/// f32での動作テスト
#[test]
fn test_infinite_line2d_f32() {
    let line: InfiniteLine2D<f32> =
        InfiniteLine2D::new(Point2D::new(1.0f32, 2.0f32), Vector2D::new(3.0f32, 4.0f32)).unwrap();

    assert!((line.direction().length() - 1.0f32).abs() < 1e-6);

    let distance = line.distance_to_point(&Point2D::new(0.0f32, 0.0f32));
    assert!(distance > 0.0f32);
}
