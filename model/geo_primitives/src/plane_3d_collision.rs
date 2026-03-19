//! Plane3D 衝突検出・距離計算実装
//!
//! BasicCollision トレイトの実装
//! Plane3D と他の幾何形状との組み合わせを実装

use crate::{InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D};
use geo_contracts::BasicCollision;
use geo_contracts::Scalar;

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// Plane3D vs Point3D
impl<T: Scalar> BasicCollision<T, Point3D<T>> for Plane3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.contains_point(*point, tolerance)
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        self.distance_to_point(*point)
    }
}

// Plane3D vs LineSegment3D
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for Plane3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の両端点が平面の異なる側にあるか、または平面上にあるか
        let start_dist = self.distance_to_point(segment.start());
        let end_dist = self.distance_to_point(segment.end());

        // 両端点が平面の異なる側にある、または少なくとも一方が平面上
        (start_dist * end_dist <= T::ZERO)
            || (start_dist.abs() <= tolerance)
            || (end_dist.abs() <= tolerance)
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の両端点が平面上にある
        self.contains_point(segment.start(), tolerance)
            && self.contains_point(segment.end(), tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        // 線分の両端点から平面への距離の最小値
        let start_dist = self.distance_to_point(segment.start()).abs();
        let end_dist = self.distance_to_point(segment.end()).abs();
        start_dist.min(end_dist)
    }
}

// Plane3D vs Ray3D
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for Plane3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        // Ray の方向と平面法線の内積をチェック
        let ray_dir = ray.direction_vector();
        let normal_vec = self.normal().as_vector();
        let denom = ray_dir.dot(&normal_vec);

        // 平行でなければ交差する可能性がある
        if denom.abs() > T::EPSILON {
            // 起点が平面上または平面の正の側にあるかチェック
            let origin_dist = self.distance_to_point(ray.origin());
            origin_dist.abs() <= tolerance || (origin_dist * denom <= T::ZERO)
        } else {
            // 平行: 起点が平面上にあれば重なる
            self.contains_point(ray.origin(), tolerance)
        }
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        // Ray が完全に平面上にある（起点が平面上かつ方向が平面に平行）
        let ray_dir = ray.direction_vector();
        let normal_vec = self.normal().as_vector();
        let denom = ray_dir.dot(&normal_vec);

        self.contains_point(ray.origin(), tolerance) && denom.abs() <= tolerance
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        // Ray の起点から平面への距離
        self.distance_to_point(ray.origin()).abs()
    }
}

// Plane3D vs InfiniteLine3D
impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for Plane3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        // 直線の方向と平面法線の内積をチェック
        let line_dir = line.direction_internal();
        let line_dir_vec = crate::Vector3D::new(line_dir.x(), line_dir.y(), line_dir.z());
        let normal_vec = self.normal().as_vector();
        let denom = line_dir_vec.dot(&normal_vec);

        // 平行でなければ必ず交差する
        if denom.abs() > T::EPSILON {
            true
        } else {
            // 平行: 直線上の点が平面上にあれば重なる
            self.contains_point(line.point_internal(), tolerance)
        }
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        // 直線が完全に平面上にある（点が平面上かつ方向が平面に平行）
        let line_dir = line.direction_internal();
        let line_dir_vec = crate::Vector3D::new(line_dir.x(), line_dir.y(), line_dir.z());
        let normal_vec = self.normal().as_vector();
        let denom = line_dir_vec.dot(&normal_vec);

        self.contains_point(line.point_internal(), tolerance) && denom.abs() <= tolerance
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        // 直線上の点から平面への距離
        self.distance_to_point(line.point_internal()).abs()
    }
}

// Plane3D vs Plane3D
impl<T: Scalar> BasicCollision<T, Plane3D<T>> for Plane3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &Plane3D<T>, tolerance: T) -> bool {
        // 法線が平行でなければ交線を持つ
        let normal1 = self.normal().as_vector();
        let normal2 = other.normal().as_vector();
        let cross = normal1.cross(&normal2);

        if cross.length() > T::EPSILON {
            true // 平行でない
        } else {
            // 平行: 同一平面かチェック
            let dist = self.distance_to_point(other.origin());
            dist.abs() <= tolerance
        }
    }

    fn overlaps(&self, other: &Plane3D<T>, tolerance: T) -> bool {
        // 同一平面かチェック（法線が平行かつ距離がゼロ）
        let normal1 = self.normal().as_vector();
        let normal2 = other.normal().as_vector();
        let cross = normal1.cross(&normal2);

        if cross.length() <= T::EPSILON {
            let dist = self.distance_to_point(other.origin());
            dist.abs() <= tolerance
        } else {
            false
        }
    }

    fn distance_to(&self, other: &Plane3D<T>) -> T {
        // 平行な平面間の距離、または0（交線を持つ場合）
        let normal1 = self.normal().as_vector();
        let normal2 = other.normal().as_vector();
        let cross = normal1.cross(&normal2);

        if cross.length() <= T::EPSILON {
            // 平行
            self.distance_to_point(other.origin()).abs()
        } else {
            // 交線を持つ
            T::ZERO
        }
    }
}
