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
    /// 標準的な円のパラメータ化に合わせて、適切な基底ベクトルを返す
    pub fn orthonormal_basis(&self) -> (crate::CadVector, crate::CadVector) {
        let n = &self.0;

        // Z軸上向きの法線の場合の特別処理
        if (n.z().abs() - 1.0).abs() < 1e-10 {
            if n.z() > 0.0 {
                // 上向きZ軸: (1,0,0)と(0,1,0)を基底とする
                return (crate::CadVector::new(1.0, 0.0, 0.0), crate::CadVector::new(0.0, 1.0, 0.0));
            } else {
                // 下向きZ軸: 右手系を維持
                return (crate::CadVector::new(-1.0, 0.0, 0.0), crate::CadVector::new(0.0, 1.0, 0.0));
            }
        }

        // 一般的な場合: Gram-Schmidt過程を使用
        let reference = if n.x().abs() < n.y().abs() && n.x().abs() < n.z().abs() {
            crate::CadVector::new(1.0, 0.0, 0.0)
        } else if n.y().abs() < n.z().abs() {
            crate::CadVector::new(0.0, 1.0, 0.0)
        } else {
            crate::CadVector::new(0.0, 0.0, 1.0)
        };

        let v1 = reference.cross(n).normalize();
        let v2 = n.cross(&v1).normalize();

        (v1, v2)
    }
}

