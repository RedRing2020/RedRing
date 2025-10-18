//! 近似計算機能のテスト

use crate::approximations::*;
use analysis::abstract_types::Scalar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_perimeter_circle() {
        // 円の場合（a = b）の周長計算
        let radius = 5.0;
        let expected = 2.0 * std::f64::consts::PI * radius;
        
        let ramanujan_i = ellipse_perimeter_ramanujan_i(radius, radius);
        let ramanujan_ii = ellipse_perimeter_ramanujan_ii(radius, radius);
        let cantrell = ellipse_perimeter_cantrell(radius, radius);
        
        // 円の場合、すべての近似式が正確な値に近いはず
        assert!((ramanujan_i - expected).abs() < 0.001);
        assert!((ramanujan_ii - expected).abs() < 0.001);
        assert!((cantrell - expected).abs() < 0.001);
    }

    #[test]
    fn test_ellipse_properties() {
        let semi_major = 5.0;
        let semi_minor = 3.0;
        
        // 離心率計算
        let eccentricity = ellipse_eccentricity(semi_major, semi_minor);
        let expected_e = (1.0 - (semi_minor / semi_major).powi(2)).sqrt();
        assert!((eccentricity - expected_e).abs() < 1e-10);
        
        // 焦点距離計算  
        let focal_distance = ellipse_focal_distance(semi_major, semi_minor);
        let expected_c = (semi_major.powi(2) - semi_minor.powi(2)).sqrt();
        assert!((focal_distance - expected_c).abs() < 1e-10);
    }

    #[test]
    fn test_bezier_curve_length() {
        // 単純な線分のベジェ曲線
        let control_points = [[0.0, 0.0], [3.0, 4.0]];
        let length = bezier_length_approximation(&control_points, 10);
        let expected_length = 5.0; // 3-4-5三角形の斜辺
        
        assert!((length - expected_length).abs() < 0.01);
    }

    #[test]
    fn test_parametric_curve_length() {
        // 単位円の1/4の弧長
        let curve_fn = |t: f64| [t.cos(), t.sin()];
        let length = parametric_curve_length(curve_fn, 0.0, std::f64::consts::PI / 2.0, 100);
        let expected_length = std::f64::consts::PI / 2.0;
        
        assert!((length - expected_length).abs() < 0.01);
    }
}