//! Cramerの公式による連立方程式ソルバー
//!
//! 小規模システム（2x2, 3x3）専用の直接解法
//! 行列式を直接計算して解を求める
use super::{LinearSolver, SolutionInfo};
use crate::linalg::{Matrix2x2, Matrix3x3, Vector2, Vector3};
use crate::linalg::scalar::Scalar;

/// Cramerの公式ソルバー
pub struct CramerSolver<T: Scalar> {
    tolerance: T,
}

impl<T: Scalar> CramerSolver<T> {
    /// 新しいCramerソルバーを作成
    pub fn new(tolerance: T) -> Self {
        Self { tolerance }
    }

    /// 2x2システムをCramerの公式で解く
    pub fn solve_2x2(
        &self,
        matrix: &Matrix2x2<T>,
        rhs: &Vector2<T>
    ) -> Result<Vector2<T>, String> {
        let det = matrix.determinant();
        if det.abs() < self.tolerance {
            return Err("Matrix is singular".to_string());
        }

        // x = det_x / det, y = det_y / det
        let det_x = Matrix2x2::new(
            rhs.x(), matrix.get(0, 1),
            rhs.y(), matrix.get(1, 1)
        ).determinant();

        let det_y = Matrix2x2::new(
            matrix.get(0, 0), rhs.x(),
            matrix.get(1, 0), rhs.y()
        ).determinant();

        Ok(Vector2::new(det_x / det, det_y / det))
    }

    /// 3x3システムをCramerの公式で解く
    pub fn solve_3x3(
        &self,
        matrix: &Matrix3x3<T>,
        rhs: &Vector3<T>
    ) -> Result<Vector3<T>, String> {
        let det = matrix.determinant();
        if det.abs() < self.tolerance {
            return Err("Matrix is singular".to_string());
        }

        // x = det_x / det, y = det_y / det, z = det_z / det
        let det_x = Matrix3x3::new(
            rhs.x(), matrix.get(0, 1), matrix.get(0, 2),
            rhs.y(), matrix.get(1, 1), matrix.get(1, 2),
            rhs.z(), matrix.get(2, 1), matrix.get(2, 2)
        ).determinant();

        let det_y = Matrix3x3::new(
            matrix.get(0, 0), rhs.x(), matrix.get(0, 2),
            matrix.get(1, 0), rhs.y(), matrix.get(1, 2),
            matrix.get(2, 0), rhs.z(), matrix.get(2, 2)
        ).determinant();

        let det_z = Matrix3x3::new(
            matrix.get(0, 0), matrix.get(0, 1), rhs.x(),
            matrix.get(1, 0), matrix.get(1, 1), rhs.y(),
            matrix.get(2, 0), matrix.get(2, 1), rhs.z()
        ).determinant();

        Ok(Vector3::new(det_x / det, det_y / det, det_z / det))
    }
}

impl<T: Scalar> LinearSolver<T> for CramerSolver<T> {
    fn solve(&self, matrix: &[Vec<T>], rhs: &[T]) -> Result<SolutionInfo<T>, String> {
        let n = matrix.len();

        match n {
            2 => {
                // 2x2の場合
                let matrix_2x2 = Matrix2x2::new(
                    matrix[0][0], matrix[0][1],
                    matrix[1][0], matrix[1][1]
                );
                let rhs_2x2 = Vector2::new(rhs[0], rhs[1]);

                let solution_vec = self.solve_2x2(&matrix_2x2, &rhs_2x2)?;
                let solution = vec![solution_vec.x(), solution_vec.y()];

                // 残差計算
                let residual = self.calculate_residual(matrix, rhs, &solution);
                Ok(SolutionInfo::direct(solution, residual))
            },
            3 => {
                // 3x3の場合
                let matrix_3x3 = Matrix3x3::new(
                    matrix[0][0], matrix[0][1], matrix[0][2],
                    matrix[1][0], matrix[1][1], matrix[1][2],
                    matrix[2][0], matrix[2][1], matrix[2][2]
                );
                let rhs_3x3 = Vector3::new(rhs[0], rhs[1], rhs[2]);

                let solution_vec = self.solve_3x3(&matrix_3x3, &rhs_3x3)?;
                let solution = vec![solution_vec.x(), solution_vec.y(), solution_vec.z()];

                // 残差計算
                let residual = self.calculate_residual(matrix, rhs, &solution);
                Ok(SolutionInfo::direct(solution, residual))
            },
            _ => Err("Cramer's rule only supports 2x2 and 3x3 systems".to_string())
        }
    }
}

impl<T: Scalar> CramerSolver<T> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cramer_2x2() {
        let matrix = vec![
            vec![2.0, 1.0],
            vec![1.0, 3.0],
        ];
        let rhs = vec![5.0, 6.0];

        let solver = CramerSolver::new(1e-15);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8).abs() < 1e-10);
        assert!((result.solution[1] - 1.4).abs() < 1e-10);
        assert!(result.converged);
    }

    #[test]
    fn test_cramer_3x3() {
        let matrix = vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ];
        let rhs = vec![8.0, -11.0, -3.0];

        let solver = CramerSolver::new(1e-15);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 2.0).abs() < 1e-10);
        assert!((result.solution[1] - 3.0).abs() < 1e-10);
        assert!((result.solution[2] - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_cramer_4x4_unsupported() {
        let matrix = vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ];
        let rhs = vec![1.0, 2.0, 3.0, 4.0];

        let solver = CramerSolver::new(1e-15);
        let result = solver.solve(&matrix, &rhs);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only supports 2x2 and 3x3"));
    }
}
