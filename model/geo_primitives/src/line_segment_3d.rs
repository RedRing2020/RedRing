//! 3次元線分（LineSegment3D）の Core 実装
//!
//! Core Foundation パターンに基づく LineSegment3D の必須機能のみ
//! 拡張機能は line_segment_3d_extensions.rs を参照

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_foundation::{
    core::linesegment_traits::{
        LineSegment3DConstructor, LineSegment3DMeasure, LineSegment3DProperties,
    },
    Scalar,
};

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
        *self.line.direction_internal()
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
        let to_point = Vector3D::from_points(&self.line.point_internal(), point);
        let t = to_point.dot(&self.line.direction_internal());

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

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

impl<T: Scalar> LineSegment3DConstructor<T> for LineSegment3D<T> {
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self> {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let end_point = Point3D::new(end.0, end.1, end.2);
        Self::new(start_point, end_point)
    }

    fn from_point_direction_length(
        start: (T, T, T),
        direction: (T, T, T),
        length: T,
    ) -> Option<Self> {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let direction_vec = Vector3D::new(direction.0, direction.1, direction.2);
        Self::from_point_direction_length(start_point, direction_vec, length)
    }

    fn unit_x() -> Self {
        let start = Point3D::origin();
        let end = Point3D::new(T::ONE, T::ZERO, T::ZERO);
        Self::new(start, end).unwrap()
    }

    // Phase 2: 追加コンストラクタ
    fn unit_y() -> Self {
        let start = Point3D::origin();
        let end = Point3D::new(T::ZERO, T::ONE, T::ZERO);
        Self::new(start, end).unwrap()
    }

    fn unit_z() -> Self {
        let start = Point3D::origin();
        let end = Point3D::new(T::ZERO, T::ZERO, T::ONE);
        Self::new(start, end).unwrap()
    }

    fn horizontal_xy(midpoint: (T, T, T), length: T) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }
        let half_length = length / (T::ONE + T::ONE);
        let start = Point3D::new(midpoint.0 - half_length, midpoint.1, midpoint.2);
        let end = Point3D::new(midpoint.0 + half_length, midpoint.1, midpoint.2);
        Self::new(start, end)
    }
}

impl<T: Scalar> LineSegment3DProperties<T> for LineSegment3D<T> {
    fn start(&self) -> (T, T, T) {
        let p = self.start();
        (p.x(), p.y(), p.z())
    }

    fn end(&self) -> (T, T, T) {
        let p = self.end();
        (p.x(), p.y(), p.z())
    }

    fn midpoint(&self) -> (T, T, T) {
        let p = self.midpoint();
        (p.x(), p.y(), p.z())
    }

    fn length(&self) -> T {
        self.length()
    }

    fn dimension(&self) -> u32 {
        3
    }

    // Phase 2: 追加プロパティ
    fn is_unit_length(&self) -> bool {
        (self.length() - T::ONE).abs() <= T::EPSILON
    }

    fn is_on_xy_plane(&self) -> bool {
        let start = self.start();
        let end = self.end();
        (start.z() - end.z()).abs() <= T::EPSILON
    }

    fn is_on_yz_plane(&self) -> bool {
        let start = self.start();
        let end = self.end();
        (start.x() - end.x()).abs() <= T::EPSILON
    }
}

impl<T: Scalar> LineSegment3DMeasure<T> for LineSegment3D<T> {
    fn measure(&self) -> T {
        self.length()
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        self.distance_to_point(&p)
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        self.contains_point(&p, T::EPSILON)
    }

    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        // 正規化パラメータ（0〜1）で線分上の点を取得
        let param = self.start_param + t * (self.end_param - self.start_param);
        let p = self.line.point_at_parameter(param);
        (p.x(), p.y(), p.z())
    }

    // Phase 2: 追加測度メソッド
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        // 直線上のパラメータを計算
        let line_param = self.line.parameter_for_point(&p);
        // 線分範囲にクランプ
        let clamped_param = if line_param < self.start_param {
            self.start_param
        } else if line_param > self.end_param {
            self.end_param
        } else {
            line_param
        };
        let result = self.line.point_at_parameter(clamped_param);
        (result.x(), result.y(), result.z())
    }

    fn distance_to_segment(&self, other: &Self) -> T {
        // 簡易実装: 各端点から他方の線分への最短距離の最小値
        let other_start_pt = other.start();
        let other_end_pt = other.end();
        let self_start_pt = self.start();
        let self_end_pt = self.end();

        let d1 = self.distance_to_point(&other_start_pt);
        let d2 = self.distance_to_point(&other_end_pt);
        let d3 = other.distance_to_point(&self_start_pt);
        let d4 = other.distance_to_point(&self_end_pt);

        let min1 = if d1 < d2 { d1 } else { d2 };
        let min2 = if d3 < d4 { d3 } else { d4 };
        if min1 < min2 {
            min1
        } else {
            min2
        }
    }

    fn direction_vector(&self) -> (T, T, T) {
        let dir = self.direction();
        (dir.x(), dir.y(), dir.z())
    }

    fn as_vector(&self) -> (T, T, T) {
        let start = self.start();
        let end = self.end();
        let v = Vector3D::from_points(&start, &end);
        (v.x(), v.y(), v.z())
    }
}
