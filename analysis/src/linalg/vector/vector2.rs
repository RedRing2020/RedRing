/// 2次元ベクトル
/// 
/// 2D幾何計算、グラフィックス、UI座標に最適化
/// 高速な演算のためコンパイル時サイズ確定

use crate::linalg::scalar::Scalar;
use std::ops::{Add, Sub, Mul, Neg};

/// 2次元固定サイズベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2<T: Scalar> {
    pub data: [T; 2],
}

impl<T: Scalar> Vector2<T> {
    /// 新しい2Dベクトルを作成
    pub fn new(x: T, y: T) -> Self {
        Self { data: [x, y] }
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// 単位ベクトル（1, 0）
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO)
    }

    /// 単位ベクトル（0, 1）
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE)
    }

    /// 単位ベクトル（1, 1）正規化済み
    pub fn one() -> Self {
        let sqrt2_inv = T::ONE / (T::ONE + T::ONE).sqrt();
        Self::new(sqrt2_inv, sqrt2_inv)
    }

    /// X成分にアクセス
    pub fn x(&self) -> T { 
        self.data[0] 
    }

    /// Y成分にアクセス
    pub fn y(&self) -> T { 
        self.data[1] 
    }

    /// 成分を設定
    pub fn set_x(&mut self, x: T) {
        self.data[0] = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.data[1] = y;
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0] + self.data[1] * other.data[1]
    }

    /// 外積（スカラー値）2Dでは z 成分のみ
    pub fn cross(&self, other: &Self) -> T {
        self.data[0] * other.data[1] - self.data[1] * other.data[0]
    }

    /// ユークリッドノルム
    pub fn norm(&self) -> T {
        (self.data[0] * self.data[0] + self.data[1] * self.data[1]).sqrt()
    }

    /// ノルムの2乗（平方根計算を避ける）
    pub fn norm_squared(&self) -> T {
        self.data[0] * self.data[0] + self.data[1] * self.data[1]
    }

    /// マンハッタン距離（L1ノルム）
    pub fn manhattan_distance(&self, other: &Self) -> T {
        (self.data[0] - other.data[0]).abs() + (self.data[1] - other.data[1]).abs()
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Result<Self, String> {
        let norm = self.norm();
        if norm.is_zero() {
            return Err("Cannot normalize zero vector".to_string());
        }
        Ok(Self::new(self.data[0] / norm, self.data[1] / norm))
    }

    /// 90度回転（反時計回り）
    pub fn rotate_90(&self) -> Self {
        Self::new(-self.data[1], self.data[0])
    }

    /// 任意角度回転（ラジアン、反時計回り）
    pub fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            self.data[0] * cos_a - self.data[1] * sin_a,
            self.data[0] * sin_a + self.data[1] * cos_a
        )
    }

    /// 他のベクトルとの角度（ラジアン）
    pub fn angle_to(&self, other: &Self) -> T {
        let dot = self.dot(other);
        let cross = self.cross(other);
        cross.atan2(dot)
    }

    /// 原点からの角度（ラジアン）
    pub fn angle(&self) -> T {
        self.data[1].atan2(self.data[0])
    }

    /// 直交ベクトル（時計回り90度回転）
    pub fn perpendicular(&self) -> Self {
        Self::new(self.data[1], -self.data[0])
    }

    /// 他のベクトルへの射影
    pub fn project_onto(&self, other: &Self) -> Result<Self, String> {
        let other_norm_sq = other.norm_squared();
        if other_norm_sq.is_zero() {
            return Err("Cannot project onto zero vector".to_string());
        }
        let scalar = self.dot(other) / other_norm_sq;
        Ok(*other * scalar)
    }

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self * (T::ONE - t) + *other * t
    }

    /// スカラー倍
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(self.data[0] * scalar, self.data[1] * scalar)
    }

    /// 要素ごとの積（Hadamard積）
    pub fn hadamard(&self, other: &Self) -> Self {
        Self::new(self.data[0] * other.data[0], self.data[1] * other.data[1])
    }

    /// 要素ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].min(other.data[0]),
            self.data[1].min(other.data[1])
        )
    }

    /// 要素ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].max(other.data[0]),
            self.data[1].max(other.data[1])
        )
    }

    /// 絶対値
    pub fn abs(&self) -> Self {
        Self::new(self.data[0].abs(), self.data[1].abs())
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Vector2<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.data[0] + other.data[0], self.data[1] + other.data[1])
    }
}

impl<T: Scalar> Sub for Vector2<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.data[0] - other.data[0], self.data[1] - other.data[1])
    }
}

impl<T: Scalar> Mul<T> for Vector2<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.data[0] * scalar, self.data[1] * scalar)
    }
}

impl<T: Scalar> Neg for Vector2<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.data[0], -self.data[1])
    }
}

/// 型エイリアス
pub type Vector2f = Vector2<f32>;
pub type Vector2d = Vector2<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_vector2_creation() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.x(), 3.0);
        assert_eq!(v.y(), 4.0);
    }

    #[test]
    fn test_vector2_operations() {
        let v1 = Vector2::new(3.0, 4.0);
        let v2 = Vector2::new(1.0, 2.0);
        
        let dot = v1.dot(&v2);
        assert_eq!(dot, 11.0); // 3*1 + 4*2 = 11
        
        let cross = v1.cross(&v2);
        assert_eq!(cross, 2.0); // 3*2 - 4*1 = 2
    }

    #[test]
    fn test_vector2_norm() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.norm(), 5.0); // 3-4-5 直角三角形
        assert_eq!(v.norm_squared(), 25.0);
    }

    #[test]
    fn test_vector2_rotation() {
        let v = Vector2::new(1.0, 0.0);
        let rotated = v.rotate(PI / 2.0);
        
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector2_arithmetic() {
        let v1 = Vector2::new(1.0, 2.0);
        let v2 = Vector2::new(3.0, 4.0);
        
        let sum = v1 + v2;
        assert_eq!(sum, Vector2::new(4.0, 6.0));
        
        let diff = v2 - v1;
        assert_eq!(diff, Vector2::new(2.0, 2.0));
        
        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector2::new(2.0, 4.0));
    }
}