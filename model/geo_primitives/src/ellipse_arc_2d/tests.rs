//! EllipseArc2D のテスト

use crate::{EllipseArc2D, Point2D};
use geo_contracts::{
    Angle, EllipseArc2DConstructor, EllipseArc2DContainment, EllipseArc2DDerived,
    EllipseArc2DDistance, EllipseArc2DEndpoint, EllipseArc2DEvaluation, EllipseArc2DProperties,
    EllipseArc2DTrimRange,
};

fn arc(start_degrees: f64, end_degrees: f64) -> EllipseArc2D<f64> {
    // 中心 (2, 3)・長半軸 4・短半軸 2 の回転なしの楕円
    EllipseArc2D::new(
        Point2D::new(2.0, 3.0),
        4.0,
        2.0,
        Angle::from_radians(0.0),
        Angle::from_degrees(start_degrees),
        Angle::from_degrees(end_degrees),
    )
    .unwrap()
}

#[test]
fn test_parameter_and_angle_apis_are_distinct_but_consistent() {
    let ellipse_arc = arc(0.0, 90.0);

    let point_from_parameter = ellipse_arc.point_at_parameter(0.5);
    let point_from_angle = ellipse_arc.point_at_angle(Angle::from_degrees(45.0));
    assert!(point_from_parameter.distance_to(point_from_angle) < 1e-10);

    assert!(
        ellipse_arc
            .start_point()
            .distance_to(Point2D::new(6.0, 3.0))
            < 1e-10
    );
    assert!(ellipse_arc.end_point().distance_to(Point2D::new(2.0, 5.0)) < 1e-10);
}

#[test]
fn test_range_crossing_zero_goes_counterclockwise() {
    // 300° から反時計回りに 60° まで（掃引角 120°）
    let crossing = arc(300.0, 60.0);
    assert!((crossing.angle_span().to_degrees() - 120.0).abs() < 1e-10);
    assert!(crossing.contains_angle(Angle::from_degrees(0.0)));
    assert!(!crossing.contains_angle(Angle::from_degrees(180.0)));
    // t=0.5 はパラメータ角 0 の点
    assert!(
        crossing
            .point_at_parameter(0.5)
            .distance_to(Point2D::new(6.0, 3.0))
            < 1e-10
    );
    // 長軸について対称な範囲なので、0° から 60° までの長さの 2 倍
    let upper = arc(0.0, 60.0);
    assert!((crossing.length() - 2.0 * upper.length()).abs() < 1e-9);
}

#[test]
fn test_distance_and_containment() {
    let ellipse_arc = arc(0.0, 90.0);

    // 角度範囲内の楕円上の点
    assert!(ellipse_arc.contains_point(Point2D::new(6.0, 3.0)));
    // 角度範囲外の楕円上の点は、近い端点までの距離
    let outside = Point2D::new(-2.0, 3.0);
    assert!(!ellipse_arc.contains_point(outside));
    assert!((ellipse_arc.distance_to_point(outside) - 20.0_f64.sqrt()).abs() < 1e-10);
}

/// 楕円全体の最近点が範囲外でも、範囲内に距離の極小点があればその点までの距離になる
#[test]
fn test_distance_uses_local_minimum_inside_the_range() {
    // 中心 (2, 3)・長半軸 4・短半軸 2 の楕円の下半分。点 (2, 4) の楕円全体の最近点は上側の短軸の端
    let lower_half = arc(180.0, 360.0);
    let point = Point2D::new(2.0, 4.0);
    // 下側の短軸の端 (2, 1) が範囲内の最近点
    assert!((lower_half.distance_to_point(point) - 3.0).abs() < 1e-12);
}

/// 楕円（長半軸 a・短半軸 b）のパラメータ start から end までの弧長（細かく分割したシンプソン則）
fn reference_length(a: f64, b: f64, start_degrees: f64, end_degrees: f64) -> f64 {
    let (start, end) = (start_degrees.to_radians(), end_degrees.to_radians());
    let speed = |t: f64| (a * a * t.sin().powi(2) + b * b * t.cos().powi(2)).sqrt();
    let n = 200_000;
    let h = (end - start) / f64::from(n);
    let sum: f64 = (0..=n)
        .map(|i| {
            let weight = if i == 0 || i == n {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            weight * speed(start + h * f64::from(i))
        })
        .sum();
    sum * h / 3.0
}

/// 一般の範囲の楕円弧の長さが、弧長の積分と一致する
#[test]
fn test_length_matches_arc_length_integral() {
    use geo_contracts::default_distance_tolerance;
    let tolerance = default_distance_tolerance::<f64>();
    // 長半軸 4・短半軸 1 の楕円の 0°〜45°（速さが範囲で大きく変わる）
    let narrow = EllipseArc2D::new(
        Point2D::new(0.0, 0.0),
        4.0,
        1.0,
        Angle::from_radians(0.0),
        Angle::from_degrees(0.0),
        Angle::from_degrees(45.0),
    )
    .unwrap();
    assert!((narrow.length() - reference_length(4.0, 1.0, 0.0, 45.0)).abs() <= tolerance);

    // 中心 (2, 3)・長半軸 4・短半軸 2 の 300°〜60° と 10°〜200°
    for (start, end, reference_end) in [(300.0, 60.0, 420.0), (10.0, 200.0, 200.0)] {
        let length = arc(start, end).length();
        assert!((length - reference_length(4.0, 2.0, start, reference_end)).abs() <= tolerance);
    }
}
