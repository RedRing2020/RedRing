//! 2x2行列（高速演算用）
//!
//! コンパイル時最適化に特化した固定サイズ行列
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

#[cfg(test)]
mod tests {
    use crate::linalg::matrix::Matrix2x2;
    use crate::linalg::vector::Vector2;
    use crate::linalg::{Mat2d, Mat2f};

    use crate::consts::test_constants::TOLERANCE_F64;

    #[test]
    fn test_matrix2x2_operations_f64() {
        let m1 = Matrix2x2::<f64>::new(1.0, 2.0, 3.0, 4.0);
        let m2 = Matrix2x2::<f64>::new(5.0, 6.0, 7.0, 8.0);

        let sum = m1 + m2;
        assert_eq!(sum.get(0, 0), 6.0);
        assert_eq!(sum.get(1, 1), 12.0);

        let det = m1.determinant();
        assert_eq!(det, -2.0); // 1*4 - 2*3 = -2
    }

    #[test]
    fn test_matrix2x2_operations_f32() {
        let m1 = Matrix2x2::<f32>::new(1.0, 2.0, 3.0, 4.0);
        let m2 = Matrix2x2::<f32>::new(5.0, 6.0, 7.0, 8.0);

        let sum = m1 + m2;
        assert_eq!(sum.get(0, 0), 6.0);
        assert_eq!(sum.get(1, 1), 12.0);

        let det = m1.determinant();
        assert_eq!(det, -2.0); // 1*4 - 2*3 = -2
    }

    #[test]
    fn test_matrix2x2_inverse() {
        let m = Matrix2x2::<f64>::new(1.0, 2.0, 3.0, 4.0);
        let inv = m.inverse().unwrap();

        let product = m * inv;
        let identity = Matrix2x2::<f64>::identity();

        // 結果が単位行列に近いことを確認
        for i in 0..2 {
            for j in 0..2 {
                assert!((product.get(i, j) - identity.get(i, j)).abs() < TOLERANCE_F64);
            }
        }
    }

    #[test]
    fn test_matrix2x2_vector_multiplication() {
        let m = Matrix2x2::<f64>::identity();
        let v = Vector2::<f64>::new(1.0, 2.0);
        let result = m.mul_vector(&v);

        assert_eq!(result.x(), 1.0);
        assert_eq!(result.y(), 2.0);
    }

    #[test]
    fn test_matrix2x2_rotation() {
        use std::f64::consts::PI;

        let rotation = Matrix2x2::<f64>::rotation(PI / 2.0); // 90度回転
        let v = Vector2::<f64>::new(1.0, 0.0);
        let rotated = rotation.mul_vector(&v);

        // (1,0) が (0,1) になることを確認（誤差を考慮）
        assert!(rotated.x().abs() < TOLERANCE_F64);
        assert!((rotated.y() - 1.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_matrix2x2_scale() {
        let scale = Matrix2x2::<f64>::scale(2.0, 3.0);
        let v = Vector2::<f64>::new(1.0, 1.0);
        let scaled = scale.mul_vector(&v);

        assert_eq!(scaled.x(), 2.0);
        assert_eq!(scaled.y(), 3.0);
    }

    #[test]
    fn test_type_aliases() {
        let m_f32 = Mat2f::identity();
        let m_f64 = Mat2d::identity();

        assert_eq!(m_f32.trace(), 2.0);
        assert_eq!(m_f64.trace(), 2.0);
    }
}
