use std::any::Any;

use super::{Point3D, Vector3D};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;

/// Represents a trimmed NURBS curve in 3D space.
/// Designed for internal use in RedRing, not strictly STEP-compliant.
#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve {
    degree: usize,                     // Degree of the curve
    control_points: Vec<Point3D>,      // Control points
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
        control_points: Vec<Point3D>,
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

    pub fn control_points(&self) -> &[Point3D] {
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
    fn evaluate(&self, _t: f64) -> Point3D {
        todo!("Implement NURBS evaluation")
    }
    fn derivative(&self, _t: f64) -> Vector3D {
        todo!("Implement NURBS derivative")
    }
    fn kind(&self) -> CurveKind3D {
        CurveKind3D::NurbsCurve
    }
    fn length(&self) -> f64 {
        todo!("Implement NURBS length")
    }
}
