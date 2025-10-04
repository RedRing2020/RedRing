use crate::geometry3d::{Circle3D, Direction3D, Point3D};
use geo_core::Scalar;

#[test]
fn circle_evaluate_quadrants() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Direction3D::unit_z();
    let circle = Circle3D::from_f64(center, 2.0, normal).unwrap();
    let p0 = circle.evaluate(0.0);
    assert!((p0.x() - 2.0).abs() < 1e-12);
    let p90 = circle.evaluate(std::f64::consts::FRAC_PI_2);
    assert!((p90.y() - 2.0).abs() < 1e-12);
}

#[test]
fn circle_contains_point() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Direction3D::unit_z();
    let circle = Circle3D::from_f64(center, 1.0, normal).unwrap();
    let inside = Point3D::new(1.0, 0.0, 0.0);
    let outside = Point3D::new(2.0, 0.0, 0.0);
    let tol = geo_core::ToleranceContext::standard();
    assert!(circle.contains_point(&inside, &tol));
    assert!(!circle.contains_point(&outside, &tol));
}

#[test]
fn circle_rotate_around_axis() {
    let center = Point3D::new(1.0, 0.0, 0.0);
    let normal = Direction3D::unit_z();
    let circle = Circle3D::from_f64(center, 1.0, normal).unwrap();
    let axis = Direction3D::unit_z();
    let rotated = circle.rotate_around_axis(&axis, std::f64::consts::FRAC_PI_2, &Point3D::new(0.0,0.0,0.0));
    // 中心 (1,0,0) -> 回転後 (0,1,0)
    assert!((rotated.center().x() - 0.0).abs() < 1e-12);
    assert!((rotated.center().y() - 1.0).abs() < 1e-12);
}
