//! 面積・体積計算のテスト

use crate::metrics::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area_calculations() {
        // 円の面積
        let radius = 2.0;
        let area = circle_area(radius);
        let expected = std::f64::consts::PI * 4.0;
        assert!((area - expected).abs() < 1e-10);

        // 三角形の面積（座標版）
        let area = triangle_area_2d(0.0, 0.0, 3.0, 0.0, 0.0, 4.0);
        assert_eq!(area, 6.0);

        // 矩形の面積
        let area = rectangle_area(5.0, 3.0);
        assert_eq!(area, 15.0);
    }

    #[test]
    fn test_polygon_area() {
        // 正方形
        let square = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        let area = polygon_area(&square);
        assert_eq!(area, 4.0);

        // 三角形
        let triangle = [[0.0, 0.0], [4.0, 0.0], [2.0, 3.0]];
        let area = polygon_area(&triangle);
        assert_eq!(area, 6.0);
    }

    #[test]
    fn test_volume_calculations() {
        // 球の体積
        let radius = 2.0;
        let volume = sphere_volume(radius);
        let expected = 4.0 * std::f64::consts::PI * 8.0 / 3.0;
        assert!((volume - expected).abs() < 1e-10);

        // 円柱の体積
        let volume = cylinder_volume(1.0, 5.0);
        let expected = std::f64::consts::PI * 5.0;
        assert!((volume - expected).abs() < 1e-10);

        // 直方体の体積
        let volume = box_volume(2.0, 3.0, 4.0);
        assert_eq!(volume, 24.0);
    }

    #[test]
    fn test_surface_area_calculations() {
        // 球の表面積
        let radius = 2.0;
        let surface_area = sphere_surface_area(radius);
        let expected = 4.0 * std::f64::consts::PI * 4.0;
        assert!((surface_area - expected).abs() < 1e-10);

        // 円柱の表面積
        let surface_area = cylinder_surface_area(1.0, 2.0);
        let expected = 2.0 * std::f64::consts::PI * (1.0 + 2.0);
        assert!((surface_area - expected).abs() < 1e-10);
    }
}
