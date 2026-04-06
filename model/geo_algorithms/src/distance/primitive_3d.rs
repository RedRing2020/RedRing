//! 3D Primitive distance algorithms
//!
//! `geo_primitives` に実装済みの各 3D 形状の距離計算メソッドを
//! `geo_algorithms` 層の公開 entrypoint としてまとめた薄いラッパー群。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{Circle3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D};
use geo_contracts::Scalar;

/// LineSegment3D-点 間の最短距離（端点クランプあり）
pub fn line_segment3d_point3d_distance<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
) -> T {
    segment.distance_to_point(point)
}

/// 逆向きラッパー: point-segment
pub fn point3d_line_segment3d_distance<T: Scalar>(
    point: &Point3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    segment.distance_to_point(point)
}

/// 無限直線3D-点 間の最短距離（垂直距離）
pub fn infinite_line3d_point3d_distance<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
) -> T {
    line.distance_to_point(point)
}

/// 逆向きラッパー: point-line
pub fn point3d_infinite_line3d_distance<T: Scalar>(
    point: &Point3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    line.distance_to_point(point)
}

/// 無限直線3D-無限直線3D 間の最短距離
pub fn infinite_line3d_infinite_line3d_distance<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
) -> T {
    line_a.distance_to_line(line_b)
}

/// Plane3D-点 間の符号付き距離（法線方向の符号を保持）
pub fn plane3d_point3d_distance<T: Scalar>(plane: &Plane3D<T>, point: &Point3D<T>) -> T {
    plane.distance_to_point(*point)
}

/// 逆向きラッパー: point-plane
pub fn point3d_plane3d_distance<T: Scalar>(point: &Point3D<T>, plane: &Plane3D<T>) -> T {
    plane.distance_to_point(*point)
}

/// Ray3D-点 間の最短距離（Ray の有効範囲を考慮）
pub fn ray3d_point3d_distance<T: Scalar>(ray: &Ray3D<T>, point: &Point3D<T>) -> T {
    ray.distance_to_point(point)
}

/// 逆向きラッパー: point-ray
pub fn point3d_ray3d_distance<T: Scalar>(point: &Point3D<T>, ray: &Ray3D<T>) -> T {
    ray.distance_to_point(point)
}

/// Circle3D-点 間の最短距離（円周への3D空間での距離）
pub fn circle3d_point3d_distance<T: Scalar>(circle: &Circle3D<T>, point: &Point3D<T>) -> T {
    circle.distance_to_point_3d(*point)
}

/// 逆向きラッパー: point-circle
pub fn point3d_circle3d_distance<T: Scalar>(point: &Point3D<T>, circle: &Circle3D<T>) -> T {
    circle.distance_to_point_3d(*point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants;

    fn standard_distance_tol() -> f64 {
        test_constants::DISTANCE_TOLERANCE_F64
    }

    #[test]
    fn infinite_line3d_distance_parallel_lines() {
        let tol = standard_distance_tol();
        let line_a = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line_b = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 2.0, 0.0),
            Point3D::new(1.0, 2.0, 0.0),
        )
        .unwrap();

        let d = infinite_line3d_infinite_line3d_distance(&line_a, &line_b);
        assert!((d - 2.0).abs() < tol);
    }

    #[test]
    fn infinite_line3d_distance_intersecting_lines() {
        let tol = standard_distance_tol();
        let line_a = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line_b = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, -1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let d = infinite_line3d_infinite_line3d_distance(&line_a, &line_b);
        assert!(d.abs() < tol);
    }
}
