use super::partition::find_span_in_non_decreasing_sequence;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_interior_span_in_non_decreasing_sequence() {
        let sequence = [0.0_f64, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0];

        assert_eq!(
            find_span_in_non_decreasing_sequence(0.25, &sequence, 2, 4),
            2
        );
        assert_eq!(
            find_span_in_non_decreasing_sequence(0.5, &sequence, 2, 4),
            3
        );
    }

    #[test]
    fn clamps_to_search_bounds_for_boundary_values() {
        let sequence = [0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];

        assert_eq!(
            find_span_in_non_decreasing_sequence(0.0, &sequence, 2, 3),
            2
        );
        assert_eq!(
            find_span_in_non_decreasing_sequence(1.0, &sequence, 2, 3),
            2
        );
    }

    #[test]
    fn handles_repeated_values_inside_sequence() {
        let sequence = [0.0_f64, 0.0, 0.25, 0.25, 0.5, 1.0];

        assert_eq!(
            find_span_in_non_decreasing_sequence(0.25, &sequence, 1, 5),
            3
        );
    }
}
