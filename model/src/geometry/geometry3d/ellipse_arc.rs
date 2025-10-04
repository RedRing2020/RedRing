use analysis::newton_arc_length;
use crate::geometry::geometry3d::{Point3D, Vector3D, Direction3D};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use geo_core::Scalar;
use analysis::EPSILON;

#[derive(Debug, Clone)]
pub struct EllipseArc {
    center: Point3D,
    major_axis: Vector3D,
    minor_axis: Vector3D,
    major_radius: Scalar,
    minor_radius: Scalar,
    start_angle: f64, // in radians
    end_angle: f64,   // in radians
}

impl EllipseArc {
    pub fn new(
        center: Point3D,
        major_axis: Vector3D,
        minor_axis: Vector3D,
        major_radius: f64,
        minor_radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Option<Self> {
        if major_axis.dot(&minor_axis).abs() > EPSILON {
            return None; // 軸が直交していない
        }
        if end_angle <= start_angle {
            return None; // 角度範囲が不正
        }
        Some(Self {
            center,
            major_axis,
            minor_axis,
            major_radius: Scalar::new(major_radius),
            minor_radius: Scalar::new(minor_radius),
            start_angle,
            end_angle,
        })
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D {
        self.center.clone()
    }

    /// 長軸ベクトルを取得
    pub fn major_axis(&self) -> Vector3D {
        self.major_axis.clone()
    }

    /// 短軸ベクトルを取得
    pub fn minor_axis(&self) -> Vector3D {
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

    /// 開始角度を取得（ラジアン）
    pub fn start_angle(&self) -> f64 {
        self.start_angle
    }

    /// 終了角度を取得（ラジアン）
    pub fn end_angle(&self) -> f64 {
        self.end_angle
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

    fn evaluate(&self, t: f64) -> Point3D {
        let theta = self.start_angle + t * (self.end_angle - self.start_angle);
        let x = theta.cos();
        let y = theta.sin();

        self.center.clone() + self.major_axis.clone() * x + self.minor_axis.clone() * y
    }

    fn derivative(&self, t: f64) -> Vector3D {
        let angle = self.start_angle + t * (self.end_angle - self.start_angle);
        let d_angle = self.end_angle - self.start_angle;

        let dx = (-self.major_radius * Scalar::new(angle.sin()) * Scalar::new(d_angle)).value();
        let dy = (self.minor_radius * Scalar::new(angle.cos()) * Scalar::new(d_angle)).value();
        self.major_axis.clone() * dx + self.minor_axis.clone() * dy
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::EllipseArc
    }

    /// 3D楕円弧の長さを数値積分で近似
    fn length(&self) -> f64 {
        let major = self.major_axis.clone();
        let minor = self.minor_axis.clone();
        let start = self.start_angle;
        let end = self.end_angle;
        let steps = 360; // 内部変数として分割数を固定

        // 速度ベクトル関数
        let evaluate = |theta: f64| {
            let dx = -theta.sin();
            let dy = theta.cos();
            major.clone() * dx + minor.clone() * dy
        };

        newton_arc_length(evaluate, start, end, steps)
    }
}
