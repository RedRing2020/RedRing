use crate::consts::test_constants::{SOLVER_TOLERANCE_F64, TOLERANCE_F64};
use crate::linalg::solver::{CramerSolver, GaussianSolver, LUSolver, LinearSolver};
use crate::linalg::DynamicMatrix;

#[cfg(test)]
mod tests {
    use super::*;

    /// 共通テストデータ
    fn get_test_2x2() -> (DynamicMatrix<f64>, Vec<f64>, Vec<f64>) {
        let matrix = DynamicMatrix::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = vec![5.0, 6.0];
        let expected = vec![1.8, 1.4];
        (matrix, rhs, expected)
    }

    fn get_test_3x3() -> (DynamicMatrix<f64>, Vec<f64>, Vec<f64>) {
        let matrix = DynamicMatrix::from_rows(vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ])
        .unwrap();
        let rhs = vec![8.0, -11.0, -3.0];
        let expected = vec![2.0, 3.0, -1.0];
        (matrix, rhs, expected)
    }

    #[test]
    fn test_gaussian_solver_2x2() {
        let (matrix, rhs, expected) = get_test_2x2();
        let solver = GaussianSolver::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        for (i, &exp) in expected.iter().enumerate() {
            assert!((result.solution[i] - exp).abs() < TOLERANCE_F64);
        }
        assert!(result.converged);
    }

    #[test]
    fn test_lu_solver_2x2() {
        let (matrix, rhs, expected) = get_test_2x2();
        let solver = LUSolver::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        for (i, &exp) in expected.iter().enumerate() {
            assert!((result.solution[i] - exp).abs() < TOLERANCE_F64);
        }
        assert!(result.converged);
    }

    #[test]
    fn test_cramer_solver_2x2() {
        let (matrix, rhs, expected) = get_test_2x2();
        let solver = CramerSolver::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        for (i, &exp) in expected.iter().enumerate() {
            assert!((result.solution[i] - exp).abs() < TOLERANCE_F64);
        }
        assert!(result.converged);
    }

    #[test]
    fn test_gaussian_solver_3x3() {
        let (matrix, rhs, expected) = get_test_3x3();
        let solver = GaussianSolver::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        for (i, &exp) in expected.iter().enumerate() {
            assert!((result.solution[i] - exp).abs() < TOLERANCE_F64);
        }
        assert!(result.converged);
    }

    #[test]
    fn test_lu_solver_3x3() {
        let (matrix, rhs, expected) = get_test_3x3();
        let solver = LUSolver::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        for (i, &exp) in expected.iter().enumerate() {
            assert!((result.solution[i] - exp).abs() < TOLERANCE_F64);
        }
        assert!(result.converged);
    }

    #[test]
    fn test_singular_matrix() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![1.0, 2.0], vec![2.0, 4.0]]).unwrap();
        let rhs = vec![3.0, 6.0];

        let solver = GaussianSolver::new(SOLVER_TOLERANCE_F64);
        assert!(solver.solve(&matrix, &rhs).is_err());
    }

    #[test]
    fn test_solver_tolerance() {
        let (matrix, rhs, _) = get_test_2x2();

        // 厳しい許容誤差
        let strict_solver = GaussianSolver::new(1e-16);
        let result = strict_solver.solve(&matrix, &rhs).unwrap();
        assert!(result.converged);

        // 緩い許容誤差
        let loose_solver = GaussianSolver::new(1e-8);
        let result = loose_solver.solve(&matrix, &rhs).unwrap();
        assert!(result.converged);
    }

    #[test]
    fn test_gaussian_solver_rejects_non_square_matrix() {
        // 非正方行列（2x3）は Err を返す
        let matrix =
            DynamicMatrix::<f64>::from_rows(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]])
                .unwrap();
        let rhs = vec![1.0_f64, 2.0];
        let solver = GaussianSolver::new(SOLVER_TOLERANCE_F64);

        assert!(solver.solve(&matrix, &rhs).is_err());
    }

    #[test]
    fn test_lu_solver_rejects_rhs_mismatch() {
        // rhs 長さ不一致は Err を返す
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = vec![5.0_f64]; // 長さ不一致
        let solver = LUSolver::new(SOLVER_TOLERANCE_F64);

        assert!(solver.solve(&matrix, &rhs).is_err());
    }

    #[test]
    fn test_solver_rejects_rhs_mismatch() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = vec![5.0_f64]; // 長さ不一致
        let solver = GaussianSolver::new(SOLVER_TOLERANCE_F64);

        assert!(solver.solve(&matrix, &rhs).is_err());
    }
}
