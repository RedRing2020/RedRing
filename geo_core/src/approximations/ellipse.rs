//! 楕円関連の近似計算

use analysis::abstract_types::Scalar;

/// 楕円の周長（ラマヌジャンの近似式I）
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
