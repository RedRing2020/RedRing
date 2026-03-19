//! EllipsoidalSolid3D の衝突判定実装
//!
//! 楕円体ソリッドとの衝突判定（含内部）を提供する。
//! 楕円体ソリッドは内部を持つ立体であり、距離計算は表面までの距離または内部からの距離を返す。

use crate::{EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Vector3D};
use geo_contracts::{BasicCollision, InfiniteLine3DProperties, Scalar};

// ============================================================================
// BasicCollision implementations for EllipsoidalSolid3D
// ============================================================================

/// EllipsoidalSolid3D と Point3D の衝突判定
impl<T: Scalar> BasicCollision<T, Point3D<T>> for EllipsoidalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        if self.contains_point(point) {
            // 内部または表面上にある場合
            T::ZERO
        } else {
            // 外部にある場合 - 表面までの距離
            self.distance_to_surface(point)
        }
    }
}

/// EllipsoidalSolid3D と LineSegment3D の衝突判定
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for EllipsoidalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn distance_to(&self, line: &LineSegment3D<T>) -> T {
        // 簡易実装: 線分の両端点との距離の最小値
        let start = line.start();
        let end = line.end();

        let dist_start = self.distance_to(&start);
        let dist_end = self.distance_to(&end);

        dist_start.min(dist_end)
    }
}

/// EllipsoidalSolid3D と Ray3D の衝突判定
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for EllipsoidalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        // 簡易実装: Ray の起点との距離
        let origin = ray.origin_internal();
        self.distance_to(&origin)
    }
}

/// EllipsoidalSolid3D と InfiniteLine3D の衝突判定
impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for EllipsoidalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        // 簡易実装: 直線上の点との距離
        let (px, py, pz) = line.point();
        let point_on_line = Point3D::new(px, py, pz);
        self.distance_to(&point_on_line)
    }
}

/// EllipsoidalSolid3D と Plane3D の衝突判定
impl<T: Scalar> BasicCollision<T, Plane3D<T>> for EllipsoidalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        // 簡易実装: 中心と平面の距離から最大半径を引く
        let center = self.center_internal();
        let distance_center = plane.distance_to_point(center).abs();

        // 最大半径（簡易的に3軸の最大値を使用）
        let max_radius = self
            .a_radius_internal()
            .max(self.b_radius_internal())
            .max(self.c_radius_internal());

        if distance_center <= max_radius {
            T::ZERO
        } else {
            distance_center - max_radius
        }
    }
}

/// EllipsoidalSolid3D と EllipsoidalSolid3D の衝突判定
impl<T: Scalar> BasicCollision<T, EllipsoidalSolid3D<T>> for EllipsoidalSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn overlaps(&self, other: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn distance_to(&self, other: &EllipsoidalSolid3D<T>) -> T {
        // 簡易実装: 中心間の距離から両方の最大半径を引く
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let center_distance = Vector3D::from_points(&center1, &center2).magnitude();

        let max_radius1 = self
            .a_radius_internal()
            .max(self.b_radius_internal())
            .max(self.c_radius_internal());

        let max_radius2 = other
            .a_radius_internal()
            .max(other.b_radius_internal())
            .max(other.c_radius_internal());

        let combined_radius = max_radius1 + max_radius2;

        if center_distance <= combined_radius {
            T::ZERO
        } else {
            center_distance - combined_radius
        }
    }
}
