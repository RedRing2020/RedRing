//! SphericalSurface3D - Collision Implementation
//!
//! 3次元球面の衝突判定実装

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D, Vector3D};
use geo_contracts::BasicCollision;
use geo_contracts::Scalar;

// ============================================================================
// SphericalSurface3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for SphericalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let center = self.center_internal();
        let distance = Vector3D::from_points(&center, point).magnitude();
        let radius = self.radius_internal();
        (distance - radius).abs() <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        let center = self.center_internal();
        let distance = Vector3D::from_points(&center, point).magnitude();
        (distance - self.radius_internal()).abs()
    }
}

// ============================================================================
// SphericalSurface3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for SphericalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の端点が球面に近いかチェック
        self.distance_to(&segment.start()) <= tolerance
            || self.distance_to(&segment.end()) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        self.distance_to_line_segment(&segment.start(), &segment.end())
    }
}

// ============================================================================
// SphericalSurface3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for SphericalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(&ray.origin_internal()) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.intersects(ray, tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.distance_to_ray(&ray.origin_internal(), &ray.direction_internal().as_vector())
    }
}

// ============================================================================
// SphericalSurface3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for SphericalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.distance_to_infinite_line(&line.point_internal(), &line.direction_internal().as_vector())
    }
}

// ============================================================================
// SphericalSurface3D vs SphericalSurface3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, SphericalSurface3D<T>> for SphericalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &SphericalSurface3D<T>, tolerance: T) -> bool {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let center_distance = Vector3D::from_points(&center1, &center2).magnitude();
        let sum_radii = self.radius_internal() + other.radius_internal();
        let diff_radii = (self.radius_internal() - other.radius_internal()).abs();

        // 球面が交差または接触する条件
        center_distance <= sum_radii + tolerance && center_distance >= diff_radii - tolerance
    }

    fn overlaps(&self, other: &SphericalSurface3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &SphericalSurface3D<T>) -> T {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let center_distance = Vector3D::from_points(&center1, &center2).magnitude();
        (center_distance - self.radius_internal() - other.radius_internal()).abs()
    }
}
