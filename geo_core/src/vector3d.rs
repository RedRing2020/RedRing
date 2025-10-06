/// 3次元ベクトル実装
/// 
/// 座標値はmm単位で格納される

use crate::scalar::Scalar;
use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};
use crate::vector::Vector;
use std::fmt;
use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};

/// 3次元ベクトル
/// 
/// 座標値はmm単位で格納される
#[derive(Debug, Clone, PartialEq)]
pub struct Vector3D {
    components: [Scalar; 3],
    tolerance_context: ToleranceContext,
}

impl Vector3D {
    pub fn new_raw(x:f64,y:f64,z:f64)->Self { Self { components:[Scalar::new(x),Scalar::new(y),Scalar::new(z)], tolerance_context: ToleranceContext::default() } }
    pub fn from_f64(x:f64,y:f64,z:f64)->Self { Self::new_raw(x,y,z) }
    pub fn new(x:Scalar,y:Scalar,z:Scalar)->Self { Self { components:[x,y,z], tolerance_context: ToleranceContext::default() } }
    pub fn x(&self)->Scalar { self.components[0].clone() }
    pub fn y(&self)->Scalar { self.components[1].clone() }
    pub fn z(&self)->Scalar { self.components[2].clone() }
    pub fn set_x(&mut self, v:Scalar){ self.components[0]=v; }
    pub fn set_y(&mut self, v:Scalar){ self.components[1]=v; }
    pub fn set_z(&mut self, v:Scalar){ self.components[2]=v; }
    pub fn zero()->Self { Self::new_raw(0.0,0.0,0.0) }
    pub fn x_axis()->Self { Self::new_raw(1.0,0.0,0.0) }
    pub fn y_axis()->Self { Self::new_raw(0.0,1.0,0.0) }
    pub fn z_axis()->Self { Self::new_raw(0.0,0.0,1.0) }
    pub fn cross(&self, other:&Self)->Self { Self::new_raw(
        self.y().value()*other.z().value()-self.z().value()*other.y().value(),
        self.z().value()*other.x().value()-self.x().value()*other.z().value(),
        self.x().value()*other.y().value()-self.y().value()*other.x().value(),
    ) }
    /// Mixed (scalar triple) product (self · (b × c))
    pub fn scalar_triple_product(&self, b:&Self, c:&Self)->Scalar {
        let cross = b.cross(c);
        self.dot(&cross)
    }
    /// Vector triple product: self × (b × c)
    pub fn vector_triple_product(&self, b:&Self, c:&Self)->Self {
        // a × (b × c) = b (a·c) - c (a·b)
        let a_dot_c = self.dot(c);
        let a_dot_b = self.dot(b);
        b.clone()*a_dot_c - c.clone()*a_dot_b
    }
}

/// 正規化された3次元方向ベクトル (移行予定: geo_primitives)
#[deprecated(note = "Will be removed; switch to geo_primitives::geometry3d::Direction3D")]
#[derive(Debug, Clone, PartialEq)]
pub struct Direction3D { vector: Vector3D }
impl Direction3D {
    pub fn from_vector(v: Vector3D, ctx:&ToleranceContext)->Option<Self>{
        if let Some(n)=v.normalize(ctx){ Some(Self{vector:n}) } else { None }
    }
    pub fn new(x:f64,y:f64,z:f64,ctx:&ToleranceContext)->Option<Self>{ Self::from_vector(Vector3D::from_f64(x,y,z), ctx) }
    pub fn as_vector(&self)->&Vector3D { &self.vector }
    pub fn to_vector(&self)->Vector3D { self.vector.clone() }
    pub fn x(&self)->Scalar { self.vector.x() }
    pub fn y(&self)->Scalar { self.vector.y() }
    pub fn z(&self)->Scalar { self.vector.z() }
    pub fn dot(&self, other:&Self)->Scalar { self.vector.dot(&other.vector) }
    pub fn cross(&self, other:&Self, ctx:&ToleranceContext)->Option<Self>{ Self::from_vector(self.vector.cross(&other.vector), ctx) }
    pub fn negate(&self)->Self { Self { vector: -self.vector.clone() } }
    pub fn orthonormal_basis(&self, ctx:&ToleranceContext)->(Vector3D,Vector3D){
        let up = if self.z().value().abs() < 0.99 { Vector3D::z_axis() } else { Vector3D::x_axis() };
        let u = self.vector.cross(&up).normalize(ctx).unwrap();
        let v = self.vector.cross(&u);
        (u,v)
    }
}

// Vector トレイト実装
impl Vector<3> for Vector3D {
    fn new(components: [Scalar; 3]) -> Self { Self { components, tolerance_context: ToleranceContext::default() } }

    fn components(&self) -> &[Scalar; 3] {
        &self.components
    }

    fn components_mut(&mut self) -> &mut [Scalar; 3] {
        &mut self.components
    }

    fn dot(&self, other: &Self) -> Scalar { self.x()*other.x() + self.y()*other.y() + self.z()*other.z() }

    fn norm(&self) -> Scalar { self.norm_squared().sqrt() }

    fn normalize(&self, context:&ToleranceContext)->Option<Self>{
        let length=self.norm();
        if length.value() < context.linear { None } else { Some(Self::new_raw(self.x().value()/length.value(), self.y().value()/length.value(), self.z().value()/length.value())) }
    }

    fn is_parallel_to(&self, other:&Self, context:&ToleranceContext)->bool { self.cross(other).is_zero(context) }

    fn component_min(&self, other:&Self)->Self { Self::new(
        if self.x().value()<other.x().value(){self.x()}else{other.x()},
        if self.y().value()<other.y().value(){self.y()}else{other.y()},
        if self.z().value()<other.z().value(){self.z()}else{other.z()},
    ) }

    fn component_max(&self, other:&Self)->Self { Self::new(
        if self.x().value()>other.x().value(){self.x()}else{other.x()},
        if self.y().value()>other.y().value(){self.y()}else{other.y()},
        if self.z().value()>other.z().value(){self.z()}else{other.z()},
    ) }

    fn abs(&self)->Self { Self::new(self.x().abs(), self.y().abs(), self.z().abs()) }
}

// ToleranceProvider実装
impl ToleranceProvider for Vector3D {
    fn tolerance_context(&self) -> &ToleranceContext {
        &self.tolerance_context
    }

    fn set_tolerance_context(&mut self, context: ToleranceContext) {
        self.tolerance_context = context;
    }
}

// TolerantEq実装
impl TolerantEq for Vector3D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.x().tolerant_eq(&other.x(), context) &&
        self.y().tolerant_eq(&other.y(), context) &&
        self.z().tolerant_eq(&other.z(), context)
    }
}

// Index実装
impl Index<usize> for Vector3D {
    type Output = Scalar;

    fn index(&self, index: usize) -> &Self::Output {
        &self.components[index]
    }
}

impl IndexMut<usize> for Vector3D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
    }
}

// 算術演算実装
impl Add for Vector3D {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x() + other.x(), self.y() + other.y(), self.z() + other.z())
    }
}

impl Sub for Vector3D {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x() - other.x(), self.y() - other.y(), self.z() - other.z())
    }
}

impl Mul<Scalar> for Vector3D {
    type Output = Self;

    fn mul(self, scalar: Scalar) -> Self::Output {
        Self::new(self.x() * scalar, self.y() * scalar, self.z() * scalar)
    }
}

impl Neg for Vector3D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x(), -self.y(), -self.z())
    }
}

// Display実装
impl fmt::Display for Vector3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl fmt::Display for Direction3D { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f,"Direction3D({}, {}, {})", self.x().value(), self.y().value(), self.z().value()) } }