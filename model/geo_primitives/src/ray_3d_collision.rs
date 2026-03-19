//! Ray3D 衝突検出・距離計算実装
//!
//! BasicCollision トレイトの実装

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D};
use geo_contracts::BasicCollision;
use geo_foundation::{Scalar, SphericalSurface3DProperties};

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// Ray3D vs Point3D
impl<T: Scalar> BasicCollision<T, Point3D<T>> for Ray3D<T> {
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

// Ray3D vs SphericalSurface3D
impl<T: Scalar> BasicCollision<T, SphericalSurface3D<T>> for Ray3D<T> {
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

// Ray3D vs Ray3D
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for Ray3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &Ray3D<T>, tolerance: T) -> bool {
        // 簡易実装: 起点間の距離をチェック
        let dist = self.origin().distance_to(&other.origin());
        dist <= tolerance
    }

    fn overlaps(&self, other: &Ray3D<T>, tolerance: T) -> bool {
        // 起点が一致し、方向が同じ
        let origin_dist = self.origin().distance_to(&other.origin());
        let dir_self = self.direction_vector();
        let dir_other = other.direction_vector();

        let dir_diff = dir_self - dir_other;

        origin_dist <= tolerance && dir_diff.length() <= tolerance
    }

    fn distance_to(&self, other: &Ray3D<T>) -> T {
        let dist1 = self.distance_to_point(&other.origin());
        let dist2 = other.distance_to_point(&self.origin());
        dist1.min(dist2)
    }
}

// Ray3D vs LineSegment3D
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for Ray3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to_line_segment(segment) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の両端点がRay上にあるかチェック
        self.contains_point(&segment.start(), tolerance)
            && self.contains_point(&segment.end(), tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        self.distance_to_line_segment(segment)
    }
}

// Ray3D vs InfiniteLine3D
impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for Ray3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to_infinite_line(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        // Rayの起点が無限直線上にあり、方向が一致するかチェック
        if !line.contains_point(&self.origin(), tolerance) {
            return false;
        }
        let dir_self = self.direction_vector();
        let dir_line = line.direction_internal();
        let dir_diff = dir_self - crate::Vector3D::new(dir_line.x(), dir_line.y(), dir_line.z());
        dir_diff.length() <= tolerance
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.distance_to_infinite_line(line)
    }
}

impl<T: Scalar> Ray3D<T> {
    /// LineSegment3Dへの最短距離を計算
    fn distance_to_line_segment(&self, segment: &LineSegment3D<T>) -> T {
        // Rayの起点から線分への距離と、線分の端点からRayへの距離の最小値
        let d1 = segment.distance_to_point(&self.origin());
        let d2 = self.distance_to_point(&segment.start());
        let d3 = self.distance_to_point(&segment.end());
        d1.min(d2).min(d3)
    }

    /// InfiniteLine3Dへの最短距離を計算
    fn distance_to_infinite_line(&self, line: &InfiniteLine3D<T>) -> T {
        // Rayの起点から無限直線への距離
        line.distance_to_point(&self.origin())
    }
}
