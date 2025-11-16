//! 楕円関連の近似計算
//!
//! 楕円周長計算、楕円の幾何学的性質など、
//! 楕円形状に特化した数値計算関数を提供する。
//! 全てScalarトレイトによるジェネリック実装。

use analysis::abstract_types::Scalar;

/// 楕円の周長（ラマヌジャンの近似式I - 標準版）
pub fn ellipse_perimeter_ramanujan_i<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    let a = semi_major;
    let b = semi_minor;
    let h = ((a - b) / (a + b)).powi(2);
    T::PI
        * (a + b)
        * (T::ONE
            + (T::from_f64(3.0) * h)
                / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
}

/// 楕円の周長（ラマヌジャンの近似式I - エイリアス）
/// analysis/curves.rsとの互換性のため
pub fn ellipse_circumference_ramanujan<T: Scalar>(major_radius: T, minor_radius: T) -> T {
    ellipse_perimeter_ramanujan_i(major_radius, minor_radius)
}

/// 楕円の周長（ラマヌジャンの近似式II、より高精度）
pub fn ellipse_perimeter_ramanujan_ii<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    let a = semi_major;
    let b = semi_minor;
    let h = ((a - b) / (a + b)).powi(2);
    let term1 = T::from_f64(3.0) * h
        / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt());
    let term2 = h.powi(2) / T::from_f64(4.0);
    T::PI * (a + b) * (T::ONE + term1 + term2)
}

/// 楕円の周長（パダン近似、中程度精度）
pub fn ellipse_perimeter_padé<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    let a = semi_major;
    let b = semi_minor;
    let h = ((a - b) / (a + b)).powi(2);
    T::PI * (a + b) * (T::ONE + h / T::from_f64(4.0) + h.powi(2) / T::from_f64(64.0))
}

/// 楕円の周長（カントレル近似、高精度）
pub fn ellipse_perimeter_cantrell<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    let a = semi_major;
    let b = semi_minor;
    if a == b {
        return T::TAU * a; // 円の場合
    }

    let h = ((a - b) / (a + b)).powi(2);
    let term1 = T::ONE;
    let term2 = h / T::from_f64(4.0);
    let term3 = h.powi(2) / T::from_f64(64.0);
    let term4 = h.powi(3) / T::from_f64(256.0);

    T::PI * (a + b) * (term1 + term2 + term3 + term4)
}

/// 楕円の離心率計算
pub fn ellipse_eccentricity<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    if semi_major == T::ZERO {
        return T::ZERO;
    }
    (T::ONE - (semi_minor / semi_major).powi(2)).sqrt()
}

/// 楕円の焦点距離計算
pub fn ellipse_focal_distance<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    if semi_major >= semi_minor {
        (semi_major.powi(2) - semi_minor.powi(2)).sqrt()
    } else {
        (semi_minor.powi(2) - semi_major.powi(2)).sqrt()
    }
}

/// 無限級数展開による楕円周長計算（高精度版）
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
pub fn ellipse_circumference_series<T: Scalar>(
    major_radius: T,
    minor_radius: T,
    terms: usize,
) -> T {
    if major_radius == minor_radius {
        return T::TAU * major_radius; // 円の場合
    }

    let a = major_radius.max(minor_radius);
    let b = major_radius.min(minor_radius);
    let e_squared = T::ONE - (b * b) / (a * a); // 離心率の二乗

    // 第2種完全楕円積分 E(e) の級数展開
    // E(e) = π/2 * [1 - Σ((2n-1)!!/(2n)!)^2 * e^(2n) / (2n-1))]
    let mut e_k = T::ONE; // E(e) の値
    let mut coeff = T::ONE;
    let mut power = e_squared;

    for n in 1..=terms {
        // (2n-1)!! / (2n)!! の計算
        coeff = coeff * T::from_f64(2.0 * n as f64 - 1.0) / T::from_f64(2.0 * n as f64);
        let term = coeff * coeff * power / T::from_f64(2.0 * n as f64 - 1.0);
        e_k = e_k - term;
        power = power * e_squared;

        // 収束判定（近似）
        if term < T::from_f64(1e-15) {
            break;
        }
    }

    // 楕円周長 = 4 * a * E(e)
    T::from_f64(4.0) * a * e_k * T::PI / T::from_f64(2.0)
}

/// 数値積分による楕円周長計算（最高精度版）
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
pub fn ellipse_circumference_numerical<T: Scalar>(
    major_radius: T,
    minor_radius: T,
    n_points: usize,
) -> T {
    if major_radius == minor_radius {
        return T::TAU * major_radius; // 円の場合
    }

    let a = major_radius;
    let b = minor_radius;
    let e_squared = T::ONE - (b * b) / (a * a);

    // ガウス・ルジャンドル求積法の実装（簡易版）
    let mut sum = T::ZERO;
    let dt = T::PI / (T::from_f64(2.0) * T::from_f64(n_points as f64));

    for i in 0..n_points {
        let t = (T::from_f64(i as f64) + T::from_f64(0.5)) * dt; // 中点公式
        let integrand = (T::ONE - e_squared * t.sin().powi(2)).sqrt();
        sum = sum + integrand * dt;
    }

    T::from_f64(4.0) * a * sum
}

/// 楕円の面積を計算
pub fn ellipse_area<T: Scalar>(major_radius: T, minor_radius: T) -> T {
    T::PI * major_radius * minor_radius
}

/// 楕円の焦点座標を計算（中心が原点、長軸がx軸の場合）
/// 戻り値: (正の焦点のx座標, 負の焦点のx座標)
pub fn ellipse_foci<T: Scalar>(major_radius: T, minor_radius: T) -> (T, T) {
    let c = ellipse_focal_distance(major_radius, minor_radius);
    (c, T::ZERO - c)
}

#[cfg(test)]
mod tests {
    use super::*;
    type F64 = f64;

    #[test]
    fn test_circle_circumference() {
        let radius = 5.0;
        let expected = 2.0 * std::f64::consts::PI * radius;

        // 円の場合、すべての手法で同じ結果になるはず
        let ramanujan = ellipse_circumference_ramanujan(radius, radius);
        let series = ellipse_circumference_series(radius, radius, 10);
        let numerical = ellipse_circumference_numerical(radius, radius, 100);

        assert!((ramanujan - expected).abs() < 1e-10);
        assert!((series - expected).abs() < 1e-10);
        assert!((numerical - expected).abs() < 1e-6);
    }

    #[test]
    fn test_ellipse_circumference_consistency() {
        let major = 10.0;
        let minor = 6.0;

        let ramanujan = ellipse_circumference_ramanujan(major, minor);
        let series = ellipse_circumference_series(major, minor, 20);
        let numerical = ellipse_circumference_numerical(major, minor, 200);

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

        let area = ellipse_area(major, minor);
        let expected_area = std::f64::consts::PI * major * minor;
        assert!((area - expected_area).abs() < 1e-10);

        let focal_dist = ellipse_focal_distance(major, minor);
        assert!((focal_dist - 4.0).abs() < 1e-10);

        let (f1, f2) = ellipse_foci(major, minor);
        assert!((f1 - 4.0).abs() < 1e-10);
        assert!((f2 - (-4.0)).abs() < 1e-10);
    }
}
