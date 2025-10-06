use super::{point3::FPoint3, vector3::FVector3, direction3::FDirection3};
use geo_core::tolerance::{ToleranceContext, TolerantEq};

#[derive(Debug, Clone, Copy)]
pub struct FPlane { origin: FPoint3, normal: FDirection3 }
impl FPlane {
    pub fn new(origin:FPoint3, normal:FDirection3)->Self{ Self{origin,normal} }
    pub fn origin(&self)->FPoint3 { self.origin }
    pub fn normal(&self)->FDirection3 { self.normal }
    pub fn distance_to_point(&self, p:&FPoint3)->f64 { let v = *p - self.origin; v.dot(&self.normal.as_vector()) }
    pub fn project_point(&self, p:&FPoint3)->FPoint3 { let d = self.distance_to_point(p); *p - self.normal.as_vector()*d }
    pub fn translate(&self, v:FVector3)->Self { Self { origin: self.origin + v, normal: self.normal } }
}
impl TolerantEq for FPlane { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { self.origin.tolerant_eq(&other.origin,ctx) && self.normal.tolerant_eq(&other.normal,ctx) }}
