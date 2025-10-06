use super::{point3::FPoint3, vector3::FVector3};
use geo_core::tolerance::{ToleranceContext, TolerantEq};

#[derive(Debug, Clone, Copy)]
pub struct FLineSegment3 { pub start: FPoint3, pub end: FPoint3 }
impl FLineSegment3 {
    pub fn new(start:FPoint3,end:FPoint3)->Self{ Self{start,end} }
    pub fn length(&self)->f64 { self.start.distance(&self.end) }
    pub fn direction(&self)->Option<FVector3>{ (self.end - self.start).normalize() }
    pub fn midpoint(&self)->FPoint3 { self.start.midpoint(&self.end) }
    pub fn vector(&self)->FVector3 { self.end - self.start }
}
impl TolerantEq for FLineSegment3 { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { self.start.tolerant_eq(&other.start,ctx) && self.end.tolerant_eq(&other.end,ctx) }}
