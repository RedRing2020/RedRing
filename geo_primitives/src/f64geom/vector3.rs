use geo_core::tolerance::{ToleranceContext, TolerantEq};

#[derive(Debug, Clone, Copy)]
pub struct FVector3 {
    data: [f64;3],
    tol: ToleranceContext,
}
impl FVector3 {
    pub fn new(x:f64,y:f64,z:f64)->Self { Self { data:[x,y,z], tol: ToleranceContext::default() } }
    pub fn x(&self)->f64 { self.data[0] }
    pub fn y(&self)->f64 { self.data[1] }
    pub fn z(&self)->f64 { self.data[2] }
    pub fn set_tol(mut self, ctx:ToleranceContext)->Self { self.tol = ctx; self }
    pub fn dot(&self, other:&Self)->f64 { self.x()*other.x() + self.y()*other.y() + self.z()*other.z() }
    pub fn cross(&self, other:&Self)->Self { Self::new(
        self.y()*other.z()-self.z()*other.y(),
        self.z()*other.x()-self.x()*other.z(),
        self.x()*other.y()-self.y()*other.x(),
    ) }
    pub fn norm(&self)->f64 { self.dot(self).sqrt() }
    pub fn normalize(&self)->Option<Self> { let n=self.norm(); if n==0.0 { None } else { Some(Self::new(self.x()/n,self.y()/n,self.z()/n)) } }
    pub fn scaled(&self, s:f64)->Self { Self::new(self.x()*s, self.y()*s, self.z()*s) }
    pub fn add_vec(&self, o:&Self)->Self { Self::new(self.x()+o.x(), self.y()+o.y(), self.z()+o.z()) }
    pub fn sub_vec(&self, o:&Self)->Self { Self::new(self.x()-o.x(), self.y()-o.y(), self.z()-o.z()) }
}
impl TolerantEq for FVector3 { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { (self.x()-other.x()).abs()<=ctx.linear && (self.y()-other.y()).abs()<=ctx.linear && (self.z()-other.z()).abs()<=ctx.linear }}
impl std::ops::Add for FVector3 { type Output=Self; fn add(self,rhs:Self)->Self{ self.add_vec(&rhs) } }
impl std::ops::Sub for FVector3 { type Output=Self; fn sub(self,rhs:Self)->Self{ self.sub_vec(&rhs) } }
impl std::ops::Neg for FVector3 { type Output=Self; fn neg(self)->Self{ Self::new(-self.x(),-self.y(),-self.z()) } }
impl std::ops::Mul<f64> for FVector3 { type Output=Self; fn mul(self,s:f64)->Self{ self.scaled(s) } }
impl std::ops::Mul<FVector3> for f64 { type Output=FVector3; fn mul(self,v:FVector3)->FVector3{ v.scaled(self) } }
