/// CAD Direction - modelからの正規化方向ベクトル構造体移植
/// 
/// 正規化されたベクトルを表現し、CAD計算での方向性を保証

use geo_core::{Vector3D, ToleranceContext, Vector as VectorTrait, TolerantEq};

/// CAD用正規化方向ベクトル（modelからの移植 / Vector3D ベース移行）
#[derive(Debug, Clone)]
pub struct CadDirection(Vector3D);

impl PartialEq for CadDirection {
    fn eq(&self, other: &Self) -> bool {
        let mut ctx = ToleranceContext::default();
        ctx.linear = 1e-10;
        TolerantEq::tolerant_eq(&self.0, &other.0, &ctx)
    }
}

impl CadDirection {
    #[deprecated(note = "Use from_vector_vec3 with Vector3D directly")] 
    pub fn from_vector(v: crate::CadVector) -> Option<Self> {
        Self::from_vector_vec3(v.0.clone())
    }

    pub fn from_vector_vec3(v: Vector3D) -> Option<Self> { let ctx = ToleranceContext::default(); if VectorTrait::norm(&v) == 0.0 { None } else { Some(CadDirection(v.normalize_or_zero(&ctx))) } }

    pub fn x(&self) -> f64 { self.0.x_val() }
    pub fn y(&self) -> f64 { self.0.y_val() }
    pub fn z(&self) -> f64 { self.0.z_val() }

    pub fn normalize(&self) -> CadDirection { self.clone() }

    pub fn as_vector(&self) -> Vector3D { self.0.clone() }
    pub fn to_vector(&self) -> Vector3D { self.0.clone() }

    pub fn dot(&self, other: &Self) -> f64 { self.0.dot(&other.0) }

    pub fn cross(&self, other: &Self) -> Vector3D { self.0.cross(&other.0) }

    /// 直交正規基底を計算（円の評価で使用）
    #[deprecated(note = "Use orthonormal_basis_vec3 for Vector3D outputs")] 
    pub fn orthonormal_basis(&self) -> (crate::CadVector, crate::CadVector) {
        let (u,v) = self.orthonormal_basis_vec3();
        (crate::CadVector(u.clone()), crate::CadVector(v.clone()))
    }

    pub fn orthonormal_basis_vec3(&self) -> (Vector3D, Vector3D) {
        let n = &self.0;
        let z_aligned = n.z_val().abs() > 0.999_999;
        let mut u = if z_aligned {
            Vector3D::from_f64(1.0, 0.0, 0.0)
        } else {
            let cand = Vector3D::from_f64(-n.y_val(), n.x_val(), 0.0);
            if VectorTrait::norm(&cand) < 1e-14 { Vector3D::from_f64(0.0, 1.0, 0.0) } else { 
                let ctx = ToleranceContext::default();
                cand.normalize_or_zero(&ctx)
            }
        };
        if !z_aligned {
            let ctx = ToleranceContext::default();
            u = u.normalize_or_zero(&ctx);
        }
        let v = {
            let cross = n.cross(&u);
            let ctx = ToleranceContext::default();
            cross.normalize_or_zero(&ctx)
        };
        (u, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cad_direction() {
    let v = Vector3D::from_f64(3.0, 4.0, 0.0);
    let dir = CadDirection::from_vector_vec3(v).unwrap();
        
        // 正規化されているか確認
        let norm = (dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_orthonormal_basis() {
    let dir = CadDirection::from_vector_vec3(Vector3D::from_f64(0.0, 0.0, 1.0)).unwrap();
    let (u, v) = dir.orthonormal_basis_vec3();
        
        // 直交性確認
    assert!(u.dot(&v).abs() < 1e-10);
        use geo_core::Vector as _; // bring trait
        assert!((VectorTrait::norm(&u) - 1.0).abs() < 1e-10);
        assert!((VectorTrait::norm(&v) - 1.0).abs() < 1e-10);
    }
}