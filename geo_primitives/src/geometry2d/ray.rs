//! Ray - ジェネリック2Dレイ（半無限直線）の実装
//!
//! 起点と方向を持つ2次元半無限直線をサポート
//! 新しい設計では、InfiniteLine2Dを継承した制約版として実装されます

use crate::geometry2d::{Direction, Point, Vector};
use geo_foundation::abstract_types::geometry::{
    Direction as DirectionTrait,
    InfiniteLine2D,
    Ray2D as Ray2DTrait,
    // RayOps,              // TODO: 後で有効化
    // RayIntersection,     // TODO: 後で有効化
};
use geo_foundation::{Angle, Scalar};

/// ジェネリック2Dレイ（半無限直線）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray<T: Scalar> {
    /// レイの起点
    origin: Point<T>,
    /// レイの方向（正規化済み）
    direction: Direction<T>,
}

impl<T: Scalar> Ray<T> {
    /// 新しいRayを作成
    pub fn new(origin: Point<T>, direction: Direction<T>) -> Self {
        Self { origin, direction }
    }

    /// 起点と方向ベクトルからRayを作成
    pub fn from_origin_and_vector(origin: Point<T>, direction_vector: Vector<T>) -> Option<Self> {
        Direction::from_vector(direction_vector).map(|dir| Self::new(origin, dir))
    }

    /// 2点を通るレイを作成（fromからtoward方向）
    pub fn from_two_points(from: Point<T>, toward: Point<T>) -> Option<Self> {
        let direction_vector = toward - from;
        Self::from_origin_and_vector(from, direction_vector)
    }

    /// X軸正方向のレイを作成
    pub fn x_axis(origin: Point<T>) -> Self {
        Self::new(origin, Direction::positive_x())
    }

    /// Y軸正方向のレイを作成
    pub fn y_axis(origin: Point<T>) -> Self {
        Self::new(origin, Direction::positive_y())
    }

    /// レイを平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        Self::new(self.origin + *vector, self.direction)
    }

    /// レイを指定角度回転（起点中心）
    pub fn rotate(&self, angle: Angle<T>) -> Self {
        let rotated_origin = self.origin.rotate(angle.to_radians());
        let rotated_direction =
            Direction::from_angle(self.direction.to_angle() + angle.to_radians());
        Self::new(rotated_origin, rotated_direction)
    }

    /// レイをミラー反転（Y軸に対して）
    pub fn mirror_y(&self) -> Self {
        let mirrored_origin = Point::new(-self.origin.x(), self.origin.y());
        let mirrored_direction = Direction::from_angle(-self.direction.to_angle());
        Self::new(mirrored_origin, mirrored_direction)
    }

    /// レイをミラー反転（X軸に対して）
    pub fn mirror_x(&self) -> Self {
        let mirrored_origin = Point::new(self.origin.x(), -self.origin.y());
        let angle = Angle::from_radians(-self.direction.to_angle());
        let mirrored_direction = Direction::from_angle(angle.to_radians());
        Self::new(mirrored_origin, mirrored_direction)
    }

    /// レイ固有：制約付きの点取得（t >= 0のみ）
    pub fn point_at_parameter_ray(&self, t: T) -> Option<Point<T>> {
        if t >= T::ZERO {
            let direction_vector = self.direction.to_vector();
            Some(self.origin + direction_vector * t)
        } else {
            None // 半無限直線なので t < 0 は無効
        }
    }

    /// レイ固有：制約付きの点判定（前方のみ）
    pub fn contains_point_ray(&self, point: &Point<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point_unlimited(point);
        if distance <= tolerance {
            let param = self.parameter_at_point_unlimited(point);
            param >= -tolerance // 許容誤差を考慮して前方判定
        } else {
            false
        }
    }

    /// 制約なしの距離計算（InfiniteLineの機能）
    pub fn distance_to_point_unlimited(&self, point: &Point<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();
        let cross_product = to_point.cross_2d(&direction_vector);
        cross_product.abs()
    }

    /// 制約なしのパラメータ計算（InfiniteLineの機能）
    pub fn parameter_at_point_unlimited(&self, point: &Point<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();
        to_point.dot(&direction_vector)
    }

    /// 点から直線（レイを無限に延長したもの）への距離を計算
    pub fn distance_to_line(&self, point: &Point<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();

        // 点からレイの直線への投影
        let projection_length = to_point.dot(&direction_vector);
        let projection_point = self.origin + direction_vector * projection_length;
        point.distance_to(&projection_point)
    }

    /// レイの起点を取得（InfiniteLineの origin メソッドと互換）
    pub fn origin(&self) -> Point<T> {
        self.origin
    }

    /// レイの方向を取得（InfiniteLineの direction メソッドと互換）
    pub fn direction(&self) -> Direction<T> {
        self.direction
    }

    /// 制約なしの点取得（InfiniteLineの point_at_parameter メソッドと互換）
    pub fn point_at_parameter(&self, t: T) -> Point<T> {
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

// 新しいトレイト実装
impl<T: Scalar> InfiniteLine2D<T> for Ray<T> {
    type Point = Point<T>;
    type Vector = Vector<T>;
    type Direction = Direction<T>;
    type Error = String; // TODO: 適切なエラー型を定義

    fn origin(&self) -> Self::Point {
        self.origin
    }

    fn direction(&self) -> Self::Direction {
        self.direction
    }

    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool {
        let distance = self.distance_to_point(point);
        distance <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        let to_point = *point - self.origin;
        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.dot(&direction_vec);
        let projection = direction_vec * projection_length;
        let perpendicular = to_point - projection;
        perpendicular.norm()
    }

    fn closest_point(&self, point: &Self::Point) -> Self::Point {
        let to_point = *point - self.origin;
        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.dot(&direction_vec);
        self.origin + direction_vec * projection_length
    }

    fn point_at_parameter(&self, t: T) -> Self::Point {
        let direction_vec = self.direction.to_vector();
        self.origin + direction_vec * t
    }

    fn parameter_at_point(&self, point: &Self::Point) -> T {
        let to_point = *point - self.origin;
        let direction_vec = self.direction.to_vector();
        to_point.dot(&direction_vec)
    }

    fn intersect_line(&self, other: &Self) -> Option<Self::Point> {
        // 2つの無限直線の交点を計算
        let dir1 = self.direction.to_vector();
        let dir2 = other.direction.to_vector();

        // 平行チェック
        let cross = dir1.cross_2d(&dir2);
        if cross.abs() < T::from_f64(1e-10) {
            return None; // 平行または同一直線
        }

        // 交点計算
        let origin_diff = other.origin - self.origin;
        let t = origin_diff.cross_2d(&dir2) / cross;
        Some(self.point_at_parameter(t))
    }

    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool {
        let dir1 = self.direction.to_vector();
        let dir2 = other.direction.to_vector();
        let cross = dir1.cross_2d(&dir2);
        cross.abs() <= tolerance
    }

    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool {
        // 平行かつ片方の起点がもう片方の直線上にある
        if !self.is_parallel_to(other, tolerance) {
            return false;
        }
        let distance = self.distance_to_point(&other.origin);
        distance <= tolerance
    }

    fn normal_vector(&self) -> Self::Vector {
        let dir = self.direction.to_vector();
        Vector::new(-dir.y(), dir.x())
    }
}

impl<T: Scalar> Ray2DTrait<T> for Ray<T> {
    // Ray2DはInfiniteLine2Dを継承するので、デフォルト実装を使用
}

// TODO: RayOps実装は後回し
/*
impl<T: Scalar> RayOps<T> for Ray<T> {
    fn point_at(&self, t: T) -> Option<Self::Point> {
        self.point_at_parameter_ray(t)
    }

    fn parameter_at_point(&self, point: &Self::Point) -> Option<T> {
        let param = self.parameter_at_point(point);
        if param >= T::ZERO {
            Some(param)
        } else {
            None
        }
    }

    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point_ray(point, tolerance)
    }
}

impl<T: Scalar> RayIntersection<T> for Ray<T> {
    fn intersect_ray(&self, other: &Self) -> Option<Self::Point> {
        // レイ同士の交点計算（両方のt >= 0が必要）
        let t1_opt = self.parameter_at_intersection_unlimited(other);
        let t2_opt = other.parameter_at_intersection_unlimited(self);

        match (t1_opt, t2_opt) {
            (Some(t1), Some(t2)) if t1 >= T::ZERO && t2 >= T::ZERO => {
                self.point_at_parameter_ray(t1)
            }
            _ => None,
        }
    }
}
*/

// 型エイリアス（後方互換性確保）
/// f64版の2D Ray（デフォルト）
pub type Ray2D<T> = Ray<T>;
pub type Ray2DF64 = Ray<f64>;

/// f32版の2D Ray（高速演算用）
pub type Ray2DF32 = Ray<f32>;
