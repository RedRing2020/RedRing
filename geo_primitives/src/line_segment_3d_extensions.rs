//! 3次元線分（LineSegment3D）の Extension 実装
//!
//! Core Foundation パターンに基づく LineSegment3D の拡張機能
//! 高度な幾何計算、交差判定、変換処理等を提供

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Vector3D};
use geo_foundation::Scalar;

// ============================================================================
// Extension Implementation (高度な機能)
// ============================================================================

impl<T: Scalar> LineSegment3D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================

    /// X軸方向の線分を作成
    pub fn x_axis_segment(start: Point3D<T>, length: T) -> Option<Self> {
        Self::from_point_direction_length(start, Vector3D::unit_x(), length)
    }

    /// Y軸方向の線分を作成
    pub fn y_axis_segment(start: Point3D<T>, length: T) -> Option<Self> {
        Self::from_point_direction_length(start, Vector3D::unit_y(), length)
    }

    /// Z軸方向の線分を作成
    pub fn z_axis_segment(start: Point3D<T>, length: T) -> Option<Self> {
        Self::from_point_direction_length(start, Vector3D::unit_z(), length)
    }

    // ========================================================================
    // Advanced Calculation Methods (Extension)
    // ========================================================================

    /// ベクトル表現を取得（始点から終点へのベクトル）
    pub fn vector(&self) -> Vector3D<T> {
        let start = self.start();
        let end = self.end();
        Vector3D::from_points(&start, &end)
    }

    /// 基盤となる無限直線を取得
    pub fn infinite_line(&self) -> InfiniteLine3D<T> {
        *self.line()
    }

    /// 正規化されたパラメータ（0〜1）での点を取得
    pub fn point_at_normalized_parameter(&self, t: T) -> Point3D<T> {
        if t < T::ZERO || t > T::ONE {
            // パラメータが範囲外の場合は最も近い端点を返す
            if t < T::ZERO {
                return self.start();
            } else {
                return self.end();
            }
        }

        let param = self.start_param() + t * (self.end_param() - self.start_param());
        self.line().point_at_parameter(param)
    }

    /// 点を線分に投影（線分内に制限）
    pub fn project_point_to_segment(&self, point: &Point3D<T>) -> Point3D<T> {
        let projected_param = self.line().parameter_for_point(point);

        // パラメータを線分の範囲内に制限
        let clamped_param = if projected_param < self.start_param() {
            self.start_param()
        } else if projected_param > self.end_param() {
            self.end_param()
        } else {
            projected_param
        };

        self.line().point_at_parameter(clamped_param)
    }

    /// 点から線分上の点へのパラメータを取得（0〜1範囲外も可能）
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let line_param = self.line().parameter_for_point(point);
        let segment_length = self.end_param() - self.start_param();

        if segment_length <= T::ZERO {
            T::ZERO // 退化した線分の場合
        } else {
            (line_param - self.start_param()) / segment_length
        }
    }

    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================

    /// 線分を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector3D<T>) -> Self {
        let new_start = self.start() + *vector;
        let new_end = self.end() + *vector;
        Self::new(new_start, new_end).unwrap()
    }

    /// 線分を指定倍率で拡大縮小（始点を基準）
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        let start = self.start();
        let direction = self.vector();
        let new_end = start + direction * factor;

        Some(Self::new(start, new_end).unwrap())
    }

    /// 方向を反転した線分を取得
    pub fn reverse(&self) -> Self {
        Self::new(self.end(), self.start()).unwrap()
    }

    // ========================================================================
    // Spatial Relationship Methods (Extension)
    // ========================================================================

    /// 他の線分と平行かを判定
    pub fn is_parallel(&self, other: &Self, tolerance: T) -> bool {
        self.line().is_parallel(other.line(), tolerance)
    }

    /// 線分を指定の比率で分割（0〜1）
    pub fn split_at(&self, t: T) -> (Self, Self) {
        let split_point = self.start() + (self.end() - self.start()) * t;

        let first = Self::new(self.start(), split_point).unwrap();
        let second = Self::new(split_point, self.end()).unwrap();

        (first, second)
    }

    /// 他の線分との最短距離
    pub fn distance_to_segment(&self, other: &Self) -> T {
        // 簡略化実装：各端点から相手の線分への最短距離の最小値
        let distances = [
            self.distance_to_point(&other.start()),
            self.distance_to_point(&other.end()),
            other.distance_to_point(&self.start()),
            other.distance_to_point(&self.end()),
        ];

        distances.iter().fold(
            distances[0],
            |min, &dist| {
                if dist < min {
                    dist
                } else {
                    min
                }
            },
        )
    }

    // ========================================================================
    // Advanced Geometric Operations (Extension)
    // ========================================================================

    /// 線分の中間点を複数取得（等間隔分割）
    pub fn subdivide(&self, segments: usize) -> Vec<Point3D<T>> {
        if segments == 0 {
            return vec![];
        }

        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = T::from_usize(i) / T::from_usize(segments);
            points.push(self.point_at_normalized_parameter(t));
        }
        points
    }

    /// 線分上の最も近い点のパラメータを取得（0〜1範囲に制限）
    pub fn closest_parameter_on_segment(&self, point: &Point3D<T>) -> T {
        let line_param = self.line().parameter_for_point(point);
        let segment_length = self.end_param() - self.start_param();

        if segment_length.abs() < T::EPSILON {
            return T::ZERO; // 長さゼロの線分
        }

        let normalized_param = (line_param - self.start_param()) / segment_length;

        // 0〜1範囲に制限
        if normalized_param < T::ZERO {
            T::ZERO
        } else if normalized_param > T::ONE {
            T::ONE
        } else {
            normalized_param
        }
    }

    /// 線分が指定した点を含むかの詳細判定
    pub fn contains_point_detailed(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // まず無限直線上にあるかチェック
        if !self.line().contains_point(point, tolerance) {
            return false;
        }

        // パラメータが線分の範囲内にあるかチェック
        let param = self.line().parameter_for_point(point);
        param >= self.start_param() - tolerance && param <= self.end_param() + tolerance
    }
}

// ============================================================================
// Extension Foundation trait implementations
// ============================================================================

// TODO: Extension Foundation traits need migration to new traits system
/*
impl<T: Scalar> ExtensionFoundation<T> for LineSegment3D<T> {
    // 高度な変形操作
    // 空間関係操作
    // 次元変換操作
}
*/
