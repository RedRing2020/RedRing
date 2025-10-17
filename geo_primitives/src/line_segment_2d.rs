//! 2次元線分（LineSegment2D）の Core 実装
//!
//! Core Foundation パターンに基づく LineSegment2D の必須機能のみ
//! 拡張機能は line_segment_2d_extensions.rs を参照

use crate::{BBox2D, InfiniteLine2D, Point2D, Vector2D};
use geo_foundation::Scalar;

/// 2次元平面の線分
///
/// 始点と終点を持つ有限の線分
/// 内部的に InfiniteLine2D とパラメータ範囲を使用
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment2D<T: Scalar> {
    line: InfiniteLine2D<T>, // 基盤となる無限直線
    start_param: T,          // 始点のパラメータ
    end_param: T,            // 終点のパラメータ
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> LineSegment2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 始点と終点から線分を作成
    pub fn new(start: Point2D<T>, end: Point2D<T>) -> Option<Self> {
        let line = InfiniteLine2D::from_two_points(start, end)?;

        Some(Self {
            line,
            start_param: T::ZERO,
            end_param: start.distance_to(&end),
        })
    }

    /// 点と方向ベクトル、長さから線分を作成
    pub fn from_point_direction_length(
        start: Point2D<T>,
        direction: Vector2D<T>,
        length: T,
    ) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }

        let line = InfiniteLine2D::new(start, direction)?;

        Some(Self {
            line,
            start_param: T::ZERO,
            end_param: length,
        })
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 始点を取得
    pub fn start(&self) -> Point2D<T> {
        self.line.point_at_parameter(self.start_param)
    }

    /// 終点を取得
    pub fn end(&self) -> Point2D<T> {
        self.line.point_at_parameter(self.end_param)
    }

    /// 中点を取得
    pub fn midpoint(&self) -> Point2D<T> {
        let mid_param = (self.start_param + self.end_param) / (T::ONE + T::ONE);
        self.line.point_at_parameter(mid_param)
    }

    /// 線分の長さを取得
    pub fn length(&self) -> T {
        (self.end_param - self.start_param).abs()
    }

    /// 方向ベクトルを取得（正規化済み）
    pub fn direction(&self) -> Vector2D<T> {
        if self.end_param >= self.start_param {
            *self.line.direction()
        } else {
            (*self.line.direction()).negate()
        }
    }

    /// ベクトル表現を取得（始点から終点へのベクトル）
    pub fn vector(&self) -> Vector2D<T> {
        let start = self.start();
        let end = self.end();
        Vector2D::from_points(start, end)
    }

    // ========================================================================
    // Core Parametric Methods
    // ========================================================================

    /// 正規化されたパラメータ（0〜1）での点を取得
    pub fn point_at_normalized_parameter(&self, t: T) -> Point2D<T> {
        if t < T::ZERO || t > T::ONE {
            // パラメータが範囲外の場合は最も近い端点を返す
            if t < T::ZERO {
                return self.start();
            } else {
                return self.end();
            }
        }

        let param = self.start_param + t * (self.end_param - self.start_param);
        self.line.point_at_parameter(param)
    }

    /// 点から線分上の点へのパラメータを取得（0〜1範囲外も可能）
    pub fn parameter_for_point(&self, point: &Point2D<T>) -> T {
        let line_param = self.line.parameter_for_point(point);
        let segment_length = self.end_param - self.start_param;

        if segment_length.abs() < T::EPSILON {
            T::ZERO // 長さゼロの線分
        } else {
            (line_param - self.start_param) / segment_length
        }
    }

    // ========================================================================
    // Core Containment Methods
    // ========================================================================

    /// 点が線分上にあるかを判定
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // まず無限直線上にあるかチェック
        if !self.line.contains_point(point, tolerance) {
            return false;
        }

        // パラメータが線分の範囲内にあるかチェック
        let param = self.line.parameter_for_point(point);
        param >= self.start_param - tolerance && param <= self.end_param + tolerance
    }

    /// 点を線分に投影（線分内に制限）
    pub fn project_point_to_segment(&self, point: &Point2D<T>) -> Point2D<T> {
        let projected_param = self.line.parameter_for_point(point);

        // パラメータを線分の範囲内に制限
        let clamped_param = if projected_param < self.start_param {
            self.start_param
        } else if projected_param > self.end_param {
            self.end_param
        } else {
            projected_param
        };

        self.line.point_at_parameter(clamped_param)
    }

    /// 点から境界ボックスの境界への最短距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let projected = self.project_point_to_segment(point);
        point.distance_to(&projected)
    }

    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> BBox2D<T> {
        BBox2D::new(self.start(), self.end())
    }

    // ========================================================================
    // Internal Accessor Methods (for Extension implementation)
    // ========================================================================

    /// 基盤となる無限直線を取得（Extension用）
    pub fn line(&self) -> &InfiniteLine2D<T> {
        &self.line
    }

    /// 開始パラメータを取得（Extension用）
    pub fn start_parameter(&self) -> T {
        self.start_param
    }

    /// 終了パラメータを取得（Extension用）
    pub fn end_parameter(&self) -> T {
        self.end_param
    }
}

// ============================================================================
// ============================================================================
// Helper Methods (Foundation traits converted to methods)
// ============================================================================

impl<T: Scalar> LineSegment2D<T> {
    /// 方向を反転
    pub fn reverse_direction(&self) -> Self {
        Self {
            line: self.line,
            start_param: self.end_param,
            end_param: self.start_param,
        }
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    /// 接線ベクトルを取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector2D<T> {
        self.direction()
    }

    /// 境界上判定（線分では点上判定と同じ）
    pub fn on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }
}
