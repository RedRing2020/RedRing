//! CadVector は非推奨です。今後は `geo_core::Vector3D` を直接利用してください。
//! 段階的移行のため、既存 API を薄いラッパで保持しています。

use std::ops::{Add, Sub, Mul, Neg};
use geo_core::{Vector3D, ToleranceContext, TolerantEq, Vector as VectorTrait};

#[derive(Debug, Clone)]
pub struct CadVector(pub Vector3D);

impl PartialEq for CadVector {
    fn eq(&self, other: &Self) -> bool { self.tolerant_eq(other, 1e-10) }
}

impl CadVector {
    #[deprecated(note = "Use Vector3D::from_f64 instead of CadVector::new")] 
    pub fn new(x: f64, y: f64, z: f64) -> Self { Self(Vector3D::from_f64(x,y,z)) }

    #[deprecated(note = "Use Vector3D::from_f64(0.0,0.0,0.0) or Vector3D::zero if implemented")] 
    pub fn zero() -> Self { Self(Vector3D::from_f64(0.0,0.0,0.0)) }

    #[deprecated(note = "Use .0.x_val() on Vector3D")] pub fn x(&self) -> f64 { self.0.x_val() }
    #[deprecated(note = "Use .0.y_val() on Vector3D")] pub fn y(&self) -> f64 { self.0.y_val() }
    #[deprecated(note = "Use .0.z_val() on Vector3D")] pub fn z(&self) -> f64 { self.0.z_val() }
    #[deprecated(note = "Use VectorTrait::norm(&v) on Vector3D")] pub fn norm(&self) -> f64 { VectorTrait::norm(&self.0) }
    #[deprecated(note = "Use self.0.dot(other.0) on Vector3D")] pub fn dot(&self, other: &Self) -> f64 { self.0.dot(&other.0) }

    #[deprecated(note = "Use v0.0.cross(&v1.0) directly")] 
    pub fn cross(&self, other: &Self) -> Self { Self(self.0.cross(&other.0)) }

    #[deprecated(note = "Use v.0.clone() * f64 directly")] pub fn scale(&self, factor: f64) -> Self { Self(self.0.clone() * factor) }

    #[deprecated(note = "Construct via Vector3D::from_f64(b.x-a.x, ...) after migrating points")] 
    pub fn between(a: &crate::CadPoint, b: &crate::CadPoint) -> Self { Self(Vector3D::from_f64(b.x() - a.x(), b.y() - a.y(), b.z() - a.z())) }

    pub fn tolerant_eq(&self, other: &Self, tolerance: f64) -> bool {
        let mut context = ToleranceContext::default();
        context.linear = tolerance;
        TolerantEq::tolerant_eq(&self.0, &other.0, &context)
    }

    #[deprecated(note = "Use Vector3D::normalize_or_zero() then wrap if needed")] 
    pub fn normalize(&self) -> Self { let ctx = ToleranceContext::default(); Self(self.0.normalize_or_zero(&ctx)) }

    /// 直接内部の `Vector3D` 参照を取得（移行用途）。
    pub fn as_inner(&self) -> &Vector3D { &self.0 }
}

impl Add for CadVector { type Output = CadVector; fn add(self, rhs: CadVector) -> CadVector { CadVector(self.0 + rhs.0) } }
impl Sub for CadVector { type Output = CadVector; fn sub(self, rhs: CadVector) -> CadVector { CadVector(self.0 - rhs.0) } }
impl Mul<f64> for CadVector { type Output = CadVector; fn mul(self, scalar: f64) -> CadVector { CadVector(self.0 * scalar) } }
impl Neg for CadVector { type Output = CadVector; fn neg(self) -> CadVector { CadVector(-self.0) } }

// analysis クレート互換（必要なら残す）
impl analysis::NormedVector for CadVector { fn norm(&self) -> f64 { VectorTrait::norm(&self.0) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(deprecated)]
    #[test]
    fn test_cad_vector_operations_deprecated() {
        let v1 = CadVector::new(1.0,0.0,0.0);
        let v2 = CadVector::new(0.0,1.0,0.0);
        let cross = v1.cross(&v2);
        assert!((cross.z() - 1.0).abs() < 1e-10);
        let dot = v1.dot(&v2); assert!(dot.abs() < 1e-10);
    }
}