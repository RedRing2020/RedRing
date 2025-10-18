//! 曲線近似計算のテスト

use crate::approximations::*;

#[cfg(test)]
mod curve_tests {
    use super::*;

    #[test]
    fn test_bezier_curve_length() {
        // 単純な線分のベジェ曲線
        let control_points = [[0.0, 0.0], [3.0, 4.0]];
        let length = bezier_length_approximation(&control_points, 10);
        let expected_length = 5.0_f64; // 3-4-5三角形の斜辺

        assert!((length - expected_length).abs() < 0.01_f64);
    }

    #[test]
    fn test_parametric_curve_length() {
        // 単位円の1/4の弧長
        let curve_fn = |t: f64| [t.cos(), t.sin()];
        let length = parametric_curve_length(curve_fn, 0.0, std::f64::consts::PI / 2.0, 100);
        let expected_length = std::f64::consts::PI / 2.0;

        assert!((length - expected_length).abs() < 0.01_f64);
    }
}
