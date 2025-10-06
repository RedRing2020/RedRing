use geo_core::tolerance::{ToleranceContext, TolerantEq};
use super::vector3::FVector3;

#[derive(Debug, Clone, Copy)]
pub struct FDirection3 { v: FVector3 }
impl FDirection3 {
    pub fn from_vector(v:FVector3)->Option<Self>{ v.normalize().map(|n|Self{v:n}) }
    pub fn new(x:f64,y:f64,z:f64)->Option<Self>{ Self::from_vector(FVector3::new(x,y,z)) }
    pub fn x(&self)->f64 { self.v.x() }
    pub fn y(&self)->f64 { self.v.y() }
    pub fn z(&self)->f64 { self.v.z() }
    pub fn as_vector(&self)->FVector3 { self.v }
    pub fn orthonormal_basis(&self)->(FVector3,FVector3){
        let up = if self.z().abs() < 0.99 { FVector3::new(0.0,0.0,1.0)} else { FVector3::new(1.0,0.0,0.0)};
        let u = self.v.cross(&up).normalize().unwrap_or(FVector3::new(1.0,0.0,0.0));
        let v = self.v.cross(&u);
        (u,v)
    }
}
impl TolerantEq for FDirection3 { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { (self.x()-other.x()).abs()<=ctx.linear && (self.y()-other.y()).abs()<=ctx.linear && (self.z()-other.z()).abs()<=ctx.linear }}
