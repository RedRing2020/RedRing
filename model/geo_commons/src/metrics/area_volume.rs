//! 面積・体積計算関数
//!
//! 基本図形の面積・体積計算を提供
//! geo_primitives での利用を想定した共通計算関数群

use analysis::abstract_types::Scalar;

/// 円の面積計算
pub fn circle_area<T: Scalar>(radius: T) -> T {
    T::PI * radius * radius
}

/// 楕円の面積計算
pub fn ellipse_area<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    T::PI * semi_major * semi_minor
}

/// 三角形の面積計算（ヘロンの公式）
pub fn triangle_area<T: Scalar>(a: T, b: T, c: T) -> T {
    let s = (a + b + c) / T::from_f64(2.0);
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

/// 三角形の面積計算（座標から）
pub fn triangle_area_from_coords<T: Scalar>(p1: (T, T), p2: (T, T), p3: (T, T)) -> T {
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let (x3, y3) = p3;

    ((x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)) / T::from_f64(2.0)).abs()
}

/// 多角形の面積計算（靴紐公式）
pub fn polygon_area<T: Scalar>(vertices: &[(T, T)]) -> T {
    if vertices.len() < 3 {
        return T::ZERO;
    }

    let mut area = T::ZERO;
    let n = vertices.len();

    for i in 0..n {
        let j = (i + 1) % n;
        area += vertices[i].0 * vertices[j].1;
        area -= vertices[j].0 * vertices[i].1;
    }

    area.abs() / T::from_f64(2.0)
}

/// 球の体積計算
pub fn sphere_volume<T: Scalar>(radius: T) -> T {
    T::from_f64(4.0) / T::from_f64(3.0) * T::PI * radius.powi(3)
}

/// 円柱の体積計算
pub fn cylinder_volume<T: Scalar>(radius: T, height: T) -> T {
    T::PI * radius * radius * height
}

/// 円錐の体積計算
pub fn cone_volume<T: Scalar>(radius: T, height: T) -> T {
    T::PI * radius * radius * height / T::from_f64(3.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_circle_area() {
        // 半径1の円
        let area = circle_area(1.0f64);
        assert_relative_eq!(area, std::f64::consts::PI, epsilon = 1e-10);

        // 半径2の円
        let area = circle_area(2.0f64);
        assert_relative_eq!(area, 4.0 * std::f64::consts::PI, epsilon = 1e-10);
    }

    #[test]
    fn test_ellipse_area() {
        // 円（a = b = 1）
        let area = ellipse_area(1.0f64, 1.0f64);
        assert_relative_eq!(area, std::f64::consts::PI, epsilon = 1e-10);

        // 楕円（a = 2, b = 1）
        let area = ellipse_area(2.0f64, 1.0f64);
        assert_relative_eq!(area, 2.0 * std::f64::consts::PI, epsilon = 1e-10);
    }

    #[test]
    fn test_sphere_volume() {
        // 半径1の球
        let volume = sphere_volume(1.0f64);
        let expected = 4.0 / 3.0 * std::f64::consts::PI;
        assert_relative_eq!(volume, expected, epsilon = 1e-10);
    }

    #[test]
    fn test_cylinder_volume() {
        // 半径1、高さ2の円柱
        let volume = cylinder_volume(1.0f64, 2.0f64);
        assert_relative_eq!(volume, 2.0 * std::f64::consts::PI, epsilon = 1e-10);
    }
}
