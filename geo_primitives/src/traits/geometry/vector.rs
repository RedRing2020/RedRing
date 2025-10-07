/// CAD Vector - modelからの基本ベクトル構造体移植
///
/// ハイブリッド統合: geo_core::Vector3Dをベースにmodel CAD API互換性を提供

use std::ops::{Add, Sub, Mul, Neg};
use geo_foundation::ToleranceContext;
use crate::geometry3d::Vector3D as GeoVector3D;

/// CAD用3Dベクトル（modelからの移植）
/// ハイブリッド統合: geo_core::Vector3Dをベースにmodel CAD API互換性を提供
#[derive(Debug, Clone)]
pub struct CadVector {
    inner: GeoVector3D,
}

impl PartialEq for CadVector {
    fn eq(&self, other: &Self) -> bool {
        self.tolerant_eq(other, 1e-10)
    }
}

impl CadVector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoVector3D::new(x, y, z)
        }
    }

    pub fn x(&self) -> f64 { self.inner.x() }
    pub fn y(&self) -> f64 { self.inner.y() }
    pub fn z(&self) -> f64 { self.inner.z() }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn norm(&self) -> f64 {
        self.inner.norm()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.inner.dot(&other.inner)
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            inner: self.inner.cross(&other.inner)
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            inner: self.inner * factor
        }
    }

    pub fn between(a: &crate::CadPoint, b: &crate::CadPoint) -> Self {
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
        let context = ToleranceContext::new(tolerance);
        // Vector3DのTolerantEq実装がないため、直接比較
        (self.inner.x() - other.inner.x()).abs() < context.tolerance() &&
        (self.inner.y() - other.inner.y()).abs() < context.tolerance() &&
        (self.inner.z() - other.inner.z()).abs() < context.tolerance()
    }

    pub fn normalize(&self) -> Self {
        let len = self.norm();
        if len == 0.0 {
            Self::zero()
        } else {
            Self::new(self.x() / len, self.y() / len, self.z() / len)
        }
    }
}

impl Add for CadVector {
    type Output = CadVector;
    fn add(self, rhs: CadVector) -> CadVector {
        CadVector::from_geo_core(self.inner + rhs.inner)
    }
}

impl Sub for CadVector {
    type Output = CadVector;
    fn sub(self, rhs: CadVector) -> CadVector {
        CadVector::from_geo_core(self.inner - rhs.inner)
    }
}

impl Mul<f64> for CadVector {
    type Output = CadVector;
    fn mul(self, scalar: f64) -> CadVector {
        CadVector::from_geo_core(self.inner * scalar)
    }
}

impl Neg for CadVector {
    type Output = CadVector;
    fn neg(self) -> CadVector {
        CadVector::from_geo_core(-self.inner)
    }
}


