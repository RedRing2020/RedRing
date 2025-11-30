//! Triangle3D 交点計算実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{LineSegment3D, Point3D, Ray3D, Triangle3D, Vector3D};
use geo_foundation::{
    extensions::{BasicIntersection, SelfIntersection},
    Scalar,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// Triangle3D vs Point3D
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for Triangle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        if self.distance_to_point(point) <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// Triangle3D vs LineSegment3D（線分と三角形の交点）
impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for Triangle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_triangle_segment_intersection(self, segment, tolerance)
    }
}

// Triangle3D vs Ray3D（Rayと三角形の交点）
impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for Triangle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ray: &Ray3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_triangle_ray_intersection(self, ray, tolerance)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Triangle3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 三角形は自己交差しない
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Triangle3D と LineSegment3D の交点を計算（簡易実装）
fn calculate_triangle_segment_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // 線分の両端点が三角形上にあるかチェック
    let start = segment.start();
    let end = segment.end();

    if triangle.distance_to_point(&start) <= tolerance {
        return Some(start);
    }
    if triangle.distance_to_point(&end) <= tolerance {
        return Some(end);
    }

    // 簡易実装: 平面との交点計算は複雑なため、None を返す
    // 完全な実装は Möller-Trumbore アルゴリズムなどが必要
    None
}

/// Triangle3D と Ray3D の交点を計算（Möller-Trumbore アルゴリズムの簡易版）
fn calculate_triangle_ray_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    // Möller-Trumbore アルゴリズム
    let v0 = triangle.vertex_a_internal();
    let v1 = triangle.vertex_b_internal();
    let v2 = triangle.vertex_c_internal();

    let edge1 = Vector3D::from_points(&v0, &v1);
    let edge2 = Vector3D::from_points(&v0, &v2);

    let ray_dir = ray.direction_vector();
    let h = ray_dir.cross(&edge2);
    let a = edge1.dot(&h);

    // Ray が三角形平面に平行な場合
    if a.abs() < T::EPSILON {
        return None;
    }

    let f = T::ONE / a;
    let s = Vector3D::from_points(&v0, &ray.origin());
    let u = f * s.dot(&h);

    if u < T::ZERO || u > T::ONE {
        return None;
    }

    let q = s.cross(&edge1);
    let v = f * ray_dir.dot(&q);

    if v < T::ZERO || u + v > T::ONE {
        return None;
    }

    // 交点のパラメータ t を計算
    let t = f * edge2.dot(&q);

    // t >= 0 のみ有効（Ray の範囲内）
    if t >= T::ZERO {
        let intersection = Point3D::new(
            ray.origin().x() + t * ray_dir.x(),
            ray.origin().y() + t * ray_dir.y(),
            ray.origin().z() + t * ray_dir.z(),
        );
        Some(intersection)
    } else {
        None
    }
}
