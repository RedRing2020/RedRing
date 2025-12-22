//! Arc3D - Intersection Implementation
//!
//! 3次元円弧の交差計算実装

use crate::{Arc3D, InfiniteLine3D, LineSegment3D, Point3D, Ray3D};
use geo_foundation::{extensions::BasicIntersection, Scalar};

// ============================================================================
// Arc3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for Arc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        let center = self.center();
        let to_center = *point - center;
        let distance = to_center.magnitude();

        // 1. 円周上にあるか確認
        if (distance - self.radius()).abs() > tolerance {
            return None;
        }

        // 2. 円弧平面上にあるか確認
        let to_point = *point - center;
        let normal = self.normal().as_vector();
        let plane_distance = to_point.dot(&normal).abs();
        if plane_distance > tolerance {
            return None;
        }

        // 3. 角度範囲内にあるか確認
        if !self.contains_point_angle(*point) {
            return None;
        }

        Some(*point)
    }
}

// ============================================================================
// Arc3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for Arc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 円弧の端点が線分に近ければ返す
        let start = self.start_point();
        let end = self.end_point();

        let dist_start_to_seg_start =
            crate::Vector3D::from_points(&start, &segment.start()).magnitude();
        let dist_start_to_seg_end =
            crate::Vector3D::from_points(&start, &segment.end()).magnitude();

        if dist_start_to_seg_start <= tolerance || dist_start_to_seg_end <= tolerance {
            return Some(start);
        }

        let dist_end_to_seg_start =
            crate::Vector3D::from_points(&end, &segment.start()).magnitude();
        let dist_end_to_seg_end = crate::Vector3D::from_points(&end, &segment.end()).magnitude();

        if dist_end_to_seg_start <= tolerance || dist_end_to_seg_end <= tolerance {
            return Some(end);
        }

        None
    }
}

// ============================================================================
// Arc3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for Arc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ray: &Ray3D<T>, tolerance: T) -> Option<Self::Point> {
        let start = self.start_point();
        let end = self.end_point();
        let ray_origin = ray.origin_internal();

        let dist_start = crate::Vector3D::from_points(&start, &ray_origin).magnitude();
        let dist_end = crate::Vector3D::from_points(&end, &ray_origin).magnitude();

        if dist_start <= tolerance {
            Some(start)
        } else if dist_end <= tolerance {
            Some(end)
        } else {
            None
        }
    }
}

// ============================================================================
// Arc3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for Arc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Option<Self::Point> {
        let start = self.start_point();
        let end = self.end_point();
        let line_point = line.point_internal();

        let dist_start = crate::Vector3D::from_points(&start, &line_point).magnitude();
        let dist_end = crate::Vector3D::from_points(&end, &line_point).magnitude();

        if dist_start <= tolerance {
            Some(start)
        } else if dist_end <= tolerance {
            Some(end)
        } else {
            None
        }
    }
}

// ============================================================================
// Arc3D vs Arc3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc3D<T>> for Arc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, other: &Arc3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 各端点間の最短距離の点を返す
        let self_start = self.start_point();
        let self_end = self.end_point();
        let other_start = other.start_point();
        let other_end = other.end_point();

        let dist1 = crate::Vector3D::from_points(&self_start, &other_start).magnitude();
        let dist2 = crate::Vector3D::from_points(&self_start, &other_end).magnitude();
        let dist3 = crate::Vector3D::from_points(&self_end, &other_start).magnitude();
        let dist4 = crate::Vector3D::from_points(&self_end, &other_end).magnitude();

        let min_dist = dist1.min(dist2).min(dist3).min(dist4);

        if min_dist <= tolerance {
            // 最小距離に対応する点を返す
            if min_dist == dist1 || min_dist == dist2 {
                Some(self_start)
            } else {
                Some(self_end)
            }
        } else {
            None
        }
    }
}
