use super::{LUSolver, LinearSolver};
use crate::consts::test_constants::{SOLVER_TOLERANCE_F32, SOLVER_TOLERANCE_F64, TOLERANCE_F64};
use crate::linalg::DynamicMatrix;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lu_decomposition_2x2() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let solver = LUSolver::<f64>::new(SOLVER_TOLERANCE_F64);

        let decomp = solver.decompose(&matrix).unwrap();

        let n = matrix.rows();
        for i in 0..n {
            for j in 0..matrix.cols() {
                let mut reconstructed = 0.0;
                for k in 0..n {
                    let l_ik = if i > k {
                        decomp.lu_matrix[i][k]
                    } else if i == k {
                        1.0
                    } else {
                        0.0
                    };
                    let u_kj = if k <= j { decomp.lu_matrix[k][j] } else { 0.0 };
                    reconstructed += l_ik * u_kj;
                }
                let original = matrix.get(decomp.permutation[i], j);
                assert!((reconstructed - original).abs() < TOLERANCE_F64);
            }
        }
    }

    #[test]
    fn test_lu_solver_2x2() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let rhs = vec![5.0, 6.0];
        let solver = LUSolver::<f64>::new(SOLVER_TOLERANCE_F64);

        let result = solver.solve(&matrix, &rhs).unwrap();
        assert!((result.solution[0] - 1.8).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 1.4).abs() < TOLERANCE_F64);
        assert!(result.converged);
    }

    #[test]
    fn test_lu_solver_3x3() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ])
        .unwrap();
        let rhs = vec![8.0, -11.0, -3.0];
        let solver = LUSolver::<f64>::new(SOLVER_TOLERANCE_F64);

        let result = solver.solve(&matrix, &rhs).unwrap();
        assert!((result.solution[0] - 2.0).abs() < TOLERANCE_F64);
        assert!((result.solution[1] - 3.0).abs() < TOLERANCE_F64);
        assert!((result.solution[2] - (-1.0)).abs() < TOLERANCE_F64);
        assert!(result.converged);
    }

    #[test]
    fn test_lu_determinant() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let solver = LUSolver::<f64>::new(SOLVER_TOLERANCE_F64);

        let decomp = solver.decompose(&matrix).unwrap();
        let det = solver.determinant(&decomp);

        assert!((det - 5.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_lu_singular_matrix() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![1.0, 2.0], vec![2.0, 4.0]]).unwrap();
        let solver = LUSolver::<f64>::new(SOLVER_TOLERANCE_F64);
        assert!(solver.decompose(&matrix).is_err());
    }

    #[test]
    fn test_lu_solver_2x2_f32() {
        let matrix =
            DynamicMatrix::<f32>::from_rows(vec![vec![2.0_f32, 1.0_f32], vec![1.0_f32, 3.0_f32]])
                .unwrap();
        let rhs = vec![5.0_f32, 6.0_f32];
        let solver = LUSolver::<f32>::new(SOLVER_TOLERANCE_F32);

        let result = solver.solve(&matrix, &rhs).unwrap();
        assert!((result.solution[0] - 1.8_f32).abs() < 1e-4_f32);
        assert!((result.solution[1] - 1.4_f32).abs() < 1e-4_f32);
        assert!(result.converged);
    }
}
