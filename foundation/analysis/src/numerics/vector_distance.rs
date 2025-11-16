//! ベクトル・距離数値計算（純粋な数値操作）

use crate::abstract_types::Scalar;

// =============================================================================
// Distance Functions (点間距離計算)
// =============================================================================

/// 点と点の距離計算（汎用N次元）
pub fn point_distance<T: Scalar>(p1: &[T], p2: &[T]) -> T {
    assert_eq!(p1.len(), p2.len(), "Points must have same dimension");

    let sum_of_squares: T = p1
        .iter()
        .zip(p2.iter())
        .map(|(a, b)| (*a - *b) * (*a - *b))
        .fold(T::ZERO, |acc, x| acc + x);

    sum_of_squares.sqrt()
}

/// 点と点の距離の二乗計算（平方根計算の回避）
pub fn point_distance_squared<T: Scalar>(p1: &[T], p2: &[T]) -> T {
    assert_eq!(p1.len(), p2.len(), "Points must have same dimension");

    p1.iter()
        .zip(p2.iter())
        .map(|(a, b)| (*a - *b) * (*a - *b))
        .fold(T::ZERO, |acc, x| acc + x)
}

/// 2D点間の距離計算（最適化版）
pub fn point_distance_2d<T: Scalar>(x1: T, y1: T, x2: T, y2: T) -> T {
    let dx = x1 - x2;
    let dy = y1 - y2;
    (dx * dx + dy * dy).sqrt()
}

/// 3D点間の距離計算（最適化版）
pub fn point_distance_3d<T: Scalar>(x1: T, y1: T, z1: T, x2: T, y2: T, z2: T) -> T {
    let dx = x1 - x2;
    let dy = y1 - y2;
    let dz = z1 - z2;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// マンハッタン距離（L1ノルム）
pub fn manhattan_distance<T: Scalar>(p1: &[T], p2: &[T]) -> T {
    assert_eq!(p1.len(), p2.len(), "Points must have same dimension");

    p1.iter()
        .zip(p2.iter())
        .map(|(a, b)| (*a - *b).abs())
        .fold(T::ZERO, |acc, x| acc + x)
}

/// チェビシェフ距離（L∞ノルム）
pub fn chebyshev_distance<T: Scalar>(p1: &[T], p2: &[T]) -> T {
    assert_eq!(p1.len(), p2.len(), "Points must have same dimension");

    p1.iter()
        .zip(p2.iter())
        .map(|(a, b)| (*a - *b).abs())
        .fold(T::ZERO, |acc, x| acc.max(x))
}

/// ミンコフスキー距離（一般化Lpノルム）
pub fn minkowski_distance<T: Scalar>(p1: &[T], p2: &[T], p: T) -> T {
    assert_eq!(p1.len(), p2.len(), "Points must have same dimension");

    let sum: T = p1
        .iter()
        .zip(p2.iter())
        .map(|(a, b)| (*a - *b).abs().powf(p))
        .fold(T::ZERO, |acc, x| acc + x);

    sum.powf(T::ONE / p)
}

// =============================================================================
// Length Functions (ベクトル・線分長さ計算)
// =============================================================================

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
