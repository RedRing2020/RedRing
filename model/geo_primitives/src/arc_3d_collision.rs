//! Arc3D - Collision Implementation
//!
//! 3次元円弧の衝突判定実装

use crate::{Arc3D, InfiniteLine3D, LineSegment3D, Point3D, Ray3D};
use geo_foundation::{extensions::BasicCollision, Scalar};

// ============================================================================
// Arc3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for Arc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // 簡易実装: 始点と終点からの距離チェック
        let start = self.start_point();
        let end = self.end_point();

        let dist_start = crate::Vector3D::from_points(&start, point).magnitude();
        let dist_end = crate::Vector3D::from_points(&end, point).magnitude();

        dist_start <= tolerance || dist_end <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        let start = self.start_point();
        let end = self.end_point();

        let dist_start = crate::Vector3D::from_points(&start, point).magnitude();
        let dist_end = crate::Vector3D::from_points(&end, point).magnitude();

        dist_start.min(dist_end)
    }
}

// ============================================================================
// Arc3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for Arc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 簡易実装: 円弧の端点と線分の端点での距離チェック
        let arc_start = self.start_point();
        let arc_end = self.end_point();

        self.distance_to(&segment.start()) <= tolerance
            || self.distance_to(&segment.end()) <= tolerance
            || {
                let dist_to_arc_start =
                    crate::Vector3D::from_points(&arc_start, &segment.start()).magnitude();
                let dist_to_arc_end =
                    crate::Vector3D::from_points(&arc_end, &segment.start()).magnitude();
                dist_to_arc_start <= tolerance || dist_to_arc_end <= tolerance
            }
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        let dist_start = self.distance_to(&segment.start());
        let dist_end = self.distance_to(&segment.end());
        dist_start.min(dist_end)
    }
}

// ============================================================================
// Arc3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for Arc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(&ray.origin_internal()) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.intersects(ray, tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.distance_to(&ray.origin_internal())
    }
}

// ============================================================================
// Arc3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for Arc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(&line.point_internal()) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.distance_to(&line.point_internal())
    }
}

// ============================================================================
// Arc3D vs Arc3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc3D<T>> for Arc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &Arc3D<T>, tolerance: T) -> bool {
        // 簡易実装: 各端点間の距離チェック
        let self_start = self.start_point();
        let self_end = self.end_point();
        let other_start = other.start_point();
        let other_end = other.end_point();

        let dist1 = crate::Vector3D::from_points(&self_start, &other_start).magnitude();
        let dist2 = crate::Vector3D::from_points(&self_start, &other_end).magnitude();
        let dist3 = crate::Vector3D::from_points(&self_end, &other_start).magnitude();
        let dist4 = crate::Vector3D::from_points(&self_end, &other_end).magnitude();

        dist1 <= tolerance || dist2 <= tolerance || dist3 <= tolerance || dist4 <= tolerance
    }

    fn overlaps(&self, other: &Arc3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &Arc3D<T>) -> T {
        let self_start = self.start_point();
        let self_end = self.end_point();
        let other_start = other.start_point();
        let other_end = other.end_point();

        let dist1 = crate::Vector3D::from_points(&self_start, &other_start).magnitude();
        let dist2 = crate::Vector3D::from_points(&self_start, &other_end).magnitude();
        let dist3 = crate::Vector3D::from_points(&self_end, &other_start).magnitude();
        let dist4 = crate::Vector3D::from_points(&self_end, &other_end).magnitude();

        dist1.min(dist2).min(dist3).min(dist4)
    }
}
