use crate::consts::test_constants::{SOLVER_TOLERANCE_F64, TOLERANCE_F64};
use crate::linalg::solver::{
    CramerSolver, DynamicMatrixLinearSolver, GaussianSolver, LUSolver, LinearSolver,
};
use crate::linalg::{DynamicMatrix, Vector};

#[cfg(test)]
mod tests {
    use super::*;

    /// 共通テストデータ
    fn get_test_2x2() -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let rhs = vec![5.0, 6.0];
        let expected = vec![1.8, 1.4];
        (matrix, rhs, expected)
    }

    fn get_test_3x3() -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
        let matrix = vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ];
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
        let matrix = vec![vec![1.0, 2.0], vec![2.0, 4.0]]; // 特異行列
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
    fn test_gaussian_solver_dynamic_matrix_adapter() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = Vector::new(vec![5.0, 6.0]);
        let solver = GaussianSolver::new(SOLVER_TOLERANCE_F64);

        let result = solver.solve_dynamic(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 1.4).abs() < TOLERANCE_F64);
        assert!(result.converged);
    }

    #[test]
    fn test_lu_solver_dynamic_matrix_adapter() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = Vector::new(vec![5.0, 6.0]);
        let solver = LUSolver::new(SOLVER_TOLERANCE_F64);

        let result = solver.solve_dynamic(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 1.4).abs() < TOLERANCE_F64);
        assert!(result.converged);
    }

    #[test]
    fn test_dynamic_matrix_adapter_rejects_rhs_mismatch() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = Vector::new(vec![5.0]);
        let solver = GaussianSolver::new(SOLVER_TOLERANCE_F64);

        let error = solver.solve_dynamic(&matrix, &rhs).unwrap_err();
        assert_eq!(error, "RHS dimension mismatch");
    }
}
