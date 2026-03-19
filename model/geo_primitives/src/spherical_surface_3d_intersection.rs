//! SphericalSurface3D - Intersection Implementation
//!
//! 3次元球面の交差計算実装

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D, Vector3D};
use geo_contracts::BasicIntersection;
use geo_contracts::Scalar;

// ============================================================================
// SphericalSurface3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for SphericalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        let center = self.center_internal();
        let distance = Vector3D::from_points(&center, point).magnitude();
        let radius = self.radius_internal();

        if (distance - radius).abs() <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// SphericalSurface3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for SphericalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 線分の端点が球面上にあれば返す
        let center = self.center_internal();
        let radius = self.radius_internal();

        let dist_start = Vector3D::from_points(&center, &segment.start()).magnitude();
        if (dist_start - radius).abs() <= tolerance {
            return Some(segment.start());
        }

        let dist_end = Vector3D::from_points(&center, &segment.end()).magnitude();
        if (dist_end - radius).abs() <= tolerance {
            return Some(segment.end());
        }

        None
    }
}

// ============================================================================
// SphericalSurface3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for SphericalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ray: &Ray3D<T>, tolerance: T) -> Option<Self::Point> {
        let center = self.center_internal();
        let radius = self.radius_internal();
        let ray_origin = ray.origin_internal();

        let distance = Vector3D::from_points(&center, &ray_origin).magnitude();

        if (distance - radius).abs() <= tolerance {
            Some(ray_origin)
        } else {
            None
        }
    }
}

// ============================================================================
// SphericalSurface3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for SphericalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Option<Self::Point> {
        let center = self.center_internal();
        let radius = self.radius_internal();
        let line_point = line.point_internal();

        let distance = Vector3D::from_points(&center, &line_point).magnitude();

        if (distance - radius).abs() <= tolerance {
            Some(line_point)
        } else {
            None
        }
    }
}

// ============================================================================
// SphericalSurface3D vs SphericalSurface3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, SphericalSurface3D<T>> for SphericalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(
        &self,
        other: &SphericalSurface3D<T>,
        tolerance: T,
    ) -> Option<Self::Point> {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let center_vec = Vector3D::from_points(&center1, &center2);
        let center_distance = center_vec.magnitude();
        let sum_radii = self.radius_internal() + other.radius_internal();

        // 球面が接触する場合、接点を返す
        if (center_distance - sum_radii).abs() <= tolerance {
            let t = self.radius_internal() / center_distance;
            let intersection_point = Point3D::new(
                center1.x() + center_vec.x() * t,
                center1.y() + center_vec.y() * t,
                center1.z() + center_vec.z() * t,
            );
            Some(intersection_point)
        } else {
            None
        }
    }
}
