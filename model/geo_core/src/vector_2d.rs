//! 2次元ベクトル（Vector2D）の実装
//!
//! geo_core内部実装として、全機能を統合

#[cfg(test)]
mod tests;

use analysis::abstract_types::{Angle, Scalar};

use std::ops::{Add, Mul, Neg, Sub};

/// 2次元ベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Vector2D<T> {
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

    /// ベクトルの長さの二乗を取得
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    /// ベクトルの長さ（ノルム）を計算
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

    /// 内積を計算
    pub fn dot(&self, other: Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// 外積のZ成分を計算（2Dでは実際の外積ではなく、Z成分のスカラー値）
    pub fn cross(&self, other: Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// 2つの点からベクトルを作成（to - from）
    pub fn from_points(from: crate::Point2D<T>, to: crate::Point2D<T>) -> Self {
        from.vector_to(to)
    }

    /// ベクトルがゼロベクトルかを判定
    pub fn is_zero(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }

    /// ベクトルを90度回転（反時計回り）
    pub fn rotate_90(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// ベクトルを90度回転（時計回り）
    pub fn rotate_neg_90(&self) -> Self {
        Self::new(self.y, -self.x)
    }

    /// ベクトルを反転（-selfと同じ）
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y)
    }

    /// ベクトル間の角度を取得
    pub fn angle_to(&self, other: Self) -> Angle<T> {
        let dot = self.dot(other);
        let cross = self.cross(other);
        Angle::from_radians(cross.atan2(dot))
    }

    /// 2つのベクトル間で線形補間
    pub fn lerp(&self, other: Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// 投影ベクトルを計算（このベクトルをotherに投影）
    pub fn project_onto(&self, other: Self) -> Self {
        let other_len_sq = other.length_squared();
        if other_len_sq <= T::ZERO {
            Self::zero()
        } else {
            other * (self.dot(other) / other_len_sq)
        }
    }

    /// 成分ごとの最小値
    pub fn min(&self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// 成分ごとの最大値
    pub fn max(&self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// 成分ごとの絶対値
    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }
}

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

impl<T: Scalar> From<Vector2D<T>> for analysis::linalg::vector::Vector2<T> {
    fn from(vector: Vector2D<T>) -> Self {
        analysis::linalg::vector::Vector2::new(vector.x(), vector.y())
    }
}

impl<T: Scalar> From<analysis::linalg::vector::Vector2<T>> for Vector2D<T> {
    fn from(vector: analysis::linalg::vector::Vector2<T>) -> Self {
        Vector2D::new(vector.x(), vector.y())
    }
}
