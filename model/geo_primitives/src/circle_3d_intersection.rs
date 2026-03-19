//! Circle3D - Intersection Implementation
//!
//! 3次元円の交差計算実装

use crate::{Circle3D, InfiniteLine3D, LineSegment3D, Point3D, Ray3D};
use geo_contracts::BasicIntersection;
use geo_contracts::Scalar;

// ============================================================================
// Circle3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for Circle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        if self.contains_point_3d(*point) || self.distance_to_point_3d(*point) <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// Circle3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for Circle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, _tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 線分の開始点が円に近ければ返す
        if self.contains_point_3d(segment.start()) {
            return Some(segment.start());
        }
        if self.contains_point_3d(segment.end()) {
            return Some(segment.end());
        }
        None
    }
}

// ============================================================================
// Circle3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for Circle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ray: &Ray3D<T>, _tolerance: T) -> Option<Self::Point> {
        // 簡易実装: レイの原点が円上にあれば返す
        if self.contains_point_3d(ray.origin_internal()) {
            Some(ray.origin_internal())
        } else {
            None
        }
    }
}

// ============================================================================
// Circle3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for Circle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, _tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 直線の原点が円上にあれば返す
        if self.contains_point_3d(line.point_internal()) {
            Some(line.point_internal())
        } else {
            None
        }
    }
}

// ============================================================================
// Circle3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle3D<T>> for Circle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, _other: &Circle3D<T>, _tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 未実装
        None
    }
}
