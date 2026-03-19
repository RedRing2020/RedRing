//! Circle3D - Collision Implementation
//!
//! 3次元円の衝突判定実装

use crate::{Circle3D, InfiniteLine3D, LineSegment3D, Point3D, Ray3D, Vector3D};
use geo_contracts::BasicCollision;
use geo_contracts::Scalar;

// ============================================================================
// Circle3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for Circle3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to_point_3d(*point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        self.distance_to_point_3d(*point)
    }
}

// ============================================================================
// Circle3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for Circle3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 簡易実装: 線分の端点と中点での距離チェック
        let dist_start = self.distance_to(&segment.start());
        let dist_end = self.distance_to(&segment.end());

        if dist_start <= tolerance || dist_end <= tolerance {
            return true;
        }

        // 中点チェック
        let mid_x = (segment.start().x() + segment.end().x()) / T::from_f64(2.0);
        let mid_y = (segment.start().y() + segment.end().y()) / T::from_f64(2.0);
        let mid_z = (segment.start().z() + segment.end().z()) / T::from_f64(2.0);
        let mid_point = Point3D::new(mid_x, mid_y, mid_z);

        self.distance_to(&mid_point) <= tolerance
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
// Circle3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for Circle3D<T> {
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
// Circle3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for Circle3D<T> {
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
// Circle3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for Circle3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &Circle3D<T>, tolerance: T) -> bool {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let to_other = Vector3D::from_points(&center1, &center2);
        let center_distance = to_other.magnitude();
        let sum_radii = self.radius_internal() + other.radius_internal();

        center_distance <= sum_radii + tolerance
    }

    fn overlaps(&self, other: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &Circle3D<T>) -> T {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let to_other = Vector3D::from_points(&center1, &center2);
        let center_distance = to_other.magnitude();
        (center_distance - self.radius_internal() - other.radius_internal()).max(T::ZERO)
    }
}
