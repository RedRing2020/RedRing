use super::newton::{
    newton_inverse, newton_inverse_generic, newton_solve, newton_solve_2d, newton_solve_generic,
    newton_solve_multivariate, newton_solve_multivariate_bounded,
    newton_solve_multivariate_bounded_with_solver, newton_solve_multivariate_with_solver,
    newton_solve_with_numeric_derivative_bounded_generic, MultivariateNewtonBounds,
    MultivariateNewtonOptions,
};
use crate::consts::test_constants::{INTEGRATION_TOLERANCE_STRICT, TOLERANCE_F64};
use crate::linalg::{DynamicMatrix, LUSolver, Vector};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newton_solve_wrapper_square_root_f64() {
        let f = |x: f64| x * x - 2.0;
        let df = |x: f64| 2.0 * x;
        let result = newton_solve(f, df, 1.0, 100, TOLERANCE_F64);

        assert!(result.is_some());
        let sqrt_2 = result.unwrap();
        assert!((sqrt_2 - std::f64::consts::SQRT_2).abs() < INTEGRATION_TOLERANCE_STRICT);
    }

    #[test]
    fn test_newton_inverse_wrapper_cube_root_f64() {
        let f = |x: f64| x * x * x;
        let df = |x: f64| 3.0 * x * x;
        let result = newton_inverse(f, df, 8.0, 2.0, 100, TOLERANCE_F64);

        assert!(result.is_some());
        let cube_root = result.unwrap();
        assert!((cube_root - 2.0).abs() < INTEGRATION_TOLERANCE_STRICT);
    }

    #[test]
    fn test_newton_solve_wrapper_zero_derivative() {
        let f = |x: f64| x * x;
        let df = |_: f64| 0.0;
        let result = newton_solve(f, df, 1.0, 100, TOLERANCE_F64);

        assert!(result.is_none());
    }

    #[test]
    fn test_newton_solve_generic_square_root_f64() {
        let f = |x: f64| x * x - 2.0;
        let df = |x: f64| 2.0 * x;
        let result = newton_solve_generic(f, df, 1.0_f64, 100, TOLERANCE_F64);

        assert!(result.is_some());
        let sqrt_2 = result.unwrap();
        assert!((sqrt_2 - std::f64::consts::SQRT_2).abs() < INTEGRATION_TOLERANCE_STRICT);
    }

    #[test]
    fn test_newton_solve_generic_square_root_f32() {
        let f = |x: f32| x * x - 2.0;
        let df = |x: f32| 2.0 * x;
        let result = newton_solve_generic(f, df, 1.0_f32, 100, 1e-6_f32);

        assert!(result.is_some());
        let sqrt_2 = result.unwrap();
        assert!((sqrt_2 - std::f32::consts::SQRT_2).abs() < 1e-4_f32);
    }

    #[test]
    fn test_newton_numeric_bounded_generic_keeps_solution_in_range() {
        let f = |x: f32| x * x - 2.0;
        let result = newton_solve_with_numeric_derivative_bounded_generic(
            f, 1.0_f32, 0.0_f32, 2.0_f32, 100, 1e-6_f32, 1e-4_f32,
        );

        assert!(result.is_some());
        let sqrt_2 = result.unwrap();
        assert!((0.0_f32..=2.0_f32).contains(&sqrt_2));
        assert!((sqrt_2 - std::f32::consts::SQRT_2).abs() < 1e-3_f32);
    }

    #[test]
    fn test_newton_inverse_generic_cube_root_f32() {
        let f = |x: f32| x * x * x;
        let df = |x: f32| 3.0 * x * x;
        let result = newton_inverse_generic(f, df, 8.0_f32, 2.0_f32, 100, 1e-6_f32);

        assert!(result.is_some());
        assert!((result.unwrap() - 2.0_f32).abs() < 1e-4_f32);
    }

    #[test]
    fn test_newton_solve_2d_circle_line() {
        let system = |x: f64, y: f64| {
            let f1 = x * x + y * y - 1.0;
            let f2 = x - y;
            let jacobian = [[2.0 * x, 2.0 * y], [1.0, -1.0]];
            (f1, f2, jacobian)
        };

        let result = newton_solve_2d(system, (1.0, 0.5), 100, TOLERANCE_F64);
        assert!(result.is_some());

        let (x, y) = result.unwrap();
        let expected = 1.0 / 2_f64.sqrt();
        assert!((x - expected).abs() < INTEGRATION_TOLERANCE_STRICT);
        assert!((y - expected).abs() < INTEGRATION_TOLERANCE_STRICT);
    }

    #[test]
    fn test_newton_solve_2d_singular_jacobian() {
        let system = |x: f64, y: f64| {
            let f1 = x + y;
            let f2 = x + y;
            let jacobian = [[1.0, 1.0], [1.0, 1.0]];
            (f1, f2, jacobian)
        };

        let result = newton_solve_2d(system, (1.0, 1.0), 100, TOLERANCE_F64);
        assert!(result.is_none());
    }

    #[test]
    fn test_newton_solve_2d_circle_line_f32() {
        let system = |x: f32, y: f32| {
            let f1 = x * x + y * y - 1.0_f32;
            let f2 = x - y;
            let jacobian = [[2.0_f32 * x, 2.0_f32 * y], [1.0_f32, -1.0_f32]];
            (f1, f2, jacobian)
        };

        let result = newton_solve_2d(system, (1.0_f32, 0.5_f32), 100, 1e-6_f32);
        assert!(result.is_some());

        let (x, y) = result.unwrap();
        let expected = 1.0_f32 / 2.0_f32.sqrt();
        assert!((x - expected).abs() < 1e-4_f32);
        assert!((y - expected).abs() < 1e-4_f32);
    }

    #[test]
    fn test_newton_solve_2d_singular_jacobian_f32() {
        let system = |x: f32, y: f32| {
            let f1 = x + y;
            let f2 = x + y;
            let jacobian = [[1.0_f32, 1.0_f32], [1.0_f32, 1.0_f32]];
            (f1, f2, jacobian)
        };

        let result = newton_solve_2d(system, (1.0_f32, 1.0_f32), 100, 1e-6_f32);
        assert!(result.is_none());
    }

    #[test]
    fn test_newton_solve_multivariate_circle_line_f64() {
        let system = |point: &Vector<f64>| {
            let x = point[0];
            let y = point[1];
            let residual = Vector::new(vec![x * x + y * y - 1.0, x - y]);
            let jacobian =
                DynamicMatrix::from_rows(vec![vec![2.0 * x, 2.0 * y], vec![1.0, -1.0]]).unwrap();
            (residual, jacobian)
        };

        let result =
            newton_solve_multivariate(system, Vector::new(vec![1.0, 0.5]), 100, TOLERANCE_F64);
        assert!(result.is_some());

        let solution = result.unwrap();
        let expected = 1.0 / 2_f64.sqrt();
        assert!((solution[0] - expected).abs() < INTEGRATION_TOLERANCE_STRICT);
        assert!((solution[1] - expected).abs() < INTEGRATION_TOLERANCE_STRICT);
    }

    #[test]
    fn test_newton_solve_multivariate_with_lu_solver_f32() {
        let system = |point: &Vector<f32>| {
            let x = point[0];
            let y = point[1];
            let residual = Vector::new(vec![x * x + y * y - 1.0_f32, x - y]);
            let jacobian = DynamicMatrix::from_rows(vec![
                vec![2.0_f32 * x, 2.0_f32 * y],
                vec![1.0_f32, -1.0_f32],
            ])
            .unwrap();
            (residual, jacobian)
        };

        let solver = LUSolver::new(1e-6_f32);
        let result = newton_solve_multivariate_with_solver(
            system,
            Vector::new(vec![1.0_f32, 0.5_f32]),
            &solver,
            100,
            1e-6_f32,
        );
        assert!(result.is_some());

        let solution = result.unwrap();
        let expected = 1.0_f32 / 2.0_f32.sqrt();
        assert!((solution[0] - expected).abs() < 1e-4_f32);
        assert!((solution[1] - expected).abs() < 1e-4_f32);
    }

    #[test]
    fn test_newton_solve_multivariate_rejects_shape_mismatch() {
        let system = |point: &Vector<f64>| {
            let residual = Vector::new(vec![point[0], point[1]]);
            let jacobian =
                DynamicMatrix::from_rows(vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]]).unwrap();
            (residual, jacobian)
        };

        let result =
            newton_solve_multivariate(system, Vector::new(vec![1.0, 1.0]), 10, TOLERANCE_F64);
        assert!(result.is_none());
    }

    #[test]
    fn test_newton_solve_multivariate_singular_jacobian() {
        let system = |point: &Vector<f64>| {
            let residual = Vector::new(vec![point[0] + point[1], point[0] + point[1]]);
            let jacobian = DynamicMatrix::from_rows(vec![vec![1.0, 1.0], vec![1.0, 1.0]]).unwrap();
            (residual, jacobian)
        };

        let result =
            newton_solve_multivariate(system, Vector::new(vec![1.0, 1.0]), 10, TOLERANCE_F64);
        assert!(result.is_none());
    }

    #[test]
    fn test_multivariate_newton_bounds_reject_dimension_mismatch() {
        let result = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, 0.0_f64]),
            Vector::new(vec![1.0_f64]),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_multivariate_newton_bounds_reject_inverted_axis_range() {
        let result = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, 2.0_f64]),
            Vector::new(vec![1.0_f64, 1.0_f64]),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_multivariate_newton_bounds_clamp_and_contains() {
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 2.0_f64]),
        )
        .unwrap();

        let point = Vector::new(vec![1.5_f64, -2.0_f64]);
        let clamped = bounds.clamp(&point).unwrap();

        assert_eq!(clamped, Vector::new(vec![1.0_f64, -1.0_f64]));
        assert!(bounds.contains(&clamped));
        assert!(!bounds.contains(&point));
    }

    #[test]
    fn test_multivariate_newton_bounds_contains_with_tolerance_accepts_near_boundary() {
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 2.0_f64]),
        )
        .unwrap();
        let point = Vector::new(vec![1.0005_f64, -1.0005_f64]);

        assert!(bounds.contains_with_tolerance(&point, 1e-3_f64));
    }

    #[test]
    fn test_multivariate_newton_bounds_contains_with_tolerance_rejects_outside_range() {
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 2.0_f64]),
        )
        .unwrap();
        let point = Vector::new(vec![1.01_f64, -1.01_f64]);

        assert!(!bounds.contains_with_tolerance(&point, 1e-3_f64));
    }

    #[test]
    fn test_multivariate_newton_bounds_contains_with_tolerance_zero_matches_strict_contains() {
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 2.0_f64]),
        )
        .unwrap();
        let point = Vector::new(vec![1.0_f64, -1.0_f64]);
        let outside = Vector::new(vec![1.0001_f64, -1.0_f64]);

        assert_eq!(
            bounds.contains(&point),
            bounds.contains_with_tolerance(&point, 0.0_f64)
        );
        assert_eq!(
            bounds.contains(&outside),
            bounds.contains_with_tolerance(&outside, 0.0_f64)
        );
    }

    #[test]
    fn test_multivariate_newton_bounds_contains_with_tolerance_rejects_dimension_mismatch() {
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 2.0_f64]),
        )
        .unwrap();
        let point = Vector::new(vec![0.5_f64]);

        assert!(!bounds.contains_with_tolerance(&point, 1e-3_f64));
    }

    #[test]
    fn test_multivariate_newton_bounds_contains_with_tolerance_clamps_negative_tolerance_to_zero() {
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![0.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 2.0_f64]),
        )
        .unwrap();
        let point = Vector::new(vec![1.0001_f64, -1.0_f64]);

        assert_eq!(
            bounds.contains_with_tolerance(&point, -1e-3_f64),
            bounds.contains_with_tolerance(&point, 0.0_f64)
        );
    }

    #[test]
    fn test_multivariate_newton_options_new_sets_separate_values() {
        let options = MultivariateNewtonOptions::new(25, 1e-4_f64, 1e-8_f64);

        assert_eq!(options.max_iter, 25);
        assert_eq!(options.residual_tol, 1e-4_f64);
        assert_eq!(options.step_tol, 1e-8_f64);
    }

    #[test]
    fn test_newton_solve_multivariate_bounded_clamps_initial_point() {
        let system = |point: &Vector<f64>| {
            let residual = Vector::new(vec![point[0] - 1.0]);
            let jacobian = DynamicMatrix::from_rows(vec![vec![1.0_f64]]).unwrap();
            (residual, jacobian)
        };
        let bounds =
            MultivariateNewtonBounds::new(Vector::new(vec![0.0_f64]), Vector::new(vec![1.0_f64]))
                .unwrap();
        let options = MultivariateNewtonOptions::new(20, TOLERANCE_F64, TOLERANCE_F64);

        let result = newton_solve_multivariate_bounded(
            system,
            Vector::new(vec![3.0_f64]),
            &bounds,
            &options,
        );
        assert!(result.is_some());

        let solution = result.unwrap();
        assert_eq!(solution, Vector::new(vec![1.0_f64]));
        assert!(bounds.contains(&solution));
    }

    #[test]
    fn test_newton_solve_multivariate_bounded_converges_on_boundary() {
        let system = |point: &Vector<f64>| {
            let x = point[0];
            let residual = Vector::new(vec![x * x - 1.0]);
            let jacobian = DynamicMatrix::from_rows(vec![vec![2.0_f64 * x]]).unwrap();
            (residual, jacobian)
        };
        let bounds =
            MultivariateNewtonBounds::new(Vector::new(vec![0.0_f64]), Vector::new(vec![1.0_f64]))
                .unwrap();
        let options = MultivariateNewtonOptions::new(20, TOLERANCE_F64, TOLERANCE_F64);

        let result = newton_solve_multivariate_bounded(
            system,
            Vector::new(vec![0.2_f64]),
            &bounds,
            &options,
        );
        assert!(result.is_some());

        let solution = result.unwrap();
        assert!((solution[0] - 1.0_f64).abs() < INTEGRATION_TOLERANCE_STRICT);
        assert!(bounds.contains(&solution));
    }

    #[test]
    fn test_newton_solve_multivariate_bounded_with_solver_rejects_shape_mismatch() {
        let system = |point: &Vector<f64>| {
            let residual = Vector::new(vec![point[0], point[1]]);
            let jacobian = DynamicMatrix::from_rows(vec![
                vec![1.0_f64, 0.0_f64, 0.0_f64],
                vec![0.0_f64, 1.0_f64, 0.0_f64],
            ])
            .unwrap();
            (residual, jacobian)
        };
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![-1.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 1.0_f64]),
        )
        .unwrap();
        let options = MultivariateNewtonOptions::new(10, TOLERANCE_F64, TOLERANCE_F64);
        let solver = LUSolver::new(TOLERANCE_F64);

        let result = newton_solve_multivariate_bounded_with_solver(
            system,
            Vector::new(vec![0.5_f64, 0.5_f64]),
            &bounds,
            &solver,
            &options,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_newton_solve_multivariate_bounded_singular_jacobian() {
        let system = |point: &Vector<f64>| {
            let residual = Vector::new(vec![point[0] + point[1], point[0] + point[1]]);
            let jacobian =
                DynamicMatrix::from_rows(vec![vec![1.0_f64, 1.0_f64], vec![1.0_f64, 1.0_f64]])
                    .unwrap();
            (residual, jacobian)
        };
        let bounds = MultivariateNewtonBounds::new(
            Vector::new(vec![-1.0_f64, -1.0_f64]),
            Vector::new(vec![1.0_f64, 1.0_f64]),
        )
        .unwrap();
        let options = MultivariateNewtonOptions::new(10, TOLERANCE_F64, TOLERANCE_F64);

        let result = newton_solve_multivariate_bounded(
            system,
            Vector::new(vec![0.5_f64, 0.5_f64]),
            &bounds,
            &options,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_newton_solve_multivariate_bounded_uses_separate_step_tolerance() {
        let system = |point: &Vector<f64>| {
            let residual = Vector::new(vec![point[0] - 0.8_f64]);
            let jacobian = DynamicMatrix::from_rows(vec![vec![1.0_f64]]).unwrap();
            (residual, jacobian)
        };
        let bounds =
            MultivariateNewtonBounds::new(Vector::new(vec![0.0_f64]), Vector::new(vec![1.0_f64]))
                .unwrap();
        let options = MultivariateNewtonOptions::new(1, 0.1_f64, 0.5_f64);

        let result = newton_solve_multivariate_bounded(
            system,
            Vector::new(vec![0.0_f64]),
            &bounds,
            &options,
        );

        assert!(result.is_none());
    }
}
