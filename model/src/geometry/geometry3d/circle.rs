use std::any::Any;
use crate::geometry::geometry3d::{point::Point, vector::Vector, direction::Direction};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use geo_core::Scalar;

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    center: Point,
    radius: Scalar,
    normal: Direction,
}

impl Circle {
    pub fn new(center: Point, radius: f64, normal: Direction) -> Self {
        Self { center, radius: Scalar::new(radius), normal }
    }

    /// 中心点を取得
    pub fn center(&self) -> Point {
        self.center.clone()
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 {
        self.radius.value()
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

    fn evaluate(&self, t: f64) -> geo_core::Point3D {
        let theta = t * 2.0 * std::f64::consts::PI;
        let (u_vec, v_vec) = self.normal.orthonormal_basis();

        // パラメトリック円の評価
        let u = (self.radius * Scalar::new(theta.cos())).value();
        let v = (self.radius * Scalar::new(theta.sin())).value();

        let result = self.center.clone() + u_vec * u + v_vec * v;
        result.as_geo_core().clone()
    }

    fn derivative(&self, t: f64) -> geo_core::Vector3D {
        let theta = t * 2.0 * std::f64::consts::PI;
        let (u, v) = self.normal.orthonormal_basis();
        let two_pi = Scalar::new(2.0 * std::f64::consts::PI);
        let dx = (-self.radius * Scalar::new(theta.sin()) * two_pi).value();
        let dy = (self.radius * Scalar::new(theta.cos()) * two_pi).value();
        let result = u * dx + v * dy;
        result.as_geo_core().clone()
    }

    fn length(&self) -> f64 {
        (Scalar::new(2.0 * std::f64::consts::PI) * self.radius).value()
    }

    fn parameter_hint(&self, pt: &geo_core::Point3D) -> f64 {
        // 円周上の点へのパラメータ初期値推定
        let pt_wrapped = Point::from_geo_core(pt.clone());
        let (u, v) = self.normal.orthonormal_basis();
        let rel = pt_wrapped.clone() - self.center.clone();
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
