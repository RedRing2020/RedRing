/// CAD Direction - modelからの正規化方向ベクトル構造体移植
/// 
/// 正規化されたベクトルを表現し、CAD計算での方向性を保証

/// CAD用正規化方向ベクトル（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadDirection(crate::CadVector);

impl PartialEq for CadDirection {
    fn eq(&self, other: &Self) -> bool {
        self.0.tolerant_eq(&other.0, 1e-10)
    }
}

impl CadDirection {
    pub fn from_vector(v: crate::CadVector) -> Option<Self> {
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

    pub fn as_vector(&self) -> crate::CadVector {
        self.0.clone()
    }

    pub fn to_vector(&self) -> crate::CadVector {
        self.0.clone()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.0.dot(&other.0)
    }

    pub fn cross(&self, other: &Self) -> crate::CadVector {
        self.0.cross(&other.0)
    }

    /// 直交正規基底を計算（円の評価で使用）
    pub fn orthonormal_basis(&self) -> (crate::CadVector, crate::CadVector) {
        let n = &self.0;
        
        // 最も小さい成分を見つけて直交ベクトルを作成
        let u = if n.x().abs() < n.y().abs() && n.x().abs() < n.z().abs() {
            crate::CadVector::new(1.0, 0.0, 0.0)
        } else if n.y().abs() < n.z().abs() {
            crate::CadVector::new(0.0, 1.0, 0.0)
        } else {
            crate::CadVector::new(0.0, 0.0, 1.0)
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
    fn test_cad_direction() {
        let v = crate::CadVector::new(3.0, 4.0, 0.0);
        let dir = CadDirection::from_vector(v).unwrap();
        
        // 正規化されているか確認
        let norm = (dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_orthonormal_basis() {
        let dir = CadDirection::from_vector(crate::CadVector::new(0.0, 0.0, 1.0)).unwrap();
        let (u, v) = dir.orthonormal_basis();
        
        // 直交性確認
        assert!(u.dot(&v).abs() < 1e-10);
        // 正規化確認
        assert!((u.norm() - 1.0).abs() < 1e-10);
        assert!((v.norm() - 1.0).abs() < 1e-10);
    }
}