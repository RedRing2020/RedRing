use super::point::Point;
use super::vector::Vector;

/// Trait for evaluating and analyzing 3D curves.
/// Implemented by analytic and freeform curve types (Line, Arc, NurbsCurve, etc.)
pub trait Curve3: Send + Sync {
    fn evaluate(&self, t: f64) -> Point;
    fn derivative(&self, t: f64) -> Vector;
    fn second_derivative(&self, t: f64) -> Vector;
    fn length_between(&self, t0: f64, t1: f64) -> f64;
    fn parameter_range(&self) -> (f64, f64);
    fn is_closed(&self, tolerance: f64) -> bool;
    fn is_rational(&self) -> bool;
    fn is_periodic(&self) -> bool;
}
