/// 2次元ベクトル実装
/// 
/// 座標値はmm単位で格納される
/// 2次元ベクトル実装
/// 
/// 座標値はmm単位で格納される

use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};
use crate::vector::Vector;
use std::fmt;
use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Vector2D {
    components: [f64; 2],
    tolerance_context: ToleranceContext,
}

impl PartialEq for Vector2D {
    fn eq(&self, other: &Self) -> bool {
        self.components[0] == other.components[0] && self.components[1] == other.components[1]
    }
}

impl Vector2D {
    pub fn new_raw(x: f64, y: f64) -> Self { Self { components: [x,y], tolerance_context: ToleranceContext::default() } }
    pub fn from_f64(x: f64, y: f64) -> Self { Self::new_raw(x,y) }
    /// 新しい f64 コンストラクタ（破壊的変更移行用の糖衣）
    pub fn new(x: f64, y: f64) -> Self { Self::from_f64(x,y) }
    pub fn x(&self) -> f64 { self.components[0] }
    pub fn y(&self) -> f64 { self.components[1] }
    pub fn set_x(&mut self, x: f64) { self.components[0] = x; }
    pub fn set_y(&mut self, y: f64) { self.components[1] = y; }
    pub fn cross_2d(&self, other: &Self) -> f64 { self.x()*other.y() - self.y()*other.x() }
    pub fn perpendicular(&self) -> Self { Self::new_raw(-self.y(), self.x()) }
    pub fn rotate(&self, angle: f64) -> Self { let (s,c)=angle.sin_cos(); Self::new_raw(self.x()*c - self.y()*s, self.x()*s + self.y()*c) }
    pub fn zero() -> Self { Self::new_raw(0.0,0.0) }
    pub fn x_axis() -> Self { Self::new_raw(1.0,0.0) }
    pub fn y_axis() -> Self { Self::new_raw(0.0,1.0) }
}

impl Vector<2> for Vector2D {
    fn new(components: [f64;2]) -> Self { Self { components, tolerance_context: ToleranceContext::default() } }
    fn components(&self) -> &[f64;2] { &self.components }
    fn components_mut(&mut self) -> &mut [f64;2] { &mut self.components }
    fn dot(&self, other:&Self) -> f64 { self.x()*other.x() + self.y()*other.y() }
    fn norm(&self) -> f64 { self.norm_squared().sqrt() }
    fn normalize(&self, ctx:&ToleranceContext) -> Option<Self> { let l=self.norm(); if l < ctx.linear { None } else { Some(Self::new_raw(self.x()/l,self.y()/l)) } }
    fn is_parallel_to(&self, other:&Self, ctx:&ToleranceContext) -> bool { self.cross_2d(other).abs() < ctx.linear }
    fn component_min(&self, other:&Self) -> Self { Self::new_raw(self.x().min(other.x()), self.y().min(other.y())) }
    fn component_max(&self, other:&Self) -> Self { Self::new_raw(self.x().max(other.x()), self.y().max(other.y())) }
    fn abs(&self) -> Self { Self::new_raw(self.x().abs(), self.y().abs()) }
}

impl ToleranceProvider for Vector2D {
    fn tolerance_context(&self) -> &ToleranceContext { &self.tolerance_context }
    fn set_tolerance_context(&mut self, context: ToleranceContext) { self.tolerance_context = context; }
}

impl TolerantEq for Vector2D {
    fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext) -> bool { (self.x()-other.x()).abs() <= ctx.linear && (self.y()-other.y()).abs() <= ctx.linear }
}

impl Index<usize> for Vector2D { type Output = f64; fn index(&self, i:usize)->&Self::Output { &self.components[i] } }
impl IndexMut<usize> for Vector2D { fn index_mut(&mut self, i:usize)->&mut Self::Output { &mut self.components[i] } }

impl Add for Vector2D { type Output=Self; fn add(self, rhs:Self)->Self { Self::new_raw(self.x()+rhs.x(), self.y()+rhs.y()) } }
impl Sub for Vector2D { type Output=Self; fn sub(self, rhs:Self)->Self { Self::new_raw(self.x()-rhs.x(), self.y()-rhs.y()) } }
impl Mul<f64> for Vector2D { type Output=Self; fn mul(self, s:f64)->Self { Self::new_raw(self.x()*s, self.y()*s) } }
// 逆方向乗算 f64 * Vector2D をサポート（演算子対称性向上）
impl Mul<Vector2D> for f64 { type Output = Vector2D; fn mul(self, v: Vector2D)->Vector2D { Vector2D::new_raw(v.x()*self, v.y()*self) } }
impl<'a> Mul<&'a Vector2D> for f64 { type Output = Vector2D; fn mul(self, v: &'a Vector2D)->Vector2D { Vector2D::new_raw(v.x()*self, v.y()*self) } }
impl Neg for Vector2D { type Output=Self; fn neg(self)->Self { Self::new_raw(-self.x(), -self.y()) } }

impl fmt::Display for Vector2D { fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f, "({}, {})", self.x(), self.y()) } }