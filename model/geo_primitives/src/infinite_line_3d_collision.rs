//! InfiniteLine3D 衝突検出・距離計算実装
//!
//! BasicCollision トレイトの実装

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D};
use geo_contracts::BasicCollision;
use geo_foundation::{Scalar, SphericalSurface3DProperties};

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// InfiniteLine3D vs Point3D
impl<T: Scalar> BasicCollision<T, Point3D<T>> for InfiniteLine3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    fn overlaps(&self, _point: &Point3D<T>, _tolerance: T) -> bool {
        false // 点は重なりを持たない
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        self.distance_to_point(point)
    }
}

// InfiniteLine3D vs Sphere3D
impl<T: Scalar> BasicCollision<T, SphericalSurface3D<T>> for InfiniteLine3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, sphere: &SphericalSurface3D<T>, tolerance: T) -> bool {
        let center_tuple = sphere.center();
        let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
        self.distance_to_point(&center) <= sphere.radius() + tolerance
    }

    fn overlaps(&self, _sphere: &SphericalSurface3D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, sphere: &SphericalSurface3D<T>) -> T {
        let center_tuple = sphere.center();
        let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
        let dist = self.distance_to_point(&center);
        (dist - sphere.radius()).max(T::ZERO)
    }
}

// InfiniteLine3D vs InfiniteLine3D
impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for InfiniteLine3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &InfiniteLine3D<T>, _tolerance: T) -> bool {
        // 平行でなければ交差またはねじれ
        // 同一平面上にある場合のみ交差する
        !self.is_parallel_to(other) && self.is_coplanar_with(other)
    }

    fn overlaps(&self, other: &InfiniteLine3D<T>, tolerance: T) -> bool {
        // 平行かつ距離がゼロ（同一直線）
        if !self.is_parallel_to(other) {
            return false;
        }

        let point_on_other = other.point_internal();
        self.contains_point(&point_on_other, tolerance)
    }

    fn distance_to(&self, other: &InfiniteLine3D<T>) -> T {
        // スキュー直線の最短距離
        if self.is_parallel_to(other) {
            // 平行な場合: 一方の直線上の点から他方への距離
            let point_on_other = other.point_internal();
            self.distance_to_point(&point_on_other)
        } else if self.is_coplanar_with(other) {
            // 同一平面上で交差する場合
            T::ZERO
        } else {
            // スキュー直線の最短距離を計算
            self.distance_to_line(other)
        }
    }
}

// InfiniteLine3D vs LineSegment3D
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for InfiniteLine3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to_line_segment(segment) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の両端点が無限直線上にあるかチェック
        self.contains_point(&segment.start(), tolerance)
            && self.contains_point(&segment.end(), tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        self.distance_to_line_segment(segment)
    }
}

// InfiniteLine3D vs Ray3D
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for InfiniteLine3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to_ray(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        // Rayの起点が無限直線上にあり、方向が一致するかチェック
        if !self.contains_point(&ray.origin(), tolerance) {
            return false;
        }
        let dir_self = self.direction_internal();
        let dir_ray = ray.direction_vector();
        let dir_diff = crate::Vector3D::new(dir_self.x(), dir_self.y(), dir_self.z()) - dir_ray;
        dir_diff.length() <= tolerance
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.distance_to_ray(ray)
    }
}

impl<T: Scalar> InfiniteLine3D<T> {
    /// LineSegment3Dへの最短距離を計算
    fn distance_to_line_segment(&self, segment: &LineSegment3D<T>) -> T {
        // 線分の両端点から無限直線への距離の最小値
        let d1 = self.distance_to_point(&segment.start());
        let d2 = self.distance_to_point(&segment.end());
        d1.min(d2)
    }

    /// Ray3Dへの最短距離を計算
    fn distance_to_ray(&self, ray: &Ray3D<T>) -> T {
        // Rayの起点から無限直線への距離
        self.distance_to_point(&ray.origin())
    }
}
