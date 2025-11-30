//! LineSegment3D 衝突検出・距離計算実装
//!
//! BasicCollision トレイトの実装
//! LineSegment3D と他の幾何形状との組み合わせを実装

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D};
use geo_foundation::{extensions::BasicCollision, Scalar, SphericalSurface3DProperties};

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// LineSegment3D vs Point3D
impl<T: Scalar> BasicCollision<T, Point3D<T>> for LineSegment3D<T> {
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

// LineSegment3D vs SphericalSurface3D
impl<T: Scalar> BasicCollision<T, SphericalSurface3D<T>> for LineSegment3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, sphere: &SphericalSurface3D<T>, tolerance: T) -> bool {
        self.distance_to_sphere(sphere) <= tolerance
    }

    fn overlaps(&self, sphere: &SphericalSurface3D<T>, tolerance: T) -> bool {
        // 線分の端点が球内にあるか、または交差する
        let start = self.start();
        let end = self.end();
        let center_tuple = sphere.center();
        let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);

        let start_inside = start.distance_to(&center) <= sphere.radius() + tolerance;
        let end_inside = end.distance_to(&center) <= sphere.radius() + tolerance;

        start_inside || end_inside || self.intersects(sphere, tolerance)
    }

    fn distance_to(&self, sphere: &SphericalSurface3D<T>) -> T {
        self.distance_to_sphere(sphere)
    }
}

impl<T: Scalar> LineSegment3D<T> {
    /// 球への最短距離を計算
    fn distance_to_sphere(&self, sphere: &SphericalSurface3D<T>) -> T {
        // 線分の中心から球への最短距離を計算し、半径を引く
        let center_tuple = sphere.center();
        let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);

        let distance_to_center = self.distance_to_point(&center);
        (distance_to_center - sphere.radius()).max(T::ZERO)
    }
}

// LineSegment3D vs LineSegment3D
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for LineSegment3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn overlaps(&self, other: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &LineSegment3D<T>) -> T {
        // 線分と線分の最短距離（簡易実装）
        // 各端点から相手の線分への距離の最小値
        let p1_start = self.start();
        let p1_end = self.end();
        let p2_start = other.start();
        let p2_end = other.end();

        let d1 = self.distance_to_point(&p2_start);
        let d2 = self.distance_to_point(&p2_end);
        let d3 = other.distance_to_point(&p1_start);
        let d4 = other.distance_to_point(&p1_end);

        d1.min(d2).min(d3).min(d4)
    }
}

// LineSegment3D vs Ray3D
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for LineSegment3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to_ray(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        // 線分の両端点がRay上にあるかチェック
        ray.contains_point(&self.start(), tolerance) && ray.contains_point(&self.end(), tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.distance_to_ray(ray)
    }
}

// LineSegment3D vs InfiniteLine3D
impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for LineSegment3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to_infinite_line(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        // 線分の両端点が無限直線上にあるかチェック
        line.contains_point(&self.start(), tolerance) && line.contains_point(&self.end(), tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.distance_to_infinite_line(line)
    }
}

impl<T: Scalar> LineSegment3D<T> {
    /// Ray3Dへの最短距離を計算
    fn distance_to_ray(&self, ray: &Ray3D<T>) -> T {
        // 線分の両端点からRayへの距離の最小値
        let d1 = ray.distance_to_point(&self.start());
        let d2 = ray.distance_to_point(&self.end());
        // Rayの起点から線分への距離も考慮
        let d3 = self.distance_to_point(&ray.origin());
        d1.min(d2).min(d3)
    }

    /// InfiniteLine3Dへの最短距離を計算
    fn distance_to_infinite_line(&self, line: &InfiniteLine3D<T>) -> T {
        // 線分の両端点から無限直線への距離の最小値
        let d1 = line.distance_to_point(&self.start());
        let d2 = line.distance_to_point(&self.end());
        d1.min(d2)
    }
}
