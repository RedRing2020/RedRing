//! 線分（LineSegment）の新実装
//!
//! foundation.rs の基盤トレイトに基づく LineSegment3D の実装
//! InfiniteLine3D を内部活用

use crate::{BBox3D, InfiniteLine3D, Point3D, Vector3D};
use geo_foundation::{abstract_types::geometry::core_foundation::*, Scalar};

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

impl<T: Scalar> LineSegment3D<T> {
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
        (self.end_param - self.start_param).abs()
    }

    /// 方向ベクトルを取得（正規化済み）
    pub fn direction(&self) -> Vector3D<T> {
        if self.end_param >= self.start_param {
            self.line.direction()
        } else {
            self.line.direction().negate()
        }
    }

    /// ベクトル表現を取得（始点から終点へのベクトル）
    pub fn vector(&self) -> Vector3D<T> {
        let start = self.start();
        let end = self.end();
        Vector3D::from_points(&start, &end)
    }

    /// 基盤となる無限直線を取得
    pub fn infinite_line(&self) -> InfiniteLine3D<T> {
        self.line
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

        let param = self.start_param + t * (self.end_param - self.start_param);
        self.line.point_at_parameter(param)
    }

    /// 点を線分に投影（線分内に制限）
    pub fn project_point_to_segment(&self, point: &Point3D<T>) -> Point3D<T> {
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
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let projected = self.project_point_to_segment(point);
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

    /// 点から線分上の点へのパラメータを取得（0〜1範囲外も可能）
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let line_param = self.line.parameter_for_point(point);
        let segment_length = self.end_param - self.start_param;

        if segment_length.abs() < T::EPSILON {
            T::ZERO // 長さゼロの線分
        } else {
            (line_param - self.start_param) / segment_length
        }
    }

    /// 線分を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector3D<T>) -> Self {
        let new_line = self.line.translate(vector);
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
        self.line.is_parallel(&other.line, tolerance)
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
}

// === foundation トレイト実装 ===

impl<T: Scalar> CoreFoundation<T> for LineSegment3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;
    type BBox = BBox3D<T>;

    /// 線分の境界ボックス
    fn bounding_box(&self) -> Self::BBox {
        let start = self.start();
        let end = self.end();

        let min_x = start.x().min(end.x());
        let max_x = start.x().max(end.x());
        let min_y = start.y().min(end.y());
        let max_y = start.y().max(end.y());
        let min_z = start.z().min(end.z());
        let max_z = start.z().max(end.z());

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }
}

impl<T: Scalar> BasicMetrics<T> for LineSegment3D<T> {
    /// 線分の長さ
    fn length(&self) -> Option<T> {
        Some(self.length())
    }
}

impl<T: Scalar> BasicContainment<T> for LineSegment3D<T> {
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point(point, T::TOLERANCE)
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

impl<T: Scalar> BasicParametric<T> for LineSegment3D<T> {
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

impl<T: Scalar> BasicDirectional<T> for LineSegment3D<T> {
    type Direction = Vector3D<T>;

    /// 線分の方向ベクトル
    fn direction(&self) -> Self::Direction {
        self.direction()
    }

    /// 方向を反転した線分
    fn reverse_direction(&self) -> Self {
        self.reverse()
    }
}
