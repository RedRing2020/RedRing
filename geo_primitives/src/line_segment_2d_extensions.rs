//! 2次元線分（LineSegment2D）の Extension 実装
//!
//! Core Foundation パターンに基づく LineSegment2D の拡張機能
//! 高度な幾何計算、交差判定、変換処理等を提供

use crate::{InfiniteLine2D, LineSegment2D, Point2D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Extension Implementation (高度な機能)
// ============================================================================

impl<T: Scalar> LineSegment2D<T> {
    // ========================================================================
    // Advanced Intersection Methods
    // ========================================================================

    /// 他の線分との交点を取得
    pub fn intersection_with_segment(&self, other: &Self) -> Option<Point2D<T>> {
        // 無限直線での交点を計算
        let line_intersection = self.line().intersection_with_line(other.line())?;

        // 両方の線分上にある点かチェック
        let tolerance = T::EPSILON;
        if self.contains_point(&line_intersection, tolerance)
            && other.contains_point(&line_intersection, tolerance)
        {
            Some(line_intersection)
        } else {
            None
        }
    }

    /// 直線との交点を取得
    pub fn intersection_with_line(&self, line: &InfiniteLine2D<T>) -> Option<Point2D<T>> {
        let intersection = self.line().intersection_with_line(line)?;
        let tolerance = T::EPSILON;

        if self.contains_point(&intersection, tolerance) {
            Some(intersection)
        } else {
            None
        }
    }

    /// 光線との交点を取得（Ray2D未実装のため一時的にコメントアウト）
    /*
    pub fn intersection_with_ray(&self, ray: &crate::Ray2D<T>) -> Option<Point2D<T>> {
        let intersection = self.line().intersection_with_ray(ray)?;
        let tolerance = T::EPSILON;

        if self.contains_point(&intersection, tolerance) {
            Some(intersection)
        } else {
            None
        }
    }
    */

    /// 円との交点を取得（一時的にコメントアウト - InfiniteLine2Dにメソッドなし）
    /*
    pub fn intersections_with_circle(&self, circle: &Circle2D<T>) -> Vec<Point2D<T>> {
        let line_intersections = self.line().intersections_with_circle(circle);
        let tolerance = T::EPSILON;

        line_intersections
            .into_iter()
            .filter(|point| self.contains_point(point, tolerance))
            .collect()
    }
    */
    // ========================================================================
    // Advanced Geometric Operations
    // ========================================================================

    /// 線分を指定した点で分割
    pub fn split_at_point(&self, point: &Point2D<T>) -> Option<(Self, Self)> {
        let tolerance = T::EPSILON;
        if !self.contains_point(point, tolerance) {
            return None;
        }

        let _t = self.parameter_for_point(point);
        let first = Self::new(self.start(), *point)?;
        let second = Self::new(*point, self.end())?;

        Some((first, second))
    }

    /// 線分を指定したパラメータで分割
    pub fn split_at_parameter(&self, t: T) -> Option<(Self, Self)> {
        if t < T::ZERO || t > T::ONE {
            return None;
        }

        let split_point = self.point_at_normalized_parameter(t);
        let first = Self::new(self.start(), split_point)?;
        let second = Self::new(split_point, self.end())?;

        Some((first, second))
    }

    /// 線分の一部を取得（パラメータ範囲指定）
    pub fn subsegment(&self, t_start: T, t_end: T) -> Option<Self> {
        if t_start < T::ZERO
            || t_end > T::ONE
            || t_start >= t_end
            || (t_end - t_start).abs() < T::EPSILON
        {
            return None;
        }

        let start_point = self.point_at_normalized_parameter(t_start);
        let end_point = self.point_at_normalized_parameter(t_end);

        Self::new(start_point, end_point)
    }

    /// 線分を延長（両端または一端）
    pub fn extend(&self, start_length: T, end_length: T) -> Option<Self> {
        let direction = self.direction();
        let current_length = self.length();

        if current_length < T::EPSILON {
            return None;
        }

        let new_start = self.start() - direction * start_length;
        let new_end = self.end() + direction * end_length;

        Self::new(new_start, new_end)
    }

    // ========================================================================
    // Advanced Geometric Queries
    // ========================================================================

    /// 平行性の判定
    pub fn is_parallel_to(&self, other: &Self, _tolerance: T) -> bool {
        self.line().is_parallel(other.line())
    }

    /// 垂直性の判定
    pub fn is_perpendicular_to(&self, other: &Self, _tolerance: T) -> bool {
        self.line().is_perpendicular(other.line())
    }

    /// 同一直線上にあるかの判定
    pub fn is_collinear_with(&self, other: &Self, _tolerance: T) -> bool {
        self.line().is_coincident(other.line())
    }

    /// 線分同士の最短距離
    pub fn distance_to_segment(&self, other: &Self) -> T {
        // 交差する場合は距離0
        if self.intersection_with_segment(other).is_some() {
            return T::ZERO;
        }

        // 各端点から相手の線分への距離の最小値
        let distances = [
            self.distance_to_point(&other.start()),
            self.distance_to_point(&other.end()),
            other.distance_to_point(&self.start()),
            other.distance_to_point(&self.end()),
        ];

        distances
            .into_iter()
            .fold(distances[0], |min, d| if d < min { d } else { min })
    }

    // ========================================================================
    // Missing Methods for Tests
    // ========================================================================

    /// X軸方向の線分を作成
    pub fn x_axis_segment(start: Point2D<T>, length: T) -> Option<Self> {
        Self::from_point_direction_length(start, crate::Vector2D::unit_x(), length)
    }

    /// Y軸方向の線分を作成
    pub fn y_axis_segment(start: Point2D<T>, length: T) -> Option<Self> {
        Self::from_point_direction_length(start, crate::Vector2D::unit_y(), length)
    }

    /// 水平線分を作成（X軸に平行）
    pub fn horizontal_segment(y: T, x_start: T, x_end: T) -> Option<Self> {
        let start = Point2D::new(x_start, y);
        let end = Point2D::new(x_end, y);
        Self::new(start, end)
    }

    /// 垂直線分を作成（Y軸に平行）
    pub fn vertical_segment(x: T, y_start: T, y_end: T) -> Option<Self> {
        let start = Point2D::new(x, y_start);
        let end = Point2D::new(x, y_end);
        Self::new(start, end)
    }

    /// 原点から指定点への線分を作成
    pub fn from_origin(end: Point2D<T>) -> Option<Self> {
        Self::new(Point2D::origin(), end)
    }

    /// 線分を平行移動
    pub fn translate(&self, offset: &crate::Vector2D<T>) -> Self {
        let new_start = self.start() + *offset;
        let new_end = self.end() + *offset;
        Self::new(new_start, new_end).expect("Translation should preserve segment validity")
    }

    /// 線分を指定比率でスケール
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        let center = self.midpoint();
        let new_start = center + (self.start() - center) * factor;
        let new_end = center + (self.end() - center) * factor;

        Self::new(new_start, new_end)
    }

    /// 方向を反転した線分を取得
    pub fn reverse(&self) -> Self {
        Self::new(self.end(), self.start()).expect("Reverse should preserve segment validity")
    }

    /// 線分を指定の比率で分割（0〜1）
    pub fn split_at(&self, t: T) -> (Self, Self) {
        let split_point = self.point_at_normalized_parameter(t);
        let first = Self::new(self.start(), split_point).expect("Split should preserve validity");
        let second = Self::new(split_point, self.end()).expect("Split should preserve validity");
        (first, second)
    }

    /// 線分を延長して指定長さにする
    pub fn extend_to_length(&self, new_length: T) -> Option<Self> {
        if new_length <= T::ZERO {
            return None;
        }

        let direction = self.direction();
        let new_end = self.start() + direction * new_length;
        Self::new(self.start(), new_end)
    }

    /// 両端を指定長さずつ延長する
    pub fn extend_both_ends(&self, start_extension: T, end_extension: T) -> Self {
        let direction = self.direction();
        let new_start = self.start() - direction * start_extension;
        let new_end = self.end() + direction * end_extension;
        Self::new(new_start, new_end).expect("Extension should preserve validity")
    }

    /// 線分の面積（常に0）
    pub fn area(&self) -> T {
        T::ZERO
    }

    /// 線分の重心（中点）
    pub fn centroid(&self) -> Point2D<T> {
        self.midpoint()
    }

    /// 線分の勾配を取得（水平でない場合）
    pub fn slope(&self) -> Option<T> {
        let vector = self.vector();
        if vector.x().abs() < T::EPSILON {
            None // 垂直線は勾配無限大
        } else {
            Some(vector.y() / vector.x())
        }
    }

    /// 線分が退化しているか（長さが0またはほぼ0）
    pub fn is_degenerate(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }

    /// 指定軸周りの回転（2Dでは原点周りの回転）
    pub fn rotate_around_origin(&self, angle: Angle<T>) -> Self {
        let origin = Point2D::origin();
        let new_start = self.start().rotate_around(&origin, angle);
        let new_end = self.end().rotate_around(&origin, angle);
        Self::new(new_start, new_end).expect("Rotation should preserve segment validity")
    }
}
