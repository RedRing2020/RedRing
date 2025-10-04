use crate::geometry::geometry3d::{point::Point, vector::Vector};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use geo_core::Scalar;

use analysis::newton_arc_length;
use analysis::EPSILON;

#[derive(Debug, Clone)]
pub struct Ellipse {
    center: Point,
    major_axis: Vector,
    minor_axis: Vector,
    major_radius: Scalar,
    minor_radius: Scalar,
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
            major_radius: Scalar::new(major_radius),
            minor_radius: Scalar::new(minor_radius),
        })
    }

    /// 中心点を取得
    pub fn center(&self) -> Point {
        self.center.clone()
    }

    /// 長軸ベクトルを取得
    pub fn major_axis(&self) -> Vector {
        self.major_axis.clone()
    }

    /// 短軸ベクトルを取得
    pub fn minor_axis(&self) -> Vector {
        self.minor_axis.clone()
    }

    /// 長軸半径を取得
    pub fn major_radius(&self) -> f64 {
        self.major_radius.value()
    }

    /// 短軸半径を取得
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius.value()
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
        let theta = t * 2.0 * std::f64::consts::PI;
        let x = theta.cos();
        let y = theta.sin();

        self.center.clone() + self.major_axis.clone() * x + self.minor_axis.clone() * y
    }
    fn derivative(&self, t: f64) -> Vector {
        let angle = t * 2.0 * std::f64::consts::PI;
        let two_pi = Scalar::new(2.0 * std::f64::consts::PI);
        let dx = (-self.major_radius * Scalar::new(angle.sin()) * two_pi).value();
        let dy = (self.minor_radius * Scalar::new(angle.cos()) * two_pi).value();
        self.major_axis.clone() * dx + self.minor_axis.clone() * dy
    }
    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Ellipse
    }

    /// 楕円の周長（数値積分による近似）
    fn length(&self) -> f64 {
        let major = self.major_axis.clone();
        let minor = self.minor_axis.clone();
        let steps = 360;

        // 速度ベクトル関数
        let evaluate = |theta: f64| {
            let dx = -theta.sin();
            let dy = theta.cos();
            major.clone() * dx + minor.clone() * dy
        };

        newton_arc_length(evaluate, 0.0, 2.0 * std::f64::consts::PI, steps)
    }
}
