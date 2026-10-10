//! Circle2D のテスト

use crate::{Circle2D, Point2D, Vector2D};
use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};
use geo_contracts::PointClassification;
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
    assert!(circle.contains_point(&p1, TOLERANCE_F64));
    assert!(circle.contains_point(&p2, TOLERANCE_F64));
    assert!(circle.contains_point(&p3, TOLERANCE_F64));
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

    // contains_point は円周上にあるかを判定する
    let center = Point2D::new(2.0, 3.0);
    let on_circle = Point2D::new(7.0, 3.0); // 右端
    let outside = Point2D::new(10.0, 3.0);
    assert!(!circle.contains_point(&center, TOLERANCE_F64));
    assert!(circle.contains_point(&on_circle, TOLERANCE_F64));
    assert!(!circle.contains_point(&outside, TOLERANCE_F64));

    // classify_point は円が囲む領域に対して分類する
    assert_eq!(
        circle.classify_point(&center, TOLERANCE_F64),
        PointClassification::Inside
    );
    assert_eq!(
        circle.classify_point(&on_circle, TOLERANCE_F64),
        PointClassification::OnBoundary
    );
    assert_eq!(
        circle.classify_point(&outside, TOLERANCE_F64),
        PointClassification::Outside
    );
}

/// 許容誤差の境目での分類テスト
#[test]
fn test_point_classification_tolerance() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0).unwrap();
    let tolerance = 1e-3;

    // 円周までの距離が許容誤差以内なら内側・外側のどちらからでも OnBoundary
    for x in [5.0 - 0.5 * tolerance, 5.0 + 0.5 * tolerance] {
        let point = Point2D::new(x, 0.0);
        assert_eq!(
            circle.classify_point(&point, tolerance),
            PointClassification::OnBoundary
        );
        assert!(circle.contains_point(&point, tolerance));
    }

    // 許容誤差を超えれば Inside / Outside
    let inside = Point2D::new(5.0 - 2.0 * tolerance, 0.0);
    let outside = Point2D::new(5.0 + 2.0 * tolerance, 0.0);
    assert_eq!(
        circle.classify_point(&inside, tolerance),
        PointClassification::Inside
    );
    assert_eq!(
        circle.classify_point(&outside, tolerance),
        PointClassification::Outside
    );
    assert!(!circle.contains_point(&inside, tolerance));
    assert!(!circle.contains_point(&outside, tolerance));
}

/// trait定義の包含判定・分類テスト
#[test]
fn test_containment_trait() {
    use geo_contracts::Circle2DContainment;

    let circle = Circle2D::new(Point2D::new(0.0_f64, 0.0), 1.0).unwrap();
    assert!(Circle2DContainment::contains_point(&circle, (1.0, 0.0)));
    assert!(!Circle2DContainment::contains_point(&circle, (0.0, 0.0)));
    assert_eq!(
        Circle2DContainment::classify_point(&circle, (0.0, 0.0)),
        PointClassification::Inside
    );
    assert_eq!(
        Circle2DContainment::classify_point(&circle, (0.0, 1.0)),
        PointClassification::OnBoundary
    );
    assert_eq!(
        Circle2DContainment::classify_point(&circle, (2.0, 0.0)),
        PointClassification::Outside
    );
}

/// 距離計算テスト
#[test]
fn test_distance_calculations() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();

    // 円周までの距離のため、中心からは半径（3）
    assert_eq!(circle.distance_to_point(&Point2D::new(0.0, 0.0)), 3.0);

    // 円上の点からの距離（0）
    assert_eq!(circle.distance_to_point(&Point2D::new(3.0, 0.0)), 0.0);

    // 外部の点からの距離
    let outside_point = Point2D::new(6.0, 0.0);
    assert_eq!(circle.distance_to_point(&outside_point), 3.0);

    // 内部の点からも円周までの距離（3 - 1 = 2）
    assert_eq!(circle.distance_to_point(&Point2D::new(1.0, 0.0)), 2.0);
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

    // 円周上の判定
    assert!(!circle.contains_point(&inside, TOLERANCE_F64));
    assert!(circle.contains_point(&on_boundary, TOLERANCE_F64));
    assert!(!circle.contains_point(&outside, TOLERANCE_F64));

    // 領域に対する分類
    assert_eq!(
        circle.classify_point(&inside, TOLERANCE_F64),
        PointClassification::Inside
    );
    assert_eq!(
        circle.classify_point(&on_boundary, TOLERANCE_F64),
        PointClassification::OnBoundary
    );
    assert_eq!(
        circle.classify_point(&outside, TOLERANCE_F64),
        PointClassification::Outside
    );

    // 距離計算（円周までの距離のため、中心からは半径）
    assert_eq!(circle.distance_to_point(&inside), 2.0);
    assert_eq!(circle.distance_to_point(&on_boundary), 0.0);
    assert_eq!(circle.distance_to_point(&outside), 2.0);
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

/// 点に対応するパラメータは最近点の角度
#[test]
fn test_parameter_for_point_is_angle_of_closest_point() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    // 円周上・内部・外部の点のいずれも、中心から見た方向の角度
    for (x, y, expected) in [
        (3.0, 1.0, 0.0),
        (1.0, 2.0, std::f64::consts::FRAC_PI_2),
        (-4.0, 1.0, PI),
        (1.0, -0.5, 3.0 * std::f64::consts::FRAC_PI_2),
    ] {
        let point = Point2D::new(x, y);
        let t = circle.parameter_for_point(&point);
        assert!((t - expected).abs() < TOLERANCE_F64);
        assert!(
            circle
                .point_at_parameter(t)
                .distance_to(&circle.closest_point(&point))
                < TOLERANCE_F64
        );
    }

    // 中心は closest_point と同じく角度 0 の点を最近点とする
    assert!(circle.parameter_for_point(&Point2D::new(1.0, 1.0)).abs() < TOLERANCE_F64);
}
