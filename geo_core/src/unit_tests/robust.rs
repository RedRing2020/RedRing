/// Robust演算のユニットテスト

use crate::robust::{Orientation, RobustSolver, predicates};
use crate::vector::Vector2D;
use crate::tolerance::ToleranceContext;

#[test]
fn test_orientation() {
    let a = Vector2D::from_f64(0.0, 0.0);
    let b = Vector2D::from_f64(1.0, 0.0);
    let c = Vector2D::from_f64(0.0, 1.0);

    assert_eq!(predicates::orient_2d(&a, &b, &c), Orientation::CounterClockwise);
    assert_eq!(predicates::orient_2d(&a, &c, &b), Orientation::Clockwise);
}

#[test]
fn test_quadratic_solver() {
    let context = ToleranceContext::standard();
    let solver = RobustSolver::new(context);

    // x² - 5x + 6 = 0, solutions: x = 2, 3
    let solutions = solver.solve_quadratic(1.0, -5.0, 6.0);
    assert_eq!(solutions.len(), 2);
    assert!((solutions[0] - 3.0).abs() < 1e-10 || (solutions[0] - 2.0).abs() < 1e-10);
    assert!((solutions[1] - 3.0).abs() < 1e-10 || (solutions[1] - 2.0).abs() < 1e-10);
}