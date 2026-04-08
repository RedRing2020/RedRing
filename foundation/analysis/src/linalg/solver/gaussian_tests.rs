use super::{GaussianSolver, LinearSolver};
use crate::consts::test_constants::{SOLVER_TOLERANCE_F32, SOLVER_TOLERANCE_F64, TOLERANCE_F64};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_2x2() {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let rhs = vec![5.0, 6.0];

        let solver = GaussianSolver::<f64>::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 1.4).abs() < TOLERANCE_F64);
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

        let solver = GaussianSolver::<f64>::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 2.0).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 3.0).abs() < TOLERANCE_F64);
        assert!((result.solution[2] - (-1.0)).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_gaussian_singular_matrix() {
        let matrix = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let rhs = vec![3.0, 6.0];

        let solver = GaussianSolver::<f64>::new(SOLVER_TOLERANCE_F64);
        assert!(solver.solve(&matrix, &rhs).is_err());
    }

    #[test]
    fn test_gaussian_without_pivoting() {
        let matrix = vec![vec![0.001, 1.0], vec![2.0, 1.0]];
        let rhs = vec![1.0, 3.0];

        let solver = GaussianSolver::<f64>::new(SOLVER_TOLERANCE_F64).with_pivoting(false);
        let result = solver.solve(&matrix, &rhs);

        assert!(result.is_ok());
    }

    #[test]
    fn test_gaussian_2x2_f32() {
        let matrix = vec![vec![2.0_f32, 1.0_f32], vec![1.0_f32, 3.0_f32]];
        let rhs = vec![5.0_f32, 6.0_f32];

        let solver = GaussianSolver::<f32>::new(SOLVER_TOLERANCE_F32);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8_f32).abs() < 1e-4_f32);
        assert!((result.solution[1] - 1.4_f32).abs() < 1e-4_f32);
        assert!(result.converged);
    }
}
