//! Circle3D - migrated from geo_core (legacy)
#![allow(deprecated)]
use geo_core::{Scalar, Point3D};
use geo_core::tolerance::ToleranceContext;
use geo_core::vector::Vector3D;
use crate::geometry3d::direction3d::LegacyDirection3D;

#[derive(Debug, Clone)]
#[deprecated(note = "Use f64 canonical type geo_primitives::Circle3D (alias of FCircle3)")]
pub struct LegacyCircle3D {
    center: Point3D,
    radius: Scalar,
    normal: LegacyDirection3D,
    basis_u: Vector3D,
    basis_v: Vector3D,
}
impl LegacyCircle3D {
    pub fn new(center: Point3D, radius: Scalar, normal: LegacyDirection3D) -> Self {
        let ctx = ToleranceContext::standard();
        let (u, v) = normal.orthonormal_basis(&ctx);
        Self { center, radius, normal, basis_u: u, basis_v: v }
    }
    pub fn center(&self)->&Point3D { &self.center }
    pub fn radius(&self)->&Scalar { &self.radius }
    pub fn normal(&self)->&LegacyDirection3D { &self.normal }
    pub fn evaluate(&self, t: Scalar) -> Point3D {
        let tau = Scalar::new(std::f64::consts::TAU);
        let theta = t * tau;
        let (s, c) = (theta.sin(), theta.cos());
        Point3D::new(
            self.center.x().clone() + self.radius.clone()*c.clone()*self.basis_u.x().clone() + self.radius.clone()*s.clone()*self.basis_v.x().clone(),
            self.center.y().clone() + self.radius.clone()*c.clone()*self.basis_u.y().clone() + self.radius.clone()*s.clone()*self.basis_v.y().clone(),
            self.center.z().clone() + self.radius.clone()*c.clone()*self.basis_u.z().clone() + self.radius.clone()*s*self.basis_v.z().clone(),
        )
    }
}
