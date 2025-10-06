use geo_algorithms::numerical::*;
use geo_core::{Point2D, ToleranceContext};

#[test]
fn test_newton_solver_1d() {
    let tolerance = ToleranceContext::standard();
    let solver = NewtonSolver::new(tolerance);

    // x^2 - 4 = 0 の解（x = 2）
    let function = |x: f64| x * x - 4.0;
    let derivative = |x: f64| 2.0 * x;

    let result = solver.solve_1d(function, derivative, 1.0);
    assert!(result.is_ok());

    let (solution, info) = result.unwrap();
    assert!(info.converged);
    assert!((solution - 2.0).abs() < 1e-10);
}

#[test]
fn test_circle_fitting() {
    let tolerance = ToleranceContext::standard();
    let fitter = LeastSquaresFitter::new(tolerance);

    // 理想的な円上の点
    let center = Point2D::from_f64(1.0, 2.0);
    let radius = 3.0;
    
    let mut points = Vec::new();
    for i in 0..8 {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / 8.0;
        let x = center.x().value() + radius * angle.cos();
        let y = center.y().value() + radius * angle.sin();
        points.push(Point2D::from_f64(x, y));
    }

    let result = fitter.fit_circle(&points);
    assert!(result.is_ok());

    let (fitted_center, fitted_radius) = result.unwrap();
    assert!((fitted_center.x().value() - center.x().value()).abs() < 1e-10);
    assert!((fitted_center.y().value() - center.y().value()).abs() < 1e-10);
    assert!((fitted_radius - radius).abs() < 1e-10);
}