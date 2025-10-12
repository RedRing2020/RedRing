//! 2次元ベクトル（Vector2D）の Core 実装
//!
//! Core Foundation パターンに基づく Vector2D の必須機能のみ
//! 拡張機能は vector_2d_extensions.rs を参照

use crate::Point2D;
use geo_foundation::{
    abstract_types::geometry::core_foundation::{BasicDirectional, BasicMetrics, CoreFoundation},
    Scalar,
};
use std::ops::{Add, Mul, Neg, Sub};

/// 2次元ベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D<T: Scalar> {
    x: T,
    y: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Vector2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しいベクトルを作成
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// ゼロベクトルを取得
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// X軸単位ベクトルを取得
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO)
    }

    /// Y軸単位ベクトルを取得
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// X成分を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y成分を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// 成分を配列として取得
    pub fn components(&self) -> [T; 2] {
        [self.x, self.y]
    }

    // ========================================================================
    // Core Metrics Methods
    // ========================================================================

    /// ベクトルの長さの二乗を取得
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    /// ベクトルの長さを取得
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// ベクトルを正規化（長さを1にする）
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len <= T::ZERO {
            Self::zero()
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// 内積を計算
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// 外積のZ成分を計算（2Dでは実際の外積ではなく、Z成分のスカラー値）
    pub fn cross(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }
}

// ============================================================================
// Core Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> CoreFoundation<T> for Vector2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = crate::BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        // ベクトルは原点から終点への境界ボックス
        // 最小・最大を正しく設定
        let min_x = T::ZERO.min(self.x);
        let max_x = T::ZERO.max(self.x);
        let min_y = T::ZERO.min(self.y);
        let max_y = T::ZERO.max(self.y);

        crate::BBox2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }
}

impl<T: Scalar> BasicMetrics<T> for Vector2D<T> {
    fn length(&self) -> Option<T> {
        Some(Vector2D::length(self))
    }
}

impl<T: Scalar> BasicDirectional<T> for Vector2D<T> {
    type Direction = Vector2D<T>;

    fn direction(&self) -> Self::Direction {
        self.normalize()
    }

    fn reverse_direction(&self) -> Self {
        -*self
    }
}

// ============================================================================
// 基本演算子実装 (Basic Operator Implementations)
// ============================================================================

impl<T: Scalar> Add for Vector2D<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Scalar> Sub for Vector2D<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<T: Scalar> Mul<T> for Vector2D<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl<T: Scalar> Neg for Vector2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}
