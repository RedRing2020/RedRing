//! # Point3D - 廃止予定
//!
//! **⚠️ このモジュールは廃止予定です。`geo_primitives::Point3D` をご利用ください。**
//!
//! ## 移行例
//! ```rust
//! // 旧:
//! // use geo_core::Point3D;
//!
//! // 新:
//! use geo_primitives::Point3D;
//! ```
use crate::scalar::Scalar;
use crate::tolerance::{ToleranceContext, TolerantEq};
use crate::vector::Vector3D;

#[derive(Debug, Clone)]
#[deprecated(note = "Use geo_primitives::Point3D instead")]
pub struct Point3D {
    x: Scalar,
    y: Scalar,
    z: Scalar,
}

impl Point3D {
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self { Self { x, y, z } }
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self { Self { x: Scalar::new(x), y: Scalar::new(y), z: Scalar::new(z) } }
    pub fn origin() -> Self { Self::from_f64(0.0, 0.0, 0.0) }
    pub fn x(&self) -> &Scalar { &self.x }
    pub fn y(&self) -> &Scalar { &self.y }
    pub fn z(&self) -> &Scalar { &self.z }
    pub fn distance_to(&self, other:&Self)->Scalar {
        let dx = self.x.clone() - other.x.clone();
        let dy = self.y.clone() - other.y.clone();
        let dz = self.z.clone() - other.z.clone();
        (dx.clone()*dx + dy.clone()*dy + dz.clone()*dz).sqrt()
    }
    pub fn to_vector(&self)->Vector3D { Vector3D::new(self.x.clone(), self.y.clone(), self.z.clone()) }
    pub fn midpoint(&self, other:&Self)->Self {
        let two = Scalar::new(2.0);
        Self::new(
            (self.x.clone()+other.x.clone())/two.clone(),
            (self.y.clone()+other.y.clone())/two.clone(),
            (self.z.clone()+other.z.clone())/two,
        )
    }
    pub fn centroid(points:&[Self])->Option<Self> {
        if points.is_empty(){ return None; }
        let mut sx=Scalar::new(0.0); let mut sy=Scalar::new(0.0); let mut sz=Scalar::new(0.0);
        for p in points { sx = sx + p.x.clone(); sy = sy + p.y.clone(); sz = sz + p.z.clone(); }
        let n = Scalar::new(points.len() as f64);
        Some(Self::new(sx/n.clone(), sy/n.clone(), sz/n))
    }
}

impl TolerantEq for Point3D {
    fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool {
        self.x.tolerant_eq(&other.x, ctx) && self.y.tolerant_eq(&other.y, ctx) && self.z.tolerant_eq(&other.z, ctx)
    }
}
