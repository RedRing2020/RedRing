//! Point2D Core 実装
//!
//! Foundation統一システムに基づくPoint2Dの必須機能のみ

use crate::point_traits::{Point2DConstructor, Point2DCore, Point2DMeasure, Point2DProperties};
use crate::Vector2D;
use analysis::abstract_types::{Angle, Scalar};
use analysis::linalg::vector::Vector2;

use std::ops::{Add, Mul, Neg, Sub};

/// 2次元空間の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Point2D<T> {
    /// 新しい点を作成
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// 原点を取得
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// タプルから点を作成
    pub fn from_tuple(coords: (T, T)) -> Self {
        Self::new(coords.0, coords.1)
    }

    /// X座標を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// 座標を配列として取得
    pub fn coords(&self) -> [T; 2] {
        [self.x, self.y]
    }

    /// 座標をタプルとして取得
    pub fn to_tuple(&self) -> (T, T) {
        (self.x, self.y)
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> T {
        self.distance_squared_to(other).sqrt()
    }

    /// 他の点との距離の二乗を計算（sqrt回避で高速）
    pub fn distance_squared_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// 原点からの距離（ノルム）
    pub fn norm(&self) -> T {
        self.norm_squared().sqrt()
    }

    /// 原点からの距離の二乗
    pub fn norm_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    /// 極座標から点を作成（r: 半径, theta: 角度）
    pub fn from_polar(r: T, theta: T) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }

    /// 極座標の半径成分を取得
    pub fn polar_radius(&self) -> T {
        self.norm()
    }

    /// 極座標の角度成分を取得（ラジアン）
    pub fn polar_angle(&self) -> T {
        self.y.atan2(self.x)
    }

    /// 2点の中点を計算
    pub fn midpoint(&self, other: &Self) -> Self {
        Self::new(
            (self.x + other.x) / (T::ONE + T::ONE),
            (self.y + other.y) / (T::ONE + T::ONE),
        )
    }

    /// 別の点との線形補間（t=0で自分、t=1で相手）
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// マンハッタン距離（L1ノルム）
    pub fn manhattan_distance_to(&self, other: &Self) -> T {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// チェビシェフ距離（L∞ノルム）
    pub fn chebyshev_distance_to(&self, other: &Self) -> T {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx.max(dy)
    }
}

// 一時的に無効化した旧 Foundation 実装

/*
impl<T: Scalar> Point2DTrait<T> for Point2D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}


impl<T: Scalar> BasicContainment<T> for Point2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        *self == *point
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to(point)
    }
}
*/

impl<T: Scalar> Point2D<T> {
    /// 他の点へのベクトル
    pub fn vector_to(&self, other: &Self) -> Vector2D<T> {
        Vector2D::new(other.x - self.x, other.y - self.y)
    }

    /// 指定点周りの回転（Angle<T>型角度）
    pub fn rotate_around(&self, center: &Self, angle: Angle<T>) -> Self {
        let offset = Vector2D::new(self.x - center.x, self.y - center.y);
        let radians = angle.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();
        let rotated_x = offset.x() * cos_a - offset.y() * sin_a;
        let rotated_y = offset.x() * sin_a + offset.y() * cos_a;
        Point2D::new(center.x + rotated_x, center.y + rotated_y)
    }

    /// 指定点周りの回転（T型角度）- 後方互換性のため
    pub fn rotate_around_radians(&self, center: &Self, angle: T) -> Self {
        self.rotate_around(center, Angle::from_radians(angle))
    }

    /// 原点周りの回転（Angle<T>型角度）
    pub fn rotate(&self, angle: Angle<T>) -> Self {
        let radians = angle.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();
        let rotated_x = self.x * cos_a - self.y * sin_a;
        let rotated_y = self.x * sin_a + self.y * cos_a;
        Point2D::new(rotated_x, rotated_y)
    }

    /// 原点周りの回転（T型角度）- 後方互換性のため
    pub fn rotate_radians(&self, angle: T) -> Self {
        self.rotate(Angle::from_radians(angle))
    }

    /// 3D点に変換（Z=0）
    pub fn to_3d(&self) -> crate::Point3D<T> {
        crate::Point3D::new(self.x, self.y, T::ZERO)
    }

    /// 3D点に変換（Z指定）
    pub fn to_3d_with_z(&self, z: T) -> crate::Point3D<T> {
        crate::Point3D::new(self.x, self.y, z)
    }

    /// Vector2Dに変換
    pub fn to_vector(&self) -> Vector2D<T> {
        Vector2D::new(self.x, self.y)
    }

    /// ベクトルから点を作成
    pub fn from_vector(vector: Vector2D<T>) -> Self {
        Point2D::new(vector.x(), vector.y())
    }

    /// 原点判定
    pub fn is_origin(&self) -> bool {
        self.x.abs() <= T::EPSILON && self.y.abs() <= T::EPSILON
    }

    /// 近似等価判定
    pub fn is_approximately_equal(&self, other: &Self, tolerance: T) -> bool {
        (self.x - other.x).abs() <= tolerance && (self.y - other.y).abs() <= tolerance
    }

    /// X軸反射
    pub fn reflect_x(&self) -> Self {
        Point2D::new(-self.x, self.y)
    }

    /// Y軸反射
    pub fn reflect_y(&self) -> Self {
        Point2D::new(self.x, -self.y)
    }

    /// 原点反射
    pub fn reflect_origin(&self) -> Self {
        Point2D::new(-self.x, -self.y)
    }

    /// 平行移動
    pub fn translate(&self, vector: Vector2D<T>) -> Self {
        *self + vector
    }

    /// スケール（非均等）
    pub fn scale(&self, scale_x: T, scale_y: T) -> Self {
        Point2D::new(self.x * scale_x, self.y * scale_y)
    }

    /// 均等スケール
    pub fn scale_uniform(&self, scale: T) -> Self {
        self.scale(scale, scale)
    }
}

impl<T: Scalar> Add<Vector2D<T>> for Point2D<T> {
    type Output = Self;

    fn add(self, rhs: Vector2D<T>) -> Self::Output {
        Self::new(self.x + rhs.x(), self.y + rhs.y())
    }
}

impl<T: Scalar> Sub<Vector2D<T>> for Point2D<T> {
    type Output = Self;

    fn sub(self, rhs: Vector2D<T>) -> Self::Output {
        Self::new(self.x - rhs.x(), self.y - rhs.y())
    }
}

impl<T: Scalar> Sub<Point2D<T>> for Point2D<T> {
    type Output = Vector2D<T>;

    fn sub(self, rhs: Point2D<T>) -> Self::Output {
        Vector2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Scalar> Mul<T> for Point2D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: Scalar> Neg for Point2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T: Scalar> Default for Point2D<T> {
    fn default() -> Self {
        Self::origin()
    }
}

impl<T: Scalar> Point2DConstructor<T> for Point2D<T> {
    fn new(x: T, y: T) -> Self {
        Point2D::new(x, y)
    }

    fn origin() -> Self {
        Point2D::origin()
    }

    fn from_tuple(coords: (T, T)) -> Self {
        Point2D::from_tuple(coords)
    }

    fn from_analysis_vector(vector: &Vector2<T>) -> Self {
        Point2D::new(vector.x(), vector.y())
    }

    fn from_point(other: &Self) -> Self {
        *other
    }

    fn from_polar(r: T, theta: T) -> Self {
        Point2D::from_polar(r, theta)
    }
}

impl<T: Scalar> Point2DProperties<T> for Point2D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn coords(&self) -> [T; 2] {
        [self.x, self.y]
    }

    fn to_tuple(&self) -> (T, T) {
        (self.x, self.y)
    }

    fn to_analysis_vector(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    fn polar_radius(&self) -> T {
        Point2D::polar_radius(self)
    }
}

impl<T: Scalar> Point2DMeasure<T> for Point2D<T> {
    fn distance_to(&self, other: &Self) -> T {
        self.distance_to(other)
    }

    fn distance_squared_to(&self, other: &Self) -> T {
        self.distance_squared_to(other)
    }

    fn distance_from_origin(&self) -> T {
        let origin = Point2D::origin();
        self.distance_to(&origin)
    }

    fn norm_squared(&self) -> T {
        let origin = Point2D::origin();
        self.distance_squared_to(&origin)
    }

    fn midpoint(&self, other: &Self) -> Self {
        self.midpoint(other)
    }

    fn lerp(&self, other: &Self, t: T) -> Self {
        self.lerp(other, t)
    }

    fn manhattan_distance_to(&self, other: &Self) -> T {
        self.manhattan_distance_to(other)
    }

    fn chebyshev_distance_to(&self, other: &Self) -> T {
        self.chebyshev_distance_to(other)
    }
}

impl<T: Scalar> Point2DCore<T> for Point2D<T> {}
