//! 2次元点の数値計算実装
//!
//! 数学的な2次元点（位置）を表す型
//! Vector2との相互変換とトレイト共通化を提供

use crate::{linalg::vector::Vector2, Scalar};
use std::ops::Index;

/// 2次元点
///
/// 2次元平面内の位置を表す
/// Vector2とは概念的に異なるが、数値的には同じ構造
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Point2<T> {
    /// 新しい点を作成
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// 原点を作成
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// X座標を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// X座標を設定
    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }

    /// Y座標を設定
    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }

    /// インデックスで座標を取得 (0=x, 1=y)
    #[inline]
    pub fn get(&self, index: usize) -> T {
        match index {
            0 => self.x,
            1 => self.y,
            _ => panic!("Index out of bounds: {}", index),
        }
    }

    /// インデックスで座標を設定 (0=x, 1=y)
    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            _ => panic!("Index out of bounds: {}", index),
        }
    }

    // === 変換 ===

    /// Vector2に変換
    pub fn to_vector(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    /// Vector2から作成
    pub fn from_vector(v: Vector2<T>) -> Self {
        Self::new(v.x(), v.y())
    }

    /// 別の点への距離
    pub fn distance_to(&self, other: &Point2<T>) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// 別の点への距離の二乗
    pub fn distance_squared_to(&self, other: &Point2<T>) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// 点間の中点
    pub fn midpoint(&self, other: &Point2<T>) -> Point2<T> {
        let two = T::ONE + T::ONE;
        Point2::new((self.x + other.x) / two, (self.y + other.y) / two)
    }

    /// 線形補間
    pub fn lerp(&self, other: &Point2<T>, t: T) -> Point2<T> {
        let one_minus_t = T::ONE - t;
        Point2::new(
            self.x * one_minus_t + other.x * t,
            self.y * one_minus_t + other.y * t,
        )
    }

    /// 3次元に拡張（z = 0）
    pub fn to_3d(&self) -> crate::linalg::point3::Point3<T> {
        crate::linalg::point3::Point3::new(self.x, self.y, T::ZERO)
    }

    /// 3次元に拡張（z座標指定）
    pub fn to_3d_with_z(&self, z: T) -> crate::linalg::point3::Point3<T> {
        crate::linalg::point3::Point3::new(self.x, self.y, z)
    }
}

/// Point2からVector2への変換
impl<T: Scalar> From<Point2<T>> for Vector2<T> {
    fn from(p: Point2<T>) -> Self {
        p.to_vector()
    }
}

/// Vector2からPoint2への変換
impl<T: Scalar> From<Vector2<T>> for Point2<T> {
    fn from(v: Vector2<T>) -> Self {
        Point2::from_vector(v)
    }
}

/// Point2とPoint2の減算（点-点=ベクトル）
impl<T: Scalar> std::ops::Sub for Point2<T> {
    type Output = Vector2<T>;

    fn sub(self, rhs: Point2<T>) -> Self::Output {
        Vector2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// Point2とVector2の加算（点+ベクトル=点）
impl<T: Scalar> std::ops::Add<Vector2<T>> for Point2<T> {
    type Output = Point2<T>;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Point2::new(self.x + rhs.x(), self.y + rhs.y())
    }
}

/// Point2とVector2の減算（点-ベクトル=点）
impl<T: Scalar> std::ops::Sub<Vector2<T>> for Point2<T> {
    type Output = Point2<T>;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Point2::new(self.x - rhs.x(), self.y - rhs.y())
    }
}

// === 添え字演算子 ===

impl<T: Scalar> Index<usize> for Point2<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &T {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of bounds: {}", index),
        }
    }
}

// === 配列変換 ===

impl<T: Scalar> From<[T; 2]> for Point2<T> {
    #[inline]
    fn from(data: [T; 2]) -> Self {
        Self::new(data[0], data[1])
    }
}

// === トレイト実装 ===

/// 2次元座標アクセスの共通トレイト
///
/// Point2とVector2で座標アクセスを統一
pub trait Coordinates2D<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
}

impl<T: Scalar> Coordinates2D<T> for Point2<T> {
    fn x(&self) -> T {
        self.x
    }
    fn y(&self) -> T {
        self.y
    }
}

impl<T: Scalar> Coordinates2D<T> for Vector2<T> {
    fn x(&self) -> T {
        Vector2::x(self)
    }
    fn y(&self) -> T {
        Vector2::y(self)
    }
}
