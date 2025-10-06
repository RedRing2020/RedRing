/// CAD Point - modelからの基本点構造体移植
/// 
/// ハイブリッド統合: geo_core::Point3Dをベースにmodel CAD API互換性を提供

use std::ops::{Add, Sub};
use geo_core::{ToleranceContext, TolerantEq};
// Point3D removed - using tuple-based coordinate

/// CAD用3D点（modelからの移植）
/// ハイブリッド統合: geo_core::Point3Dをベースにmodel CAD API互換性を提供
#[derive(Debug, Clone)]
pub struct CadPoint {
    inner: GeoPoint3D,
}

impl PartialEq for CadPoint {
    fn eq(&self, other: &Self) -> bool {
        let mut context = ToleranceContext::default();
        context.linear = 1e-10;
        self.tolerant_eq(other, &context)
    }
}

impl CadPoint {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoPoint3D::new(x, y, z)
        }
    }

    pub fn x(&self) -> f64 { self.inner.x() }
    pub fn y(&self) -> f64 { self.inner.y() }  
    pub fn z(&self) -> f64 { self.inner.z() }

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

    /// f64タプルから作成
    pub fn from_tuple(coord: (f64, f64, f64)) -> Self {
        Self::new(coord.0, coord.1, coord.2)  
    }

    /// f64タプルに変換
    pub fn to_tuple(&self) -> (f64, f64, f64) {
        (self.x(), self.y(), self.z())
    }
}

impl TolerantEq for CadPoint {
    fn tolerant_eq(&self, other: &Self, tolerance: &ToleranceContext) -> bool {
        let dx = self.x() - other.x();
        let dy = self.y() - other.y();
        let dz = self.z() - other.z();
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        distance <= tolerance.linear
    }
}

// geo-core互換レイヤー
use crate::geometry3d::Point3D as GeoPoint3D;

impl From<GeoPoint3D> for CadPoint {
    fn from(point: GeoPoint3D) -> Self {
        Self { inner: point }
    }
}

impl From<CadPoint> for GeoPoint3D {
    fn from(cad_point: CadPoint) -> Self {
        cad_point.inner
    }
}

impl Add<crate::CadVector> for CadPoint {
    type Output = Self;
    
    fn add(self, vector: crate::CadVector) -> Self::Output {
        self.translate(&vector)
    }
}

impl Sub for CadPoint {
    type Output = crate::CadVector;
    
    fn sub(self, other: Self) -> Self::Output {
        crate::CadVector::new(
            self.x() - other.x(),
            self.y() - other.y(),
            self.z() - other.z()
        )
    }
}