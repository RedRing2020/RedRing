use crate::geometry_kind::CurveKind2D;
use std::any::Any;

/// NOTE: Removed dependency on old model::geometry, using associated types for Point/Vector.
/// Curve2D: Abstract trait for 2D curves
///
/// Common interface for curve types (Line, Arc, Ellipse, etc.).
/// Provides classification via CurveKind2D and basic operations like evaluation, derivative, length.
pub trait Curve2D: Any {
    type Point;
    type Vector;
    /// Downcast for type identification
    fn as_any(&self) -> &dyn Any;

    /// Returns curve classification
    fn kind(&self) -> CurveKind2D;

    /// Evaluate point at parameter t
    fn evaluate(&self, t: f64) -> Self::Point;

    /// Calculate first derivative vector at parameter t
    fn derivative(&self, t: f64) -> Self::Vector;

    /// Calculate curve length
    fn length(&self) -> f64;

    /// Returns parameter domain [t_min, t_max]
    fn domain(&self) -> (f64, f64);

    /// Check if parameter is within valid domain
    fn is_valid_parameter(&self, t: f64) -> bool {
        let (t_min, t_max) = self.domain();
        t >= t_min && t <= t_max
    }

    /// Split curve at specified parameter (default implementation)
    #[allow(clippy::type_complexity)]
    fn split(
        &self,
        _t: f64,
    ) -> Option<(
        Box<dyn Curve2D<Point = Self::Point, Vector = Self::Vector>>,
        Box<dyn Curve2D<Point = Self::Point, Vector = Self::Vector>>,
    )> {
        None
    }

    /// Reverse curve direction (default implementation)
    #[allow(clippy::type_complexity)]
    fn reverse(&self) -> Option<Box<dyn Curve2D<Point = Self::Point, Vector = Self::Vector>>> {
        None
    }
}
