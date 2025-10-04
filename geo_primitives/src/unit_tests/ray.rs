use crate::geometry3d::{Ray3D, Direction3D};
use geo_core::{Point3D, Scalar};

#[test]
fn ray_evaluate_positive() {
    let origin = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
    let dir = Direction3D::unit_x();
    let ray = Ray3D::new(origin, dir);
    let p = ray.evaluate(Scalar::new(2.0)).unwrap();
    assert!((p.x().value() - 2.0).abs() < 1e-12);
}

#[test]
fn ray_evaluate_negative_none() {
    let origin = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
    let dir = Direction3D::unit_x();
    let ray = Ray3D::new(origin, dir);
    assert!(ray.evaluate(Scalar::new(-1.0)).is_none());
}
