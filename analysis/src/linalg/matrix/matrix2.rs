//! 2x2行列（高速演算用）
//!
//! コンパイル時最適化に特化した固定サイズ行列
//! グラフィックス処理とCAD計算の両方に対応
use crate::linalg::scalar::Scalar;
use crate::linalg::vector::Vector2;
use std::ops::{Add, Mul};

/// 2x2行列
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix2x2<T: Scalar> {
    pub data: [[T; 2]; 2],
}

impl<T: Scalar> Matrix2x2<T> {
    pub fn new(a11: T, a12: T, a21: T, a22: T) -> Self {
        Self {
            data: [[a11, a12], [a21, a22]],
        }
    }

    pub fn zeros() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO, T::ZERO)
    }

    pub fn identity() -> Self {
        Self::new(T::ONE, T::ZERO, T::ZERO, T::ONE)
    }

    pub fn determinant(&self) -> T {
        self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]
    }

    pub fn trace(&self) -> T {
        self.data[0][0] + self.data[1][1]
    }

    pub fn transpose(&self) -> Self {
        Self::new(
            self.data[0][0],
            self.data[1][0],
            self.data[0][1],
            self.data[1][1],
        )
    }

    pub fn inverse(&self) -> Result<Self, String> {
        let det = self.determinant();
        if det.is_zero() {
            return Err("Matrix is singular".to_string());
        }

        Ok(Self::new(
            self.data[1][1] / det,
            -self.data[0][1] / det,
            -self.data[1][0] / det,
            self.data[0][0] / det,
        ))
    }

    pub fn mul_vector(&self, vec: &Vector2<T>) -> Vector2<T> {
        Vector2::new(
            self.data[0][0] * vec.x() + self.data[0][1] * vec.y(),
            self.data[1][0] * vec.x() + self.data[1][1] * vec.y(),
        )
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
        for i in 0..2 {
            for j in 0..2 {
                sum = sum + self.data[i][j] * self.data[i][j];
            }
        }
        sum.sqrt()
    }

    /// 回転行列を作成（ラジアン）
    pub fn rotation(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(cos_a, -sin_a, sin_a, cos_a)
    }

    /// スケール行列を作成
    pub fn scale(sx: T, sy: T) -> Self {
        Self::new(sx, T::ZERO, T::ZERO, sy)
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Matrix2x2<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::new(
            self.data[0][0] + other.data[0][0],
            self.data[0][1] + other.data[0][1],
            self.data[1][0] + other.data[1][0],
            self.data[1][1] + other.data[1][1],
        )
    }
}

impl<T: Scalar> Mul<T> for Matrix2x2<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        Self::new(
            self.data[0][0] * scalar,
            self.data[0][1] * scalar,
            self.data[1][0] * scalar,
            self.data[1][1] * scalar,
        )
    }
}

impl<T: Scalar> Mul for Matrix2x2<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::new(
            self.data[0][0] * other.data[0][0] + self.data[0][1] * other.data[1][0],
            self.data[0][0] * other.data[0][1] + self.data[0][1] * other.data[1][1],
            self.data[1][0] * other.data[0][0] + self.data[1][1] * other.data[1][0],
            self.data[1][0] * other.data[0][1] + self.data[1][1] * other.data[1][1],
        )
    }
}

/// 型エイリアス
pub type Matrix2x2f = Matrix2x2<f32>;
pub type Matrix2x2d = Matrix2x2<f64>;
