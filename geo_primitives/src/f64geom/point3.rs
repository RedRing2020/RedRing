use geo_core::tolerance::{ToleranceContext, TolerantEq};
use super::vector3::FVector3;

#[derive(Debug, Clone, Copy)]
pub struct FPoint3 { data:[f64;3] }
impl FPoint3 {
    pub fn new(x:f64,y:f64,z:f64)->Self { Self{data:[x,y,z]} }
    pub fn origin()->Self { Self::new(0.0,0.0,0.0) }
    pub fn x(&self)->f64 { self.data[0] }
    pub fn y(&self)->f64 { self.data[1] }
    pub fn z(&self)->f64 { self.data[2] }
    pub fn to_vector(&self)->FVector3 { FVector3::new(self.x(), self.y(), self.z()) }
    pub fn distance_to(&self, other:&Self)->f64 { ((self.x()-other.x()).powi(2) + (self.y()-other.y()).powi(2) + (self.z()-other.z()).powi(2)).sqrt() }
    pub fn distance(&self, other:&Self)->f64 { self.distance_to(other) }
    pub fn midpoint(&self, other:&Self)->Self { Self::new((self.x()+other.x())*0.5,(self.y()+other.y())*0.5,(self.z()+other.z())*0.5) }
}
impl TolerantEq for FPoint3 { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { self.distance_to(other) <= ctx.linear }}

// Point +/- Vector
impl std::ops::Add<FVector3> for FPoint3 { type Output=FPoint3; fn add(self,v:FVector3)->FPoint3 { FPoint3::new(self.x()+v.x(), self.y()+v.y(), self.z()+v.z()) } }
impl std::ops::Sub<FVector3> for FPoint3 { type Output=FPoint3; fn sub(self,v:FVector3)->FPoint3 { FPoint3::new(self.x()-v.x(), self.y()-v.y(), self.z()-v.z()) } }
// Point - Point = Vector
impl std::ops::Sub for FPoint3 { type Output=FVector3; fn sub(self,other:FPoint3)->FVector3 { FVector3::new(self.x()-other.x(), self.y()-other.y(), self.z()-other.z()) } }
