//! EllipseArc2D のテスト

use super::*;

fn create_test_ellipse_arc() -> EllipseArc2D<f64> {
    let center = Point2D::new(2.0, 3.0);
    let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).unwrap();

    EllipseArc2D::new(ellipse, Angle::from_degrees(0.0), Angle::from_degrees(90.0))
}

#[test]
fn test_parameter_and_angle_apis_are_distinct_but_consistent() {
    let ellipse_arc = create_test_ellipse_arc();

    let point_from_parameter = ellipse_arc.point_at_parameter(0.5);
    let point_from_angle = ellipse_arc.point_at_angle(std::f64::consts::PI / 4.0);

    assert!((point_from_parameter.x() - point_from_angle.x()).abs() < 1e-10);
    assert!((point_from_parameter.y() - point_from_angle.y()).abs() < 1e-10);
}
