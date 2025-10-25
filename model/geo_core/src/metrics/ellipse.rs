//! 楕円の幾何学的計算モジュール
//!
//! 楕円の周長、面積、焦点などの特化した計算機能を提供する。

use std::f64::consts::PI;

// =============================================================================
// 楕円周長計算の各種近似手法
// =============================================================================

/// ラマヌジャンの近似式による楕円周長計算（f64特化版）
///
/// # 数値解析について
/// 楕円の正確な周長は第2種完全楕円積分で表現され、解析的解がないため近似が必要です。
///
/// ## 使用している手法: ラマヌジャンの近似式 (1914)
/// - 精度: 相対誤差 < 5×10⁻⁵ (離心率0.99でも高精度)
/// - 計算量: O(1) - 平方根1回のみ
/// - 適用範囲: すべての楕円形状
///
/// # Arguments
/// * `major_radius` - 楕円の長軸半径
/// * `minor_radius` - 楕円の短軸半径
///
/// # Returns
/// 楕円の周長の近似値
pub fn ellipse_circumference_ramanujan_f64(major_radius: f64, minor_radius: f64) -> f64 {
    let a = major_radius;
    let b = minor_radius;
    let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
    PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
}

/// 無限級数展開による楕円周長計算（高精度版、f64特化）
///
/// より高精度だが計算量が大きい手法。高精度が必要な場合に使用。
///
/// # Arguments
/// * `major_radius` - 楕円の長軸半径
/// * `minor_radius` - 楕円の短軸半径
/// * `terms` - 級数の項数（精度と計算量のトレードオフ）
///
/// # Returns
/// 楕円の周長の高精度近似値
pub fn ellipse_circumference_series_f64(major_radius: f64, minor_radius: f64, terms: usize) -> f64 {
    if major_radius == minor_radius {
        return 2.0 * PI * major_radius; // 円の場合
    }

    let a = major_radius.max(minor_radius);
    let b = major_radius.min(minor_radius);
    let e_squared = 1.0 - (b * b) / (a * a); // 離心率の二乗

    // 第2種完全楕円積分 E(e) の級数展開
    // E(e) = π/2 * [1 - Σ((2n-1)!!/(2n)!)^2 * e^(2n) / (2n-1))]
    let mut e_k = 1.0; // E(e) の値
    let mut coeff = 1.0;
    let mut power = e_squared;

    for n in 1..=terms {
        // (2n-1)!! / (2n)!! の計算
        coeff *= (2.0 * n as f64 - 1.0) / (2.0 * n as f64);
        let term = coeff * coeff * power / (2.0 * n as f64 - 1.0);
        e_k -= term;
        power *= e_squared;

        // 収束判定
        if term.abs() < 1e-15 {
            break;
        }
    }

    // 楕円周長 = 4 * a * E(e)
    4.0 * a * e_k * PI / 2.0
}

/// 数値積分による楕円周長計算（最高精度版、f64特化）
///
/// ガウス・ルジャンドル求積法による数値積分。最高精度だが計算コスト高。
///
/// # Arguments
/// * `major_radius` - 楕円の長軸半径
/// * `minor_radius` - 楕円の短軸半径
/// * `n_points` - 積分点数（精度と計算量のトレードオフ）
///
/// # Returns
/// 楕円の周長の数値積分値
pub fn ellipse_circumference_numerical_f64(
    major_radius: f64,
    minor_radius: f64,
    n_points: usize,
) -> f64 {
    if major_radius == minor_radius {
        return 2.0 * PI * major_radius; // 円の場合
    }

    let a = major_radius;
    let b = minor_radius;
    let e_squared = 1.0 - (b * b) / (a * a);

    // ガウス・ルジャンドル求積法の実装（簡易版）
    let mut sum = 0.0;
    let dt = PI / (2.0 * n_points as f64);

    for i in 0..n_points {
        let t = (i as f64 + 0.5) * dt; // 中点公式
        let integrand = (1.0 - e_squared * t.sin().powi(2)).sqrt();
        sum += integrand * dt;
    }

    4.0 * a * sum
}

/// 楕円の離心率を計算（f64特化版）
///
/// # Arguments
/// * `major_radius` - 楕円の長軸半径
/// * `minor_radius` - 楕円の短軸半径
///
/// # Returns
/// 離心率 (0 ≤ e < 1)
pub fn ellipse_eccentricity_f64(major_radius: f64, minor_radius: f64) -> f64 {
    if major_radius <= minor_radius {
        0.0
    } else {
        (1.0 - (minor_radius * minor_radius) / (major_radius * major_radius)).sqrt()
    }
}

// =============================================================================
// 楕円の様々な幾何学的性質
// =============================================================================

/// 楕円の面積を計算（f64特化版）
pub fn ellipse_area_f64(major_radius: f64, minor_radius: f64) -> f64 {
    PI * major_radius * minor_radius
}

/// 楕円の焦点距離を計算（f64特化版）
pub fn ellipse_focal_distance_f64(major_radius: f64, minor_radius: f64) -> f64 {
    if major_radius <= minor_radius {
        0.0
    } else {
        (major_radius * major_radius - minor_radius * minor_radius).sqrt()
    }
}

/// 楕円の焦点座標を計算（中心が原点、長軸がx軸の場合、f64特化版）
pub fn ellipse_foci_f64(major_radius: f64, minor_radius: f64) -> (f64, f64) {
    let c = ellipse_focal_distance_f64(major_radius, minor_radius);
    (c, -c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_circumference() {
        let radius = 5.0;
        let expected = 2.0 * PI * radius;

        // 円の場合、すべての手法で同じ結果になるはず
        assert!((ellipse_circumference_ramanujan_f64(radius, radius) - expected).abs() < 1e-10);
        assert!((ellipse_circumference_series_f64(radius, radius, 10) - expected).abs() < 1e-10);
        assert!((ellipse_circumference_numerical_f64(radius, radius, 100) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_ellipse_circumference_consistency() {
        let major = 10.0;
        let minor = 6.0;

        let ramanujan = ellipse_circumference_ramanujan_f64(major, minor);
        let series = ellipse_circumference_series_f64(major, minor, 20);
        let numerical = ellipse_circumference_numerical_f64(major, minor, 200);

        // 各手法の結果が十分近いことを確認
        assert!(
            (ramanujan - series).abs() / series < 1e-4,
            "Ramanujan vs Series: {} vs {}",
            ramanujan,
            series
        );
        assert!(
            (series - numerical).abs() / numerical < 1e-4,
            "Series vs Numerical: {} vs {}",
            series,
            numerical
        );
        assert!(
            (ramanujan - numerical).abs() / numerical < 1e-4,
            "Ramanujan vs Numerical: {} vs {}",
            ramanujan,
            numerical
        );
    }

    #[test]
    fn test_ellipse_properties() {
        let major = 5.0;
        let minor = 3.0;

        assert!((ellipse_area_f64(major, minor) - PI * major * minor).abs() < 1e-10);
        assert!((ellipse_focal_distance_f64(major, minor) - 4.0).abs() < 1e-10);

        let (f1, f2) = ellipse_foci_f64(major, minor);
        assert!((f1 - 4.0).abs() < 1e-10);
        assert!((f2 - (-4.0)).abs() < 1e-10);
    }
}
