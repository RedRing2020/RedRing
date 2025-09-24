use crate::geometry2d::kind::CurveKind2;
use crate::geometry2d::{point::Point, vector::Vector};

/// Trait for all 2D curves in RedRing
pub trait Curve2 {
    /// Evaluate the curve at parameter t
    fn evaluate(&self, t: f64) -> Point;

    /// Compute the first derivative at parameter t
    fn derivative(&self, t: f64) -> Vector;

    /// Return the valid parameter range (start, end)
    fn parameter_range(&self) -> (f64, f64);

    /// Whether the curve is geometrically closed
    fn is_closed(&self) -> bool;

    /// Return the curve kind identifier
    fn kind(&self) -> CurveKind2;
}