use std::ops::{Add, Sub, Mul, Neg};
use crate::geometry::geometry3d;
use crate::geometry::geometry3d::point::Point;
use crate::geometry_trait::normed::Normed;
use crate::geometry_trait::normalize::Normalize;
use geo_core::{Vector3D as GeoVector3D, Vector as GeoVector, ToleranceContext, TolerantEq, Scalar};

/// 3D vector with geo_core numerical foundation
/// ハイブリッド統合: geo_core::Vector3Dをベースにmodel CAD API互換性を提供
#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    inner: GeoVector3D,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { 
            inner: GeoVector3D::from_f64(x, y, z) 
        }
    }

    pub fn x(&self) -> f64 { self.inner.x().value() }
    pub fn y(&self) -> f64 { self.inner.y().value() }
    pub fn z(&self) -> f64 { self.inner.z().value() }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0) 
    }



    pub fn norm(&self) -> f64 {
        GeoVector::norm(&self.inner).value()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        GeoVector::dot(&self.inner, &other.inner).value()
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            inner: self.inner.cross(&other.inner)
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            inner: self.inner.clone() * Scalar::new(factor)
        }
    }

    pub fn between(a: &Point, b: &Point) -> Self {
        Self::new(b.x() - a.x(), b.y() - a.y(), b.z() - a.z())
    }

    // geo_core統合メソッド
    pub fn as_geo_core(&self) -> &GeoVector3D {
        &self.inner
    }

    pub fn from_geo_core(geo_vec: GeoVector3D) -> Self {
        Self { inner: geo_vec }
    }

    pub fn tolerant_eq(&self, other: &Self, tolerance: f64) -> bool {
        let mut context = ToleranceContext::default();
        context.linear = tolerance;
        TolerantEq::tolerant_eq(&self.inner, &other.inner, &context)
    }
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector::from_geo_core(self.inner + rhs.inner)
    }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Vector) -> Vector {
        Vector::from_geo_core(self.inner - rhs.inner)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, scalar: f64) -> Vector {
        Vector::from_geo_core(self.inner * Scalar::new(scalar))
    }
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector::from_geo_core(-self.inner)
    }
}

impl Normed for geometry3d::vector::Vector {
    fn norm(&self) -> f64 {
        self.norm()
    }
}

impl Normalize for geometry3d::vector::Vector {
    fn normalize(&self) -> Self {
        let len = self.norm();
        if len == 0.0 {
            Self::zero()
        } else {
            Self::new(self.x() / len, self.y() / len, self.z() / len)
        }
    }
}

// analysis クレートとの互換性のためのトレイト実装
impl analysis::NormedVector for Vector {
    fn norm(&self) -> f64 {
        self.norm()
    }
}
