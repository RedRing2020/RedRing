use analysis::abstract_types::Scalar;
use geo_contracts::EllipseCalculation;

const ANALYSIS_NUMERICAL_REFERENCE_POINTS: usize = 1000;
const ANALYSIS_SERIES_TERMS: usize = 20;

pub(crate) fn compare_approximation_methods<T, E>(ellipse: &E) -> Vec<(&'static str, T, T)>
where
    T: Scalar,
    E: EllipseCalculation<T> + ?Sized,
{
    let numerical_ref = ellipse.perimeter_numerical(ANALYSIS_NUMERICAL_REFERENCE_POINTS);
    let ramanujan_i = ellipse.perimeter_ramanujan_i();
    let ramanujan_ii = ellipse.perimeter_ramanujan_ii();
    let pade = ellipse.perimeter_pade();
    let cantrell = ellipse.perimeter_cantrell();
    let series = ellipse.perimeter_series(ANALYSIS_SERIES_TERMS);

    vec![
        (
            "Ramanujan I",
            ramanujan_i,
            ((ramanujan_i - numerical_ref) / numerical_ref).abs(),
        ),
        (
            "Ramanujan II",
            ramanujan_ii,
            ((ramanujan_ii - numerical_ref) / numerical_ref).abs(),
        ),
        ("Pade", pade, ((pade - numerical_ref) / numerical_ref).abs()),
        (
            "Cantrell",
            cantrell,
            ((cantrell - numerical_ref) / numerical_ref).abs(),
        ),
        (
            "Series(20)",
            series,
            ((series - numerical_ref) / numerical_ref).abs(),
        ),
    ]
}
