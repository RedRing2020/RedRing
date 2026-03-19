//! SphericalSolid3D の衝突判定実装
//!
//! 球ソリッドとの衝突判定（含内部）を提供する。
//! 球ソリッドは内部を持つ立体であり、距離計算は表面までの距離または内部からの距離を返す。

use crate::{
    Circle3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSolid3D, Triangle3D,
    Vector3D,
};
use geo_contracts::BasicCollision;
use geo_contracts::{Circle3DProperties, Scalar};

// ============================================================================
// BasicCollision implementations for SphericalSolid3D
// ============================================================================

/// SphericalSolid3D と Point3D の衝突判定
impl<T: Scalar> BasicCollision<T, Point3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        let center = self.center_internal();
        let to_point = Vector3D::from_points(&center, point);
        let distance_from_center = to_point.magnitude();
        let radius = self.radius_internal();

        if distance_from_center <= radius {
            // 内部または表面上にある場合
            T::ZERO
        } else {
            // 外部にある場合
            distance_from_center - radius
        }
    }
}

/// SphericalSolid3D と Circle3D の衝突判定
impl<T: Scalar> BasicCollision<T, Circle3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        // 簡易実装：円の中心との距離
        // TODO: より正確な円と球ソリッドの距離計算
        let (cx, cy, cz) = circle.center();
        let center = Point3D::new(cx, cy, cz);
        self.distance_to(&center)
    }
}

/// SphericalSolid3D と LineSegment3D の衝突判定
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn distance_to(&self, line: &LineSegment3D<T>) -> T {
        self.distance_to_line_segment(&line.start(), &line.end())
    }
}

/// SphericalSolid3D と Ray3D の衝突判定
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.distance_to_ray(
            &ray.origin_internal(),
            &ray.direction_internal().as_vector(),
        )
    }
}

/// SphericalSolid3D と InfiniteLine3D の衝突判定
impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.distance_to_infinite_line(
            &line.point_internal(),
            &line.direction_internal().as_vector(),
        )
    }
}

/// SphericalSolid3D と Triangle3D の衝突判定
impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        use geo_contracts::Triangle3DProperties;

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
        self.distance_to(triangle) <= tolerance
    }

    fn distance_to(&self, triangle: &Triangle3D<T>) -> T {
        use geo_contracts::Triangle3DProperties;

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

/// SphericalSolid3D と Plane3D の衝突判定
impl<T: Scalar> BasicCollision<T, Plane3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        let center = self.center_internal();
        let distance_center_to_plane = plane.distance_to_point(center).abs();
        let radius = self.radius_internal();

        if distance_center_to_plane <= radius {
            // 平面が球を貫通
            T::ZERO
        } else {
            distance_center_to_plane - radius
        }
    }
}

/// SphericalSolid3D と SphericalSolid3D の衝突判定
impl<T: Scalar> BasicCollision<T, SphericalSolid3D<T>> for SphericalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn overlaps(&self, other: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn distance_to(&self, other: &SphericalSolid3D<T>) -> T {
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let center_distance = Vector3D::from_points(&center1, &center2).magnitude();

        let radius1 = self.radius_internal();
        let radius2 = other.radius_internal();

        if center_distance <= (radius1 + radius2) {
            // 交差または包含
            T::ZERO
        } else {
            center_distance - radius1 - radius2
        }
    }
}
