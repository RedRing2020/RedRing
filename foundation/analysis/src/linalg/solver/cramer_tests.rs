use super::{CramerSolver, LinearSolver};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cramer_2x2() {
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let rhs = vec![5.0, 6.0];

        let solver = CramerSolver::<f64>::new(1e-15_f64);
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

        let solver = CramerSolver::<f64>::new(1e-15_f64);
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

        let solver = CramerSolver::<f64>::new(1e-15_f64);
        let result = solver.solve(&matrix, &rhs);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only supports 2x2 and 3x3"));
    }
}
