//! 長さ計算のテスト

use crate::metrics::*;

#[cfg(test)]
mod length_tests {
    use super::*;

    #[test]
    fn test_length_calculations() {
        // 円弧長
        let arc = arc_length(2.0, std::f64::consts::PI / 2.0);
        let expected = std::f64::consts::PI;
        assert!((arc - expected).abs() < 1e-10);

        // 楕円弧長近似
        let ellipse_arc =
            ellipse_arc_length_approximation(5.0, 3.0, 0.0, std::f64::consts::PI / 2.0);
        let expected_avg_radius = 4.0; // (5+3)/2
        let expected_ellipse_arc = expected_avg_radius * std::f64::consts::PI / 2.0;
        assert!((ellipse_arc - expected_ellipse_arc).abs() < 1e-10);
    }

    #[test]
    fn test_distance_calculations() {
        // 汎用的な距離計算は analysis::metrics に移動されました
        // このテストでは geometry 専用の距離計算のみをテスト
        // 現在は機能が空なので、将来の実装を待つ
    }
}
