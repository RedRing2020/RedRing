use super::range::is_within_closed_range;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_bounds_and_tolerance_band() {
        assert!(is_within_closed_range(5.0_f64, 0.0, 10.0, 0.0));
        assert!(is_within_closed_range(0.0_f64, 0.0, 10.0, 0.0));
        assert!(is_within_closed_range(10.0_f64, 0.0, 10.0, 0.0));
        assert!(is_within_closed_range(10.05_f64, 0.0, 10.0, 0.1));
        assert!(is_within_closed_range(-0.05_f64, 0.0, 10.0, -0.1));
        assert!(!is_within_closed_range(10.2_f64, 0.0, 10.0, 0.1));
        assert!(!is_within_closed_range(-0.2_f64, 0.0, 10.0, 0.1));
    }

    #[test]
    fn rejects_non_finite_values_and_inverted_range() {
        assert!(!is_within_closed_range(f64::NAN, 0.0, 10.0, 0.1));
        assert!(!is_within_closed_range(
            5.0_f64,
            f64::NEG_INFINITY,
            10.0,
            0.1
        ));
        assert!(!is_within_closed_range(5.0_f64, 0.0, f64::INFINITY, 0.1));
        assert!(!is_within_closed_range(5.0_f64, 10.0, 0.0, 0.1));
    }
}
