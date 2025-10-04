use crate::geometry3d::direction::Direction3D;
use geo_core::{ToleranceContext, TolerantEq};

#[test]
fn rotate_parallel_axis_no_change() {
    let d = Direction3D::unit_x();
    let axis = Direction3D::unit_x();
    let rotated = d.rotate_around_axis(&axis, std::f64::consts::FRAC_PI_2);
    assert!((rotated.x() - d.x()).abs() < 1e-12);
    assert!((rotated.y() - d.y()).abs() < 1e-12);
    assert!((rotated.z() - d.z()).abs() < 1e-12);
}

#[test]
fn rotate_z_axis_90deg() {
    let d = Direction3D::unit_x();
    let axis = Direction3D::unit_z();
    let rotated = d.rotate_around_axis(&axis, std::f64::consts::FRAC_PI_2);
    assert!(rotated.x().abs() < 1e-12);
    assert!((rotated.y() - 1.0).abs() < 1e-12);
    assert!(rotated.z().abs() < 1e-12);
}

#[test]
fn rotate_full_circle_identity() {
    let d = Direction3D::from_f64(0.3, -0.7, 0.64).unwrap();
    let axis = Direction3D::unit_z();
    let rotated = d.rotate_around_axis(&axis, 2.0 * std::f64::consts::PI);
    assert!((rotated.x() - d.x()).abs() < 1e-12);
    assert!((rotated.y() - d.y()).abs() < 1e-12);
    assert!((rotated.z() - d.z()).abs() < 1e-12);
}

#[test]
fn tolerant_eq_direction3d() {
    let ctx = ToleranceContext::standard();
    let a = Direction3D::unit_x();
    let b = Direction3D::from_f64(1.0 + 1e-9, 1e-9, 0.0).unwrap();
    assert!(a.tolerant_eq(&b, &ctx));
}
