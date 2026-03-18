//! ConicalSurface3D - Collision Implementation
//!
//! 3次元円錐サーフェスの衝突判定実装
//!
//! 円錐サーフェスは厚みのない曲面であり、距離計算は表面までの最短距離を返す。

use crate::{
    Circle3D, ConicalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D,
    Vector3D,
};
use geo_contracts::BasicCollision;
use geo_foundation::Scalar;

// ============================================================================
// ConicalSurface3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for ConicalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to(point);
        distance <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 点から円錐サーフェスまでの最短距離
        let center = self.center_internal();
        let axis = self.axis_internal();
        let to_point = Vector3D::from_points(&center, point);

        // 軸方向の投影
        let axis_projection = to_point.dot(&axis.as_vector());

        // その高さでの期待半径
        let expected_radius =
            self.radius_internal() + axis_projection * self.semi_angle_internal().tan();

        // 半径方向距離
        let perpendicular = to_point - axis.as_vector() * axis_projection;
        let radial_distance = perpendicular.magnitude();

        // サーフェスまでの距離（半径方向のずれ）
        (radial_distance - expected_radius).abs()
    }
}

// ============================================================================
// ConicalSurface3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for ConicalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        use geo_foundation::Circle3DProperties;
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.distance_to(&center_point) <= tolerance + circle.radius()
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        use geo_foundation::Circle3DProperties;
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        let dist_to_center = self.distance_to(&center_point);
        if dist_to_center > circle.radius() {
            dist_to_center - circle.radius()
        } else {
            T::ZERO
        }
    }
}

// ============================================================================
// ConicalSurface3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for ConicalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(&segment.start()) <= tolerance
            || self.distance_to(&segment.end()) <= tolerance
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
// ConicalSurface3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for ConicalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        let ref_point = line.point_at_parameter(T::ZERO);
        self.distance_to(&ref_point) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        let ref_point = line.point_at_parameter(T::ZERO);
        self.distance_to(&ref_point)
    }
}

// ============================================================================
// ConicalSurface3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for ConicalSurface3D<T> {
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
// ConicalSurface3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for ConicalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        use geo_foundation::Triangle3DProperties;

        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

        self.distance_to(&va) <= tolerance
            || self.distance_to(&vb) <= tolerance
            || self.distance_to(&vc) <= tolerance
    }

    fn overlaps(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        self.intersects(triangle, tolerance)
    }

    fn distance_to(&self, triangle: &Triangle3D<T>) -> T {
        use geo_foundation::Triangle3DProperties;

        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

        let dist_a = self.distance_to(&va);
        let dist_b = self.distance_to(&vb);
        let dist_c = self.distance_to(&vc);

        dist_a.min(dist_b).min(dist_c)
    }
}

// ============================================================================
// ConicalSurface3D vs Plane3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for ConicalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        let center = self.center_internal();
        self.distance_to(&plane.origin()) <= tolerance
            || plane.distance_to_point(center) <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.intersects(plane, tolerance)
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        let center = self.center_internal();
        plane
            .distance_to_point(center)
            .min(self.distance_to(&plane.origin()))
    }
}

// ============================================================================
// ConicalSurface3D vs ConicalSurface3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, ConicalSurface3D<T>> for ConicalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &ConicalSurface3D<T>, tolerance: T) -> bool {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        self.distance_to(&center2) <= tolerance || other.distance_to(&center1) <= tolerance
    }

    fn overlaps(&self, other: &ConicalSurface3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &ConicalSurface3D<T>) -> T {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        self.distance_to(&center2).min(other.distance_to(&center1))
    }
}
