//! Direction3D - 正規化された3D方向ベクトル//! Direction3D - migrated from geo_core (legacy normalized 3D direction)use geo_core::tolerance::{ToleranceContext, TolerantEq};use geo_core::tolerance::{ToleranceContext, TolerantEq};use geo_core::tolerance::{ToleranceContext, TolerantEq};use geo_core::tolerance::{ToleranceContext, TolerantEq};use geo_core::tolerance::{ToleranceContext, TolerantEq};//! Direction3D - migrated from geo_core (legacy normalized 3D direction)



use super::vector3d::Vector3D;



/// 正規化された3Dベクトル（方向ベクトル）use super::vector3d::Vector3D;use super::vector3d::Vector3D;

#[derive(Debug, Clone, Copy)]

pub struct Direction3D {

    v: Vector3D,

}/// 正規化された3Dベクトル（方向ベクトル）use super::vector3d::Vector3D;



impl Direction3D {#[derive(Debug, Clone, Copy)]

    /// ベクトルから方向ベクトルを作成（正規化）

    pub fn from_vector(v: Vector3D) -> Option<Self> {pub struct Direction3D {#[derive(Debug, Clone, Copy)]

        v.normalize().map(|n| Self { v: n })

    }    v: Vector3D,



    /// 成分から方向ベクトルを作成}pub struct Direction3D { use super::vector3d::Vector3D;

    pub fn new(x: f64, y: f64, z: f64) -> Option<Self> {

        Self::from_vector(Vector3D::new(x, y, z))

    }

impl Direction3D {    v: Vector3D

    /// X成分を取得

    pub fn x(&self) -> f64 {    /// ベクトルから方向ベクトルを作成（正規化）

        self.v.x()

    }    pub fn from_vector(v: Vector3D) -> Option<Self> {}#[derive(Debug, Clone, Copy)]



    /// Y成分を取得        v.normalize().map(|n| Self { v: n })

    pub fn y(&self) -> f64 {

        self.v.y()    }

    }



    /// Z成分を取得

    pub fn z(&self) -> f64 {    /// 成分から方向ベクトルを作成impl Direction3D {pub struct Direction3D { use super::vector3d::Vector3D;

        self.v.z()

    }    pub fn new(x: f64, y: f64, z: f64) -> Option<Self> {



    /// 長さ（常に1.0）        Self::from_vector(Vector3D::new(x, y, z))    pub fn from_vector(v:Vector3D)->Option<Self>{

    pub fn length(&self) -> f64 {

        1.0    }

    }

        v.normalize().map(|n|Self{v:n})     v: Vector3D

    /// 内部ベクトルを取得

    pub fn as_vector(&self) -> &Vector3D {    /// X成分を取得

        &self.v

    }    pub fn x(&self) -> f64 {    }



    /// ベクトルに変換        self.v.x()

    pub fn to_vector(&self) -> Vector3D {

        self.v    }    }#[derive(Debug, Clone, Copy)]

    }



    /// 内積

    pub fn dot(&self, other: &Self) -> f64 {    /// Y成分を取得    pub fn new(x:f64,y:f64,z:f64)->Option<Self>{

        self.v.dot(&other.v)

    }    pub fn y(&self) -> f64 {



    /// 外積        self.v.y()        Self::from_vector(Vector3D::new(x,y,z))

    pub fn cross(&self, other: &Self) -> Option<Self> {

        let result = self.v.cross(&other.v);    }

        Self::from_vector(result)

    }    }



    /// 反転    /// Z成分を取得

    pub fn negate(&self) -> Self {

        Self { v: -self.v }    pub fn z(&self) -> f64 {    impl Direction3D {pub struct Direction3D { v: Vector3D }use super::vector3d::Vector3D;#![allow(deprecated)]

    }

        self.v.z()

    /// 直交する正規直交基底を生成

    pub fn orthonormal_basis(&self) -> (Vector3D, Vector3D) {    }    pub fn x(&self)->f64 { self.v.x() }

        let up = if self.z().abs() < 0.99 {

            Vector3D::z_axis()

        } else {

            Vector3D::x_axis()    /// 長さ（常に1.0）    pub fn y(&self)->f64 { self.v.y() }    pub fn from_vector(v:Vector3D)->Option<Self>{

        };

        let u = self.v.cross(&up).normalize().unwrap_or(Vector3D::x_axis());    pub fn length(&self) -> f64 {

        let v = self.v.cross(&u);

        (u, v)        1.0    pub fn z(&self)->f64 { self.v.z() }

    }

}    }

            v.normalize().map(|n|Self{v:n})

    /// 内部ベクトルを取得

    pub fn as_vector(&self) -> &Vector3D {    pub fn as_vector(&self)->Vector3D { self.v }

        &self.v

    }        }



    /// ベクトルに変換    pub fn orthonormal_basis(&self)->(Vector3D,Vector3D){

    pub fn to_vector(&self) -> Vector3D {

        self.v        let up = if self.z().abs() < 0.99 {     impl Direction3D {#[derive(Debug, Clone, Copy)]

    }

            Vector3D::new(0.0,0.0,1.0)

    /// 内積

    pub fn dot(&self, other: &Self) -> f64 {        } else {     pub fn new(x:f64,y:f64,z:f64)->Option<Self>{

        self.v.dot(&other.v)

    }            Vector3D::new(1.0,0.0,0.0)



    /// 外積        };        Self::from_vector(Vector3D::new(x,y,z))     pub fn from_vector(v:Vector3D)->Option<Self>{

    pub fn cross(&self, other: &Self) -> Option<Self> {

        let result = self.v.cross(&other.v);        let u = self.v.cross(&up).normalize().unwrap_or(Vector3D::new(1.0,0.0,0.0));

        Self::from_vector(result)

    }        let v = self.v.cross(&u);    }



    /// 反転        (u,v)

    pub fn negate(&self) -> Self {

        Self { v: -self.v }    }            v.normalize().map(|n|Self{v:n}) pub struct Direction3D { v: Vector3D }use geo_core::tolerance::{ToleranceContext, ToleranceProvider, TolerantEq};

    }

}

    /// 直交する正規直交基底を生成

    pub fn orthonormal_basis(&self) -> (Vector3D, Vector3D) {    pub fn x(&self)->f64 { self.v.x() }

        let up = if self.z().abs() < 0.99 {

            Vector3D::z_axis()impl TolerantEq for Direction3D {

        } else {

            Vector3D::x_axis()    fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool {     pub fn y(&self)->f64 { self.v.y() }    }

        };

        let u = self.v.cross(&up).normalize().unwrap_or(Vector3D::x_axis());        (self.x()-other.x()).abs()<=ctx.linear &&

        let v = self.v.cross(&u);

        (u, v)        (self.y()-other.y()).abs()<=ctx.linear &&     pub fn z(&self)->f64 { self.v.z() }

    }

}        (self.z()-other.z()).abs()<=ctx.linear



// Display実装は別クレートで実装    }        impl Direction3D {

}
    pub fn as_vector(&self)->Vector3D { self.v }

        pub fn new(x:f64,y:f64,z:f64)->Option<Self>{

    pub fn orthonormal_basis(&self)->(Vector3D,Vector3D){

        let up = if self.z().abs() < 0.99 { Vector3D::new(0.0,0.0,1.0)} else { Vector3D::new(1.0,0.0,0.0)};        Self::from_vector(Vector3D::new(x,y,z))     pub fn from_vector(v:Vector3D)->Option<Self>{ v.normalize().map(|n|Self{v:n}) }#[derive(Debug, Clone, Copy)]use geo_core::vector::{Vector3D, Vector};

        let u = self.v.cross(&up).normalize().unwrap_or(Vector3D::new(1.0,0.0,0.0));

        let v = self.v.cross(&u);    }

        (u,v)

    }        pub fn new(x:f64,y:f64,z:f64)->Option<Self>{ Self::from_vector(Vector3D::new(x,y,z)) }

}

    pub fn x(&self)->f64 { self.v.x() }

impl TolerantEq for Direction3D {

    fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool {     pub fn y(&self)->f64 { self.v.y() }    pub fn x(&self)->f64 { self.v.x() }pub struct Direction3D { v: Vector3D }use std::fmt;

        (self.x()-other.x()).abs()<=ctx.linear &&

        (self.y()-other.y()).abs()<=ctx.linear &&     pub fn z(&self)->f64 { self.v.z() }

        (self.z()-other.z()).abs()<=ctx.linear

    }        pub fn y(&self)->f64 { self.v.y() }

}
    pub fn as_vector(&self)->Vector3D { self.v }

        pub fn z(&self)->f64 { self.v.z() }impl Direction3D {

    pub fn orthonormal_basis(&self)->(Vector3D,Vector3D){

        let up = if self.z().abs() < 0.99 { Vector3D::new(0.0,0.0,1.0)} else { Vector3D::new(1.0,0.0,0.0)};    pub fn as_vector(&self)->Vector3D { self.v }

        let u = self.v.cross(&up).normalize().unwrap_or(Vector3D::new(1.0,0.0,0.0));

        let v = self.v.cross(&u);    pub fn orthonormal_basis(&self)->(Vector3D,Vector3D){    pub fn from_vector(v:Vector3D)->Option<Self>{ v.normalize().map(|n|Self{v:n}) }#[derive(Debug, Clone, PartialEq)]

        (u,v)

    }        let up = if self.z().abs() < 0.99 { Vector3D::new(0.0,0.0,1.0)} else { Vector3D::new(1.0,0.0,0.0)};

}

        let u = self.v.cross(&up).normalize().unwrap_or(Vector3D::new(1.0,0.0,0.0));    pub fn new(x:f64,y:f64,z:f64)->Option<Self>{ Self::from_vector(Vector3D::new(x,y,z)) }#[deprecated(note = "Use f64 canonical type geo_primitives::Direction3D (alias of FDirection3)")]

impl TolerantEq for Direction3D {

    fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool {         let v = self.v.cross(&u);

        (self.x()-other.x()).abs()<=ctx.linear &&

        (self.y()-other.y()).abs()<=ctx.linear &&         (u,v)    pub fn x(&self)->f64 { self.v.x() }pub struct LegacyDirection3D { vector: Vector3D }

        (self.z()-other.z()).abs()<=ctx.linear

    }    }

}
}    pub fn y(&self)->f64 { self.v.y() }impl LegacyDirection3D {

impl TolerantEq for Direction3D { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { (self.x()-other.x()).abs()<=ctx.linear && (self.y()-other.y()).abs()<=ctx.linear && (self.z()-other.z()).abs()<=ctx.linear }}
    pub fn z(&self)->f64 { self.v.z() }    pub fn from_vector(v: Vector3D, context: &ToleranceContext) -> Option<Self> {

    pub fn as_vector(&self)->Vector3D { self.v }        if let Some(n) = v.normalize(context) { Some(Self { vector: n }) } else { None }

    pub fn orthonormal_basis(&self)->(Vector3D,Vector3D){    }

        let up = if self.z().abs() < 0.99 { Vector3D::new(0.0,0.0,1.0)} else { Vector3D::new(1.0,0.0,0.0)};    pub fn new(x: f64, y: f64, z: f64, context: &ToleranceContext) -> Option<Self> {

        let u = self.v.cross(&up).normalize().unwrap_or(Vector3D::new(1.0,0.0,0.0));        Self::from_vector(Vector3D::from_f64(x,y,z), context)

        let v = self.v.cross(&u);    }

        (u,v)    pub fn as_vector(&self) -> &Vector3D { &self.vector }

    }    pub fn to_vector(&self) -> Vector3D { self.vector.clone() }

}    pub fn x(&self)->f64 { self.vector.x().value() }

impl TolerantEq for Direction3D { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { (self.x()-other.x()).abs()<=ctx.linear && (self.y()-other.y()).abs()<=ctx.linear && (self.z()-other.z()).abs()<=ctx.linear }}    pub fn y(&self)->f64 { self.vector.y().value() }
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
// Display実装は別クレートで実装
impl ToleranceProvider for LegacyDirection3D { fn tolerance_context(&self)->&ToleranceContext { self.vector.tolerance_context() } fn set_tolerance_context(&mut self, ctx:ToleranceContext){ self.vector.set_tolerance_context(ctx)} }
impl TolerantEq for LegacyDirection3D { fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool { (self.x()-other.x()).abs() <= ctx.linear && (self.y()-other.y()).abs()<=ctx.linear && (self.z()-other.z()).abs()<=ctx.linear }}
