//! LU分解による連立方程式ソルバー
//!
//! Doolittle法によるLU分解を実装
//! 部分ピボット選択付きで数値安定性を確保
use super::{LinearSolver, SolutionInfo};
use crate::abstract_types::Scalar;
use crate::linalg::DynamicMatrix;

/// LU分解ソルバー
pub struct LUSolver<T: Scalar> {
    tolerance: T,
    use_partial_pivoting: bool,
}

/// LU分解の結果
#[derive(Debug, Clone)]
pub struct LUDecomposition<T: Scalar> {
    pub lu_matrix: Vec<Vec<T>>,  // LとUを同じ行列に格納
    pub permutation: Vec<usize>, // 置換行列
    pub determinant_sign: i32,   // 行列式の符号
}

impl<T: Scalar> LUSolver<T> {
    /// 新しいLU分解ソルバーを作成
    pub fn new(tolerance: T) -> Self {
        Self {
            tolerance,
            use_partial_pivoting: true,
        }
    }

    /// 部分ピボット選択の有無を設定
    pub fn with_pivoting(mut self, use_pivoting: bool) -> Self {
        self.use_partial_pivoting = use_pivoting;
        self
    }

    /// LU分解を実行（Doolittle法）
    pub fn decompose(&self, matrix: &DynamicMatrix<T>) -> Result<LUDecomposition<T>, String> {
        let n = matrix.rows();

        // 入力検証
        if n == 0 || matrix.cols() != n {
            return Err("Matrix must be square".to_string());
        }

        // LU行列を初期化（元の行列をコピー）
        let mut lu_matrix: Vec<Vec<T>> = (0..n)
            .map(|i| (0..n).map(|j| matrix.get(i, j)).collect())
            .collect();
        let mut permutation: Vec<usize> = (0..n).collect();
        let mut determinant_sign = 1i32;

        // LU分解の実行
        for k in 0..n {
            // 部分ピボット選択
            if self.use_partial_pivoting {
                let pivot_row = self.find_pivot(&lu_matrix, k)?;
                if pivot_row != k {
                    // 行交換
                    lu_matrix.swap(k, pivot_row);
                    permutation.swap(k, pivot_row);
                    determinant_sign *= -1;
                }
            }

            // 特異性チェック
            if lu_matrix[k][k].abs() < self.tolerance {
                return Err(format!(
                    "Matrix is singular at diagonal element ({}, {})",
                    k, k
                ));
            }

            // L部分の計算（下三角）
            for i in (k + 1)..n {
                lu_matrix[i][k] = lu_matrix[i][k] / lu_matrix[k][k];
            }

            // U部分の更新（上三角）
            for i in (k + 1)..n {
                for j in (k + 1)..n {
                    lu_matrix[i][j] = lu_matrix[i][j] - lu_matrix[i][k] * lu_matrix[k][j];
                }
            }
        }

        Ok(LUDecomposition {
            lu_matrix,
            permutation,
            determinant_sign,
        })
    }

    /// ピボット行を見つける
    fn find_pivot(&self, lu_matrix: &[Vec<T>], k: usize) -> Result<usize, String> {
        let n = lu_matrix.len();
        let mut max_row = k;
        let mut max_val = lu_matrix[k][k].abs();

        #[allow(clippy::needless_range_loop)]
        for i in (k + 1)..n {
            let val = lu_matrix[i][k].abs();
            if val > max_val {
                max_val = val;
                max_row = i;
            }
        }

        if max_val < self.tolerance {
            return Err(format!("Matrix is singular at column {}", k));
        }

        Ok(max_row)
    }

    /// LU分解を使って連立方程式を解く
    pub fn solve_with_decomposition(
        &self,
        lu_decomp: &LUDecomposition<T>,
        rhs: &[T],
    ) -> Result<Vec<T>, String> {
        let n = rhs.len();
        if n != lu_decomp.lu_matrix.len() {
            return Err("RHS dimension mismatch".to_string());
        }

        // 置換を適用してRHSを並び替え
        let mut permuted_rhs = vec![T::ZERO; n];
        for i in 0..n {
            permuted_rhs[i] = rhs[lu_decomp.permutation[i]];
        }

        // 前進代入（Ly = Pb）
        let mut y = vec![T::ZERO; n];
        for i in 0..n {
            y[i] = permuted_rhs[i];
            for j in 0..i {
                y[i] = y[i] - lu_decomp.lu_matrix[i][j] * y[j];
            }
        }

        // 後退代入（Ux = y）
        let mut x = vec![T::ZERO; n];
        for i in (0..n).rev() {
            x[i] = y[i];
            for j in (i + 1)..n {
                x[i] = x[i] - lu_decomp.lu_matrix[i][j] * x[j];
            }
            x[i] /= lu_decomp.lu_matrix[i][i];
        }

        Ok(x)
    }

    /// 行列式を計算
    pub fn determinant(&self, lu_decomp: &LUDecomposition<T>) -> T {
        let mut det = T::from_f64(lu_decomp.determinant_sign as f64);

        for i in 0..lu_decomp.lu_matrix.len() {
            det *= lu_decomp.lu_matrix[i][i];
        }

        det
    }

    /// 残差を計算
    fn calculate_residual(&self, matrix: &DynamicMatrix<T>, rhs: &[T], solution: &[T]) -> T {
        let n = matrix.rows();
        let mut residual = T::ZERO;

        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            let mut sum = T::ZERO;
            for j in 0..n {
                sum += matrix.get(i, j) * solution[j];
            }
            let diff = sum - rhs[i];
            residual += diff * diff;
        }

        residual.sqrt()
    }
}

impl<T: Scalar> LinearSolver<T> for LUSolver<T> {
    fn solve(&self, matrix: &DynamicMatrix<T>, rhs: &[T]) -> Result<SolutionInfo<T>, String> {
        // LU分解を実行
        let lu_decomp = self.decompose(matrix)?;

        // 連立方程式を解く
        let solution = self.solve_with_decomposition(&lu_decomp, rhs)?;

        // 残差計算
        let residual = self.calculate_residual(matrix, rhs, &solution);

        Ok(SolutionInfo::direct(solution, residual))
    }
}
