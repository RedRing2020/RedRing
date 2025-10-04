use crate::geometry3d::{InfiniteLine3D, Direction3D};
use geo_core::{Point3D, Scalar, ToleranceContext};

#[test]
fn infinite_line_parameter_and_contains() {
    let p1 = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
    let p2 = Point3D::new(Scalar::new(2.0), Scalar::new(0.0), Scalar::new(0.0));
    let line = InfiniteLine3D::from_points(&p1,&p2).unwrap();
    let p_mid = Point3D::new(Scalar::new(1.0), Scalar::new(0.0), Scalar::new(0.0));
    let tol = ToleranceContext::standard();
    assert!(line.contains_point(&p_mid, &tol));
    let t = line.parameter_of_point(&p_mid, &tol).unwrap();
    assert!((t.value() - 1.0).abs() < 1e-12);
}
