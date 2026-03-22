use analysis::abstract_types::Scalar;

const ADAPTIVE_PERIMETER_LOOSE_ACCURACY_THRESHOLD: f64 = 1e-3;
const ADAPTIVE_PERIMETER_MEDIUM_ACCURACY_THRESHOLD: f64 = 1e-6;
const ADAPTIVE_PERIMETER_HIGH_ACCURACY_THRESHOLD: f64 = 1e-9;
const ADAPTIVE_PERIMETER_SERIES_COST_THRESHOLD: f64 = 2.0;

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
    fn perimeter_adaptive(&self, target_accuracy: T, max_computation_cost: T) -> T {
        if target_accuracy >= T::from_f64(ADAPTIVE_PERIMETER_LOOSE_ACCURACY_THRESHOLD) {
            self.perimeter_pade()
        } else if target_accuracy >= T::from_f64(ADAPTIVE_PERIMETER_MEDIUM_ACCURACY_THRESHOLD) {
            self.perimeter_ramanujan_i()
        } else if target_accuracy >= T::from_f64(ADAPTIVE_PERIMETER_HIGH_ACCURACY_THRESHOLD) {
            self.perimeter_ramanujan_ii()
        } else if max_computation_cost >= T::from_f64(ADAPTIVE_PERIMETER_SERIES_COST_THRESHOLD) {
            self.perimeter_series(20)
        } else {
            self.perimeter_cantrell()
        }
    }
}

/// Accuracy analysis contract for comparing approximation methods.
pub trait EllipseAccuracyAnalysis<T: Scalar>: EllipseCalculation<T> {
    fn compare_approximation_methods(&self) -> Vec<(&'static str, T, T)> {
        let numerical_ref = self.perimeter_numerical(1000);

        vec![
            (
                "Ramanujan I",
                self.perimeter_ramanujan_i(),
                ((self.perimeter_ramanujan_i() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Ramanujan II",
                self.perimeter_ramanujan_ii(),
                ((self.perimeter_ramanujan_ii() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Pade",
                self.perimeter_pade(),
                ((self.perimeter_pade() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Cantrell",
                self.perimeter_cantrell(),
                ((self.perimeter_cantrell() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Series(20)",
                self.perimeter_series(20),
                ((self.perimeter_series(20) - numerical_ref) / numerical_ref).abs(),
            ),
        ]
    }
}
