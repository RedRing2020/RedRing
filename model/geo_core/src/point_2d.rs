//! Point2D Core 実装
//!
//! Foundation統一システムに基づくPoint2Dの必須機能のみ

#[cfg(test)]
mod tests;

use crate::Vector2D;
use analysis::abstract_types::Scalar;

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

    /// 座標をタプルとして取得
    pub fn to_tuple(&self) -> (T, T) {
        (self.x, self.y)
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: Self) -> T {
        self.distance_squared_to(other).sqrt()
    }

    /// 他の点との距離の二乗を計算（sqrt回避で高速）
    pub fn distance_squared_to(&self, other: Self) -> T {
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

    /// 2点の中点を計算
    pub fn midpoint(&self, other: Self) -> Self {
        Self::new(
            (self.x + other.x) / (T::ONE + T::ONE),
            (self.y + other.y) / (T::ONE + T::ONE),
        )
    }

    /// 別の点との線形補間（t=0で自分、t=1で相手）
    pub fn lerp(&self, other: Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

impl<T: Scalar> Point2D<T> {
    /// 他の点へのベクトル
    pub fn vector_to(&self, other: Self) -> Vector2D<T> {
        Vector2D::new(other.x - self.x, other.y - self.y)
    }

    /// Vector2Dに変換
    pub fn to_vector(&self) -> Vector2D<T> {
        Vector2D::new(self.x, self.y)
    }

    /// ベクトルから点を作成
    pub fn from_vector(vector: Vector2D<T>) -> Self {
        Point2D::new(vector.x(), vector.y())
    }

    /// スケール（非均等）
    pub fn scale(&self, scale_x: T, scale_y: T) -> Self {
        Point2D::new(self.x * scale_x, self.y * scale_y)
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

impl<T: Scalar> From<Point2D<T>> for analysis::linalg::vector::Vector2<T> {
    fn from(point: Point2D<T>) -> Self {
        analysis::linalg::vector::Vector2::new(point.x(), point.y())
    }
}

impl<T: Scalar> From<analysis::linalg::vector::Vector2<T>> for Point2D<T> {
    fn from(vector: analysis::linalg::vector::Vector2<T>) -> Self {
        Point2D::new(vector.x(), vector.y())
    }
}
