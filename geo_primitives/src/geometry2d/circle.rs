//! Circle2D - f64 ベース円プリミティブ (renamed from circle2d.rs)
//!
//! 旧ファイル名: `circle2d.rs`

use geo_core::Point2D;
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
    pub fn evaluate(&self, t: f64) -> Point2D {
        let theta = t * std::f64::consts::TAU; let (s,c)=theta.sin_cos();
        Point2D::new(self.center.x() + self.radius * c, self.center.y() + self.radius * s)
    }
    pub fn derivative(&self, t: f64) -> (f64,f64) {
        let theta = t * std::f64::consts::TAU; let (s,c)=theta.sin_cos(); let dtheta=std::f64::consts::TAU; (-self.radius * s * dtheta, self.radius * c * dtheta)
    }
    pub fn length(&self) -> f64 { std::f64::consts::TAU * self.radius }
    pub fn arc_length(&self, span: Angle) -> f64 { span.radians().abs() * self.radius }
    pub fn distance_to_point(&self, p:&Point2D) -> f64 { let dx=p.x()-self.center.x(); let dy=p.y()-self.center.y(); (dx*dx+dy*dy).sqrt()-self.radius }
    pub fn contains_point(&self, p:&Point2D, ctx:&ToleranceContext) -> bool { self.distance_to_point(p).abs() <= ctx.linear }
}

impl GeometricPrimitive for Circle2D {
    fn primitive_kind(&self) -> PrimitiveKind { PrimitiveKind::Circle }
    fn bounding_box(&self) -> BoundingBox {
        let cx = self.center.x();
        let cy = self.center.y();
        let r = self.radius;
        BoundingBox::from_2d(
            Point2D::new(cx - r, cy - r),
            Point2D::new(cx + r, cy + r),
        )
    }
    fn measure(&self) -> Option<f64> { Some(self.length()) }
}
