//! EllipseArc3D のテスト

use crate::{Direction3D, EllipseArc3D, Point3D};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{
    Angle, Bounded, EllipseArc3DConstructor, EllipseArc3DEndpoint, EllipseArc3DEvaluation,
    EllipseArc3DProperties,
};

fn create_test_ellipse_arc() -> EllipseArc3D<f64> {
    // 中心 (2, 3, 1)・XY 平面・長軸 X 方向・長半軸 4・短半軸 2、45°〜135°
    EllipseArc3D::new(
        Point3D::new(2.0, 3.0, 1.0),
        4.0,
        2.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
        Angle::from_degrees(45.0),
        Angle::from_degrees(135.0),
    )
    .unwrap()
}

#[test]
fn test_ellipse_arc_creation() {
    let ellipse_arc = create_test_ellipse_arc();

    let tolerance = TOLERANCE_F64;
    assert!(
        ellipse_arc
            .center()
            .distance_to(Point3D::new(2.0, 3.0, 1.0))
            < tolerance
    );
    assert!((ellipse_arc.semi_major_axis() - 4.0_f64).abs() < tolerance);
    assert!((ellipse_arc.semi_minor_axis() - 2.0_f64).abs() < tolerance);
    assert!((ellipse_arc.start_angle().to_degrees() - 45.0).abs() < tolerance);
    assert!((ellipse_arc.end_angle().to_degrees() - 135.0).abs() < tolerance);
    assert!((ellipse_arc.angle_span().to_degrees() - 90.0).abs() < tolerance);
}

#[test]
fn test_parameter_and_angle_apis_are_distinct_but_consistent() {
    let ellipse_arc = create_test_ellipse_arc();

    let mid_from_parameter = ellipse_arc.point_at_parameter(0.5);
    let mid_from_angle = ellipse_arc.point_at_angle(Angle::from_degrees(90.0));
    assert!(mid_from_parameter.distance_to(mid_from_angle) < TOLERANCE_F64);
    // パラメータ角 90° は短軸方向の端
    assert!(mid_from_angle.distance_to(Point3D::new(2.0, 5.0, 1.0)) < TOLERANCE_F64);
}

#[test]
fn test_aabb_contains_arc_and_touches_extremes() {
    let ellipse_arc = create_test_ellipse_arc();
    let aabb = ellipse_arc.aabb().unwrap();

    // 45°〜135° は短軸方向の端（y 最大）を含む
    assert!((aabb.max().y() - 5.0).abs() < TOLERANCE_F64);
    assert!((aabb.min().x() - ellipse_arc.end_point().x()).abs() < TOLERANCE_F64);
    assert!((aabb.max().x() - ellipse_arc.start_point().x()).abs() < TOLERANCE_F64);
    assert!(
        (aabb.min().z() - 1.0).abs() < TOLERANCE_F64
            && (aabb.max().z() - 1.0).abs() < TOLERANCE_F64
    );
    for i in 0..=100 {
        let p = ellipse_arc.point_at_parameter(f64::from(i) / 100.0);
        assert!(p.y() <= aabb.max().y() + TOLERANCE_F64 && p.y() >= aabb.min().y() - TOLERANCE_F64);
        assert!(p.x() <= aabb.max().x() + TOLERANCE_F64 && p.x() >= aabb.min().x() - TOLERANCE_F64);
    }
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
    use geo_contracts::{default_distance_tolerance, EllipseArc3DDerived};
    let tolerance = default_distance_tolerance::<f64>();
    // 長半軸 4・短半軸 2 の 45°〜135°
    let arc = create_test_ellipse_arc();
    assert!((arc.length() - reference_length(4.0, 2.0, 45.0, 135.0)).abs() <= tolerance);

    // 長半軸 4・短半軸 1 の 0°〜45°
    let narrow = EllipseArc3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        4.0,
        1.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
        Angle::from_degrees(0.0),
        Angle::from_degrees(45.0),
    )
    .unwrap();
    assert!((narrow.length() - reference_length(4.0, 1.0, 0.0, 45.0)).abs() <= tolerance);
}
