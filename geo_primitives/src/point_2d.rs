//! 2次元点（Point2D）の Core 実装
//!
//! Core Foundation パターンに基づく Point2D の必須機能のみ
//! 拡張機能は point_2d_extensions.rs を参照

use crate::Vector2D;
use geo_foundation::{
    abstract_types::geometry::core_foundation::{BasicContainment, CoreFoundation},
    Scalar,
};

use std::ops::{Add, Mul, Neg, Sub};

/// 2次元空間の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D<T: Scalar> {
    x: T,
    y: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Point2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

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

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

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

    // ========================================================================
    // Core Distance Methods (基本距離計算)
    // ========================================================================

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// 他の点との距離の二乗を計算（sqrt回避で高速）
    pub fn distance_squared_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// 原点からの距離（ノルム）
    pub fn norm(&self) -> T {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 原点からの距離の二乗
    pub fn norm_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }
}

// ============================================================================
// Core Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> CoreFoundation<T> for Point2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = crate::BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        crate::BBox2D::from_point(*self)
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

// ============================================================================
// 基本演算子実装 (Basic Operator Implementations)
// ============================================================================

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
