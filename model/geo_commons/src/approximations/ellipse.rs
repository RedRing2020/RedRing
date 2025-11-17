//! 楕円の近似計算・数値計算モジュール
//!
//! 楕円の周囲長、面積、焦点などの計算を提供します。
//! 複数の近似手法と数値積分による高精度計算をサポート。
//!
//! ## 主要機能
//! - **周囲長近似**: ラマヌジャン近似I/II、パダン近似、カントレル近似
//! - **数値計算**: 級数展開、数値積分による高精度計算
//! - **基本要素**: 離心率、焦点距離、面積、焦点座標
//!
//! ## 使用例
//! ```rust
//! use geo_commons::approximations::ellipse::*;
//!
//! let a = 5.0f64; // 長半径
//! let b = 3.0f64; // 短半径
//!
//! let perimeter = ellipse_perimeter_ramanujan_ii(a, b);
//! let area = geo_commons::ellipse_area(a, b);
//! let eccentricity = ellipse_eccentricity(a, b);
//! ```
//!
//! ---
//! © RedRing Project

use analysis::abstract_types::Scalar;

/// ラマヌジャン近似I による楕円周囲長計算
///
/// 精度: 中程度（通常の用途に適用）
/// 計算コスト: 低
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
///
/// # Returns
/// 楕円の周囲長の近似値
pub fn ellipse_perimeter_ramanujan_i<T: Scalar>(a: T, b: T) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    let pi = T::PI;
    (a + b)
        * pi
        * (T::ONE
            + (T::from_f64(3.0) * h)
                / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
}

/// ラマヌジャン近似II による楕円周囲長計算
///
/// 精度: 高（ほぼ全ての実用目的に適用）
/// 計算コスト: 中
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
///
/// # Returns
/// 楕円の周囲長の高精度近似値
pub fn ellipse_perimeter_ramanujan_ii<T: Scalar>(a: T, b: T) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    let pi = T::PI;
    let numerator = T::from_f64(3.0) * h;
    let denominator = T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt();

    (a + b) * pi * (T::ONE + numerator / denominator)
}

/// パダン近似による楕円周囲長計算
///
/// 精度: 非常に高（科学技術計算向け）
/// 計算コスト: 高
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
///
/// # Returns
/// 楕円の周囲長の高精度近似値
pub fn ellipse_perimeter_padé<T: Scalar>(a: T, b: T) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    let pi = T::PI;

    let term1 = T::from_f64(64.0) - T::from_f64(3.0) * h.powi(2);
    let term2 = T::from_f64(256.0) - T::from_f64(48.0) * h - T::from_f64(21.0) * h.powi(2);

    (a + b) * pi * (T::ONE + h * term1 / term2)
}

/// カントレル近似による楕円周囲長計算
///
/// 精度: 高（工学計算向け）
/// 計算コスト: 中
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
///
/// # Returns
/// 楕円の周囲長の近似値
pub fn ellipse_perimeter_cantrell<T: Scalar>(a: T, b: T) -> T {
    let pi = T::PI;
    let a_plus_b = a + b;
    let _a_minus_b = a - b;
    let sqrt_ab = (a * b).sqrt();

    pi * (T::from_f64(1.5) * a_plus_b - sqrt_ab)
}

/// 級数展開による楕円周囲長計算
///
/// 精度: 可変（項数に依存）
/// 計算コスト: 高（項数に比例）
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
/// * `terms` - 級数の項数（推奨: 10-50）
///
/// # Returns
/// 楕円の周囲長の級数近似値
pub fn ellipse_circumference_series<T: Scalar>(a: T, b: T, terms: usize) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    let pi = T::PI;

    let mut sum = T::ONE;
    let mut coefficient = T::ONE;
    let mut h_power = h;

    for n in 1..=terms {
        let n_t = T::from_usize(n);
        coefficient = coefficient * (T::from_f64(2.0) * n_t - T::ONE) / (T::from_f64(2.0) * n_t);
        sum += coefficient.powi(2) * h_power;
        h_power *= h;
    }

    pi * (a + b) * sum
}

/// 数値積分による楕円周囲長計算（シンプソン法）
///
/// 精度: 非常に高（数値誤差のみ）
/// 計算コスト: 非常に高
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
/// * `n_intervals` - 積分区間数（推奨: 1000以上）
///
/// # Returns
/// 楕円の周囲長の数値積分値
pub fn ellipse_circumference_numerical<T: Scalar>(a: T, b: T, n_intervals: usize) -> T {
    let pi_2 = T::PI / T::from_f64(2.0);
    let h = pi_2 / T::from_usize(n_intervals);

    let integrand = |t: T| -> T {
        let sin_t = t.sin();
        let cos_t = t.cos();
        (a.powi(2) * sin_t.powi(2) + b.powi(2) * cos_t.powi(2)).sqrt()
    };

    let mut sum = integrand(T::ZERO) + integrand(pi_2);

    for i in 1..n_intervals {
        let t = T::from_usize(i) * h;
        if i % 2 == 0 {
            sum += T::from_f64(2.0) * integrand(t);
        } else {
            sum += T::from_f64(4.0) * integrand(t);
        }
    }

    T::from_f64(4.0) * h * sum / T::from_f64(3.0)
}

/// 楕円の離心率を計算
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
///
/// # Returns
/// 離心率 e（0 ≤ e < 1）
pub fn ellipse_eccentricity<T: Scalar>(a: T, b: T) -> T {
    (T::ONE - (b / a).powi(2)).sqrt()
}

/// 楕円の焦点距離を計算
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
///
/// # Returns
/// 中心から焦点までの距離
pub fn ellipse_focal_distance<T: Scalar>(a: T, b: T) -> T {
    (a.powi(2) - b.powi(2)).sqrt()
}

/// 楕円の焦点座標を計算（中心が原点の場合）
///
/// # Arguments
/// * `a` - 長半径（a ≥ b > 0）
/// * `b` - 短半径（b > 0）
///
/// # Returns
/// 焦点座標のタプル ((x1, y1), (x2, y2))
pub fn ellipse_foci<T: Scalar>(a: T, b: T) -> ((T, T), (T, T)) {
    let c = ellipse_focal_distance(a, b);
    ((-c, T::ZERO), (c, T::ZERO))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_ellipse_perimeter_ramanujan_i() {
        let a = 5.0;
        let b = 3.0;
        let perimeter = ellipse_perimeter_ramanujan_i(a, b);

        // 期待値は約25.53（参考値）
        assert!(perimeter > 25.0 && perimeter < 26.0);
        assert_abs_diff_eq!(perimeter, 25.526999519494658, epsilon = 0.01);
    }

    #[test]
    fn test_ellipse_perimeter_ramanujan_ii() {
        let a = 5.0;
        let b = 3.0;
        let perimeter = ellipse_perimeter_ramanujan_ii(a, b);

        // ラマヌジャンII近似は高精度
        assert!(perimeter > 25.5 && perimeter < 25.6);
        assert_abs_diff_eq!(perimeter, 25.526999519494658, epsilon = 0.001);
    }

    #[test]
    fn test_ellipse_perimeter_pade() {
        let a = 5.0;
        let b = 3.0;
        let perimeter = ellipse_perimeter_padé(a, b);

        // パダン近似は非常に高精度
        assert!(perimeter > 25.52 && perimeter < 25.54);
    }

    #[test]
    fn test_ellipse_perimeter_cantrell() {
        let a = 5.0;
        let b = 3.0;
        let perimeter = ellipse_perimeter_cantrell(a, b);

        // カントレル近似の妥当性確認
        assert!(perimeter > 24.0 && perimeter < 27.0);
    }

    #[test]
    fn test_ellipse_circumference_series() {
        let a = 5.0;
        let b = 3.0;
        let perimeter = ellipse_circumference_series(a, b, 20);

        // 級数展開は項数が多いほど高精度
        assert!(perimeter > 25.0 && perimeter < 26.0);
        assert_abs_diff_eq!(perimeter, 25.526999519494658, epsilon = 0.1);
    }

    #[test]
    fn test_ellipse_circumference_numerical() {
        let a = 5.0;
        let b = 3.0;
        let perimeter = ellipse_circumference_numerical(a, b, 1000);

        // 数値積分は最も高精度
        assert_abs_diff_eq!(perimeter, 25.526999519494658, epsilon = 0.01);
    }

    #[test]
    fn test_ellipse_eccentricity() {
        let a = 5.0;
        let b = 3.0;
        let eccentricity = ellipse_eccentricity(a, b);

        // e = sqrt(1 - (b/a)^2) = sqrt(1 - 9/25) = sqrt(16/25) = 0.8
        assert_abs_diff_eq!(eccentricity, 0.8, epsilon = 1e-10);
    }

    #[test]
    fn test_ellipse_focal_distance() {
        let a = 5.0;
        let b = 3.0;
        let focal_distance = ellipse_focal_distance(a, b);

        // c = sqrt(a^2 - b^2) = sqrt(25 - 9) = 4
        assert_abs_diff_eq!(focal_distance, 4.0, epsilon = 1e-10);
    }

    #[test]
    fn test_ellipse_area() {
        let a = 5.0;
        let b = 3.0;
        let area = crate::metrics::area_volume::ellipse_area(a, b);

        // Area = π * a * b = π * 5 * 3 = 15π
        assert_abs_diff_eq!(area, 15.0 * std::f64::consts::PI, epsilon = 1e-10);
    }

    #[test]
    fn test_ellipse_foci() {
        let a = 5.0;
        let b = 3.0;
        let ((x1, y1), (x2, y2)) = ellipse_foci(a, b);

        // 焦点は (±4, 0)
        assert_abs_diff_eq!(x1, -4.0, epsilon = 1e-10);
        assert_abs_diff_eq!(y1, 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(x2, 4.0, epsilon = 1e-10);
        assert_abs_diff_eq!(y2, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_circle_case() {
        // 円の場合（a = b）の特殊ケース
        let a = 3.0;
        let b = 3.0;

        let perimeter = ellipse_perimeter_ramanujan_ii(a, b);
        let expected_circumference = 2.0 * std::f64::consts::PI * a;
        assert_abs_diff_eq!(perimeter, expected_circumference, epsilon = 1e-10);

        let eccentricity = ellipse_eccentricity(a, b);
        assert_abs_diff_eq!(eccentricity, 0.0, epsilon = 1e-10);

        let focal_distance = ellipse_focal_distance(a, b);
        assert_abs_diff_eq!(focal_distance, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_generic_types_f32() {
        let a = 5.0f32;
        let b = 3.0f32;

        let perimeter = ellipse_perimeter_ramanujan_ii(a, b);
        let area = crate::metrics::area_volume::ellipse_area(a, b);
        let eccentricity = ellipse_eccentricity(a, b);

        assert!(perimeter > 25.0f32);
        assert!(area > 47.0f32);
        assert!((eccentricity - 0.8f32).abs() < 1e-6f32);
    }

    #[test]
    fn test_edge_cases() {
        // 極端に扁平な楕円
        let a = 10.0;
        let b = 0.1;
        let eccentricity = ellipse_eccentricity(a, b);
        assert!(eccentricity > 0.99);

        // 非常に小さな楕円
        let a = 1e-3;
        let b = 5e-4;
        let area = crate::metrics::area_volume::ellipse_area(a, b);
        assert!(area > 0.0);
        assert_abs_diff_eq!(area, std::f64::consts::PI * 1e-3 * 5e-4, epsilon = 1e-12);
    }
}
