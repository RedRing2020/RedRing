use super::{point3::FPoint3, vector3::FVector3, direction3::FDirection3, plane::FPlane};
use geo_core::tolerance::{ToleranceContext, TolerantEq};

#[derive(Debug, Clone, Copy)]
pub struct FCircle3 { center: FPoint3, normal: FDirection3, radius: f64 }
impl FCircle3 {
    pub fn new(center:FPoint3, normal:FDirection3, radius:f64)->Option<Self>{ if radius>=0.0 { Some(Self{center,normal,radius}) } else { None } }
    pub fn center(&self)->FPoint3 { self.center }
    pub fn normal(&self)->FDirection3 { self.normal }
    pub fn radius(&self)->f64 { self.radius }
    pub fn plane(&self)->FPlane { FPlane::new(self.center, self.normal) }
    pub fn point_at(&self, t:f64)->FPoint3 {
        let (s,c)=t.sin_cos();
        let (u,v)=self.normal.orthonormal_basis();
        // Convention: t=0 -> center + v*radius, t=pi/2 -> center + u*radius
        // (swap to match right-handed basis expectation in tests)
        self.center + (u*s + v*c)*self.radius
    }
}
impl TolerantEq for FCircle3 { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { self.center.tolerant_eq(&other.center,ctx) && self.normal.tolerant_eq(&other.normal,ctx) && (self.radius-other.radius).abs()<=ctx.linear }}
