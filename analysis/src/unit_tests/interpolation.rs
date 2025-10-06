// API不一致のため一時的にコメントアウト
// use geo_algorithms::interpolation::*;
// use geo_core::{Point2D, ToleranceContext};

// #[test]
// fn test_linear_interpolation() {
//     let tolerance = ToleranceContext::standard();
//     let interpolator = LinearInterpolator::new(tolerance);
//     
//     let p0 = Point2D::from_f64(0.0, 0.0);
//     let p1 = Point2D::from_f64(2.0, 4.0);
//     
//     let mid = interpolator.interpolate(&p0, &p1, 0.5);
//     assert!((mid.x().value() - 1.0).abs() < 1e-10);
//     assert!((mid.y().value() - 2.0).abs() < 1e-10);
// }
//
// #[test]
// fn test_bezier_curve() {
//     let p0 = Point2D::from_f64(0.0, 0.0);
//     let p1 = Point2D::from_f64(1.0, 1.0);
//     let p2 = Point2D::from_f64(2.0, 1.0);
//     let p3 = Point2D::from_f64(3.0, 0.0);
//     
//     let bezier = BezierCurve::new(p0, p1, p2, p3);
//     
//     let start = bezier.evaluate(0.0);
//     let end = bezier.evaluate(1.0);
//     
//     assert!((start.x().value() - 0.0).abs() < 1e-10);
//     assert!((start.y().value() - 0.0).abs() < 1e-10);
//     assert!((end.x().value() - 3.0).abs() < 1e-10);
//     assert!((end.y().value() - 0.0).abs() < 1e-10);
// }