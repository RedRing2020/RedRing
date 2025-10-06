/// CAD Vector - modelからの基本ベクトル構造体移植
///
/// ハイブリッド統合: geo_core::Vector3Dをベースにmodel CAD API互換性を提供

use std::ops::{Add, Sub, Mul, Neg};
use geo_core::{Vector3D as GeoVector3D, Scalar, ToleranceContext, TolerantEq};

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
        geo_core::Vector::norm(&self.inner).value()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        geo_core::Vector::dot(&self.inner, &other.inner).value()
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
        let mut context = ToleranceContext::default();
        context.linear = tolerance;
        TolerantEq::tolerant_eq(&self.inner, &other.inner, &context)
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
        CadVector::from_geo_core(self.inner * Scalar::new(scalar))
    }
}

impl Neg for CadVector {
    type Output = CadVector;
    fn neg(self) -> CadVector {
        CadVector::from_geo_core(-self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cad_vector_operations() {
        let v1 = CadVector::new(1.0, 0.0, 0.0);
        let v2 = CadVector::new(0.0, 1.0, 0.0);

        let cross = v1.cross(&v2);
        assert!((cross.z() - 1.0).abs() < 1e-10);

        let dot = v1.dot(&v2);
        assert!(dot.abs() < 1e-10);
    }

    #[test]
    fn test_cad_vector_norm() {
        let v = CadVector::new(3.0, 4.0, 0.0);
        assert!((v.norm() - 5.0).abs() < 1e-10);
    }
}
