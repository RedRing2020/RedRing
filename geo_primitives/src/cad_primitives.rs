/// CAD統合層 - modelからの基本プリミティブ移植
/// 
/// Phase 2: model(Scalar基礎＋CAD設計)のプリミティブ形状をgeo_primitivesに移動
/// ハイブリッド統合パターンを維持しつつ、geo_primitivesでCAD機能を提供

use std::ops::{Add, Sub, Mul, Neg};
use geo_core::{Point3D as GeoPoint3D, Vector3D as GeoVector3D, Scalar, ToleranceContext, TolerantEq};

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

    pub fn translate(&self, vector: &CadVector) -> Self {
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

impl Add<CadVector> for CadPoint {
    type Output = CadPoint;
    fn add(self, rhs: CadVector) -> CadPoint {
        self.translate(&rhs)
    }
}

impl Sub<CadPoint> for CadPoint {
    type Output = CadVector;
    fn sub(self, rhs: CadPoint) -> CadVector {
        CadVector::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

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

    pub fn between(a: &CadPoint, b: &CadPoint) -> Self {
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

// analysis クレートとの互換性のためのトレイト実装
impl analysis::NormedVector for CadVector {
    fn norm(&self) -> f64 {
        self.norm()
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

/// CAD用正規化方向ベクトル（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadDirection(CadVector);

impl PartialEq for CadDirection {
    fn eq(&self, other: &Self) -> bool {
        self.0.tolerant_eq(&other.0, 1e-10)
    }
}

impl CadDirection {
    pub fn from_vector(v: CadVector) -> Option<Self> {
        let len = v.norm();
        if len == 0.0 {
            None
        } else {
            Some(CadDirection(v.normalize()))
        }
    }

    pub fn x(&self) -> f64 { self.0.x() }
    pub fn y(&self) -> f64 { self.0.y() }
    pub fn z(&self) -> f64 { self.0.z() }

    pub fn normalize(&self) -> CadDirection {
        CadDirection::from_vector(self.0.clone()).unwrap()
    }

    pub fn as_vector(&self) -> CadVector {
        self.0.clone()
    }

    pub fn to_vector(&self) -> CadVector {
        self.0.clone()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.0.dot(&other.0)
    }

    pub fn cross(&self, other: &Self) -> CadVector {
        self.0.cross(&other.0)
    }

    /// 直交正規基底を計算（円の評価で使用）
    pub fn orthonormal_basis(&self) -> (CadVector, CadVector) {
        let n = &self.0;
        
        // 最も小さい成分を見つけて直交ベクトルを作成
        let u = if n.x().abs() < n.y().abs() && n.x().abs() < n.z().abs() {
            CadVector::new(1.0, 0.0, 0.0)
        } else if n.y().abs() < n.z().abs() {
            CadVector::new(0.0, 1.0, 0.0)
        } else {
            CadVector::new(0.0, 0.0, 1.0)
        };
        
        let v1 = n.cross(&u).normalize();
        let v2 = n.cross(&v1).normalize();
        
        (v1, v2)
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
    fn test_cad_vector_operations() {
        let v1 = CadVector::new(1.0, 0.0, 0.0);
        let v2 = CadVector::new(0.0, 1.0, 0.0);
        
        let cross = v1.cross(&v2);
        assert!((cross.z() - 1.0).abs() < 1e-10);
        
        let dot = v1.dot(&v2);
        assert!(dot.abs() < 1e-10);
    }

    #[test]
    fn test_cad_direction() {
        let v = CadVector::new(3.0, 4.0, 0.0);
        let dir = CadDirection::from_vector(v).unwrap();
        
        // 正規化されているか確認
        let norm = (dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }
}