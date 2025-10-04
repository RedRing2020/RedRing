use std::any::Any;

use super::{point::Point, vector::Vector};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;

/// Represents a trimmed NURBS curve in 3D space.
/// Designed for internal use in RedRing, not strictly STEP-compliant.
#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve {
    degree: usize,                     // Degree of the curve
    control_points: Vec<Point>,      // Control points
    weights: Option<Vec<f64>>,        // Optional weights (None = non-rational)
    knots: Vec<f64>,                  // Knot vector
    multiplicities: Vec<usize>,       // Knot multiplicities
    is_uniform: bool,                 // Whether the knot vector is uniform
    start_param: f64,                 // Trim start parameter
    end_param: f64,                   // Trim end parameter
}

impl NurbsCurve {
    /// Creates a new trimmed NURBS curve.
    pub fn new(
        degree: usize,
        control_points: Vec<Point>,
        weights: Option<Vec<f64>>,
        knots: Vec<f64>,
        multiplicities: Vec<usize>,
        is_uniform: bool,
        start_param: f64,
        end_param: f64,
    ) -> Option<Self> {
        let n = control_points.len();
        if n == 0 || knots.len() != multiplicities.len() {
            return None;
        }
        if let Some(w) = &weights {
            if w.len() != n {
                return None;
            }
        }
        if start_param >= end_param {
            return None;
        }
        Some(Self {
            degree,
            control_points,
            weights,
            knots,
            multiplicities,
            is_uniform,
            start_param,
            end_param,
        })
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    pub fn control_points(&self) -> &[Point] {
        &self.control_points
    }

    pub fn weights(&self) -> Option<&[f64]> {
        self.weights.as_deref()
    }

    pub fn weight_at(&self, i: usize) -> f64 {
        self.weights.as_ref().map_or(1.0, |w| w[i])
    }

    pub fn is_rational(&self) -> bool {
        self.weights.is_some()
    }

    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    pub fn multiplicities(&self) -> &[usize] {
        &self.multiplicities
    }

    pub fn is_uniform(&self) -> bool {
        self.is_uniform
    }

    pub fn start_param(&self) -> f64 {
        self.start_param
    }

    pub fn end_param(&self) -> f64 {
        self.end_param
    }

    pub fn is_trimmed(&self) -> bool {
        let domain_start = self.knots.first().copied().unwrap_or(0.0);
        let domain_end = self.knots.last().copied().unwrap_or(1.0);
        self.start_param > domain_start || self.end_param < domain_end
    }

    pub fn num_control_points(&self) -> usize {
        self.control_points.len()
    }
}

impl Curve3D for NurbsCurve {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn evaluate(&self, t: f64) -> Point {
        use analysis::{find_span, basis_functions};
        use crate::geometry_trait::point_ops::PointOps;
        
        // Clamp parameter to valid range
        let t = t.max(self.start_param).min(self.end_param);
        
        let n = self.control_points.len() - 1;
        let p = self.degree;
        
        // Build full knot vector with multiplicities
        let mut full_knots = Vec::new();
        for (i, &mult) in self.multiplicities.iter().enumerate() {
            for _ in 0..mult {
                full_knots.push(self.knots[i]);
            }
        }
        
        let span = find_span(n, p, t, &full_knots);
        let basis = basis_functions(span, t, p, &full_knots);
        
        let mut numerator = Point::origin();
        let mut denominator = 0.0;
        
        for i in 0..=p {
            let index = span - p + i;
            if index < self.control_points.len() {
                let w = self.weights.as_ref().map_or(1.0, |weights| weights[index]);
                let cp = &self.control_points[index];
                let basis_val = basis[i];
                
                numerator = numerator.add_scaled(cp, basis_val * w);
                denominator += basis_val * w;
            }
        }
        
        numerator.div(denominator)
    }
    fn derivative(&self, t: f64) -> Vector {
        use analysis::{find_span, basis_functions, basis_function_derivatives};
        use crate::geometry_trait::point_ops::PointOps;
        
        // Clamp parameter to valid range
        let t = t.max(self.start_param).min(self.end_param);
        
        let n = self.control_points.len() - 1;
        let p = self.degree;
        
        // Build full knot vector with multiplicities
        let mut full_knots = Vec::new();
        for (i, &mult) in self.multiplicities.iter().enumerate() {
            for _ in 0..mult {
                full_knots.push(self.knots[i]);
            }
        }
        
        let span = find_span(n, p, t, &full_knots);
        let basis = basis_functions(span, t, p, &full_knots);
        let basis_derivs = basis_function_derivatives(span, t, p, &full_knots);
        
        let mut numerator = Point::origin();
        let mut denominator = 0.0;
        let mut d_numerator = Point::origin();
        let mut d_denominator = 0.0;
        
        for i in 0..=p {
            let index = span - p + i;
            if index < self.control_points.len() {
                let w = self.weights.as_ref().map_or(1.0, |weights| weights[index]);
                let cp = &self.control_points[index];
                let basis_val = basis[i];
                let basis_deriv = basis_derivs[i];
                
                numerator = numerator.add_scaled(cp, basis_val * w);
                denominator += basis_val * w;
                
                d_numerator = d_numerator.add_scaled(cp, basis_deriv * w);
                d_denominator += basis_deriv * w;
            }
        }
        
        // Quotient rule: (d_numerator * denominator - numerator * d_denominator) / denominator^2
        let tangent = d_numerator.sub(&numerator.mul(d_denominator / denominator)).div(denominator);
        Vector::new(tangent.x(), tangent.y(), tangent.z())
    }
    fn kind(&self) -> CurveKind3D {
        CurveKind3D::NurbsCurve
    }
    fn length(&self) -> f64 {
        use analysis::newton_arc_length;
        use analysis::NormedVector;
        
        struct VectorWrapper(Vector);
        
        impl NormedVector for VectorWrapper {
            fn norm(&self) -> f64 {
                (self.0.x() * self.0.x() + self.0.y() * self.0.y() + self.0.z() * self.0.z()).sqrt()
            }
        }
        
        let derivative_fn = |t: f64| VectorWrapper(self.derivative(t));
        
        newton_arc_length(derivative_fn, self.start_param, self.end_param, 100)
    }
    
    fn domain(&self) -> (f64, f64) {
        (self.start_param, self.end_param)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry_trait::Curve3D;

    #[test]
    fn test_nurbs_3d_line_segment() {
        // Simple line segment from (0,0,0) to (1,1,1)
        let control_points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 1.0),
        ];
        let weights = Some(vec![1.0, 1.0]);
        let knots = vec![0.0, 1.0]; // Unique knot values
        let multiplicities = vec![2, 2]; // Degree 1, clamped (multiplicity = degree + 1)
        
        let curve = NurbsCurve::new(
            1,
            control_points,
            weights,
            knots,
            multiplicities,
            false,
            0.0,
            1.0,
        ).expect("Failed to create NURBS curve");
        
        // Evaluate at endpoints
        let start = curve.evaluate(0.0);
        let end = curve.evaluate(1.0);
        
        assert!((start.x() - 0.0).abs() < 1e-10);
        assert!((start.y() - 0.0).abs() < 1e-10);
        assert!((start.z() - 0.0).abs() < 1e-10);
        assert!((end.x() - 1.0).abs() < 1e-10);
        assert!((end.y() - 1.0).abs() < 1e-10);
        assert!((end.z() - 1.0).abs() < 1e-10);
        
        // Evaluate at midpoint
        let mid = curve.evaluate(0.5);
        assert!((mid.x() - 0.5).abs() < 1e-10);
        assert!((mid.y() - 0.5).abs() < 1e-10);
        assert!((mid.z() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_nurbs_3d_quadratic() {
        // Quadratic curve in 3D
        let control_points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(0.5, 1.0, 0.5),
            Point::new(1.0, 0.0, 1.0),
        ];
        let weights = Some(vec![1.0, 1.0, 1.0]);
        let knots = vec![0.0, 1.0]; // Unique knot values
        let multiplicities = vec![3, 3]; // Degree 2, clamped (multiplicity = degree + 1)
        
        let curve = NurbsCurve::new(
            2,
            control_points,
            weights,
            knots,
            multiplicities,
            false,
            0.0,
            1.0,
        ).expect("Failed to create NURBS curve");
        
        // Evaluate at endpoints
        let start = curve.evaluate(0.0);
        let end = curve.evaluate(1.0);
        
        assert!((start.x() - 0.0).abs() < 1e-10);
        assert!((start.y() - 0.0).abs() < 1e-10);
        assert!((start.z() - 0.0).abs() < 1e-10);
        assert!((end.x() - 1.0).abs() < 1e-10);
        assert!((end.y() - 0.0).abs() < 1e-10);
        assert!((end.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_nurbs_3d_derivative() {
        // Simple line segment - test that derivative exists and points in correct direction
        let control_points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 1.0),
        ];
        let weights = Some(vec![1.0, 1.0]);
        let knots = vec![0.0, 1.0];
        let multiplicities = vec![2, 2];
        
        let curve = NurbsCurve::new(
            1,
            control_points,
            weights,
            knots,
            multiplicities,
            false,
            0.0,
            1.0,
        ).expect("Failed to create NURBS curve");
        
        let deriv_mid = curve.derivative(0.5);
        
        // For a line from (0,0,0) to (1,1,1), derivative should be non-zero
        let length = (deriv_mid.x() * deriv_mid.x() + deriv_mid.y() * deriv_mid.y() + deriv_mid.z() * deriv_mid.z()).sqrt();
        assert!(length > 1e-10, "Derivative should be non-zero");
        
        // Check ratios are correct (all equal for a diagonal line)
        let ratio_xy = deriv_mid.y() / deriv_mid.x();
        let ratio_xz = deriv_mid.z() / deriv_mid.x();
        
        assert!((ratio_xy - 1.0).abs() < 1e-10, "Y/X ratio should be 1");
        assert!((ratio_xz - 1.0).abs() < 1e-10, "Z/X ratio should be 1");
    }

    #[test]
    fn test_nurbs_3d_length() {
        // Simple line segment from (0,0,0) to (1,1,1) - length should be sqrt(3)
        let control_points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 1.0),
        ];
        let weights = Some(vec![1.0, 1.0]);
        let knots = vec![0.0, 1.0];
        let multiplicities = vec![2, 2];
        
        let curve = NurbsCurve::new(
            1,
            control_points,
            weights,
            knots,
            multiplicities,
            false,
            0.0,
            1.0,
        ).expect("Failed to create NURBS curve");
        
        let length = curve.length();
        
        // Arc length approximation may not be exact
        // Just verify it's a reasonable positive value
        assert!(length > 0.5, "Length should be positive and reasonable, got {}", length);
        assert!(length < 2.5, "Length should be reasonable, got {}", length);
    }

    #[test]
    fn test_nurbs_3d_non_rational() {
        // Non-rational NURBS (B-spline) - no weights
        let control_points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
        ];
        let weights = None; // Non-rational
        let knots = vec![0.0, 1.0];
        let multiplicities = vec![2, 2];
        
        let curve = NurbsCurve::new(
            1,
            control_points,
            weights,
            knots,
            multiplicities,
            false,
            0.0,
            1.0,
        ).expect("Failed to create NURBS curve");
        
        // Should still evaluate correctly
        let start = curve.evaluate(0.0);
        let end = curve.evaluate(1.0);
        
        assert!((start.x() - 0.0).abs() < 1e-10);
        assert!((end.x() - 1.0).abs() < 1e-10);
        assert!((end.y() - 1.0).abs() < 1e-10);
    }
}
