//! 動的サイズ行列
//!
//! 行数・列数を実行時に持つ汎用行列。
//! 内部は row-major の 1 次元配列で保持する。
use crate::abstract_types::Scalar;
use crate::linalg::vector::Vector;

/// 動的サイズ行列（row-major 格納）
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicMatrix<T: Scalar> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T: Scalar> DynamicMatrix<T> {
    /// 行列を作成
    pub fn new(rows: usize, cols: usize, data: Vec<T>) -> Result<Self, String> {
        if rows == 0 || cols == 0 {
            return Err("Matrix dimensions must be greater than zero".to_string());
        }

        if data.len() != rows * cols {
            return Err("Matrix data length does not match dimensions".to_string());
        }

        Ok(Self { rows, cols, data })
    }

    /// ゼロ行列を作成
    pub fn zeros(rows: usize, cols: usize) -> Result<Self, String> {
        Self::new(rows, cols, vec![T::ZERO; rows * cols])
    }

    /// 行ごとのデータから構築
    pub fn from_rows(rows_data: Vec<Vec<T>>) -> Result<Self, String> {
        if rows_data.is_empty() {
            return Err("Matrix must have at least one row".to_string());
        }

        let rows = rows_data.len();
        let cols = rows_data[0].len();

        if cols == 0 {
            return Err("Matrix must have at least one column".to_string());
        }

        if rows_data.iter().any(|row| row.len() != cols) {
            return Err("All matrix rows must have the same length".to_string());
        }

        let data = rows_data.into_iter().flatten().collect();
        Self::new(rows, cols, data)
    }

    /// 行数
    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// 列数
    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// 形状
    #[inline]
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// 内部 row-major 配列への参照
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// 要素を取得
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[self.index_of(row, col)]
    }

    /// 要素を設定
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        let index = self.index_of(row, col);
        self.data[index] = value;
    }

    /// `Vec<Vec<T>>` に変換
    pub fn to_vec2d(&self) -> Vec<Vec<T>> {
        self.data
            .chunks(self.cols)
            .map(|row| row.to_vec())
            .collect()
    }

    /// 転置行列を返す
    pub fn transpose(&self) -> Self {
        let mut data = vec![T::ZERO; self.data.len()];

        for row in 0..self.rows {
            for col in 0..self.cols {
                data[col * self.rows + row] = self.get(row, col);
            }
        }

        Self {
            rows: self.cols,
            cols: self.rows,
            data,
        }
    }

    /// 行列ベクトル積
    pub fn mul_vector(&self, vector: &Vector<T>) -> Result<Vector<T>, String> {
        if self.cols != vector.len() {
            return Err("Matrix and vector dimensions mismatch".to_string());
        }

        let mut result = vec![T::ZERO; self.rows];
        for (row_index, result_cell) in result.iter_mut().enumerate() {
            let row_start = row_index * self.cols;
            let row_end = row_start + self.cols;
            *result_cell = self.data[row_start..row_end]
                .iter()
                .zip(vector.data().iter())
                .map(|(matrix_value, vector_value)| *matrix_value * *vector_value)
                .fold(T::ZERO, |acc, value| acc + value);
        }

        Ok(Vector::new(result))
    }

    #[inline]
    fn index_of(&self, row: usize, col: usize) -> usize {
        assert!(row < self.rows, "row index out of bounds");
        assert!(col < self.cols, "column index out of bounds");
        row * self.cols + col
    }
}
