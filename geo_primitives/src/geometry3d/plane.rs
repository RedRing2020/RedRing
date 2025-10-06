//! Plane - migrated from geo_core (legacy)
#![allow(deprecated)]
use geo_core::{Scalar, Point3D};
use geo_core::tolerance::ToleranceContext;
use geo_core::vector::{Vector3D, Vector};
use crate::geometry3d::direction3d::LegacyDirection3D;

#[derive(Debug, Clone)]
#[deprecated(note = "Use f64 canonical type geo_primitives::Plane (alias of FPlane)")]
pub struct LegacyPlane {
    origin: Point3D,
    u_axis: Vector3D,
    v_axis: Vector3D,
    normal: LegacyDirection3D,
}
impl LegacyPlane {
    pub fn new(origin: Point3D, u_axis: Vector3D, v_axis: Vector3D) -> Option<Self> {
        let context = ToleranceContext::standard();
    let normal = LegacyDirection3D::from_vector(u_axis.cross(&v_axis), &context)?;
        Some(Self { origin, u_axis, v_axis, normal })
    }
    pub fn from_point_and_normal(origin: Point3D, normal: LegacyDirection3D) -> Self {
        let context = ToleranceContext::standard();
        let (u_axis, v_axis) = normal.orthonormal_basis(&context);
        Self { origin, u_axis, v_axis, normal }
    }
    pub fn from_three_points(p1:&Point3D,p2:&Point3D,p3:&Point3D)->Option<Self>{
        let u_axis = Vector3D::new(
            p2.x().clone()-p1.x().clone(),
            p2.y().clone()-p1.y().clone(),
            p2.z().clone()-p1.z().clone(),
        );
        let v_axis = Vector3D::new(
            p3.x().clone()-p1.x().clone(),
            p3.y().clone()-p1.y().clone(),
            p3.z().clone()-p1.z().clone(),
        );
        Self::new(p1.clone(), u_axis, v_axis)
    }
    pub fn origin(&self)->&Point3D { &self.origin }
    pub fn normal(&self)->&LegacyDirection3D { &self.normal }
    pub fn u_axis(&self)->&Vector3D { &self.u_axis }
    pub fn v_axis(&self)->&Vector3D { &self.v_axis }
    pub fn distance_to_point(&self, point:&Point3D)->Scalar {
        let to_point = Vector3D::new(
            point.x().clone()-self.origin.x().clone(),
            point.y().clone()-self.origin.y().clone(),
            point.z().clone()-self.origin.z().clone(),
        );
        to_point.dot(self.normal.as_vector()).abs()
    }
    pub fn project_point(&self, point:&Point3D)->Point3D {
        let to_point = Vector3D::new(
            point.x().clone()-self.origin.x().clone(),
            point.y().clone()-self.origin.y().clone(),
            point.z().clone()-self.origin.z().clone(),
        );
        let distance = to_point.dot(self.normal.as_vector());
    Point3D::new(
            point.x().clone() - distance.clone()*Scalar::new(self.normal.x()),
            point.y().clone() - distance.clone()*Scalar::new(self.normal.y()),
            point.z().clone() - distance*Scalar::new(self.normal.z()),
        )
    }
}
