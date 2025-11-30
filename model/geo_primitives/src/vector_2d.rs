//! 2次元ベクトル（Vector2D）の Core 実装
//!
//! Core Foundation パターンに基づく Vector2D の必須機能のみ
//! 拡張機能は vector_2d_extensions.rs を参照

use geo_foundation::{core::vector_traits, Scalar};
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

    /// ベクトルの長さ（ノルム）を計算
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// ベクトルの大きさ（長さの別名）
    pub fn magnitude(&self) -> T {
        self.length()
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

// ============================================================================
// Legacy Foundation Trait Implementations (Temporarily Disabled)
// ============================================================================

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

impl<T: Scalar> std::ops::Div<T> for Vector2D<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl<T: Scalar> Neg for Vector2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T: Scalar> From<(T, T)> for Vector2D<T> {
    fn from(tuple: (T, T)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

// ============================================================================
// geo_foundation abstracts trait implementations
// ============================================================================

/// geo_foundation::core::Vector2D<T> トレイト実装
impl<T: Scalar> vector_traits::Vector2D<T> for Vector2D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}
