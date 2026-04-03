use analysis::abstract_types::Scalar;

/// Unified contract for ellipse-oriented calculation capabilities.
pub trait EllipseCalculation<T: Scalar> {
    /// Point type used by each concrete shape implementation.
    type Point;

    fn semi_major_axis(&self) -> T;
    fn semi_minor_axis(&self) -> T;

    fn perimeter_ramanujan_i(&self) -> T;
    fn perimeter_ramanujan_ii(&self) -> T;
    fn perimeter_pade(&self) -> T;
    fn perimeter_cantrell(&self) -> T;
    fn perimeter_series(&self, terms: usize) -> T;
    fn perimeter_numerical(&self, n_points: usize) -> T;

    fn eccentricity(&self) -> T;
    fn focal_distance(&self) -> T;
    fn area(&self) -> T;
    fn foci(&self) -> (Self::Point, Self::Point);
}

/// Strategy contract for adaptive perimeter calculation.
pub trait EllipseAdaptiveCalculation<T: Scalar>: EllipseCalculation<T> {
    fn perimeter_adaptive(&self, target_accuracy: T, max_computation_cost: T) -> T;
}

/// Accuracy analysis contract for comparing approximation methods.
pub trait EllipseAccuracyAnalysis<T: Scalar>: EllipseCalculation<T> {
    fn compare_approximation_methods(&self) -> Vec<(&'static str, T, T)>;
}
