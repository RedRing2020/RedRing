//! EllipseArc3D のテスト

use crate::{Direction3D, EllipseArc3D, Point3D};
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

    let tolerance = 1e-10_f64;
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
    assert!(mid_from_parameter.distance_to(mid_from_angle) < 1e-10);
    // パラメータ角 90° は短軸方向の端
    assert!(mid_from_angle.distance_to(Point3D::new(2.0, 5.0, 1.0)) < 1e-10);
}

#[test]
fn test_aabb_contains_arc_and_touches_extremes() {
    let ellipse_arc = create_test_ellipse_arc();
    let aabb = ellipse_arc.aabb().unwrap();

    // 45°〜135° は短軸方向の端（y 最大）を含む
    assert!((aabb.max().y() - 5.0).abs() < 1e-10);
    assert!((aabb.min().x() - ellipse_arc.end_point().x()).abs() < 1e-10);
    assert!((aabb.max().x() - ellipse_arc.start_point().x()).abs() < 1e-10);
    assert!((aabb.min().z() - 1.0).abs() < 1e-10 && (aabb.max().z() - 1.0).abs() < 1e-10);
    for i in 0..=100 {
        let p = ellipse_arc.point_at_parameter(f64::from(i) / 100.0);
        assert!(p.y() <= aabb.max().y() + 1e-12 && p.y() >= aabb.min().y() - 1e-12);
        assert!(p.x() <= aabb.max().x() + 1e-12 && p.x() >= aabb.min().x() - 1e-12);
    }
}
