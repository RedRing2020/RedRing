use crate::geometry3d::{Arc3D, Circle3D, Direction3D, Point3D};
use geo_core::Scalar;

#[test]
fn arc_length_half_circle() {
    let center = Point3D::new(0.0,0.0,0.0);
    let normal = Direction3D::unit_z();
    let circle = Circle3D::from_f64(center, 1.0, normal).unwrap();
    let arc = Arc3D::new(circle, Scalar::new(0.0), Scalar::new(std::f64::consts::PI));
    assert!((arc.length().value() - std::f64::consts::PI).abs() < 1e-12);
}

#[test]
fn arc_evaluate_endpoints() {
    let arc = Arc3D::from_f64(0.0,0.0,0.0, 1.0, 0.0,0.0,1.0, 0.0, std::f64::consts::PI).unwrap();
    let start = arc.start_point();
    let end = arc.end_point();
    assert!((start.x() - 1.0).abs() < 1e-12);
    assert!((end.x() + 1.0).abs() < 1e-12);
}
