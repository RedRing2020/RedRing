use analysis::numerics::integration::{newton_arc_length, trapezoidal_rule, NormedVector};

#[derive(Clone)]
struct TestVector2D {
    x: f64,
    y: f64,
}

impl NormedVector for TestVector2D {
    fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[test]
fn test_newton_arc_length_line() {
    let line_derivative = |_t: f64| TestVector2D { x: 1.0, y: 0.0 };
    let arc_length = newton_arc_length(line_derivative, 0.0, 1.0, 100);

    assert!((arc_length - 1.0).abs() < 1e-10);
}

#[test]
fn test_newton_arc_length_circle() {
    let circle_derivative = |t: f64| TestVector2D {
        x: -t.sin(),
        y: t.cos(),
    };

    let arc_length = newton_arc_length(circle_derivative, 0.0, std::f64::consts::PI, 1000);
    assert!((arc_length - std::f64::consts::PI).abs() < 1e-3);
}

#[test]
fn test_trapezoidal_rule_quadratic() {
    let result = trapezoidal_rule(|x| x * x, 0.0, 1.0, 1000);
    assert!((result - 1.0 / 3.0).abs() < 1e-6);
}

#[test]
fn test_trapezoidal_rule_sine() {
    let result = trapezoidal_rule(|x| x.sin(), 0.0, std::f64::consts::PI, 10000);
    assert!((result - 2.0).abs() < 1e-4);
}
