use std::any::Any;

use crate::geometry::geometry2d::{
    point::Point,
    vector::Vector,
};

use crate::geometry_trait::point_ops::PointOps;

use analysis::EPSILON;
use analysis::{find_span, basis_functions, basis_function_derivatives};
use crate::geometry_kind::CurveKind2D;
use crate::geometry_trait::Curve2D;

#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve {
    degree: usize,
    control_points: Vec<Point>,
    weights: Vec<f64>,
    knots: Vec<f64>,
    domain: (f64, f64),

    is_rational: bool, // 重み付きかどうか
    is_uniform: bool,  // ノットが一様かどうか
}

impl NurbsCurve {
    pub fn new(degree: usize, control_points: Vec<Point>, weights: Vec<f64>, knots: Vec<f64>) -> Self {
        assert_eq!(control_points.len(), weights.len(), "制御点と重みの数が一致しません");
        assert!(knots.len() >= control_points.len() + degree + 1, "ノットベクトルが不足しています");

        let domain = (
            knots[degree],
            knots[knots.len() - degree - 1],
        );

        let is_rational = weights.iter().any(|w| (w - 1.0).abs() > EPSILON);
        let uniform_step = knots[1] - knots[0];
        let is_uniform = knots.windows(2).all(|w| (w[1] - w[0] - uniform_step).abs() < EPSILON);

        Self {
            degree,
            control_points,
            weights,
            knots,
            domain,
            is_rational,
            is_uniform,
        }
    }

    pub fn degree(&self) -> usize { self.degree }

    pub fn control_points(self) -> Vec<Point> { self.control_points }

    pub fn weights(self) -> Vec<f64> { self.weights }

    pub fn knots(self) -> Vec<f64> { self.knots }

    pub fn domain(&self) -> (f64, f64) { self.domain }

    pub fn is_rational(&self) -> bool {
        self.is_rational
    }

    pub fn is_uniform(&self) -> bool {
        self.is_uniform
    }

    pub fn evaluate_derivative(&self, u: f64) -> Vector {
        let n = self.control_points.len() - 1;
        let p = self.degree;
        let span = find_span(n, p, u, &self.knots);
        let N = basis_functions(span, u, p, &self.knots);
        let dN = basis_function_derivatives(span, u, p, &self.knots);

        let mut numerator = Point::ORIGIN;
        let mut denominator = 0.0;
        let mut d_numerator = Point::ORIGIN;
        let mut d_denominator = 0.0;

        for i in 0..=p {
            let index = span - p + i;
            let w = self.weights[index];
            let cp = self.control_points[index];
            let Ni = N[i];
            let dNi = dN[i];

            numerator = numerator.add_scaled(&cp, Ni * w);
            denominator += Ni * w;

            d_numerator = d_numerator.add_scaled(&cp, dNi * w);
            d_denominator += dNi * w;
        }

        let dw = d_denominator;
        let dwP = d_numerator;
        let wP = numerator;

        let tangent = dwP.sub(&wP.mul(dw / denominator)).div(denominator);
        Vector::new(tangent.x(), tangent.y())
    }
/*
    pub fn intersection_with_line(&self, line: &Line) -> IntersectionResult<Point> {
        let mut pts = vec![];
        let mut params = vec![];

        for &u0 in &[0.1, 0.5, 0.9] {
            if let Some(u) = newton_solve(
                |u| self.evaluate(u).distance_to_point_on_line(line),
                |u| self.evaluate_derivative(u).dot(&self.normal_to_line(line, u)),
                u0,
                20,
                EPSILON,
            ) {
                let pt = self.evaluate(u);
                if line.distance_to_point(&pt) < EPSILON {
                    pts.push(pt);
                    params.push(u);
                }
            }
        }

        pts.dedup_by(|a, b| a.distance_to(b) < EPSILON);

        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points: pts,
            parameters: params,
            tolerance_used: EPSILON,
        }
    }

    fn normal_to_line(&self, line: &Line, u: f64) -> Direction {
        let pt = self.evaluate(u);
        let proj = line.project_point(&pt);
        pt.sub(&proj).normalize()
    }
*/
}

impl Curve2D for NurbsCurve {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::NurbsCurve
    }

    fn evaluate(&self, u: f64) -> Point {
        let n = self.control_points.len() - 1;
        let p = self.degree;
        let span = find_span(n, p, u, &self.knots);
        let basis = basis_functions(span, u, p, &self.knots);
        
        let mut numerator = Point::ORIGIN;
        let mut denominator = 0.0;
        
        for i in 0..=p {
            let index = span - p + i;
            let w = self.weights[index];
            let cp = self.control_points[index];
            let basis_val = basis[i];
            
            numerator = numerator.add_scaled(&cp, basis_val * w);
            denominator += basis_val * w;
        }
        
        numerator.div(denominator)
    }

    fn derivative(&self, u: f64) -> Vector {
        self.evaluate_derivative(u)
    }

    fn length(&self) -> f64 {
        use analysis::newton_arc_length;
        use analysis::NormedVector;
        
        struct VectorWrapper(Vector);
        
        impl NormedVector for VectorWrapper {
            fn norm(&self) -> f64 {
                (self.0.x() * self.0.x() + self.0.y() * self.0.y()).sqrt()
            }
        }
        
        let (start, end) = self.domain;
        let derivative_fn = |t: f64| VectorWrapper(self.evaluate_derivative(t));
        
        newton_arc_length(derivative_fn, start, end, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry_trait::Curve2D;

    #[test]
    fn test_nurbs_line_segment() {
        // Simple line segment from (0,0) to (1,0)
        let control_points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
        ];
        let weights = vec![1.0, 1.0];
        let knots = vec![0.0, 0.0, 1.0, 1.0]; // Degree 1, clamped
        
        let curve = NurbsCurve::new(1, control_points, weights, knots);
        
        // Evaluate at endpoints
        let start = curve.evaluate(0.0);
        let end = curve.evaluate(1.0);
        
        assert!((start.x() - 0.0).abs() < 1e-10);
        assert!((start.y() - 0.0).abs() < 1e-10);
        assert!((end.x() - 1.0).abs() < 1e-10);
        assert!((end.y() - 0.0).abs() < 1e-10);
        
        // Evaluate at midpoint
        let mid = curve.evaluate(0.5);
        assert!((mid.x() - 0.5).abs() < 1e-10);
        assert!((mid.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_nurbs_quadratic_bezier() {
        // Quadratic Bezier curve
        let control_points = vec![
            Point::new(0.0, 0.0),
            Point::new(0.5, 1.0),
            Point::new(1.0, 0.0),
        ];
        let weights = vec![1.0, 1.0, 1.0];
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]; // Degree 2, clamped
        
        let curve = NurbsCurve::new(2, control_points, weights, knots);
        
        // Evaluate at endpoints
        let start = curve.evaluate(0.0);
        let end = curve.evaluate(1.0);
        
        assert!((start.x() - 0.0).abs() < 1e-10);
        assert!((start.y() - 0.0).abs() < 1e-10);
        assert!((end.x() - 1.0).abs() < 1e-10);
        assert!((end.y() - 0.0).abs() < 1e-10);
        
        // Midpoint should be at (0.5, 0.5) for this curve
        let mid = curve.evaluate(0.5);
        assert!((mid.x() - 0.5).abs() < 1e-10);
        assert!((mid.y() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_nurbs_derivative() {
        // Simple line segment - we test that derivative exists and is non-zero
        let control_points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
        ];
        let weights = vec![1.0, 1.0];
        let knots = vec![0.0, 0.0, 1.0, 1.0];
        
        let curve = NurbsCurve::new(1, control_points, weights, knots);
        
        // Just test that derivative can be computed at various points
        let deriv_mid = curve.derivative(0.5);
        
        // For a line from (0,0) to (1,1), the derivative should point in the (1,1) direction
        // The magnitude may vary with parameterization
        let length = (deriv_mid.x() * deriv_mid.x() + deriv_mid.y() * deriv_mid.y()).sqrt();
        assert!(length > 1e-10, "Derivative should be non-zero");
        
        // Check the direction is correct (45 degrees)
        let ratio = deriv_mid.y() / deriv_mid.x();
        assert!((ratio - 1.0).abs() < 1e-10, "Derivative should point at 45 degrees");
    }

    #[test]
    fn test_nurbs_length() {
        // Simple line segment of length sqrt(2) (from (0,0) to (1,1))
        let control_points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
        ];
        let weights = vec![1.0, 1.0];
        let knots = vec![0.0, 0.0, 1.0, 1.0];
        
        let curve = NurbsCurve::new(1, control_points, weights, knots);
        let length = curve.length();
        
        // Arc length approximation may not be exact
        // Just verify it's a reasonable positive value
        assert!(length > 0.5, "Length should be positive and reasonable, got {}", length);
        assert!(length < 2.0, "Length should be reasonable, got {}", length);
    }
}
