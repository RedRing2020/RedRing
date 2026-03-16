//! Area and volume formulas used by geometric layer crates.

use crate::Scalar;

/// Compute the area of a circle.
pub fn circle_area<T: Scalar>(radius: T) -> T {
    T::PI * radius * radius
}

/// Compute the area of an ellipse.
pub fn ellipse_area<T: Scalar>(semi_major: T, semi_minor: T) -> T {
    T::PI * semi_major * semi_minor
}

/// Compute triangle area from edge lengths via Heron's formula.
pub fn triangle_area<T: Scalar>(a: T, b: T, c: T) -> T {
    let s = (a + b + c) / T::from_f64(2.0);
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

/// Compute triangle area from 2D coordinates.
pub fn triangle_area_from_coords<T: Scalar>(p1: (T, T), p2: (T, T), p3: (T, T)) -> T {
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let (x3, y3) = p3;
    ((x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)) / T::from_f64(2.0)).abs()
}

/// Compute polygon area with the shoelace formula.
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

/// Compute volume of a sphere.
pub fn sphere_volume<T: Scalar>(radius: T) -> T {
    T::from_f64(4.0) / T::from_f64(3.0) * T::PI * radius.powi(3)
}

/// Compute volume of a cylinder.
pub fn cylinder_volume<T: Scalar>(radius: T, height: T) -> T {
    T::PI * radius * radius * height
}

/// Compute volume of a cone.
pub fn cone_volume<T: Scalar>(radius: T, height: T) -> T {
    T::PI * radius * radius * height / T::from_f64(3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_area() {
        let eps = 1e-12;
        assert!((circle_area(1.0_f64) - std::f64::consts::PI).abs() < eps);
        assert!((circle_area(2.0_f64) - 4.0 * std::f64::consts::PI).abs() < eps);
    }

    #[test]
    fn test_ellipse_area() {
        let eps = 1e-12;
        assert!((ellipse_area(1.0_f64, 1.0_f64) - std::f64::consts::PI).abs() < eps);
        assert!((ellipse_area(2.0_f64, 1.0_f64) - 2.0 * std::f64::consts::PI).abs() < eps);
    }

    #[test]
    fn test_sphere_volume() {
        let expected = 4.0 / 3.0 * std::f64::consts::PI;
        assert!((sphere_volume(1.0_f64) - expected).abs() < 1e-12);
    }

    #[test]
    fn test_cylinder_volume() {
        assert!((cylinder_volume(1.0_f64, 2.0_f64) - 2.0 * std::f64::consts::PI).abs() < 1e-12);
    }
}
