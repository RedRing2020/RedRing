use analysis::abstract_types::Scalar;

/// Unified contract for ellipse-oriented calculation capabilities.
pub trait EllipseCalculation<T: Scalar> {
    /// Point type used by each concrete shape implementation.
    type Point;

    fn semi_major_axis(&self) -> T;
    fn semi_minor_axis(&self) -> T;

    fn circumference_ramanujan_i(&self) -> T;
    fn circumference_ramanujan_ii(&self) -> T;
    fn circumference_pade(&self) -> T;
    fn circumference_cantrell(&self) -> T;
    fn circumference_series(&self, terms: usize) -> T;
    fn circumference_numerical(&self, n_points: usize) -> T;

    fn eccentricity(&self) -> T;
    fn focal_distance(&self) -> T;
    fn area(&self) -> T;
    fn foci(&self) -> (Self::Point, Self::Point);
}
