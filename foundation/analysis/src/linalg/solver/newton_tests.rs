use super::newton::{
    newton_inverse, newton_inverse_generic, newton_solve, newton_solve_2d, newton_solve_generic,
    newton_solve_with_numeric_derivative_bounded_generic,
};
use crate::consts::test_constants::{INTEGRATION_TOLERANCE_STRICT, TOLERANCE_F64};

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
        assert!(sqrt_2 >= 0.0_f32 && sqrt_2 <= 2.0_f32);
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
}
