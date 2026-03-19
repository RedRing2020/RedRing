//! Plane3D 交点計算実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Vector3D};
use geo_contracts::Scalar;
use geo_contracts::{BasicIntersection, SelfIntersection};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// Plane3D vs Point3D
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for Plane3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        if self.contains_point(*point, tolerance) {
            Some(*point)
        } else {
            None
        }
    }
}

// Plane3D vs LineSegment3D（線分と平面の交点）
impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for Plane3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_plane_line_segment_intersection(self, segment, tolerance)
    }
}

// Plane3D vs Ray3D（Rayと平面の交点）
impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for Plane3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ray: &Ray3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_plane_ray_intersection(self, ray, tolerance)
    }
}

// Plane3D vs InfiniteLine3D（無限直線と平面の交点）
impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for Plane3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, _tolerance: T) -> Option<Self::Point> {
        calculate_plane_infinite_line_intersection(self, line)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Plane3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 平面は自己交差しない
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Plane3D と LineSegment3D の交点を計算
fn calculate_plane_line_segment_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    let end = segment.end();
    let direction = Vector3D::from_points(&start, &end);

    if direction.is_zero() {
        // 退化した線分（点）
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    // 平行チェック
    if denom.abs() <= T::EPSILON {
        // 平行: 線分が平面上にあればいずれかの端点を返す
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    // 交点のパラメータ t を計算
    let to_plane = Vector3D::from_points(&start, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

    // t が [0, 1] の範囲内にあるかチェック
    if t >= T::ZERO && t <= T::ONE {
        let intersection = Point3D::new(
            start.x() + t * direction.x(),
            start.y() + t * direction.y(),
            start.z() + t * direction.z(),
        );
        Some(intersection)
    } else {
        None
    }
}

/// Plane3D と Ray3D の交点を計算
fn calculate_plane_ray_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    let direction = ray.direction_vector();

    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    // 平行チェック
    if denom.abs() <= T::EPSILON {
        // 平行: Ray の起点が平面上にあれば起点を返す
        return if plane.contains_point(origin, tolerance) {
            Some(origin)
        } else {
            None
        };
    }

    // 交点のパラメータ t を計算
    let to_plane = Vector3D::from_points(&origin, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

    // t >= 0 のみ有効（Ray の範囲内）
    if t >= T::ZERO {
        let intersection = Point3D::new(
            origin.x() + t * direction.x(),
            origin.y() + t * direction.y(),
            origin.z() + t * direction.z(),
        );
        Some(intersection)
    } else {
        None
    }
}

/// Plane3D と InfiniteLine3D の交点を計算
fn calculate_plane_infinite_line_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
) -> Option<Point3D<T>> {
    let line_point = line.point_internal();
    let line_dir = line.direction_internal();
    let line_dir_vec = Vector3D::new(line_dir.x(), line_dir.y(), line_dir.z());

    let normal = plane.normal().as_vector();
    let denom = line_dir_vec.dot(&normal);

    // 平行チェック
    if denom.abs() <= T::EPSILON {
        // 平行: 直線が平面上にあれば任意の点を返す
        return if plane.distance_to_point(line_point).abs() <= T::EPSILON {
            Some(line_point)
        } else {
            None
        };
    }

    // 交点のパラメータ t を計算
    let to_plane = Vector3D::from_points(&line_point, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

    let intersection = Point3D::new(
        line_point.x() + t * line_dir_vec.x(),
        line_point.y() + t * line_dir_vec.y(),
        line_point.z() + t * line_dir_vec.z(),
    );
    Some(intersection)
}
