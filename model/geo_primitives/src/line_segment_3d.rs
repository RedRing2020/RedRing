//! 3次元線分（LineSegment3D）の Core 実装
//!
//! Core Foundation パターンに基づく LineSegment3D の必須機能のみ
//! 拡張機能は line_segment_3d_extensions.rs を参照

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元空間の線分
///
/// 始点と終点を持つ有限の線分
/// 内部的に InfiniteLine3D とパラメータ範囲を使用
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment3D<T: Scalar> {
    line: InfiniteLine3D<T>, // 基盤となる無限直線
    start_param: T,          // 始点のパラメータ
    end_param: T,            // 終点のパラメータ
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> LineSegment3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 始点と終点から線分を作成
    pub fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        let line = InfiniteLine3D::from_two_points(start, end)?;

        Some(Self {
            line,
            start_param: T::ZERO,
            end_param: start.distance_to(&end),
        })
    }

    /// 点と方向ベクトル、長さから線分を作成
    pub fn from_point_direction_length(
        start: Point3D<T>,
        direction: Vector3D<T>,
        length: T,
    ) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }

        let line = InfiniteLine3D::new(start, direction)?;

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
    pub fn start(&self) -> Point3D<T> {
        self.line.point_at_parameter(self.start_param)
    }

    /// 終点を取得
    pub fn end(&self) -> Point3D<T> {
        self.line.point_at_parameter(self.end_param)
    }

    /// 中点を取得
    pub fn midpoint(&self) -> Point3D<T> {
        let mid_param = (self.start_param + self.end_param) / (T::ONE + T::ONE);
        self.line.point_at_parameter(mid_param)
    }

    /// 線分の長さを取得
    pub fn length(&self) -> T {
        self.end_param - self.start_param
    }

    /// 方向ベクトルを取得
    pub fn direction(&self) -> Vector3D<T> {
        *self.line.direction()
    }

    /// 基盤となる無限直線を取得
    pub fn line(&self) -> &InfiniteLine3D<T> {
        &self.line
    }

    /// 始点のパラメータを取得
    pub fn start_param(&self) -> T {
        self.start_param
    }

    /// 終点のパラメータを取得
    pub fn end_param(&self) -> T {
        self.end_param
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// 点から線分への最短距離
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let to_point = Vector3D::from_points(&self.line.point(), point);
        let t = to_point.dot(&self.line.direction());

        // パラメータを線分の範囲内に制限
        let clamped_param = if t < self.start_param {
            self.start_param
        } else if t > self.end_param {
            self.end_param
        } else {
            t
        };

        let projected = self.line.point_at_parameter(clamped_param);
        point.distance_to(&projected)
    }

    /// 点が線分上にあるかを判定
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // まず無限直線上にあるかチェック
        if !self.line.contains_point(point, tolerance) {
            return false;
        }

        // パラメータが線分の範囲内にあるかチェック
        let param = self.line.parameter_for_point(point);
        param >= self.start_param - tolerance && param <= self.end_param + tolerance
    }

    /// 線分が退化しているか（長さが0）を判定
    pub fn is_degenerate(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }
}
