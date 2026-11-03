//! ベクトル・距離数値計算（純粋な数値操作）

use crate::abstract_types::Scalar;

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

#[cfg(test)]
mod tests {
    use super::{
        chebyshev_distance, manhattan_distance, minkowski_distance, point_distance,
        point_distance_2d, point_distance_3d, polyline_length, polyline_length_3d, vector_length,
        vector_length_2d, vector_length_3d, vector_length_squared,
    };
    use crate::consts::test_constants::TOLERANCE_F64;

    #[test]
    fn test_point_distance_calculations() {
        let distance = point_distance_2d(0.0_f64, 0.0, 3.0, 4.0);
        assert!((distance - 5.0).abs() < TOLERANCE_F64);

        let distance_3d = point_distance_3d(0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0);
        let expected = 3.0_f64.sqrt();
        assert!((distance_3d - expected).abs() < TOLERANCE_F64);

        let p1 = [0.0, 0.0, 0.0];
        let p2 = [1.0, 1.0, 1.0];
        let distance = point_distance(&p1, &p2);
        assert!((distance - 3.0_f64.sqrt()).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_distance_metrics() {
        let p1 = [0.0, 0.0];
        let p2 = [3.0, 4.0];

        let manhattan = manhattan_distance(&p1, &p2);
        assert_eq!(manhattan, 7.0);

        let chebyshev = chebyshev_distance(&p1, &p2);
        assert_eq!(chebyshev, 4.0);

        let minkowski = minkowski_distance(&p1, &p2, 2.0_f64);
        let euclidean = point_distance(&p1, &p2);
        assert!((minkowski - euclidean).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_vector_length_calculations() {
        let components = [3.0, 4.0];
        let length = vector_length(&components);
        assert_eq!(length, 5.0);

        let length_2d = vector_length_2d(3.0, 4.0);
        assert_eq!(length_2d, 5.0);

        let length_3d = vector_length_3d(1.0, 1.0, 1.0);
        let expected = 3.0_f64.sqrt();
        assert!((length_3d - expected).abs() < TOLERANCE_F64);

        let length_squared = vector_length_squared(&components);
        assert_eq!(length_squared, 25.0);
    }

    #[test]
    fn test_polyline_calculations() {
        let points_2d = [[0.0, 0.0], [3.0, 0.0], [3.0, 4.0]];
        let length_2d = polyline_length(&points_2d);
        assert_eq!(length_2d, 7.0);

        let points_3d = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        ];
        let length_3d = polyline_length_3d(&points_3d);
        assert_eq!(length_3d, 3.0);
    }
}
