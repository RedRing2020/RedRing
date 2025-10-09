//! ガウス消去法による連立方程式ソルバー
//!
//! 部分ピボット選択付きガウス消去法を実装
//! 数値安定性を考慮した一般的な直接法
use super::{LinearSolver, SolutionInfo};
use crate::abstract_types::Scalar;

/// ガウス消去法ソルバー
pub struct GaussianSolver<T: Scalar> {
    tolerance: T,
    use_partial_pivoting: bool,
}

impl<T: Scalar> GaussianSolver<T> {
    /// 新しいガウス消去法ソルバーを作成
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

    /// 拡張行列を作成
    fn create_augmented_matrix(&self, matrix: &[Vec<T>], rhs: &[T]) -> Vec<Vec<T>> {
        let n = matrix.len();
        let mut aug_matrix = Vec::with_capacity(n);

        for i in 0..n {
            let mut row = matrix[i].clone();
            row.push(rhs[i]);
            aug_matrix.push(row);
        }

        aug_matrix
    }

    /// 部分ピボット選択
    fn partial_pivot(&self, aug_matrix: &mut [Vec<T>], k: usize) -> Result<(), String> {
        let n = aug_matrix.len();
        let mut max_row = k;
        let mut max_val = aug_matrix[k][k].abs();

        // k列目で最大の絶対値を持つ行を探す
        #[allow(clippy::needless_range_loop)]
        for i in (k + 1)..n {
            let val = aug_matrix[i][k].abs();
            if val > max_val {
                max_val = val;
                max_row = i;
            }
        }

        // 特異性チェック
        if max_val < self.tolerance {
            return Err(format!("Matrix is singular at column {}", k));
        }

        // 行交換
        if max_row != k {
            aug_matrix.swap(k, max_row);
        }

        Ok(())
    }

    /// 前進消去
    fn forward_elimination(&self, aug_matrix: &mut [Vec<T>]) -> Result<(), String> {
        let n = aug_matrix.len();

        for k in 0..n {
            // 部分ピボット選択
            if self.use_partial_pivoting {
                self.partial_pivot(aug_matrix, k)?;
            } else if aug_matrix[k][k].abs() < self.tolerance {
                return Err(format!(
                    "Matrix is singular at diagonal element ({}, {})",
                    k, k
                ));
            }

            // 前進消去
            #[allow(clippy::needless_range_loop)]
            for i in (k + 1)..n {
                let factor = aug_matrix[i][k] / aug_matrix[k][k];

                for j in k..=n {
                    aug_matrix[i][j] = aug_matrix[i][j] - factor * aug_matrix[k][j];
                }
            }
        }

        Ok(())
    }

    /// 後退代入
    fn back_substitution(&self, aug_matrix: &[Vec<T>]) -> Vec<T> {
        let n = aug_matrix.len();
        let mut solution = vec![T::ZERO; n];

        for i in (0..n).rev() {
            solution[i] = aug_matrix[i][n]; // RHSの値

            for j in (i + 1)..n {
                solution[i] = solution[i] - aug_matrix[i][j] * solution[j];
            }

            solution[i] = solution[i] / aug_matrix[i][i];
        }

        solution
    }

    /// 残差を計算
    fn calculate_residual(&self, matrix: &[Vec<T>], rhs: &[T], solution: &[T]) -> T {
        let n = matrix.len();
        let mut residual = T::ZERO;

        for i in 0..n {
            let mut sum = T::ZERO;
            #[allow(clippy::needless_range_loop)]
            for j in 0..n {
                sum = sum + matrix[i][j] * solution[j];
            }
            let diff = sum - rhs[i];
            residual = residual + diff * diff;
        }

        residual.sqrt()
    }
}

impl<T: Scalar> LinearSolver<T> for GaussianSolver<T> {
    fn solve(&self, matrix: &[Vec<T>], rhs: &[T]) -> Result<SolutionInfo<T>, String> {
        let n = matrix.len();

        // 入力検証
        if n == 0 || matrix[0].len() != n || rhs.len() != n {
            return Err("Invalid matrix dimensions".to_string());
        }

        for row in matrix {
            if row.len() != n {
                return Err("Matrix must be square".to_string());
            }
        }

        // 拡張行列を作成
        let mut aug_matrix = self.create_augmented_matrix(matrix, rhs);

        // 前進消去
        self.forward_elimination(&mut aug_matrix)?;

        // 後退代入
        let solution = self.back_substitution(&aug_matrix);

        // 残差計算
        let residual = self.calculate_residual(matrix, rhs, &solution);

        Ok(SolutionInfo::direct(solution, residual))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_2x2() {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let rhs = vec![5.0, 6.0];

        let solver = GaussianSolver::new(1e-15);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8).abs() < 1e-10);
        assert!((result.solution[1] - 1.4).abs() < 1e-10);
        assert!(result.converged);
    }

    #[test]
    fn test_gaussian_3x3() {
        let matrix = vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ];
        let rhs = vec![8.0, -11.0, -3.0];

        let solver = GaussianSolver::new(1e-15);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 2.0).abs() < 1e-10);
        assert!((result.solution[1] - 3.0).abs() < 1e-10);
        assert!((result.solution[2] - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_gaussian_singular_matrix() {
        let matrix = vec![
            vec![1.0, 2.0],
            vec![2.0, 4.0], // 2倍の行
        ];
        let rhs = vec![3.0, 6.0];

        let solver = GaussianSolver::new(1e-15);
        let result = solver.solve(&matrix, &rhs);

        assert!(result.is_err());
    }

    #[test]
    fn test_gaussian_without_pivoting() {
        let matrix = vec![vec![0.001, 1.0], vec![2.0, 1.0]];
        let rhs = vec![1.0, 3.0];

        let solver = GaussianSolver::new(1e-15).with_pivoting(false);
        let result = solver.solve(&matrix, &rhs);

        // ピボット選択なしでは数値的に不安定になる可能性
        assert!(result.is_ok());
    }
}
