//! LineSegment3D Extensions 実装
//!
//! Foundation統一システムに基づくLineSegment3Dの拡張機能
//! Core機能は line_segment_3d.rs を参照

use crate::{BBox3D, LineSegment3D, Point3D, Vector3D};
use geo_foundation::{core_foundation::*, Scalar};

// ============================================================================
// Core trait implementations
// ============================================================================

// Note: Copy trait cannot be implemented due to InfiniteLine3D field not implementing Copy

impl<T: Scalar> std::fmt::Display for LineSegment3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LineSegment3D(start: {:?}, end: {:?})",
            self.start(),
            self.end()
        )
    }
}

// ============================================================================
// Extended geometric operations (moved from core)
// ============================================================================

impl<T: Scalar> LineSegment3D<T> {
    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> BBox3D<T> {
        BBox3D::from_points(&[self.start(), self.end()])
            .unwrap_or_else(|| BBox3D::from_point(self.start()))
    }

    /// パラメータでの点を取得
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let param = self.start_param() + t * (self.end_param() - self.start_param());
        self.line().point_at_parameter(param)
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    /// パラメータでの接線方向を取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector3D<T> {
        self.direction()
    }

    /// 線分の逆方向を作成
    pub fn reverse(&self) -> Self {
        Self::new(self.end(), self.start()).expect("Valid segment should always reverse")
    }

    /// 境界上判定（線分では contains_point と同じ）
    pub fn on_boundary(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    /// 線分を延長
    pub fn extend(&self, start_extension: T, end_extension: T) -> Self {
        let direction = self.direction().normalize();
        let start_offset = direction * start_extension;
        let end_offset = direction * end_extension;
        let new_start = Point3D::new(
            self.start().x() - start_offset.x(),
            self.start().y() - start_offset.y(),
            self.start().z() - start_offset.z(),
        );
        let new_end = Point3D::new(
            self.end().x() + end_offset.x(),
            self.end().y() + end_offset.y(),
            self.end().z() + end_offset.z(),
        );
        Self::new(new_start, new_end).expect("Extended segment should be valid")
    }

    /// 線分の一部を取得
    pub fn sub_segment(&self, start_t: T, end_t: T) -> Option<Self> {
        if start_t < T::ZERO || end_t > T::ONE || start_t >= end_t {
            return None;
        }

        let start_point = self.point_at_parameter(start_t);
        let end_point = self.point_at_parameter(end_t);
        Self::new(start_point, end_point)
    }

    /// 点を線分に投影
    pub fn project_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = Vector3D::from_points(&self.line().point(), point);
        let t = to_point.dot(&self.line().direction());

        // パラメータを線分の範囲内に制限
        let clamped_param = if t < self.start_param() {
            self.start_param()
        } else if t > self.end_param() {
            self.end_param()
        } else {
            t
        };

        self.line().point_at_parameter(clamped_param)
    }

    /// 線分上で点に最も近い点のパラメータを取得
    pub fn closest_parameter(&self, point: &Point3D<T>) -> T {
        let to_point = Vector3D::from_points(&self.line().point(), point);
        let line_param = to_point.dot(&self.line().direction());

        // 線分のパラメータ範囲に正規化
        if line_param < self.start_param() {
            T::ZERO
        } else if line_param > self.end_param() {
            T::ONE
        } else {
            (line_param - self.start_param()) / (self.end_param() - self.start_param())
        }
    }

    /// 線分同士の最短距離
    pub fn distance_to_segment(&self, other: &LineSegment3D<T>) -> T {
        // 簡易実装：各端点と他の線分との距離の最小値
        let d1 = self.distance_to_point(&other.start());
        let d2 = self.distance_to_point(&other.end());
        let d3 = other.distance_to_point(&self.start());
        let d4 = other.distance_to_point(&self.end());

        d1.min(d2).min(d3).min(d4)
    }

    /// 線分が平行かを判定
    pub fn is_parallel_to(&self, other: &LineSegment3D<T>, tolerance: T) -> bool {
        let cross_product = self.direction().cross(&other.direction());
        cross_product.length() <= tolerance
    }

    /// 線分が垂直かを判定
    pub fn is_perpendicular_to(&self, other: &LineSegment3D<T>, tolerance: T) -> bool {
        let dot_product = self.direction().dot(&other.direction()).abs();
        dot_product <= tolerance
    }

    /// 線分の中点を取得
    pub fn center(&self) -> Point3D<T> {
        self.midpoint()
    }

    /// 指定した比率での分割点を取得
    pub fn point_at_ratio(&self, ratio: T) -> Point3D<T> {
        self.point_at_parameter(ratio)
    }

    /// 線分を等分割する点を取得
    pub fn subdivide(&self, segments: usize) -> Vec<Point3D<T>> {
        if segments == 0 {
            return vec![];
        }

        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = T::from_f64(i as f64 / segments as f64);
            points.push(self.point_at_parameter(t));
        }
        points
    }
}

// ============================================================================
// geo_foundation trait implementations (simplified)
// ============================================================================

impl<T: Scalar> BasicMetrics<T> for LineSegment3D<T> {
    fn length(&self) -> Option<T> {
        Some(self.length())
    }
}
