//! Direction3D - migrated from geo_core (legacy normalized 3D direction)
#![allow(deprecated)]
use geo_core::tolerance::{ToleranceContext, ToleranceProvider, TolerantEq};
use geo_core::vector::{Vector3D, Vector};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
#[deprecated(note = "Use f64 canonical type geo_primitives::Direction3D (alias of FDirection3)")]
pub struct LegacyDirection3D { vector: Vector3D }
impl LegacyDirection3D {
    pub fn from_vector(v: Vector3D, context: &ToleranceContext) -> Option<Self> {
        if let Some(n) = v.normalize(context) { Some(Self { vector: n }) } else { None }
    }
    pub fn new(x: f64, y: f64, z: f64, context: &ToleranceContext) -> Option<Self> {
        Self::from_vector(Vector3D::from_f64(x,y,z), context)
    }
    pub fn as_vector(&self) -> &Vector3D { &self.vector }
    pub fn to_vector(&self) -> Vector3D { self.vector.clone() }
    pub fn x(&self)->f64 { self.vector.x().value() }
    pub fn y(&self)->f64 { self.vector.y().value() }
    pub fn z(&self)->f64 { self.vector.z().value() }
    pub fn dot(&self, other:&Self)->f64 { self.vector.dot(&other.vector).value() }
    pub fn cross(&self, other:&Self, context:&ToleranceContext)->Option<Self> { Self::from_vector(self.vector.cross(&other.vector), context) }
    pub fn negate(&self)->Self { Self { vector: -self.vector.clone() } }
    #[inline] pub fn unit_x(context:&ToleranceContext)->Self { Self::from_vector(Vector3D::from_f64(1.0,0.0,0.0), context).unwrap() }
    #[inline] pub fn unit_y(context:&ToleranceContext)->Self { Self::from_vector(Vector3D::from_f64(0.0,1.0,0.0), context).unwrap() }
    #[inline] pub fn unit_z(context:&ToleranceContext)->Self { Self::from_vector(Vector3D::from_f64(0.0,0.0,1.0), context).unwrap() }
    pub fn orthonormal_basis(&self, context:&ToleranceContext)->(Vector3D, Vector3D){
        let up = if self.z().abs() < 0.99 { Vector3D::z_axis() } else { Vector3D::x_axis() };
        let u = self.vector.cross(&up).normalize(context).unwrap();
        let v = self.vector.cross(&u);
        (u, v)
    }
}
impl fmt::Display for LegacyDirection3D { fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f, "LegacyDirection3D({}, {}, {})", self.x(), self.y(), self.z()) } }
impl ToleranceProvider for LegacyDirection3D { fn tolerance_context(&self)->&ToleranceContext { self.vector.tolerance_context() } fn set_tolerance_context(&mut self, ctx:ToleranceContext){ self.vector.set_tolerance_context(ctx)} }
impl TolerantEq for LegacyDirection3D { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { (self.x()-other.x()).abs() <= ctx.linear && (self.y()-other.y()).abs()<=ctx.linear && (self.z()-other.z()).abs()<=ctx.linear }}
