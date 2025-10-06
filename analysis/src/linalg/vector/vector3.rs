/// 3次元ベクトル
/// 
/// 3D幾何計算、物理シミュレーション、3Dグラフィックスに最適化
/// CAD/CAMの座標変換や法線ベクトル計算に使用

use crate::linalg::scalar::Scalar;
use std::ops::{Add, Sub, Mul, Neg};

/// 3次元固定サイズベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3<T: Scalar> {
    pub data: [T; 3],
}

impl<T: Scalar> Vector3<T> {
    /// 新しい3Dベクトルを作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { data: [x, y, z] }
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// X軸単位ベクトル（1, 0, 0）
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO, T::ZERO)
    }

    /// Y軸単位ベクトル（0, 1, 0）
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE, T::ZERO)
    }

    /// Z軸単位ベクトル（0, 0, 1）
    pub fn unit_z() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ONE)
    }

    /// 全成分が1のベクトル（正規化済み）
    pub fn one() -> Self {
        let sqrt3_inv = T::ONE / (T::ONE + T::ONE + T::ONE).sqrt();
        Self::new(sqrt3_inv, sqrt3_inv, sqrt3_inv)
    }

    /// X成分にアクセス
    pub fn x(&self) -> T { 
        self.data[0] 
    }

    /// Y成分にアクセス
    pub fn y(&self) -> T { 
        self.data[1] 
    }

    /// Z成分にアクセス
    pub fn z(&self) -> T { 
        self.data[2] 
    }

    /// 成分を設定
    pub fn set_x(&mut self, x: T) {
        self.data[0] = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.data[1] = y;
    }

    pub fn set_z(&mut self, z: T) {
        self.data[2] = z;
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0] + 
        self.data[1] * other.data[1] + 
        self.data[2] * other.data[2]
    }

    /// 外積（3D専用）
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.data[1] * other.data[2] - self.data[2] * other.data[1],
            self.data[2] * other.data[0] - self.data[0] * other.data[2],
            self.data[0] * other.data[1] - self.data[1] * other.data[0],
        )
    }

    /// ユークリッドノルム
    pub fn norm(&self) -> T {
        (self.data[0] * self.data[0] + 
         self.data[1] * self.data[1] + 
         self.data[2] * self.data[2]).sqrt()
    }

    /// ノルムの2乗（平方根計算を避ける）
    pub fn norm_squared(&self) -> T {
        self.data[0] * self.data[0] + 
        self.data[1] * self.data[1] + 
        self.data[2] * self.data[2]
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Result<Self, String> {
        let norm = self.norm();
        if norm.is_zero() {
            return Err("Cannot normalize zero vector".to_string());
        }
        Ok(Self::new(
            self.data[0] / norm,
            self.data[1] / norm,
            self.data[2] / norm
        ))
    }

    /// 他のベクトルとの角度（ラジアン）
    pub fn angle_to(&self, other: &Self) -> T {
        let dot = self.dot(other);
        let norms = self.norm() * other.norm();
        if norms.is_zero() {
            return T::ZERO;
        }
        (dot / norms).acos()
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

    /// 平面への射影（法線ベクトルを指定）
    pub fn project_onto_plane(&self, normal: &Self) -> Result<Self, String> {
        let projection = self.project_onto(normal)?;
        Ok(*self - projection)
    }

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self * (T::ONE - t) + *other * t
    }

    /// 球面線形補間（単位ベクトル用）
    pub fn slerp(&self, other: &Self, t: T) -> Result<Self, String> {
        let dot = self.dot(other);
        let theta = dot.acos();
        
        if theta.abs() < T::EPSILON {
            // ベクトルがほぼ同じ場合は線形補間
            return Ok(self.lerp(other, t));
        }
        
        let sin_theta = theta.sin();
        let a = ((T::ONE - t) * theta).sin() / sin_theta;
        let b = (t * theta).sin() / sin_theta;
        
        Ok(*self * a + *other * b)
    }

    /// 任意軸周りの回転（Rodriguesの回転公式）
    pub fn rotate_around_axis(&self, axis: &Self, angle: T) -> Result<Self, String> {
        let axis_normalized = axis.normalize()?;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        
        let dot_product = self.dot(&axis_normalized);
        let cross_product = axis_normalized.cross(self);
        
        Ok(*self * cos_angle + 
           cross_product * sin_angle + 
           axis_normalized * dot_product * (T::ONE - cos_angle))
    }

    /// スカラー倍
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(
            self.data[0] * scalar,
            self.data[1] * scalar,
            self.data[2] * scalar
        )
    }

    /// 要素ごとの積（Hadamard積）
    pub fn hadamard(&self, other: &Self) -> Self {
        Self::new(
            self.data[0] * other.data[0],
            self.data[1] * other.data[1],
            self.data[2] * other.data[2]
        )
    }

    /// 要素ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].min(other.data[0]),
            self.data[1].min(other.data[1]),
            self.data[2].min(other.data[2])
        )
    }

    /// 要素ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].max(other.data[0]),
            self.data[1].max(other.data[1]),
            self.data[2].max(other.data[2])
        )
    }

    /// 絶対値
    pub fn abs(&self) -> Self {
        Self::new(
            self.data[0].abs(),
            self.data[1].abs(),
            self.data[2].abs()
        )
    }

    /// 4次元ベクトルに変換（同次座標、w=1）
    pub fn to_homogeneous(&self) -> super::vector4::Vector4<T> {
        super::vector4::Vector4::new(self.data[0], self.data[1], self.data[2], T::ONE)
    }

    /// 4次元ベクトルに変換（方向ベクトル、w=0）
    pub fn to_direction(&self) -> super::vector4::Vector4<T> {
        super::vector4::Vector4::new(self.data[0], self.data[1], self.data[2], T::ZERO)
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Vector3<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::new(
            self.data[0] + other.data[0],
            self.data[1] + other.data[1],
            self.data[2] + other.data[2]
        )
    }
}

impl<T: Scalar> Sub for Vector3<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::new(
            self.data[0] - other.data[0],
            self.data[1] - other.data[1],
            self.data[2] - other.data[2]
        )
    }
}

impl<T: Scalar> Mul<T> for Vector3<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        Self::new(
            self.data[0] * scalar,
            self.data[1] * scalar,
            self.data[2] * scalar
        )
    }
}

impl<T: Scalar> Neg for Vector3<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.data[0], -self.data[1], -self.data[2])
    }
}

/// 型エイリアス
pub type Vector3f = Vector3<f32>;
pub type Vector3d = Vector3<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_vector3_creation() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn test_vector3_cross_product() {
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 1.0, 0.0);
        let cross = v1.cross(&v2);
        
        assert_eq!(cross, Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_vector3_norm() {
        let v = Vector3::new(2.0, 3.0, 6.0);
        assert_eq!(v.norm(), 7.0); // √(4 + 9 + 36) = 7
    }

    #[test]
    fn test_vector3_normalize() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        let normalized = v.normalize().unwrap();
        
        assert!((normalized.norm() - 1.0).abs() < 1e-10);
        assert!((normalized.x() - 0.6).abs() < 1e-10);
        assert!((normalized.y() - 0.8).abs() < 1e-10);
        assert!((normalized.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector3_arithmetic() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        
        let sum = v1 + v2;
        assert_eq!(sum, Vector3::new(5.0, 7.0, 9.0));
        
        let diff = v2 - v1;
        assert_eq!(diff, Vector3::new(3.0, 3.0, 3.0));
        
        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector3::new(2.0, 4.0, 6.0));
    }
}