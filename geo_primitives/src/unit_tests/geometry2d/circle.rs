//! Circle2D unit tests (separated)

use crate::geometry2d::Circle2D;
use geo_core::{Point2D, tolerance::ToleranceContext};

#[test]
fn test_circle2d_basic() {
    let c = Circle2D::new(Point2D::from_f64(0.0,0.0), 2.0);
    assert!((c.length() - 2.0*std::f64::consts::PI*2.0).abs() < 1e-10);
    let p0 = c.evaluate(0.0); // (r,0)
    assert!((p0.x().value() - 2.0).abs() < 1e-10);
    assert!(p0.y().value().abs() < 1e-10);
    let p_half = c.evaluate(0.5); // (-r,0)
    assert!((p_half.x().value() + 2.0).abs() < 1e-10);
    assert!(p_half.y().value().abs() < 1e-10);
    let (dx,dy) = c.derivative(0.0);
    assert!(dx.abs() < 1e-10);
    assert!((dy - std::f64::consts::TAU * 2.0).abs() < 1e-10);
}

#[test]
fn test_circle2d_contains() {
    let ctx = ToleranceContext::standard();
    let c = Circle2D::new(Point2D::from_f64(1.0,1.0), 1.0);
    let on = Point2D::from_f64(2.0,1.0);
    assert!(c.contains_point(&on, &ctx));
    let out = Point2D::from_f64(3.1,1.0);
    assert!(!c.contains_point(&out, &ctx));
}
