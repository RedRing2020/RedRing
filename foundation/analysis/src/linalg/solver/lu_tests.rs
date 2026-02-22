use super::{LUSolver, LinearSolver};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lu_decomposition_2x2() {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let solver = LUSolver::<f64>::new(1e-15_f64);

        let decomp = solver.decompose(&matrix).unwrap();

        let n = matrix.len();
        for i in 0..n {
            for (j, _) in matrix.iter().enumerate().take(n) {
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
                let original = matrix[decomp.permutation[i]][j];
                assert!((reconstructed - original).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_lu_solver_2x2() {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let rhs = vec![5.0, 6.0];
        let solver = LUSolver::<f64>::new(1e-15_f64);

        let result = solver.solve(&matrix, &rhs).unwrap();
        assert!((result.solution[0] - 1.8).abs() < 1e-10);
        assert!((result.solution[1] - 1.4).abs() < 1e-10);
        assert!(result.converged);
    }

    #[test]
    fn test_lu_solver_3x3() {
        let matrix = vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ];
        let rhs = vec![8.0, -11.0, -3.0];
        let solver = LUSolver::<f64>::new(1e-15_f64);

        let result = solver.solve(&matrix, &rhs).unwrap();
        assert!((result.solution[0] - 2.0).abs() < 1e-10);
        assert!((result.solution[1] - 3.0).abs() < 1e-10);
        assert!((result.solution[2] - (-1.0)).abs() < 1e-10);
        assert!(result.converged);
    }

    #[test]
    fn test_lu_determinant() {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let solver = LUSolver::<f64>::new(1e-15_f64);

        let decomp = solver.decompose(&matrix).unwrap();
        let det = solver.determinant(&decomp);

        assert!((det - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_lu_singular_matrix() {
        let matrix = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let solver = LUSolver::<f64>::new(1e-15_f64);
        assert!(solver.decompose(&matrix).is_err());
    }
}
