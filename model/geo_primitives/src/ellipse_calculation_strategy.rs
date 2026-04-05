use analysis::abstract_types::Scalar;
use geo_contracts::EllipseCalculation;

const ADAPTIVE_CIRCUMFERENCE_LOOSE_ACCURACY_THRESHOLD: f64 = 1e-3;
const ADAPTIVE_CIRCUMFERENCE_MEDIUM_ACCURACY_THRESHOLD: f64 = 1e-6;
const ADAPTIVE_CIRCUMFERENCE_HIGH_ACCURACY_THRESHOLD: f64 = 1e-9;
const ADAPTIVE_CIRCUMFERENCE_SERIES_COST_THRESHOLD: f64 = 2.0;
const ADAPTIVE_CIRCUMFERENCE_SERIES_TERMS: usize = 20;

pub(crate) fn circumference_adaptive<T, E>(
    ellipse: &E,
    target_accuracy: T,
    max_computation_cost: T,
) -> T
where
    T: Scalar,
    E: EllipseCalculation<T> + ?Sized,
{
    if target_accuracy >= T::from_f64(ADAPTIVE_CIRCUMFERENCE_LOOSE_ACCURACY_THRESHOLD) {
        ellipse.circumference_pade()
    } else if target_accuracy >= T::from_f64(ADAPTIVE_CIRCUMFERENCE_MEDIUM_ACCURACY_THRESHOLD) {
        ellipse.circumference_ramanujan_i()
    } else if target_accuracy >= T::from_f64(ADAPTIVE_CIRCUMFERENCE_HIGH_ACCURACY_THRESHOLD) {
        ellipse.circumference_ramanujan_ii()
    } else if max_computation_cost >= T::from_f64(ADAPTIVE_CIRCUMFERENCE_SERIES_COST_THRESHOLD) {
        ellipse.circumference_series(ADAPTIVE_CIRCUMFERENCE_SERIES_TERMS)
    } else {
        ellipse.circumference_cantrell()
    }
}
