use super::{GaussianSolver, LinearSolver};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_2x2() {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let rhs = vec![5.0, 6.0];

        let solver = GaussianSolver::<f64>::new(1e-15_f64);
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

        let solver = GaussianSolver::<f64>::new(1e-15_f64);
        let result = solver.solve(&matrix, &rhs).unwrap();

        assert!((result.solution[0] - 2.0).abs() < 1e-10);
        assert!((result.solution[1] - 3.0).abs() < 1e-10);
        assert!((result.solution[2] - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_gaussian_singular_matrix() {
        let matrix = vec![
            vec![1.0, 2.0],
            vec![2.0, 4.0],
        ];
        let rhs = vec![3.0, 6.0];

        let solver = GaussianSolver::<f64>::new(1e-15_f64);
        assert!(solver.solve(&matrix, &rhs).is_err());
    }

    #[test]
    fn test_gaussian_without_pivoting() {
        let matrix = vec![vec![0.001, 1.0], vec![2.0, 1.0]];
        let rhs = vec![1.0, 3.0];

        let solver = GaussianSolver::<f64>::new(1e-15_f64).with_pivoting(false);
        let result = solver.solve(&matrix, &rhs);

        assert!(result.is_ok());
    }
}
