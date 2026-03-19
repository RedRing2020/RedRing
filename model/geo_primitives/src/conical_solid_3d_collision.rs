//! ConicalSolid3D の衝突判定実装
//!
//! 円錐ソリッドとの衝突判定（含内部）を提供する。
//! 円錐ソリッドは内部を持つ立体であり、距離計算は表面までの距離または内部からの距離を返す。

use crate::{
    Circle3D, ConicalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D,
    Vector3D,
};
use geo_contracts::{BasicCollision, Circle3DProperties, Scalar};

// ============================================================================
// BasicCollision implementations for ConicalSolid3D
// ============================================================================

/// ConicalSolid3D と Point3D の衝突判定
impl<T: Scalar> BasicCollision<T, Point3D<T>> for ConicalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        let center = self.center_internal();
        let axis = self.axis_internal();
        let to_point = Vector3D::from_points(&center, point);

        // 軸方向の投影
        let axis_projection = to_point.dot(&axis.as_vector());

        // 高さ範囲外チェック
        if axis_projection < T::ZERO || axis_projection > self.height_internal() {
            // 底面または頂点側の外側
            if axis_projection < T::ZERO {
                // 底面より下
                let perpendicular = to_point - axis.as_vector() * axis_projection;
                let radial_distance = perpendicular.magnitude();
                if radial_distance <= self.radius_internal() {
                    // 真下にある場合
                    return (-axis_projection).abs();
                } else {
                    // 底面エッジからの距離
                    let edge_distance = radial_distance - self.radius_internal();
                    return (edge_distance * edge_distance + axis_projection * axis_projection)
                        .sqrt();
                }
            } else {
                // 頂点より上
                let apex = center + axis.as_vector() * self.height_internal();
                return Vector3D::from_points(&apex, point).magnitude();
            }
        }

        // その高さでの期待半径
        let t = axis_projection / self.height_internal();
        let expected_radius = self.radius_internal() * (T::ONE - t);

        // 半径方向距離
        let perpendicular = to_point - axis.as_vector() * axis_projection;
        let radial_distance = perpendicular.magnitude();

        if radial_distance <= expected_radius {
            // 内部にある場合、表面までの最短距離
            let to_surface = expected_radius - radial_distance;
            let to_base = axis_projection;
            let to_apex = self.height_internal() - axis_projection;
            to_surface.min(to_base).min(to_apex)
        } else {
            // 外部にある場合
            radial_distance - expected_radius
        }
    }
}

/// ConicalSolid3D と Circle3D の衝突判定
impl<T: Scalar> BasicCollision<T, Circle3D<T>> for ConicalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        // 簡易実装：円の中心との距離
        // TODO: より正確な円と円錐ソリッドの距離計算
        let (cx, cy, cz) = circle.center();
        let center = Point3D::new(cx, cy, cz);
        self.distance_to(&center)
    }
}

/// ConicalSolid3D と LineSegment3D の衝突判定
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for ConicalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn distance_to(&self, line: &LineSegment3D<T>) -> T {
        // 簡易実装：端点の最小距離
        // TODO: 線分と円錐ソリッドの正確な距離計算
        let start_point = line.start();
        let end_point = line.end();

        self.distance_to(&start_point)
            .min(self.distance_to(&end_point))
    }
}

/// ConicalSolid3D と Ray3D の衝突判定
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for ConicalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        // 簡易実装：始点との距離
        // TODO: 光線と円錐ソリッドの正確な距離計算
        let origin = ray.origin();
        self.distance_to(&origin)
    }
}

/// ConicalSolid3D と InfiniteLine3D の衝突判定
impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for ConicalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        use geo_contracts::InfiniteLine3DProperties;
        // 簡易実装：通過点との距離
        // TODO: 無限直線と円錐ソリッドの正確な距離計算
        let (px, py, pz) = line.point();
        let point = Point3D::new(px, py, pz);
        self.distance_to(&point)
    }
}

/// ConicalSolid3D と Triangle3D の衝突判定
impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for ConicalSolid3D<T> {
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

/// ConicalSolid3D と Plane3D の衝突判定
impl<T: Scalar> BasicCollision<T, Plane3D<T>> for ConicalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        let center = self.center_internal();
        let axis = self.axis_internal();

        // 底面中心、頂点、いくつかの底面エッジ点を平面との距離でチェック
        let apex = center + axis.as_vector() * self.height_internal();

        let dist_center = plane.distance_to_point(center);
        let dist_apex = plane.distance_to_point(apex);

        dist_center.min(dist_apex)
    }
}

/// ConicalSolid3D と ConicalSolid3D の衝突判定
impl<T: Scalar> BasicCollision<T, ConicalSolid3D<T>> for ConicalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &ConicalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn overlaps(&self, other: &ConicalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn distance_to(&self, other: &ConicalSolid3D<T>) -> T {
        // 簡易実装：中心間距離と半径の組み合わせ
        // TODO: より正確な円錐ソリッド同士の距離計算
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let center_distance = Vector3D::from_points(&center1, &center2).magnitude();

        let max_radius1 = self.radius_internal();
        let max_radius2 = other.radius_internal();

        if center_distance > max_radius1 + max_radius2 {
            center_distance - max_radius1 - max_radius2
        } else {
            T::ZERO
        }
    }
}
