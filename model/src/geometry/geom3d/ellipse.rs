use crate::geom3::{point::Point, vector::Vector};
use crate::geom3::kind::CurveKind3;
use crate::geom3::curve::curve_trait::Curve3;

#[derive(Debug, Clone)]
pub struct Ellipse {
    pub center: Point,
    pub major_axis: Vector,
    pub minor_axis: Vector,
    pub major_radius: f64,
    pub minor_radius: f64,
}

impl Ellipse {
    /// Constructs a new Ellipse, validating that major and minor axes are orthogonal.
    pub fn new(
        center: Point,
        major_axis: Vector,
        minor_axis: Vector,
        major_radius: f64,
        minor_radius: f64,
    ) -> Option<Self> {
        let epsilon = 1e-10;
        let dot = major_axis.dot(minor_axis);

        if dot.abs() > epsilon {
            return None; // Axes are not orthogonal
        }

        Some(Self {
            center,
            major_axis,
            minor_axis,
            major_radius,
            minor_radius,
        })
    }
}

impl Curve3 for Ellipse {
    fn evaluate(&self, t: f64) -> Point {
        let x = self.major_radius * t.cos();
        let y = self.minor_radius * t.sin();
        self.center + self.major_axis * x + self.minor_axis * y
    }

    fn derivative(&self, t: f64) -> Vector {
        let dx = -self.major_radius * t.sin();
        let dy = self.minor_radius * t.cos();
        self.major_axis * dx + self.minor_axis * dy
    }

    fn parameter_range(&self) -> (f64, f64) {
        (0.0, 2.0 * std::f64::consts::PI)
    }

    fn is_closed(&self) -> bool {
        true
    }

    fn kind(&self) -> CurveKind3 {
        CurveKind3::Ellipse
    }
}