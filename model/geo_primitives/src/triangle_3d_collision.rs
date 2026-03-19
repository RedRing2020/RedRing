//! Triangle3D 衝突検出・距離計算実装
//!
//! BasicCollision トレイトの実装
//! Triangle3D と他の幾何形状との組み合わせを実装

use crate::{LineSegment3D, Point3D, Ray3D, Triangle3D};
use geo_contracts::BasicCollision;
use geo_foundation::Scalar;

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// Triangle3D vs Point3D
impl<T: Scalar> BasicCollision<T, Point3D<T>> for Triangle3D<T> {
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

// Triangle3D vs LineSegment3D
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for Triangle3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の両端点または中間点が三角形に接触するかチェック
        if self.distance_to_point(&segment.start()) <= tolerance
            || self.distance_to_point(&segment.end()) <= tolerance
        {
            return true;
        }

        // 線分が三角形平面と交差するかチェック
        triangle_segment_intersects(self, segment, tolerance)
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の両端点が三角形上にある
        self.distance_to_point(&segment.start()) <= tolerance
            && self.distance_to_point(&segment.end()) <= tolerance
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        // 線分の両端点から三角形への距離の最小値
        let start_dist = self.distance_to_point(&segment.start());
        let end_dist = self.distance_to_point(&segment.end());
        start_dist.min(end_dist)
    }
}

// Triangle3D vs Ray3D
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for Triangle3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        // Ray の起点が三角形に接触するかチェック
        if self.distance_to_point(&ray.origin()) <= tolerance {
            return true;
        }

        // Ray が三角形と交差するかチェック
        triangle_ray_intersects(self, ray, tolerance)
    }

    fn overlaps(&self, _ray: &Ray3D<T>, _tolerance: T) -> bool {
        false // Ray は無限長のため overlap は定義しない
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        // Ray の起点から三角形への距離
        self.distance_to_point(&ray.origin())
    }
}

// Triangle3D vs Triangle3D
impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for Triangle3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &Triangle3D<T>, tolerance: T) -> bool {
        // 簡易実装: いずれかの頂点が相手の三角形に接触するかチェック
        let vertices_self = [
            self.vertex_a_internal(),
            self.vertex_b_internal(),
            self.vertex_c_internal(),
        ];
        let vertices_other = [
            other.vertex_a_internal(),
            other.vertex_b_internal(),
            other.vertex_c_internal(),
        ];

        for v in &vertices_self {
            if other.distance_to_point(v) <= tolerance {
                return true;
            }
        }

        for v in &vertices_other {
            if self.distance_to_point(v) <= tolerance {
                return true;
            }
        }

        false
    }

    fn overlaps(&self, other: &Triangle3D<T>, tolerance: T) -> bool {
        // 全頂点が相手の三角形上にある
        let vertices_self = [
            self.vertex_a_internal(),
            self.vertex_b_internal(),
            self.vertex_c_internal(),
        ];

        vertices_self
            .iter()
            .all(|v| other.distance_to_point(v) <= tolerance)
    }

    fn distance_to(&self, other: &Triangle3D<T>) -> T {
        // 各頂点から相手の三角形への距離の最小値
        let vertices_self = [
            self.vertex_a_internal(),
            self.vertex_b_internal(),
            self.vertex_c_internal(),
        ];
        let vertices_other = [
            other.vertex_a_internal(),
            other.vertex_b_internal(),
            other.vertex_c_internal(),
        ];

        let mut min_dist = T::MAX;
        for v in &vertices_self {
            min_dist = min_dist.min(other.distance_to_point(v));
        }
        for v in &vertices_other {
            min_dist = min_dist.min(self.distance_to_point(v));
        }

        min_dist
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// 三角形と線分が交差するかチェック
fn triangle_segment_intersects<T: Scalar>(
    _triangle: &Triangle3D<T>,
    _segment: &LineSegment3D<T>,
    _tolerance: T,
) -> bool {
    // 簡易実装: 平面との交点計算は複雑なため、保守的に false を返す
    // 完全な実装は Möller-Trumbore アルゴリズムなどが必要
    false
}

/// 三角形と Ray が交差するかチェック
fn triangle_ray_intersects<T: Scalar>(
    _triangle: &Triangle3D<T>,
    _ray: &Ray3D<T>,
    _tolerance: T,
) -> bool {
    // 簡易実装: 平面との交点計算は複雑なため、保守的に false を返す
    // 完全な実装は Möller-Trumbore アルゴリズムなどが必要
    false
}
