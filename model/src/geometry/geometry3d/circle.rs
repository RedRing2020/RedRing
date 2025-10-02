use std::any::Any;
use crate::geometry::geometry3d::{point::Point, vector::Vector, direction::Direction};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
    pub normal: Direction,
}

impl Circle {
    pub fn new(center: Point, radius: f64, normal: Direction) -> Self {
        Self { center, radius, normal }
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
        // t: [0, 1] → θ: [0, 2π]
        let theta = t * 2.0 * std::f64::consts::PI;
        // 3D円の座標計算
        // normalから直交基底(u, v)を生成
        let (u, v) = self.normal.orthonormal_basis();
        let x = self.radius * theta.cos();
        let y = self.radius * theta.sin();
        self.center + u * x + v * y
    }

    fn derivative(&self, t: f64) -> Vector {
        let theta = t * 2.0 * std::f64::consts::PI;
        let (u, v) = self.normal.orthonormal_basis();
        let dx = -self.radius * theta.sin() * 2.0 * std::f64::consts::PI;
        let dy =  self.radius * theta.cos() * 2.0 * std::f64::consts::PI;
        u * dx + v * dy
    }

    fn length(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    fn parameter_hint(&self, pt: &Point) -> f64 {
        // 円周上の点へのパラメータ初期値推定
        let (u, v) = self.normal.orthonormal_basis();
        let rel = *pt - self.center;
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
