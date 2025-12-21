//! CylindricalSurface3D - Collision Implementation
//!
//! 3次元円柱サーフェスの衝突判定実装
//!
//! 円柱サーフェスは無限に延びる円柱面であり、軸からの距離が
//! 半径に等しい点の集合として定義される。

use crate::{
    Circle3D, CylindricalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D,
    Triangle3D, Vector3D,
};
use geo_foundation::{extensions::BasicCollision, Scalar};

// ============================================================================
// CylindricalSurface3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to(point);
        distance <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 点から軸への垂直距離を計算
        let center = self.center_internal();
        let axis = self.axis();
        let to_point = Vector3D::from_points(&center, point);
        
        // 軸方向成分を除去して、軸に垂直な成分のみを取得
        let axis_component = to_point.dot(&axis.as_vector());
        let perpendicular = to_point - axis.as_vector() * axis_component;
        let radial_distance = perpendicular.magnitude();
        
        // 円柱面までの距離は、軸からの距離と半径の差の絶対値
        (radial_distance - self.radius()).abs()
    }
}

// ============================================================================
// CylindricalSurface3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        use geo_foundation::Circle3DProperties;
        // 円の中心点との距離をチェック
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.distance_to(&center_point) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        use geo_foundation::Circle3DProperties;
        // 簡易実装: 円の中心点との距離を返す
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.distance_to(&center_point)
    }
}

// ============================================================================
// CylindricalSurface3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の端点との距離をチェック
        self.distance_to(&segment.start()) <= tolerance
            || self.distance_to(&segment.end()) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        // 簡易実装: 両端点との距離の最小値
        let dist_start = self.distance_to(&segment.start());
        let dist_end = self.distance_to(&segment.end());
        dist_start.min(dist_end)
    }
}

// ============================================================================
// CylindricalSurface3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        // 光線の始点との距離をチェック
        self.distance_to(&ray.origin_internal()) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.intersects(ray, tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        // 簡易実装: 光線の始点との距離
        self.distance_to(&ray.origin_internal())
    }
}

// ============================================================================
// CylindricalSurface3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        // 直線上の点との距離をチェック
        self.distance_to(&line.point_internal()) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        // 簡易実装: 直線上の基準点との距離
        self.distance_to(&line.point_internal())
    }
}

// ============================================================================
// CylindricalSurface3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        use geo_foundation::Triangle3DProperties;
        
        // 三角形の各頂点との距離をチェック
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
        
        // 簡易実装: 三角形の頂点との距離の最小値
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
// CylindricalSurface3D vs Plane3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        // 円柱の中心点が平面に近いかチェック
        let center = self.center_internal();
        plane.distance_to(&center).abs() <= tolerance + self.radius()
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.intersects(plane, tolerance)
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        // 簡易実装: 中心点から平面への距離から半径を引いた値
        let center = self.center_internal();
        let dist_to_plane = plane.distance_to(&center).abs();
        if dist_to_plane > self.radius() {
            dist_to_plane - self.radius()
        } else {
            T::ZERO
        }
    }
}

// ============================================================================
// CylindricalSurface3D vs CylindricalSurface3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, CylindricalSurface3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &CylindricalSurface3D<T>, tolerance: T) -> bool {
        // 2つの円柱サーフェスの軸が平行かどうかで場合分け
        let axis1 = self.axis();
        let axis2 = other.axis();
        
        // 軸の平行度チェック
        let cross = axis1.as_vector().cross(&axis2.as_vector());
        let is_parallel = cross.magnitude() < tolerance;
        
        if is_parallel {
            // 軸が平行な場合: 軸間の距離が半径の和以下なら交差
            let center1 = self.center_internal();
            let center2 = other.center_internal();
            let between_centers = Vector3D::from_points(&center1, &center2);
            
            // 軸方向成分を除去
            let axis_component = between_centers.dot(&axis1.as_vector());
            let perpendicular = between_centers - axis1.as_vector() * axis_component;
            let axis_distance = perpendicular.magnitude();
            
            let radius_sum = self.radius() + other.radius();
            let radius_diff = (self.radius() - other.radius()).abs();
            
            axis_distance <= radius_sum + tolerance && axis_distance >= radius_diff - tolerance
        } else {
            // 軸が交差する場合: 簡易実装として、中心点間の距離をチェック
            let center1 = self.center_internal();
            let center2 = other.center_internal();
            let dist = Vector3D::from_points(&center1, &center2).magnitude();
            dist <= (self.radius() + other.radius()) + tolerance
        }
    }

    fn overlaps(&self, other: &CylindricalSurface3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &CylindricalSurface3D<T>) -> T {
        // 簡易実装: 中心点間の距離から両半径の和を引いた値
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let dist = Vector3D::from_points(&center1, &center2).magnitude();
        let radius_sum = self.radius() + other.radius();
        if dist > radius_sum {
            dist - radius_sum
        } else {
            T::ZERO
        }
    }
}
