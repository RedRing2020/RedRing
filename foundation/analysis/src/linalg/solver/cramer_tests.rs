use super::{CramerSolver, LinearSolver};
use crate::consts::test_constants::{SOLVER_TOLERANCE_F32, SOLVER_TOLERANCE_F64, TOLERANCE_F64};
use crate::linalg::DynamicMatrix;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cramer_2x2() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = vec![5.0, 6.0];

        let solver = CramerSolver::<f64>::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 1.4).abs() < TOLERANCE_F64);
        assert!(result.converged);
    }

    #[test]
    fn test_cramer_3x3() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ])
        .unwrap();
        let rhs = vec![8.0, -11.0, -3.0];

        let solver = CramerSolver::<f64>::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 2.0).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 3.0).abs() < TOLERANCE_F64);
        assert!((result.solution[2] - (-1.0)).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_cramer_4x4_unsupported() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ])
        .unwrap();
        let rhs = vec![1.0, 2.0, 3.0, 4.0];

        let solver = CramerSolver::<f64>::new(SOLVER_TOLERANCE_F64);
        let result = solver.solve(&matrix, &rhs);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only supports 2x2 and 3x3"));
    }

    #[test]
    fn test_cramer_2x2_f32() {
        let matrix =
            DynamicMatrix::<f32>::from_rows(vec![vec![2.0_f32, 1.0_f32], vec![1.0_f32, 3.0_f32]])
                .unwrap();
        let rhs = vec![5.0_f32, 6.0_f32];

        let solver = CramerSolver::<f32>::new(SOLVER_TOLERANCE_F32);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 1.8_f32).abs() < 1e-4_f32);
        assert!((result.solution[1] - 1.4_f32).abs() < 1e-4_f32);
        assert!(result.converged);
    }
}
