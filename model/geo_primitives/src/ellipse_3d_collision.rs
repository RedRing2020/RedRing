//! Ellipse3D - Collision Implementation
//!
//! 3次元楕円の衝突判定実装
//! Phase 3.1: BasicCollision 実装

use crate::{Arc3D, Circle3D, Ellipse3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D, Vector3D};
use geo_foundation::{extensions::BasicCollision, Scalar};

// ============================================================================
// Ellipse3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        self.distance_to_point(point)
    }
}

// ============================================================================
// Ellipse3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        let dist = self.distance_to(&circle.center_internal());
        dist <= circle.radius_internal() + tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        let center_dist = Vector3D::from_points(&self.center(), &circle.center_internal()).length();
        center_dist + self.semi_major_axis() <= circle.radius_internal() + tolerance
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        let dist = self.distance_to(&circle.center_internal());
        (dist - circle.radius_internal()).max(T::ZERO)
    }
}

// ============================================================================
// Ellipse3D vs Arc3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, arc: &Arc3D<T>, tolerance: T) -> bool {
        let dist = self.distance_to(&arc.center());
        dist <= arc.radius() + tolerance
    }

    fn overlaps(&self, _arc: &Arc3D<T>, _tolerance: T) -> bool {
        false // 簡易実装
    }

    fn distance_to(&self, arc: &Arc3D<T>) -> T {
        let dist = self.distance_to(&arc.center());
        (dist - arc.radius()).max(T::ZERO)
    }
}

// ============================================================================
// Ellipse3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
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
        self.distance_to(&segment.start()) <= tolerance
            && self.distance_to(&segment.end()) <= tolerance
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        let dist_start = self.distance_to(&segment.start());
        let dist_end = self.distance_to(&segment.end());
        dist_start.min(dist_end)
    }
}

// ============================================================================
// Ellipse3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(&line.point_internal()) <= tolerance
    }

    fn overlaps(&self, _line: &InfiniteLine3D<T>, _tolerance: T) -> bool {
        false // 直線と楕円は重なりを持たない
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.distance_to(&line.point_internal())
    }
}

// ============================================================================
// Ellipse3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(&ray.origin_internal()) <= tolerance
    }

    fn overlaps(&self, _ray: &Ray3D<T>, _tolerance: T) -> bool {
        false
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.distance_to(&ray.origin_internal())
    }
}

// ============================================================================
// Ellipse3D vs Plane3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        // 楕円の中心から平面までの距離
        let dist = plane.distance_to_point(self.center());
        dist <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        // 楕円が平面上にあるか確認
        let dist = plane.distance_to_point(self.center());
        if dist > tolerance {
            return false;
        }

        // 楕円の法線と平面の法線が平行か確認
        let normal_dot = self.normal().dot(&plane.normal_internal()).abs();
        normal_dot >= T::ONE - tolerance
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        plane.distance_to_point(self.center())
    }
}

// ============================================================================
// Ellipse3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        // 三角形の頂点が楕円の近傍にあるか
        if self.distance_to(&triangle.vertex_a_internal()) <= tolerance
            || self.distance_to(&triangle.vertex_b_internal()) <= tolerance
            || self.distance_to(&triangle.vertex_c_internal()) <= tolerance
        {
            return true;
        }

        // 楕円の中心が三角形の近傍にあるか
        let centroid = triangle.centroid();
        self.distance_to(&centroid) <= tolerance
    }

    fn overlaps(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        self.distance_to(&triangle.vertex_a_internal()) <= tolerance
            && self.distance_to(&triangle.vertex_b_internal()) <= tolerance
            && self.distance_to(&triangle.vertex_c_internal()) <= tolerance
    }

    fn distance_to(&self, triangle: &Triangle3D<T>) -> T {
        let dist_a = self.distance_to(&triangle.vertex_a_internal());
        let dist_b = self.distance_to(&triangle.vertex_b_internal());
        let dist_c = self.distance_to(&triangle.vertex_c_internal());

        dist_a.min(dist_b).min(dist_c)
    }
}

// ============================================================================
// Ellipse3D vs Ellipse3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ellipse3D<T>> for Ellipse3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &Ellipse3D<T>, tolerance: T) -> bool {
        let center_dist = Vector3D::from_points(&self.center(), &other.center()).length();
        let sum_semi_major = self.semi_major_axis() + other.semi_major_axis();

        center_dist <= sum_semi_major + tolerance
    }

    fn overlaps(&self, other: &Ellipse3D<T>, tolerance: T) -> bool {
        let dist_to_other = self.distance_to(&other.center());
        dist_to_other <= tolerance
    }

    fn distance_to(&self, other: &Ellipse3D<T>) -> T {
        let center_dist = Vector3D::from_points(&self.center(), &other.center()).length();
        let radii_sum = self.semi_major_axis() + other.semi_major_axis();
        (center_dist - radii_sum).max(T::ZERO)
    }
}
