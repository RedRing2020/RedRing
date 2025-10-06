//! LineSegment3D - migrated from geo_core (legacy)
#![allow(deprecated)]
use geo_core::{Scalar, Point3D};
use geo_core::vector::{Vector3D, Vector};

/// 3D線分 (geo_core から移行)
#[derive(Debug, Clone)]
#[deprecated(note = "Use f64 canonical type geo_primitives::LineSegment3D (alias of FLineSegment3)")]
pub struct LegacyLineSegment3D {
    start: Point3D,
    end: Point3D,
}
impl LegacyLineSegment3D {
    pub fn new(start: Point3D, end: Point3D) -> Self { Self { start, end } }
    pub fn start(&self) -> &Point3D { &self.start }
    pub fn end(&self) -> &Point3D { &self.end }
    pub fn direction(&self) -> Vector3D { Vector3D::new(
        self.end.x().clone()-self.start.x().clone(),
        self.end.y().clone()-self.start.y().clone(),
        self.end.z().clone()-self.start.z().clone(),
    ) }
    pub fn midpoint(&self) -> Point3D { self.start.midpoint(&self.end) }
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let direction = self.direction();
        let to_point = Vector3D::new(
            point.x().clone()-self.start.x().clone(),
            point.y().clone()-self.start.y().clone(),
            point.z().clone()-self.start.z().clone(),
        );
        let length_sq = direction.dot(&direction);
        if length_sq.value().abs() < 1e-12 { return self.start.distance_to(point); }
        let t = to_point.dot(&direction) / length_sq;
        let t_clamped = if t.value() < 0.0 { Scalar::new(0.0) } else if t.value() > 1.0 { Scalar::new(1.0) } else { t };
        let one_minus_t = Scalar::new(1.0) - t_clamped.clone();
        Point3D::new(
            one_minus_t.clone()*self.start.x().clone() + t_clamped.clone()*self.end.x().clone(),
            one_minus_t.clone()*self.start.y().clone() + t_clamped.clone()*self.end.y().clone(),
            one_minus_t*self.start.z().clone() + t_clamped*self.end.z().clone(),
        ).distance_to(point)
    }
}
