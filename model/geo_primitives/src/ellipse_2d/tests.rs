//! Ellipse2D のテスト

use crate::{Angle, Ellipse2D, Point2D};
use geo_contracts::{
    default_distance_tolerance, Bounded, Ellipse2DConstructor, Ellipse2DContainment,
    Ellipse2DDistance, Ellipse2DEvaluation, Ellipse2DProjection, Ellipse2DProperties,
    PointClassification,
};

use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};

fn no_rotation() -> Angle<f64> {
    Angle::from_radians(0.0)
}

#[test]
fn test_ellipse_creation() {
    let center = Point2D::new(2.0, 3.0);
    let ellipse = Ellipse2D::new(center, 5.0, 3.0, no_rotation()).unwrap();

    assert_eq!(ellipse.center(), center);
    assert_eq!(ellipse.semi_major_axis(), 5.0);
    assert_eq!(ellipse.semi_minor_axis(), 3.0);
    assert_eq!(ellipse.rotation(), no_rotation());

    // 不正な楕円（短軸が長軸より大きい・負の半軸）
    assert!(Ellipse2D::new(center, 3.0, 5.0, no_rotation()).is_none());
    assert!(Ellipse2D::new(center, -1.0, 2.0, no_rotation()).is_none());
}

#[test]
fn test_point_at_parameter() {
    let ellipse = Ellipse2D::new(Point2D::origin(), 4.0, 2.0, no_rotation()).unwrap();

    // Ellipse core parameter は local angle domain (0..2π)
    for (t, expected) in [
        (0.0, Point2D::new(4.0, 0.0)),
        (std::f64::consts::PI / 2.0, Point2D::new(0.0, 2.0)),
        (std::f64::consts::PI, Point2D::new(-4.0, 0.0)),
    ] {
        assert!(ellipse.point_at_parameter(t).distance_to(expected) < TOLERANCE_F64);
    }

    // 回転した楕円の t=0 は長軸方向の端
    let rotated = Ellipse2D::new(
        Point2D::new(1.0, 1.0),
        2.0,
        1.0,
        Angle::from_radians(std::f64::consts::FRAC_PI_2),
    )
    .unwrap();
    assert!(
        rotated
            .point_at_parameter(0.0)
            .distance_to(Point2D::new(1.0, 3.0))
            < TOLERANCE_F64
    );
}

#[test]
fn test_point_at_parameter_lies_on_ellipse() {
    let ellipse = Ellipse2D::new(Point2D::origin(), 5.0, 3.0, no_rotation()).unwrap();

    let p = ellipse.point_at_parameter(std::f64::consts::PI / 6.0);
    let normalized_distance_squared = (p.x() / 5.0).powi(2) + (p.y() / 3.0).powi(2);
    assert!((normalized_distance_squared - 1.0).abs() < TOLERANCE_F64);
    assert!(ellipse.distance_to_point(p) < TOLERANCE_F64);
}

#[test]
fn test_closest_point() {
    let ellipse = Ellipse2D::new(Point2D::origin(), 4.0, 2.0, no_rotation()).unwrap();
    let closest = ellipse.closest_point(Point2D::new(10.0, 0.0));

    assert!(closest.distance_to(Point2D::new(4.0, 0.0)) < TOLERANCE_F64);
    assert!((ellipse.distance_to_point(Point2D::new(10.0, 0.0)) - 6.0).abs() < TOLERANCE_F64);
}

#[test]
fn test_aabb() {
    // 軸に平行な楕円
    let ellipse = Ellipse2D::new(Point2D::new(2.0, 3.0), 4.0, 2.0, no_rotation()).unwrap();
    let bbox = ellipse.aabb().unwrap();
    assert_eq!(bbox.min_point(), Point2D::new(-2.0, 1.0));
    assert_eq!(bbox.max_point(), Point2D::new(6.0, 5.0));

    // 90° 回転した楕円は幅と高さが入れ替わる
    let rotated = Ellipse2D::new(
        Point2D::new(0.0, 0.0),
        4.0,
        2.0,
        Angle::from_radians(std::f64::consts::FRAC_PI_2),
    )
    .unwrap();
    let bbox = rotated.aabb().unwrap();
    assert!(bbox.min_point().distance_to(Point2D::new(-2.0, -4.0)) < TOLERANCE_F64);
    assert!(bbox.max_point().distance_to(Point2D::new(2.0, 4.0)) < TOLERANCE_F64);
}

#[test]
fn test_circumference_approximation() {
    // 円の場合
    let circle_ellipse = Ellipse2D::new(Point2D::origin(), 3.0, 3.0, no_rotation()).unwrap();
    let expected_circle_circumference = 2.0 * std::f64::consts::PI * 3.0;
    assert!(
        (circle_ellipse.circumference() - expected_circle_circumference).abs()
            / expected_circle_circumference
            < 0.01
    );

    // 一般的な楕円
    let circumference = Ellipse2D::new(Point2D::origin(), 5.0, 3.0, no_rotation())
        .unwrap()
        .circumference();
    assert!(circumference > 2.0 * std::f64::consts::PI * 3.0);
    assert!(circumference < 2.0 * std::f64::consts::PI * 5.0);
}

#[test]
fn test_f32_compatibility() {
    let ellipse = Ellipse2D::new(
        Point2D::new(0.0f32, 0.0f32),
        3.0f32,
        2.0f32,
        Angle::from_radians(0.0f32),
    )
    .unwrap();

    assert_eq!(ellipse.semi_major_axis(), 3.0f32);
    assert!(ellipse.distance_to_point(Point2D::new(3.0f32, 0.0f32)) < TOLERANCE_F32);

    let bbox = ellipse.aabb().unwrap();
    assert_eq!(bbox.min_point(), Point2D::new(-3.0f32, -2.0f32));
    assert_eq!(bbox.max_point(), Point2D::new(3.0f32, 2.0f32));
}

#[test]
fn test_ellipse_contains_point() {
    let center = Point2D::new(0.0, 0.0);
    let ellipse = Ellipse2D::new(center, 4.0, 2.0, no_rotation()).unwrap();

    // 楕円上の点
    assert!(ellipse.contains_point(Point2D::new(4.0, 0.0)));
    assert!(ellipse.contains_point(Point2D::new(0.0, -2.0)));

    // 楕円が囲む領域の内部の点・外部の点は楕円上にない
    assert!(!ellipse.contains_point(center));
    assert!(!ellipse.contains_point(Point2D::new(1.0, 0.5)));
    assert!(!ellipse.contains_point(Point2D::new(5.0, 3.0)));
}

#[test]
fn test_ellipse_distance_and_closest_point_to_curve() {
    // 長半軸 2・短半軸 1 の楕円を 30° 回転し、中心を (1, -1) に置く
    let rotation = std::f64::consts::FRAC_PI_6;
    let center = Point2D::new(1.0, -1.0);
    let ellipse = Ellipse2D::new(center, 2.0, 1.0, Angle::from_radians(rotation)).unwrap();
    let to_world = |x: f64, y: f64| {
        Point2D::new(
            center.x() + x * rotation.cos() - y * rotation.sin(),
            center.y() + x * rotation.sin() + y * rotation.cos(),
        )
    };

    // 中心から楕円までの距離は短半軸の長さ
    assert!((ellipse.distance_to_point(center) - 1.0).abs() < TOLERANCE_F64);

    // 局所座標 (1, 0.5) の内部の点。距離と最近点は曲線のパラメータ方程式の数値解で求めた値
    let inside = to_world(1.0, 0.5);
    assert!((ellipse.distance_to_point(inside) - 0.349_605_694_569_673).abs() < TOLERANCE_F64);
    let closest = ellipse.closest_point(inside);
    let expected = to_world(1.110_726_977_965_884, 0.831_607_717_078_608);
    assert!(closest.distance_to(expected) < TOLERANCE_F64);

    // 最近点のパラメータで評価した点は最近点と一致する
    let t = ellipse.parameter_for_point(inside);
    assert!(ellipse.point_at_parameter(t).distance_to(closest) < TOLERANCE_F64);
}

#[test]
fn test_ellipse_classify_point() {
    // 長半軸 4・短半軸 2 の楕円を 90° 回転（長軸が Y 軸方向）
    let ellipse = Ellipse2D::new(
        Point2D::new(0.0, 0.0),
        4.0,
        2.0,
        Angle::from_radians(std::f64::consts::FRAC_PI_2),
    )
    .unwrap();
    let tolerance = default_distance_tolerance::<f64>();
    let classify = |x: f64, y: f64| ellipse.classify_point(Point2D::new(x, y));

    assert_eq!(classify(0.0, 0.0), PointClassification::Inside);
    assert_eq!(classify(0.0, 4.0), PointClassification::OnBoundary);
    assert_eq!(classify(-2.0, 0.0), PointClassification::OnBoundary);
    assert_eq!(classify(3.0, 0.0), PointClassification::Outside);

    // 許容誤差の境目
    assert_eq!(
        classify(0.0, 4.0 - 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(0.0, 4.0 + 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(0.0, 4.0 - 2.0 * tolerance),
        PointClassification::Inside
    );
    assert_eq!(
        classify(0.0, 4.0 + 2.0 * tolerance),
        PointClassification::Outside
    );

    // classify_point が OnBoundary のときだけ contains_point が真
    for (x, y) in [
        (0.0, 0.0),
        (0.0, 4.0),
        (3.0, 0.0),
        (0.0, 4.0 + 2.0 * tolerance),
    ] {
        let point = Point2D::new(x, y);
        assert_eq!(
            ellipse.contains_point(point),
            ellipse.classify_point(point) == PointClassification::OnBoundary
        );
    }
}
