//! ベクトル・距離数値計算のテスト

use crate::numerics::vector_distance::{
    chebyshev_distance, manhattan_distance, minkowski_distance, point_distance, point_distance_2d,
    point_distance_3d, polyline_length, polyline_length_3d, vector_length, vector_length_2d,
    vector_length_3d, vector_length_squared,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance_calculations() {
        // 2D点間距離
        let distance = point_distance_2d(0.0_f64, 0.0, 3.0, 4.0);
        assert!((distance - 5.0).abs() < 1e-10);

        // 3D点間距離
        let distance_3d = point_distance_3d(0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0);
        let expected = 3.0_f64.sqrt();
        assert!((distance_3d - expected).abs() < 1e-10);

        // N次元点間距離
        let p1 = [0.0, 0.0, 0.0];
        let p2 = [1.0, 1.0, 1.0];
        let distance = point_distance(&p1, &p2);
        assert!((distance - 3.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_distance_metrics() {
        let p1 = [0.0, 0.0];
        let p2 = [3.0, 4.0];

        // マンハッタン距離
        let manhattan = manhattan_distance(&p1, &p2);
        assert_eq!(manhattan, 7.0);

        // チェビシェフ距離
        let chebyshev = chebyshev_distance(&p1, &p2);
        assert_eq!(chebyshev, 4.0);

        // ミンコフスキー距離（p=2, ユークリッド距離と同等）
        let minkowski = minkowski_distance(&p1, &p2, 2.0_f64);
        let euclidean = point_distance(&p1, &p2);
        assert!((minkowski - euclidean).abs() < 1e-10);
    }

    #[test]
    fn test_vector_length_calculations() {
        // ベクトルの長さ
        let components = [3.0, 4.0];
        let length = vector_length(&components);
        assert_eq!(length, 5.0);

        // 2Dベクトルの長さ
        let length_2d = vector_length_2d(3.0, 4.0);
        assert_eq!(length_2d, 5.0);

        // 3Dベクトルの長さ
        let length_3d = vector_length_3d(1.0, 1.0, 1.0);
        let expected = 3.0_f64.sqrt();
        assert!((length_3d - expected).abs() < 1e-10);

        // ベクトルの長さの二乗
        let length_squared = vector_length_squared(&components);
        assert_eq!(length_squared, 25.0);
    }

    #[test]
    fn test_polyline_calculations() {
        // 2Dポリラインの長さ
        let points_2d = [[0.0, 0.0], [3.0, 0.0], [3.0, 4.0]];
        let length_2d = polyline_length(&points_2d);
        assert_eq!(length_2d, 7.0);

        // 3Dポリラインの長さ
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
