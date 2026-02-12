//! 2x2行列（高速演算用）
//!
//! コンパイル時最適化に特化した固定サイズ行列
//! グラフィックス処理とCAD計算の両方に対応
use crate::abstract_types::Scalar;
use crate::linalg::vector::Vector2;
use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

/// 2x2行列（行優先格納）
///
/// 内部データは行優先で格納されています。
/// GPU転送時は `to_column_major()` で列優先に変換してください。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix2x2<T: Scalar> {
    data: [[T; 2]; 2],
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

    // === アクセサメソッド ===

    /// 要素を取得
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[row][col]
    }

    /// 要素を設定
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.data[row][col] = value;
    }

    /// 行を取得
    #[inline]
    pub fn get_row(&self, row: usize) -> [T; 2] {
        self.data[row]
    }

    /// 列を取得
    #[inline]
    pub fn get_column(&self, col: usize) -> [T; 2] {
        [self.data[0][col], self.data[1][col]]
    }

    /// 行を設定
    #[inline]
    pub fn set_row(&mut self, row: usize, values: [T; 2]) {
        self.data[row] = values;
    }

    /// 列を設定
    #[inline]
    pub fn set_column(&mut self, col: usize, values: [T; 2]) {
        self.data[0][col] = values[0];
        self.data[1][col] = values[1];
    }

    /// 内部データへの参照（行優先）
    #[inline]
    pub fn as_row_major(&self) -> &[[T; 2]; 2] {
        &self.data
    }

    // === イテレータ ===

    /// 全要素を行優先でイテレート
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data.iter().flat_map(|row| row.iter()).copied()
    }

    /// 各行をイテレート
    pub fn rows(&self) -> impl Iterator<Item = [T; 2]> + '_ {
        self.data.iter().copied()
    }

    /// 各列をイテレート
    pub fn columns(&self) -> impl Iterator<Item = [T; 2]> + '_ {
        (0..2).map(move |col| self.get_column(col))
    }

    // === GPU用変換 ===

    /// 列優先形式に変換（wgpu/OpenGL用）
    #[inline]
    pub fn to_column_major(&self) -> [[T; 2]; 2] {
        [
            [self.data[0][0], self.data[1][0]],
            [self.data[0][1], self.data[1][1]],
        ]
    }

    /// 列優先形式から構築（wgpu/OpenGL用）
    #[inline]
    pub fn from_column_major(data: [[T; 2]; 2]) -> Self {
        Self {
            data: [[data[0][0], data[1][0]], [data[0][1], data[1][1]]],
        }
    }

    // === 基本演算 ===

    /// フロベニウスノルム
    pub fn frobenius_norm(&self) -> T {
        let mut sum = T::ZERO;
        for i in 0..2 {
            for j in 0..2 {
                sum += self.data[i][j] * self.data[i][j];
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

// === 添え字演算子（互換性維持） ===

impl<T: Scalar> Index<usize> for Matrix2x2<T> {
    type Output = [T; 2];
    #[inline]
    fn index(&self, row: usize) -> &[T; 2] {
        &self.data[row]
    }
}

impl<T: Scalar> IndexMut<usize> for Matrix2x2<T> {
    #[inline]
    fn index_mut(&mut self, row: usize) -> &mut [T; 2] {
        &mut self.data[row]
    }
}

// === 演算子オーバーロード ===

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

impl<T: Scalar> Sub for Matrix2x2<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::new(
            self.data[0][0] - other.data[0][0],
            self.data[0][1] - other.data[0][1],
            self.data[1][0] - other.data[1][0],
            self.data[1][1] - other.data[1][1],
        )
    }
}

impl<T: Scalar> Neg for Matrix2x2<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(
            -self.data[0][0],
            -self.data[0][1],
            -self.data[1][0],
            -self.data[1][1],
        )
    }
}

// === 配列変換 ===

impl<T: Scalar> From<[[T; 2]; 2]> for Matrix2x2<T> {
    /// 行優先配列から構築
    #[inline]
    fn from(data: [[T; 2]; 2]) -> Self {
        Self { data }
    }
}

/// 型エイリアス
pub type Matrix2x2f = Matrix2x2<f32>;
pub type Matrix2x2d = Matrix2x2<f64>;
