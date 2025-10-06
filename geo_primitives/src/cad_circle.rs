/// CAD Circle - modelからの円構造体移植
/// 
/// Scalar基礎計算を使用した高精度円演算を提供

use geo_core::Scalar;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cad_circle_basic() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let normal = CadDirection::from_vector(CadVector::new(0.0, 0.0, 1.0)).unwrap();
        let circle = CadCircle::new(center, 1.0, normal);

        assert_eq!(circle.radius(), 1.0);
        assert!((circle.length() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_cad_circle_evaluation() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let normal = CadDirection::from_vector(CadVector::new(0.0, 0.0, 1.0)).unwrap();
        let circle = CadCircle::new(center, 1.0, normal);

        let point = circle.evaluate(0.0); // t=0の点
        assert!((point.x() - 1.0).abs() < 1e-10);
        assert!(point.y().abs() < 1e-10);
        assert!(point.z().abs() < 1e-10);
    }
}