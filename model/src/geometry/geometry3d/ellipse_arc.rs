use crate::analysis::numeric::newton_arc_length;
use crate::geometry::geometry3d::{point::Point, vector::Vector, direction::Direction};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;

#[derive(Debug, Clone)]
pub struct EllipseArc {
    pub center: Point,
    pub major_axis: Vector,
    pub minor_axis: Vector,
    pub major_radius: f64,
    pub minor_radius: f64,
    pub start_angle: f64, // in radians
    pub end_angle: f64,   // in radians
}

impl EllipseArc {
    pub fn new(
        center: Point,
        major_axis: Vector,
        minor_axis: Vector,
        major_radius: f64,
        minor_radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Option<Self> {
        let epsilon = 1e-10;
        if major_axis.dot(&minor_axis).abs() > epsilon {
            return None; // 軸が直交していない
        }
        if end_angle <= start_angle {
            return None; // 角度範囲が不正
        }
        Some(Self {
            center,
            major_axis,
            minor_axis,
            major_radius,
            minor_radius,
            start_angle,
            end_angle,
        })
    }

    fn parameter_range(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    fn is_closed(&self) -> bool {
        false
    }
}

impl Curve3D for EllipseArc {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn evaluate(&self, t: f64) -> Point {
        let angle = self.start_angle + (self.end_angle - self.start_angle) * t;
        let x = self.major_radius * angle.cos();
        let y = self.minor_radius * angle.sin();
        self.center + self.major_axis * x + self.minor_axis * y
    }

    fn derivative(&self, t: f64) -> Vector {
        let angle = self.start_angle + (self.end_angle - self.start_angle) * t;
        let d_angle = self.end_angle - self.start_angle;
        let dx = -self.major_radius * angle.sin() * d_angle;
        let dy = self.minor_radius * angle.cos() * d_angle;
        self.major_axis * dx + self.minor_axis * dy
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::EllipseArc
    }

    /// 3D楕円弧の長さを数値積分で近似
    fn length(&self) -> f64 {
        let major = self.major_axis;
        let minor = self.minor_axis;
        let a = self.major_radius;
        let b = self.minor_radius;
        let start = self.start_angle;
        let end = self.end_angle;
        let steps = 360; // 内部変数として分割数を固定

        // 速度ベクトル関数
        let evaluate = |theta: f64| {
            let dx = -a * theta.sin();
            let dy =  b * theta.cos();
            major * dx + minor * dy
        };

        newton_arc_length(evaluate, start, end, steps)
    }
}
