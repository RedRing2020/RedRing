/// CAD Circle - modelからの円構造体移植
///
/// Scalar基礎計算を使用した高精度円演算を提供

// geo_core参照を削除 - f64を直接使用
use crate::{CadPoint, CadVector, CadDirection};

/// CAD Circle（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadCircle {
    center: CadPoint,
    radius: Scalar,
    normal: CadDirection,
}

impl CadCircle {
    pub fn new(center: CadPoint, radius: f64, normal: CadDirection) -> Self {
        Self { center, radius: Scalar::new(radius), normal }
    }

    /// 中心点を取得
    pub fn center(&self) -> CadPoint {
        self.center.clone()
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 {
        self.radius.value()
    }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> CadDirection {
        self.normal.clone()
    }

    /// パラメトリック評価
    pub fn evaluate(&self, t: f64) -> CadPoint {
        let theta = t * 2.0 * std::f64::consts::PI;
        let (u_vec, v_vec) = self.normal.orthonormal_basis();

        // パラメトリック円の評価
        let u = (self.radius * Scalar::new(theta.cos())).value();
        let v = (self.radius * Scalar::new(theta.sin())).value();

        self.center.clone() + u_vec * u + v_vec * v
    }

    /// 導関数計算
    pub fn derivative(&self, t: f64) -> CadVector {
        let theta = t * 2.0 * std::f64::consts::PI;
        let (u, v) = self.normal.orthonormal_basis();
        let two_pi = Scalar::new(2.0 * std::f64::consts::PI);
        let dx = (-self.radius * Scalar::new(theta.sin()) * two_pi).value();
        let dy = (self.radius * Scalar::new(theta.cos()) * two_pi).value();
        u * dx + v * dy
    }

    /// 円周長
    pub fn length(&self) -> f64 {
        (Scalar::new(2.0 * std::f64::consts::PI) * self.radius).value()
    }

    /// ドメイン
    pub fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}

// テストコードはunit_tests/cad_tests.rsに移動
