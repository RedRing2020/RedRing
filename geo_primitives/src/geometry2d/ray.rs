//! Ray2D - ジェネリック2Dレイ（半無限直線）の実装
//!
//! 起点と方向を持つ2次元半無限直線をサポート
//! 新しい設計では、InfiniteLine2Dを継承した制約版として実装されます

use crate::geometry2d::{Direction2D, Point2D, Vector};
use geo_foundation::abstract_types::geometry::{
    Direction,
    Direction2D as Direction2DTrait,
    // Ray2D トレイトは新しい設計では InfiniteLine2D を継承
};
use geo_foundation::{Angle, Scalar};

/// ジェネリック2Dレイ（半無限直線）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray2D<T: Scalar> {
    /// レイの起点
    origin: Point2D<T>,
    /// レイの方向（正規化済み）
    direction: Direction2D<T>,
}

impl<T: Scalar> Ray2D<T> {
    /// 新しいRay2Dを作成
    pub fn new(origin: Point2D<T>, direction: Direction2D<T>) -> Self {
        Self { origin, direction }
    }

    /// 起点と方向ベクトルからRay2Dを作成
    pub fn from_origin_and_vector(origin: Point2D<T>, direction_vector: Vector<T>) -> Option<Self> {
        Direction2D::from_vector(direction_vector).map(|dir| Self::new(origin, dir))
    }

    /// 2点を通るレイを作成（fromからtoward方向）
    pub fn from_two_points(from: Point2D<T>, toward: Point2D<T>) -> Option<Self> {
        let direction_vector = toward - from;
        Self::from_origin_and_vector(from, direction_vector)
    }

    /// X軸正方向のレイを作成
    pub fn x_axis(origin: Point2D<T>) -> Self {
        Self::new(origin, Direction2D::positive_x())
    }

    /// Y軸正方向のレイを作成
    pub fn y_axis(origin: Point2D<T>) -> Self {
        Self::new(origin, Direction2D::positive_y())
    }

    /// レイを平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        Self::new(self.origin + *vector, self.direction)
    }

    /// レイを指定角度回転（起点中心）
    pub fn rotate(&self, angle: Angle<T>) -> Self {
        let rotated_origin = self.origin.rotate(angle.to_radians());
        let rotated_direction =
            Direction2D::from_angle(self.direction.to_angle() + angle.to_radians());
        Self::new(rotated_origin, rotated_direction)
    }

    /// レイをミラー反転（Y軸に対して）
    pub fn mirror_y(&self) -> Self {
        let mirrored_origin = Point2D::new(-self.origin.x(), self.origin.y());
        let mirrored_direction = Direction2D::from_angle(-self.direction.to_angle());
        Self::new(mirrored_origin, mirrored_direction)
    }

    /// レイをミラー反転（X軸に対して）
    pub fn mirror_x(&self) -> Self {
        let mirrored_origin = Point2D::new(self.origin.x(), -self.origin.y());
        let angle = Angle::from_radians(-self.direction.to_angle());
        let mirrored_direction = Direction2D::from_angle(angle.to_radians());
        Self::new(mirrored_origin, mirrored_direction)
    }

    /// レイ固有：制約付きの点取得（t >= 0のみ）
    pub fn point_at_parameter_ray(&self, t: T) -> Option<Point2D<T>> {
        if t >= T::ZERO {
            let direction_vector = self.direction.to_vector();
            Some(self.origin + direction_vector * t)
        } else {
            None // 半無限直線なので t < 0 は無効
        }
    }

    /// レイ固有：制約付きの点判定（前方のみ）
    pub fn contains_point_ray(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point_unlimited(point);
        if distance <= tolerance {
            let param = self.parameter_at_point_unlimited(point);
            param >= -tolerance // 許容誤差を考慮して前方判定
        } else {
            false
        }
    }

    /// 制約なしの距離計算（InfiniteLineの機能）
    pub fn distance_to_point_unlimited(&self, point: &Point2D<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();
        let cross_product = to_point.cross_2d(&direction_vector);
        cross_product.abs()
    }

    /// 制約なしのパラメータ計算（InfiniteLineの機能）
    pub fn parameter_at_point_unlimited(&self, point: &Point2D<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();
        to_point.dot(&direction_vector)
    }

    /// 点から直線（レイを無限に延長したもの）への距離を計算
    pub fn distance_to_line(&self, point: &Point2D<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();

        // 点からレイの直線への投影
        let projection_length = to_point.dot(&direction_vector);
        let projection_point = self.origin + direction_vector * projection_length;
        point.distance_to(&projection_point)
    }

    /// レイの起点を取得（InfiniteLineの origin メソッドと互換）
    pub fn origin(&self) -> Point2D<T> {
        self.origin
    }

    /// レイの方向を取得（InfiniteLineの direction メソッドと互換）
    pub fn direction(&self) -> Direction2D<T> {
        self.direction
    }

    /// 制約なしの点取得（InfiniteLineの point_at_parameter メソッドと互換）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let direction_vector = self.direction.to_vector();
        self.origin + direction_vector * t
    }

    /// 他のレイと平行かどうかを判定
    pub fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool {
        self.direction.is_parallel(&other.direction, tolerance)
    }

    /// 他のレイと同一かどうかを判定
    pub fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool {
        // 方向が平行かつ両方の起点が同一直線上にある
        if !self.is_parallel_to(other, tolerance) {
            return false;
        }

        // 一方の起点がもう一方の直線上にあるかチェック（レイの範囲を超えても良い）
        let distance_to_line = self.distance_to_line(&other.origin);
        distance_to_line <= tolerance
    }
}

// 型エイリアス（後方互換性確保）
/// f64版の2D Ray（デフォルト）
pub type Ray2DF64 = Ray2D<f64>;

/// f32版の2D Ray（高速演算用）
pub type Ray2DF32 = Ray2D<f32>;
