use std::any::Any;
use crate::geometry::geometry3d::{point::Point, vector::Vector, direction::Direction};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    center: Point,
    radius: f64,
    normal: Direction,
}

impl Circle {
    pub fn new(center: Point, radius: f64, normal: Direction) -> Self {
        Self { center, radius, normal }
    }

    /// 中心点を取得
    pub fn center(&self) -> Point {
        self.center.clone()
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> Direction {
        self.normal.clone()
    }

    fn parameter_range(&self) -> (f64, f64) {
        (0.0, 2.0 * std::f64::consts::PI)
    }

    fn is_closed(&self) -> bool {
        true
    }
}

impl Curve3D for Circle {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Circle
    }

    fn evaluate(&self, t: f64) -> Point {
        let theta = t * 2.0 * std::f64::consts::PI;
        let (u_vec, v_vec) = self.normal.orthonormal_basis();

        // パラメトリック円の評価
        let u = self.radius * theta.cos();
        let v = self.radius * theta.sin();

        self.center.clone() + u_vec * u + v_vec * v
    }

    fn derivative(&self, t: f64) -> Vector {
        let theta = t * 2.0 * std::f64::consts::PI;
        let (u, v) = self.normal.orthonormal_basis();
        let two_pi = 2.0 * std::f64::consts::PI;
        let dx = (-self.radius * theta.sin() * two_pi);
        let dy = (self.radius * theta.cos() * two_pi);
        u * dx + v * dy
    }

    fn length(&self) -> f64 {
        (2.0 * std::f64::consts::PI) * self.radius
    }

    fn parameter_hint(&self, pt: &Point) -> f64 {
        // 円周上の点へのパラメータ初期値推定
        let (u, v) = self.normal.orthonormal_basis();
        let rel = pt.clone() - self.center.clone();
        let x = rel.dot(&u);
        let y = rel.dot(&v);
        let theta = y.atan2(x);
        // [0, 2π] → [0, 1]
        (theta.rem_euclid(2.0 * std::f64::consts::PI)) / (2.0 * std::f64::consts::PI)
    }



    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}
