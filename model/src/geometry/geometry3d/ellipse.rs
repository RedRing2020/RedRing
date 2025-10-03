use crate::geometry::geometry3d::{point::Point, vector::Vector};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;

use analysis::newton_arc_length;
use analysis::EPSILON;

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
        let dot = major_axis.dot(&minor_axis);

        if dot.abs() > EPSILON {
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

    fn parameter_range(&self) -> (f64, f64) {
        (0.0, 2.0 * std::f64::consts::PI)
    }

    fn is_closed(&self) -> bool {
        true
    }
}

impl Curve3D for Ellipse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn evaluate(&self, t: f64) -> Point {
        let angle = t * 2.0 * std::f64::consts::PI;
        let x = self.major_radius * angle.cos();
        let y = self.minor_radius * angle.sin();
        self.center + self.major_axis * x + self.minor_axis * y
    }
    fn derivative(&self, t: f64) -> Vector {
        let angle = t * 2.0 * std::f64::consts::PI;
        let dx = -self.major_radius * angle.sin() * 2.0 * std::f64::consts::PI;
        let dy =  self.minor_radius * angle.cos() * 2.0 * std::f64::consts::PI;
        self.major_axis * dx + self.minor_axis * dy
    }
    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Ellipse
    }

    /// 楕円の周長（数値積分による近似）
    fn length(&self) -> f64 {
        let a = self.major_radius;
        let b = self.minor_radius;
        let major = self.major_axis;
        let minor = self.minor_axis;
        let steps = 360;

        // 速度ベクトル関数
        let evaluate = |theta: f64| {
            let dx = -a * theta.sin();
            let dy =  b * theta.cos();
            major * dx + minor * dy
        };

        newton_arc_length(evaluate, 0.0, 2.0 * std::f64::consts::PI, steps)
    }
}
