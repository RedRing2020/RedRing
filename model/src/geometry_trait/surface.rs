use crate::geometry_kind::SurfaceKind;

pub trait Surface {
    type Point;
    type Vector;

    /// Evaluate the surface at (u, v)
    fn evaluate(&self, u: f64, v: f64) -> Self::Point;

    /// Partial derivative with respect to u
    fn derivative_u(&self, u: f64, v: f64) -> Self::Vector;

    /// Partial derivative with respect to v
    fn derivative_v(&self, u: f64, v: f64) -> Self::Vector;

    /// Return the valid parameter range in u direction
    fn parameter_range_u(&self) -> (f64, f64);

    /// Return the valid parameter range in v direction
    fn parameter_range_v(&self) -> (f64, f64);

    /// Whether the surface is closed in u direction
    fn is_closed_u(&self) -> bool;

    /// Whether the surface is closed in v direction
    fn is_closed_v(&self) -> bool;

    /// Return the surface kind identifier
    fn kind(&self) -> SurfaceKind;
}
