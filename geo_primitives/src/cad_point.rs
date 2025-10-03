/// CAD Point - modelからの基本点構造体移植
/// 
/// ハイブリッド統合: geo_core::Point3Dをベースにmodel CAD API互換性を提供

use std::ops::{Add, Sub};
use geo_core::{Point3D as GeoPoint3D, ToleranceContext, TolerantEq};

/// CAD用3D点（modelからの移植）
/// ハイブリッド統合: geo_core::Point3Dをベースにmodel CAD API互換性を提供
#[derive(Debug, Clone)]
pub struct CadPoint {
    inner: GeoPoint3D,
}

impl PartialEq for CadPoint {
    fn eq(&self, other: &Self) -> bool {
        self.tolerant_eq(other, 1e-10)
    }
}

impl CadPoint {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoPoint3D::from_f64(x, y, z)
        }
    }

    pub fn x(&self) -> f64 { self.inner.x().value() }
    pub fn y(&self) -> f64 { self.inner.y().value() }  
    pub fn z(&self) -> f64 { self.inner.z().value() }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x() - other.x();
        let dy = self.y() - other.y();
        let dz = self.z() - other.z();
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn translate(&self, vector: &crate::CadVector) -> Self {
        Self::new(
            self.x() + vector.x(),
            self.y() + vector.y(),
            self.z() + vector.z()
        )
    }

    // geo_core統合メソッド
    pub fn as_geo_core(&self) -> &GeoPoint3D {
        &self.inner
    }

    pub fn from_geo_core(geo_point: GeoPoint3D) -> Self {
        Self { inner: geo_point }
    }

    pub fn tolerant_eq(&self, other: &Self, tolerance: f64) -> bool {
        let mut context = ToleranceContext::default();
        context.linear = tolerance;
        TolerantEq::tolerant_eq(&self.inner, &other.inner, &context)
    }
}

impl Add<crate::CadVector> for CadPoint {
    type Output = CadPoint;
    fn add(self, rhs: crate::CadVector) -> CadPoint {
        self.translate(&rhs)
    }
}

impl Sub<CadPoint> for CadPoint {
    type Output = crate::CadVector;
    fn sub(self, rhs: CadPoint) -> crate::CadVector {
        crate::CadVector::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cad_point_basic() {
        let p = CadPoint::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_cad_point_distance() {
        let p1 = CadPoint::new(0.0, 0.0, 0.0);
        let p2 = CadPoint::new(3.0, 4.0, 0.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 1e-10);
    }
}