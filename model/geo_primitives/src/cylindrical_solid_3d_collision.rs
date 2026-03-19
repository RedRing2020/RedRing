//! CylindricalSolid3D - Collision Implementation
//!
//! 3次元円柱ソリッドの衝突判定実装
//!
//! 円柱ソリッドは有限の立体であり、内部判定と表面判定の両方が必要。

use crate::{
    Circle3D, CylindricalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D,
    Triangle3D, Vector3D,
};
use geo_contracts::BasicCollision;
use geo_contracts::Scalar;

// ============================================================================
// CylindricalSolid3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for CylindricalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to(point);
        distance <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // Solid の場合、overlaps は内部または表面上にあることを意味
        // 既存の contains_point_internal を使用し、tolerance 考慮のため distance_to で判定
        self.distance_to(point) <= tolerance
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 点から円柱ソリッドまでの最短距離
        let center = self.center_internal();
        let axis = self.axis();
        let to_point = Vector3D::from_points(&center, point);

        // 軸方向の投影
        let axis_projection = to_point.dot(&axis.as_vector());

        // 高さ範囲内かチェック
        let height = self.height();
        let axis_dist = if axis_projection < T::ZERO {
            -axis_projection // 底面より下
        } else if axis_projection > height {
            axis_projection - height // 上面より上
        } else {
            T::ZERO // 高さ範囲内
        };

        // 軸からの半径方向距離
        let perpendicular = to_point - axis.as_vector() * axis_projection;
        let radial_distance = perpendicular.magnitude();
        let radius = self.radius();
        let radial_dist = if radial_distance > radius {
            radial_distance - radius
        } else {
            T::ZERO
        };

        // 距離の合成（両方ゼロなら内部または表面上）
        if axis_dist.is_zero() && radial_dist.is_zero() {
            T::ZERO
        } else if axis_dist.is_zero() {
            radial_dist
        } else if radial_dist.is_zero() {
            axis_dist
        } else {
            (axis_dist * axis_dist + radial_dist * radial_dist).sqrt()
        }
    }
}

// ============================================================================
// CylindricalSolid3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for CylindricalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        use geo_contracts::Circle3DProperties;
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.distance_to(&center_point) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        use geo_contracts::Circle3DProperties;
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.distance_to(&center_point)
    }
}

// ============================================================================
// CylindricalSolid3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for CylindricalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 両端点が内部または表面上にあるか、または貫通しているか
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
// CylindricalSolid3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for CylindricalSolid3D<T> {
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
// CylindricalSolid3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for CylindricalSolid3D<T> {
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
// CylindricalSolid3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for CylindricalSolid3D<T> {
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
        self.intersects(triangle, tolerance)
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

// ============================================================================
// CylindricalSolid3D vs Plane3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for CylindricalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        // 円柱の中心点が平面に近いかチェック
        let center = self.center_internal();
        let dist_center = plane.distance_to(&center).abs();

        // 簡易判定：中心からの距離が半径+高さ以内
        let max_extent = self.radius() + self.height();
        dist_center <= tolerance + max_extent
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.intersects(plane, tolerance)
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        // 簡易実装：中心点から平面への距離
        let center = self.center_internal();
        let dist = plane.distance_to(&center).abs();
        let max_extent = self.radius() + self.height();
        if dist > max_extent {
            dist - max_extent
        } else {
            T::ZERO
        }
    }
}

// ============================================================================
// CylindricalSolid3D vs CylindricalSolid3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, CylindricalSolid3D<T>> for CylindricalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        // 簡易実装：中心点間の距離をチェック
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let dist = Vector3D::from_points(&center1, &center2).magnitude();
        let max_extent = self.radius() + self.height() + other.radius() + other.height();
        dist <= max_extent + tolerance
    }

    fn overlaps(&self, other: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &CylindricalSolid3D<T>) -> T {
        // 簡易実装：中心点間の距離から最大範囲を引く
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let dist = Vector3D::from_points(&center1, &center2).magnitude();
        let max_extent = self.radius() + self.height() + other.radius() + other.height();
        if dist > max_extent {
            dist - max_extent
        } else {
            T::ZERO
        }
    }
}
