//! 長さ計算関連の計算（geometry専用）

use analysis::abstract_types::Scalar;

/// 円弧長計算
pub fn arc_length<T: Scalar>(radius: T, angle_span: T) -> T {
    radius * angle_span
}

/// 楕円弧長計算（近似）
pub fn ellipse_arc_length_approximation<T: Scalar>(
    semi_major: T,
    semi_minor: T,
    start_angle: T,
    end_angle: T,
) -> T {
    // 簡単な近似：平均半径 × 角度範囲
    let avg_radius = (semi_major + semi_minor) / T::from_f64(2.0);
    avg_radius * (end_angle - start_angle).abs()
}
