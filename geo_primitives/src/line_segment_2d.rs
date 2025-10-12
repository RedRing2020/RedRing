//! 2次元線分（LineSegment2D）の実装
//!
//! foundation.rs の基盤トレイトに基づく LineSegment2D の実装
//! InfiniteLine2D を内部活用

use crate::{BBox2D, InfiniteLine2D, Point2D, Vector2D};
use geo_foundation::{
    abstract_types::geometry::core_foundation::*, tolerance_migration::DefaultTolerances,
    GeometryContext, Scalar,
};

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

impl<T: Scalar> LineSegment2D<T> {
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

    /// X軸方向の線分を作成
    pub fn x_axis_segment(start: Point2D<T>, length: T) -> Option<Self> {
        Self::from_point_direction_length(start, Vector2D::unit_x(), length)
    }

    /// Y軸方向の線分を作成
    pub fn y_axis_segment(start: Point2D<T>, length: T) -> Option<Self> {
        Self::from_point_direction_length(start, Vector2D::unit_y(), length)
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
            self.line.direction()
        } else {
            self.line.direction().negate()
        }
    }

    /// ベクトル表現を取得（始点から終点へのベクトル）
    pub fn vector(&self) -> Vector2D<T> {
        let start = self.start();
        let end = self.end();
        Vector2D::from_points(start, end)
    }

    /// 基盤となる無限直線を取得
    pub fn infinite_line(&self) -> InfiniteLine2D<T> {
        self.line
    }

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

    /// 点から線分への最短距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let projected = self.project_point_to_segment(point);
        point.distance_to(&projected)
    }

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

    /// 線分を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector2D<T>) -> Self {
        let new_line = self.line.translate(*vector);
        Self {
            line: new_line,
            start_param: self.start_param,
            end_param: self.end_param,
        }
    }

    /// 線分を指定倍率で拡大縮小（始点を基準）
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        Some(Self {
            line: self.line,
            start_param: self.start_param,
            end_param: self.start_param + (self.end_param - self.start_param) * factor,
        })
    }

    /// 方向を反転した線分を取得
    pub fn reverse(&self) -> Self {
        Self {
            line: self.line,
            start_param: self.end_param,
            end_param: self.start_param,
        }
    }

    /// 線分が退化しているか（長さが0）を判定
    pub fn is_degenerate(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }

    /// 他の線分と平行かどうかを判定
    pub fn is_parallel(&self, other: &Self, tolerance: T) -> bool {
        self.line.is_parallel_with_tolerance(&other.line, tolerance)
    }

    /// 他の線分と垂直かどうかを判定
    pub fn is_perpendicular(&self, other: &Self, tolerance: T) -> bool {
        self.line
            .is_perpendicular_with_tolerance(&other.line, tolerance)
    }

    /// 線分を指定の比率で分割（0〜1）
    pub fn split_at(&self, t: T) -> (Self, Self) {
        let split_param = self.start_param + t * (self.end_param - self.start_param);

        let first = Self {
            line: self.line,
            start_param: self.start_param,
            end_param: split_param,
        };

        let second = Self {
            line: self.line,
            start_param: split_param,
            end_param: self.end_param,
        };

        (first, second)
    }

    /// 他の線分との最短距離
    pub fn distance_to_segment(&self, other: &Self) -> T {
        // 各端点から相手の線分への最短距離の最小値
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

    /// 他の線分との交点を計算
    pub fn intersection_with_segment(&self, other: &Self) -> Option<Point2D<T>> {
        // まず無限直線同士の交点を求める
        let line_intersection = self.line.intersection_with_line(&other.line)?;

        // 交点が両方の線分上にあるかチェック（デフォルト距離許容誤差を使用）
        let tolerance = DefaultTolerances::distance::<T>();
        if self.contains_point(&line_intersection, tolerance)
            && other.contains_point(&line_intersection, tolerance)
        {
            Some(line_intersection)
        } else {
            None
        }
    }

    /// 線分の面積（常に0）
    pub fn area(&self) -> T {
        T::ZERO
    }

    /// 線分の重心（中点）
    pub fn centroid(&self) -> Point2D<T> {
        self.midpoint()
    }

    /// 指定軸周りの回転（2Dでは原点周りの回転）
    pub fn rotate_around_origin(&self, angle: T) -> Self {
        let new_line = self.line.rotate_around_origin(angle);
        Self {
            line: new_line,
            start_param: self.start_param,
            end_param: self.end_param,
        }
    }

    /// 線分の勾配を取得（水平でない場合）
    pub fn slope(&self) -> Option<T> {
        self.line.slope()
    }

    /// 水平線分かどうかを判定
    pub fn is_horizontal(&self, tolerance: T) -> bool {
        self.line.is_horizontal(tolerance)
    }

    /// 垂直線分かどうかを判定
    pub fn is_vertical(&self, tolerance: T) -> bool {
        self.line.is_vertical(tolerance)
    }

    /// 2つの線分が平行で同じ直線上にあるかを判定
    pub fn is_collinear(&self, other: &Self, tolerance: T) -> bool {
        self.line.is_same_line(&other.line, tolerance)
    }

    /// 線分を延長して指定長さにする
    pub fn extend_to_length(&self, new_length: T) -> Option<Self> {
        if new_length <= T::ZERO {
            return None;
        }

        Some(Self {
            line: self.line,
            start_param: self.start_param,
            end_param: self.start_param + new_length,
        })
    }

    /// 両端を指定長さずつ延長する
    pub fn extend_both_ends(&self, start_extension: T, end_extension: T) -> Self {
        Self {
            line: self.line,
            start_param: self.start_param - start_extension,
            end_param: self.end_param + end_extension,
        }
    }

    // === コンテキスト対応メソッド（アプリケーションレベルの許容誤差制御） ===

    /// コンテキストを使用した点の包含判定
    pub fn contains_point_with_context(
        &self,
        point: &Point2D<T>,
        context: &GeometryContext<T>,
    ) -> bool {
        self.contains_point(point, context.tolerances.distance_tolerance)
    }

    /// コンテキストを使用した平行判定
    pub fn is_parallel_with_context(&self, other: &Self, context: &GeometryContext<T>) -> bool {
        self.is_parallel(other, context.tolerances.angle_tolerance)
    }

    /// コンテキストを使用した垂直判定
    pub fn is_perpendicular_with_context(
        &self,
        other: &Self,
        context: &GeometryContext<T>,
    ) -> bool {
        self.is_perpendicular(other, context.tolerances.angle_tolerance)
    }

    /// コンテキストを使用した共線判定
    pub fn is_collinear_with_context(&self, other: &Self, context: &GeometryContext<T>) -> bool {
        self.is_collinear(other, context.tolerances.angle_tolerance)
    }

    /// コンテキストを使用した水平線分判定
    pub fn is_horizontal_with_context(&self, context: &GeometryContext<T>) -> bool {
        self.is_horizontal(context.tolerances.angle_tolerance)
    }

    /// コンテキストを使用した垂直線分判定
    pub fn is_vertical_with_context(&self, context: &GeometryContext<T>) -> bool {
        self.is_vertical(context.tolerances.angle_tolerance)
    }

    /// コンテキストを使用した退化判定
    pub fn is_degenerate_with_context(&self, context: &GeometryContext<T>) -> bool {
        self.is_degenerate(context.tolerances.length_tolerance)
    }

    /// コンテキストを使用した線分間交点計算
    pub fn intersection_with_segment_context(
        &self,
        other: &Self,
        context: &GeometryContext<T>,
    ) -> Option<Point2D<T>> {
        // まず無限直線同士の交点を求める
        let line_intersection = self.line.intersection_with_line(&other.line)?;

        // 交点が両方の線分上にあるかチェック（コンテキストの距離許容誤差を使用）
        if self.contains_point(&line_intersection, context.tolerances.distance_tolerance)
            && other.contains_point(&line_intersection, context.tolerances.distance_tolerance)
        {
            Some(line_intersection)
        } else {
            None
        }
    }
}

// === foundation トレイト実装 ===

impl<T: Scalar> CoreFoundation<T> for LineSegment2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = BBox2D<T>;

    /// 線分の境界ボックス
    fn bounding_box(&self) -> Self::BBox {
        let start = self.start();
        let end = self.end();

        let min_x = start.x().min(end.x());
        let max_x = start.x().max(end.x());
        let min_y = start.y().min(end.y());
        let max_y = start.y().max(end.y());

        BBox2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }
}

impl<T: Scalar> BasicMetrics<T> for LineSegment2D<T> {
    /// 線分の長さ
    fn length(&self) -> Option<T> {
        Some(self.length())
    }
}

impl<T: Scalar> BasicContainment<T> for LineSegment2D<T> {
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        self.contains_point(point, tolerance)
    }

    /// 点が線分の境界上にあるかを判定（線分では contains_point と同じ）
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    /// 点から線分への最短距離
    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to_point(point)
    }
}

impl<T: Scalar> BasicParametric<T> for LineSegment2D<T> {
    /// パラメータ範囲（0から1）
    fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    /// 正規化されたパラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point {
        self.point_at_normalized_parameter(t)
    }

    /// 正規化されたパラメータでの接線ベクトル
    fn tangent_at_parameter(&self, _t: T) -> Self::Vector {
        // 線分の接線ベクトルは一定（方向ベクトル）
        self.direction() * self.length()
    }
}

impl<T: Scalar> BasicDirectional<T> for LineSegment2D<T> {
    type Direction = Vector2D<T>;

    /// 線分の方向ベクトル
    fn direction(&self) -> Self::Direction {
        self.direction()
    }

    /// 方向を反転した線分
    fn reverse_direction(&self) -> Self {
        self.reverse()
    }
}
