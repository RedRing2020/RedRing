/// 3次元ベクトル実装（f64ベース、modelトレイト互換）

use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};
use std::fmt;
use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};

/// 3次元ベクトル（f64ベース）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self { Self::new(x, y, z) }
    pub fn new_raw(x: f64, y: f64, z: f64) -> Self { Self::new(x, y, z) }
    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }
    pub fn set_x(&mut self, v: f64) { self.x = v; }
    pub fn set_y(&mut self, v: f64) { self.y = v; }
    pub fn set_z(&mut self, v: f64) { self.z = v; }
    pub fn zero() -> Self { Self::new(0.0, 0.0, 0.0) }
    pub fn x_axis() -> Self { Self::new(1.0, 0.0, 0.0) }
    pub fn y_axis() -> Self { Self::new(0.0, 1.0, 0.0) }
    pub fn z_axis() -> Self { Self::new(0.0, 0.0, 1.0) }

    /// ベクトルの長さ
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 正規化（modelトレイト互換 - ゼロベクトルはゼロベクトルを返す）
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// ノルム（長さ）- modelトレイト互換
    pub fn norm(&self) -> f64 {
        self.length()
    }

    /// スカラー加算（スケール加算）- modelトレイト互換
    pub fn add_scaled(&self, other: &Self, scale: f64) -> Self {
        Self::new(
            self.x + other.x * scale,
            self.y + other.y * scale,
            self.z + other.z * scale,
        )
    }
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    /// Mixed (scalar triple) product (self · (b × c))
    pub fn scalar_triple_product(&self, b: &Self, c: &Self) -> f64 {
        let cross = b.cross(c);
        self.dot(&cross)
    }
    /// Vector triple product: self × (b × c)
    pub fn vector_triple_product(&self, b: &Self, c: &Self) -> Self {
        // a × (b × c) = b (a·c) - c (a·b)
        let a_dot_c = self.dot(c);
        let a_dot_b = self.dot(b);
        *b * a_dot_c - *c * a_dot_b
    }
}

/// 正規化された3次元方向ベクトル (移行予定: geo_primitives)
#[deprecated(note = "Will be removed; switch to geo_primitives::geometry3d::Direction3D")]
#[derive(Debug, Clone, PartialEq)]
pub struct Direction3D { vector: Vector3D }
impl Direction3D {
    pub fn from_vector(v: Vector3D, _ctx: &ToleranceContext) -> Option<Self> {
        let len = v.length();
        if len == 0.0 {
            None
        } else {
            Some(Self { vector: v.normalize() })
        }
    }
    pub fn new(x:f64,y:f64,z:f64,ctx:&ToleranceContext)->Option<Self>{ Self::from_vector(Vector3D::from_f64(x,y,z), ctx) }
    pub fn as_vector(&self)->&Vector3D { &self.vector }
    pub fn to_vector(&self)->Vector3D { self.vector.clone() }
    pub fn x(&self) -> f64 { self.vector.x() }
    pub fn y(&self) -> f64 { self.vector.y() }
    pub fn z(&self) -> f64 { self.vector.z() }
    pub fn dot(&self, other: &Self) -> f64 { self.vector.dot(&other.vector) }
    pub fn cross(&self, other:&Self, ctx:&ToleranceContext)->Option<Self>{ Self::from_vector(self.vector.cross(&other.vector), ctx) }
    pub fn negate(&self)->Self { Self { vector: -self.vector.clone() } }
    pub fn orthonormal_basis(&self, _ctx: &ToleranceContext) -> (Vector3D, Vector3D) {
        let up = if self.z().abs() < 0.99 { Vector3D::z_axis() } else { Vector3D::x_axis() };
        let u = self.vector.cross(&up).normalize();
        let v = self.vector.cross(&u);
        (u, v)
    }
}

// f64ベースの基本操作メソッド
impl Vector3D {
    /// 2つのベクトルが平行かどうかを判定
    pub fn is_parallel_to(&self, other: &Self, tolerance: f64) -> bool {
        self.cross(other).length() <= tolerance
    }

    /// 成分ごとの最小値
    pub fn component_min(&self, other: &Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    /// 成分ごとの最大値
    pub fn component_max(&self, other: &Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    /// 絶対値
    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
}

// ToleranceProvider実装
impl ToleranceProvider for Vector3D {
    fn tolerance_context(&self) -> &ToleranceContext {
        &ToleranceContext::default()
    }

    fn set_tolerance_context(&mut self, _context: ToleranceContext) {
        // f64ベースの実装では許容誤差は内部に保持しない
    }
}

// TolerantEq実装
impl TolerantEq for Vector3D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        let diff = *self - *other;
        diff.length() <= context.linear_tolerance()
    }
}

// Index実装
impl Index<usize> for Vector3D {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds for Vector3D"),
        }
    }
}

impl IndexMut<usize> for Vector3D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds for Vector3D"),
        }
    }
}

// 算術演算実装
impl Add for Vector3D {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vector3D {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for Vector3D {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Neg for Vector3D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

// 表示実装
impl fmt::Display for Vector3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector3D({:.6}, {:.6}, {:.6})", self.x, self.y, self.z)
    }
}

impl fmt::Display for Direction3D { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        write!(f, "Direction3D({:.6}, {:.6}, {:.6})", self.x(), self.y(), self.z()) 
    } 
}