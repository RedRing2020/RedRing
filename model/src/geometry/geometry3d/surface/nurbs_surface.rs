use super::super::point::Point;
use super::super::vector::Vector;
use super::surface::Surface;
use super::kind::SurfaceKind;

/// Represents a NURBS surface with optional weights.
/// Control points are stored in a flattened 1D array (row-major: u + v * u_count).
#[derive(Debug, Clone)]
pub struct NurbsSurface {
    pub control_points: Vec<Point>,
    pub weights: Option<Vec<f64>>,
    pub u_count: usize,
    pub v_count: usize,
    pub u_knots: Vec<f64>,
    pub v_knots: Vec<f64>,
    pub u_multiplicities: Vec<usize>,
    pub v_multiplicities: Vec<usize>,
    pub u_degree: usize,
    pub v_degree: usize,
    pub is_uniform_u: bool,
    pub is_uniform_v: bool,
}

impl Surface for NurbsSurface {
    fn kind(&self) -> SurfaceKind {
        SurfaceKind::NurbsSurface
    }
}

impl NurbsSurface {
    /// Constructs a new NurbsSurface, validating input dimensions.
    pub fn try_new(
        control_points: Vec<Point>,
        weights: Option<Vec<f64>>,
        u_count: usize,
        v_count: usize,
        u_knots: Vec<f64>,
        v_knots: Vec<f64>,
        u_multiplicities: Vec<usize>,
        v_multiplicities: Vec<usize>,
        u_degree: usize,
        v_degree: usize,
        is_uniform_u: bool,
        is_uniform_v: bool,
    ) -> Option<Self> {
        let expected_len = u_count * v_count;

        if control_points.len() != expected_len {
            return None;
        }

        if let Some(w) = &weights {
            if w.len() != expected_len {
                return None;
            }
        }

        Some(Self {
            control_points,
            weights,
            u_count,
            v_count,
            u_knots,
            v_knots,
            u_multiplicities,
            v_multiplicities,
            u_degree,
            v_degree,
            is_uniform_u,
            is_uniform_v,
        })
    }

    /// Returns control point at (u_index, v_index)
    pub fn control_point_at(&self, u: usize, v: usize) -> &Point {
        &self.control_points[u + v * self.u_count]
    }

    /// Returns weight at (u_index, v_index), if rational
    pub fn weight_at(&self, u: usize, v: usize) -> Option<f64> {
        self.weights.as_ref().map(|w| w[u + v * self.u_count])
    }

    /// Returns true if surface is rational
    pub fn is_rational(&self) -> bool {
        self.weights.is_some()
    }

    /// Returns total number of control points
    pub fn total_points(&self) -> usize {
        self.u_count * self.v_count
    }
}

impl Surface for NurbsSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point {
        // 実装は後で：双方向のBスプライン基底関数による評価
        todo!("NURBS surface evaluation not yet implemented")
    }

    fn normal(&self, u: f64, v: f64) -> Vector {
        // 実装は後で：偏微分ベクトルの外積
        todo!("NURBS surface normal computation not yet implemented")
    }

    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        (
            (*self.u_knots.first().unwrap(), *self.u_knots.last().unwrap()),
            (*self.v_knots.first().unwrap(), *self.v_knots.last().unwrap()),
        )
    }

    fn is_closed_u(&self) -> bool {
        false // 拡張可能
    }

    fn is_closed_v(&self) -> bool {
        false // 拡張可能
    }

    fn kind(&self) -> SurfaceKind {
        SurfaceKind::NurbsSurface
    }
}