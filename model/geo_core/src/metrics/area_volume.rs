//! 面積・体積計算関連の計算

use analysis::abstract_types::Scalar;

/// 円の面積計算
pub fn circle_area<T: Scalar>(radius: T) -> T {
    T::PI * radius * radius
}

/// 楕円の面積計算
pub fn ellipse_area<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    T::PI * semi_major * semi_minor
}

/// 三角形の面積計算（座標版）
pub fn triangle_area_2d<T: Scalar>(x1: T, y1: T, x2: T, y2: T, x3: T, y3: T) -> T {
    ((x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)) / T::from_f64(2.0)).abs()
}

/// 三角形の面積計算（ヘロンの公式）
pub fn triangle_area_heron<T: Scalar>(a: T, b: T, c: T) -> T {
    let s = (a + b + c) / T::from_f64(2.0);
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

/// 矩形の面積計算
pub fn rectangle_area<T: Scalar>(width: T, height: T) -> T {
    width * height
}

/// 台形の面積計算
pub fn trapezoid_area<T: Scalar>(base1: T, base2: T, height: T) -> T {
    (base1 + base2) * height / T::from_f64(2.0)
}

/// 多角形の面積計算（シューレース公式）
pub fn polygon_area<T: Scalar>(vertices: &[[T; 2]]) -> T {
    if vertices.len() < 3 {
        return T::ZERO;
    }

    let mut area = T::ZERO;
    let n = vertices.len();

    for i in 0..n {
        let j = (i + 1) % n;
        area += vertices[i][0] * vertices[j][1];
        area -= vertices[j][0] * vertices[i][1];
    }

    area.abs() / T::from_f64(2.0)
}

/// 球の体積計算
pub fn sphere_volume<T: Scalar>(radius: T) -> T {
    T::from_f64(4.0) * T::PI * radius * radius * radius / T::from_f64(3.0)
}

/// 円柱の体積計算
pub fn cylinder_volume<T: Scalar>(radius: T, height: T) -> T {
    T::PI * radius * radius * height
}

/// 円錐の体積計算
pub fn cone_volume<T: Scalar>(radius: T, height: T) -> T {
    T::PI * radius * radius * height / T::from_f64(3.0)
}

/// 直方体の体積計算
pub fn box_volume<T: Scalar>(width: T, height: T, depth: T) -> T {
    width * height * depth
}

/// 球の表面積計算
pub fn sphere_surface_area<T: Scalar>(radius: T) -> T {
    T::from_f64(4.0) * T::PI * radius * radius
}

/// 円柱の表面積計算（上下面含む）
pub fn cylinder_surface_area<T: Scalar>(radius: T, height: T) -> T {
    T::from_f64(2.0) * T::PI * radius * (radius + height)
}

/// 円錐の表面積計算（底面含む）
pub fn cone_surface_area<T: Scalar>(radius: T, height: T) -> T {
    let slant_height = (radius * radius + height * height).sqrt();
    T::PI * radius * (radius + slant_height)
}
