//! Circle2D のテスト

use crate::{Circle2D, Point2D};
use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};
use geo_contracts::{
    default_distance_tolerance, Bounded, Circle2DConstructor, Circle2DContainment,
    Circle2DDistance, Circle2DEvaluation, Circle2DProjection, Circle2DProperties, CrossDistance,
    PointClassification,
};
use std::f64::consts::{PI, TAU};

/// 基本作成テスト
#[test]
fn test_circle2d_creation() {
    let center = Point2D::new(2.0, 3.0);
    let radius = 5.0;
    let circle = Circle2D::new(center, radius).unwrap();

    assert_eq!(circle.center(), center);
    assert_eq!(circle.radius(), radius);
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
    assert!(circle.center().distance_to(Point2D::new(1.5, 2.0)) < TOLERANCE_F64);
    assert!((circle.radius() - 2.5f64).abs() < TOLERANCE_F64);

    // 3点すべてが円上にあることを確認
    assert!(circle.contains_point(p1));
    assert!(circle.contains_point(p2));
    assert!(circle.contains_point(p3));
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

/// パラメータでの点取得テスト
#[test]
fn test_point_at_parameter() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // Circle core parameter は local angle domain (0..=2π)
    for (t, expected) in [
        (0.0, Point2D::new(1.0, 0.0)),
        (PI / 2.0, Point2D::new(0.0, 1.0)),
        (PI, Point2D::new(-1.0, 0.0)),
        // t=2π（360度、開始点と同じ）
        (TAU, Point2D::new(1.0, 0.0)),
    ] {
        assert!(circle.point_at_parameter(t).distance_to(expected) < TOLERANCE_F64);
    }

    // 中心をずらした円
    let shifted = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();
    assert!(
        shifted
            .point_at_parameter(PI / 2.0)
            .distance_to(Point2D::new(1.0, 3.0))
            < TOLERANCE_F64
    );
}

/// 点の包含判定・分類テスト
#[test]
fn test_point_containment() {
    let circle = Circle2D::new(Point2D::new(2.0, 3.0), 5.0).unwrap();

    // contains_point は円周上にあるかを判定する
    let center = Point2D::new(2.0, 3.0);
    let on_circle = Point2D::new(7.0, 3.0); // 右端
    let outside = Point2D::new(10.0, 3.0);
    assert!(!circle.contains_point(center));
    assert!(circle.contains_point(on_circle));
    assert!(!circle.contains_point(outside));

    // classify_point は円が囲む領域に対して分類する
    assert_eq!(circle.classify_point(center), PointClassification::Inside);
    assert_eq!(
        circle.classify_point(on_circle),
        PointClassification::OnBoundary
    );
    assert_eq!(circle.classify_point(outside), PointClassification::Outside);
}

/// 許容誤差の境目での分類テスト
#[test]
fn test_point_classification_tolerance() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0).unwrap();
    let tolerance = default_distance_tolerance::<f64>();

    // 円周までの距離が許容誤差以内なら内側・外側のどちらからでも OnBoundary
    for x in [5.0 - 0.5 * tolerance, 5.0 + 0.5 * tolerance] {
        let point = Point2D::new(x, 0.0);
        assert_eq!(
            circle.classify_point(point),
            PointClassification::OnBoundary
        );
        assert!(circle.contains_point(point));
    }

    // 許容誤差を超えれば Inside / Outside
    let inside = Point2D::new(5.0 - 2.0 * tolerance, 0.0);
    let outside = Point2D::new(5.0 + 2.0 * tolerance, 0.0);
    assert_eq!(circle.classify_point(inside), PointClassification::Inside);
    assert_eq!(circle.classify_point(outside), PointClassification::Outside);
    assert!(!circle.contains_point(inside));
    assert!(!circle.contains_point(outside));
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
    assert_eq!(circle.distance_to_point(Point2D::new(6.0, 0.0)), 3.0);

    // 内部の点からも円周までの距離（3 - 1 = 2）
    assert_eq!(circle.distance_to_point(Point2D::new(1.0, 0.0)), 2.0);
}

/// 円同士の距離テスト
#[test]
fn test_distance_between_circles() {
    let circle = Circle2D::new(Point2D::new(0.0_f64, 0.0), 1.0).unwrap();

    // 外部にある円は中心間の距離から半径の和を引く
    let apart = Circle2D::new(Point2D::new(5.0, 0.0), 2.0).unwrap();
    assert!((circle.distance_to(&apart) - 2.0).abs() < TOLERANCE_F64);

    // 交差する円は 0
    let crossing = Circle2D::new(Point2D::new(1.5, 0.0), 1.0).unwrap();
    assert_eq!(circle.distance_to(&crossing), 0.0);

    // 内部にある円は半径の差から中心間の距離を引く
    let inner = Circle2D::new(Point2D::new(0.5, 0.0), 0.25).unwrap();
    assert!((inner.distance_to(&circle) - 0.25).abs() < TOLERANCE_F64);
}

/// 最近点テスト
#[test]
fn test_closest_point() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    assert!(
        circle
            .closest_point(Point2D::new(1.0, 5.0))
            .distance_to(Point2D::new(1.0, 3.0))
            < TOLERANCE_F64
    );

    // 中心は角度 0 の点を最近点とする
    assert!(
        circle
            .closest_point(Point2D::new(1.0, 1.0))
            .distance_to(Point2D::new(3.0, 1.0))
            < TOLERANCE_F64
    );
}

/// 境界ボックステスト
#[test]
fn test_aabb() {
    let circle = Circle2D::new(Point2D::new(2.0, 3.0), 1.5).unwrap();
    let bbox = circle.aabb().unwrap();

    assert_eq!(bbox.min_point(), Point2D::new(0.5, 1.5));
    assert_eq!(bbox.max_point(), Point2D::new(3.5, 4.5));
}

/// f32での動作テスト
#[test]
fn test_circle2d_f32() {
    let circle: Circle2D<f32> = Circle2D::new(Point2D::new(1.0f32, 2.0f32), 3.0f32).unwrap();

    assert_eq!(circle.radius(), 3.0f32);
    assert!(
        (circle.distance_to_point(Point2D::new(1.0f32, 2.0f32)) - 3.0f32).abs() < TOLERANCE_F32
    );
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
        let t = circle.parameter_for_point(point);
        assert!((t - expected).abs() < TOLERANCE_F64);
        assert!(
            circle
                .point_at_parameter(t)
                .distance_to(circle.closest_point(point))
                < TOLERANCE_F64
        );
    }

    // 中心は closest_point と同じく角度 0 の点を最近点とする
    assert!(circle.parameter_for_point(Point2D::new(1.0, 1.0)).abs() < TOLERANCE_F64);
}
