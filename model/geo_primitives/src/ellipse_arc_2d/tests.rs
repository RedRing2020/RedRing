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
    // 長さは掃引角に比例する（1 周の 1/3）
    let quarter = arc(0.0, 90.0);
    assert!((crossing.length() / quarter.length() - 120.0 / 90.0).abs() < 1e-10);
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
