//! 長さ計算関連の計算

use crate::abstract_types::Scalar;

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

/// ベクトルの長さ（ノルム）
pub fn vector_length<T: Scalar>(components: &[T]) -> T {
    components
        .iter()
        .map(|&x| x * x)
        .fold(T::ZERO, |acc, x| acc + x)
        .sqrt()
}

/// ベクトルの長さの二乗
pub fn vector_length_squared<T: Scalar>(components: &[T]) -> T {
    components
        .iter()
        .map(|&x| x * x)
        .fold(T::ZERO, |acc, x| acc + x)
}

/// 2Dベクトルの長さ（最適化版）
pub fn vector_length_2d<T: Scalar>(x: T, y: T) -> T {
    (x * x + y * y).sqrt()
}

/// 3Dベクトルの長さ（最適化版）
pub fn vector_length_3d<T: Scalar>(x: T, y: T, z: T) -> T {
    (x * x + y * y + z * z).sqrt()
}

/// ポリラインの全長計算
pub fn polyline_length<T: Scalar>(points: &[[T; 2]]) -> T {
    if points.len() < 2 {
        return T::ZERO;
    }

    points
        .windows(2)
        .map(|pair| {
            let dx = pair[1][0] - pair[0][0];
            let dy = pair[1][1] - pair[0][1];
            (dx * dx + dy * dy).sqrt()
        })
        .fold(T::ZERO, |acc, length| acc + length)
}

/// 3Dポリラインの全長計算
pub fn polyline_length_3d<T: Scalar>(points: &[[T; 3]]) -> T {
    if points.len() < 2 {
        return T::ZERO;
    }

    points
        .windows(2)
        .map(|pair| {
            let dx = pair[1][0] - pair[0][0];
            let dy = pair[1][1] - pair[0][1];
            let dz = pair[1][2] - pair[0][2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(T::ZERO, |acc, length| acc + length)
}
