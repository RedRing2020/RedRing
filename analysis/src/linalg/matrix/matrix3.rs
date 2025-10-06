/// 3x3行列（高速演算用）
///
/// 3D変換、回転、投影に特化した固定サイズ行列
/// CAD計算とグラフィックス処理の両方に対応

use crate::linalg::scalar::Scalar;
use crate::linalg::vector::Vector3;
use std::ops::{Add, Mul};

/// 3x3行列
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3<T: Scalar> {
    pub data: [[T; 3]; 3],
}

impl<T: Scalar> Matrix3x3<T> {
    pub fn new(
        a11: T, a12: T, a13: T,
        a21: T, a22: T, a23: T,
        a31: T, a32: T, a33: T,
    ) -> Self {
        Self {
            data: [
                [a11, a12, a13],
                [a21, a22, a23],
                [a31, a32, a33],
            ],
        }
    }

    pub fn zeros() -> Self {
        Self::new(
            T::ZERO, T::ZERO, T::ZERO,
            T::ZERO, T::ZERO, T::ZERO,
            T::ZERO, T::ZERO, T::ZERO
        )
    }

    pub fn identity() -> Self {
        Self::new(
            T::ONE, T::ZERO, T::ZERO,
            T::ZERO, T::ONE, T::ZERO,
            T::ZERO, T::ZERO, T::ONE
        )
    }

    pub fn determinant(&self) -> T {
        let a11 = self.data[0][0];
        let a12 = self.data[0][1];
        let a13 = self.data[0][2];
        let a21 = self.data[1][0];
        let a22 = self.data[1][1];
        let a23 = self.data[1][2];
        let a31 = self.data[2][0];
        let a32 = self.data[2][1];
        let a33 = self.data[2][2];

        a11 * (a22 * a33 - a23 * a32) 
        - a12 * (a21 * a33 - a23 * a31) 
        + a13 * (a21 * a32 - a22 * a31)
    }

    pub fn trace(&self) -> T {
        self.data[0][0] + self.data[1][1] + self.data[2][2]
    }

    pub fn transpose(&self) -> Self {
        Self::new(
            self.data[0][0], self.data[1][0], self.data[2][0],
            self.data[0][1], self.data[1][1], self.data[2][1],
            self.data[0][2], self.data[1][2], self.data[2][2]
        )
    }

    pub fn inverse(&self) -> Result<Self, String> {
        let det = self.determinant();
        if det.is_zero() {
            return Err("Matrix is singular".to_string());
        }

        // Adjugate matrix / determinant
        let inv_det = T::ONE / det;
        
        Ok(Self::new(
            (self.data[1][1] * self.data[2][2] - self.data[1][2] * self.data[2][1]) * inv_det,
            (self.data[0][2] * self.data[2][1] - self.data[0][1] * self.data[2][2]) * inv_det,
            (self.data[0][1] * self.data[1][2] - self.data[0][2] * self.data[1][1]) * inv_det,
            
            (self.data[1][2] * self.data[2][0] - self.data[1][0] * self.data[2][2]) * inv_det,
            (self.data[0][0] * self.data[2][2] - self.data[0][2] * self.data[2][0]) * inv_det,
            (self.data[0][2] * self.data[1][0] - self.data[0][0] * self.data[1][2]) * inv_det,
            
            (self.data[1][0] * self.data[2][1] - self.data[1][1] * self.data[2][0]) * inv_det,
            (self.data[0][1] * self.data[2][0] - self.data[0][0] * self.data[2][1]) * inv_det,
            (self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]) * inv_det
        ))
    }

    pub fn mul_vector(&self, vec: &Vector3<T>) -> Vector3<T> {
        Vector3::new(
            self.data[0][0] * vec.x() + self.data[0][1] * vec.y() + self.data[0][2] * vec.z(),
            self.data[1][0] * vec.x() + self.data[1][1] * vec.y() + self.data[1][2] * vec.z(),
            self.data[2][0] * vec.x() + self.data[2][1] * vec.y() + self.data[2][2] * vec.z()
        )
    }

    pub fn mul_matrix(&self, other: &Self) -> Self {
        let mut result = Self::zeros();
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result.data[i][j] = result.data[i][j] + self.data[i][k] * other.data[k][j];
                }
            }
        }
        result
    }

    /// 行列の要素にアクセス
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[row][col]
    }

    /// 行列の要素を設定
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.data[row][col] = value;
    }

    /// フロベニウスノルム
    pub fn frobenius_norm(&self) -> T {
        let mut sum = T::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                sum = sum + self.data[i][j] * self.data[i][j];
            }
        }
        sum.sqrt()
    }

    /// X軸周りの回転行列（ラジアン）
    pub fn rotation_x(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            T::ONE, T::ZERO, T::ZERO,
            T::ZERO, cos_a, -sin_a,
            T::ZERO, sin_a, cos_a
        )
    }

    /// Y軸周りの回転行列（ラジアン）
    pub fn rotation_y(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            cos_a, T::ZERO, sin_a,
            T::ZERO, T::ONE, T::ZERO,
            -sin_a, T::ZERO, cos_a
        )
    }

    /// Z軸周りの回転行列（ラジアン）
    pub fn rotation_z(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            cos_a, -sin_a, T::ZERO,
            sin_a, cos_a, T::ZERO,
            T::ZERO, T::ZERO, T::ONE
        )
    }

    /// スケール行列を作成
    pub fn scale(sx: T, sy: T, sz: T) -> Self {
        Self::new(
            sx, T::ZERO, T::ZERO,
            T::ZERO, sy, T::ZERO,
            T::ZERO, T::ZERO, sz
        )
    }

    /// 平行移動行列を作成（同次座標用）
    pub fn translation(tx: T, ty: T) -> Self {
        Self::new(
            T::ONE, T::ZERO, tx,
            T::ZERO, T::ONE, ty,
            T::ZERO, T::ZERO, T::ONE
        )
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Matrix3x3<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let mut result = Self::zeros();
        for i in 0..3 {
            for j in 0..3 {
                result.data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        result
    }
}

impl<T: Scalar> Mul<T> for Matrix3x3<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        let mut result = Self::zeros();
        for i in 0..3 {
            for j in 0..3 {
                result.data[i][j] = self.data[i][j] * scalar;
            }
        }
        result
    }
}

impl<T: Scalar> Mul for Matrix3x3<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        self.mul_matrix(&other)
    }
}

/// 型エイリアス
pub type Matrix3x3f = Matrix3x3<f32>;
pub type Matrix3x3d = Matrix3x3<f64>;