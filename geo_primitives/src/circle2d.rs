//! Circle2D - f64 ベース円プリミティブ
//!
//! 幾何解析やレンダリング前処理向けの軽量 2D 円。
//! geo_core の Point2D (Scalar 座標) を保持しつつ、半径・角度評価は f64 で行う。

use geo_core::{Point2D, Scalar};
use geo_core::angle::Angle;
use geo_core::tolerance::ToleranceContext;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};

#[derive(Debug, Clone)]
pub struct Circle2D {
    center: Point2D,
    radius: f64,
}

impl Circle2D {
    pub fn new(center: Point2D, radius: f64) -> Self {
        debug_assert!(radius >= 0.0, "radius must be non-negative");
        Self { center, radius }
    }

    pub fn center(&self) -> &Point2D { &self.center }
    pub fn radius(&self) -> f64 { self.radius }

    /// パラメータ t∈[0,1] を角度 0..2π に対応させ評価
    pub fn evaluate(&self, t: f64) -> Point2D {
        let theta = t * std::f64::consts::TAU;
        let (s,c) = theta.sin_cos();
        Point2D::from_f64(
            self.center.x().value() + self.radius * c,
            self.center.y().value() + self.radius * s,
        )
    }

    /// 1 周を 0..1 でパラメトライズしたときの dt に対する微分
    pub fn derivative(&self, t: f64) -> (f64,f64) {
        let theta = t * std::f64::consts::TAU; let (s,c)=theta.sin_cos();
        let dtheta = std::f64::consts::TAU;
        // d/dt (r(c,s)) = r * dθ * (-s, c)
        (-self.radius * s * dtheta, self.radius * c * dtheta)
    }

    pub fn length(&self) -> f64 { std::f64::consts::TAU * self.radius }

    /// 中心角 span の弧長 (Angle)
    pub fn arc_length(&self, span: Angle) -> f64 { span.radians().abs() * self.radius }

    /// 点との距離 (中心からの距離 - 半径) の絶対値
    pub fn distance_to_point(&self, p: &Point2D) -> f64 {
        let dx = p.x().value() - self.center.x().value();
        let dy = p.y().value() - self.center.y().value();
        (dx*dx + dy*dy).sqrt() - self.radius
    }

    pub fn contains_point(&self, p: &Point2D, context: &ToleranceContext) -> bool {
        self.distance_to_point(p).abs() <= context.linear
    }
}

impl GeometricPrimitive for Circle2D {
    fn primitive_kind(&self) -> PrimitiveKind { PrimitiveKind::Circle }

    fn bounding_box(&self) -> BoundingBox {
        let r = Scalar::new(self.radius);
        BoundingBox::from_2d(
            Point2D::new(self.center.x().clone() - r.clone(), self.center.y().clone() - r.clone()),
            Point2D::new(self.center.x().clone() + r.clone(), self.center.y().clone() + r),
        )
    }

    fn measure(&self) -> Option<f64> { Some(self.length()) }
}

// (moved to unit_tests/circle2d.rs)
