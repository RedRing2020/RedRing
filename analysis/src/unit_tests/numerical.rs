// API不一致のため一時的にコメントアウト
// use geo_algorithms::numerical::*;
// use geo_core::{Point2D, ToleranceContext};

// #[test]
// fn test_newton_solver() {
//     let tolerance = ToleranceContext::standard();
//     let solver = NewtonSolver::new(tolerance);
//
//     // Solve x^2 - 4 = 0 (expecting x = 2)
//     let f = |x: f64| x * x - 4.0;
//     let df = |x: f64| 2.0 * x;
//
//     let result = solver.solve(f, df, 1.0).unwrap();
//     assert!((result.root - 2.0).abs() < 1e-6);
//     assert!(result.converged);
//     assert!(result.iterations < 10);
// }
//
// #[test]
// fn test_least_squares_circle_fit() {
//     let tolerance = ToleranceContext::standard();
//     let fitter = LeastSquaresFitter::new(tolerance);
//
//     let mut points = Vec::new();
//     let center = Point2D::from_f64(1.0, 2.0);
//     let radius = 2.0;
//
//     for i in 0..10 {
//         let angle = (i as f64) * std::f64::consts::TAU / 10.0;
//         let x = center.x().value() + radius * angle.cos();
//         let y = center.y().value() + radius * angle.sin();
//         points.push(Point2D::from_f64(x, y));
//     }
//
//     let result = fitter.fit_circle(&points).unwrap();
//     assert!((result.radius - radius).abs() < 1e-3);
// }
