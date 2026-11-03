//! 数値の範囲判定

use crate::Scalar;

/// 値が閉区間 `[min, max]` に許容誤差付きで含まれるかを判定する。
///
/// `min - tolerance <= value <= max + tolerance` なら `true` を返す。許容誤差は絶対値として扱う。
/// 値・下限・上限のいずれかが有限でない場合と、`min > max` の場合は `false` を返す。
pub fn is_within_closed_range<T: Scalar>(value: T, min: T, max: T, tolerance: T) -> bool {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || min > max {
        return false;
    }
    let tolerance = tolerance.abs();
    value >= min - tolerance && value <= max + tolerance
}

#[cfg(test)]
mod tests {
    use super::is_within_closed_range;

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
