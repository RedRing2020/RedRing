/// CAD Circle - modelからの円構造体移植
/// 
/// Scalar基礎計算を使用した高精度円演算を提供

use geo_core::Vector3D;
use crate::{CadPoint, CadDirection};

/// CAD Circle（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadCircle {
    center: CadPoint,
    radius: f64,               // f64 中心化
    normal: CadDirection,
    // 評価用直交基底 (normal に依存) をキャッシュ
    basis_u: Vector3D,
    basis_v: Vector3D,
}

impl CadCircle {
    pub fn new(center: CadPoint, radius: f64, normal: CadDirection) -> Self {
    let (u, v) = normal.orthonormal_basis_vec3();
        Self { center, radius, normal, basis_u: u, basis_v: v }
    }

    /// 中心点を取得
    pub fn center(&self) -> CadPoint {
        self.center.clone()
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 { self.radius }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> CadDirection {
        self.normal.clone()
    }

    /// パラメトリック評価
    pub fn evaluate(&self, t: f64) -> CadPoint { self.evaluate_f64(t) }

    pub fn evaluate_f64(&self, t: f64) -> CadPoint {
        let theta = t * std::f64::consts::TAU; // 2πt
        let (s, c) = theta.sin_cos();
    // CadPoint + Vector3D サポートが無いので一旦 f64 で構築
    let p = self.center.clone();
    let nx = p.x() + self.basis_u.x_val() * (self.radius * c) + self.basis_v.x_val() * (self.radius * s);
    let ny = p.y() + self.basis_u.y_val() * (self.radius * c) + self.basis_v.y_val() * (self.radius * s);
    let nz = p.z() + self.basis_u.z_val() * (self.radius * c) + self.basis_v.z_val() * (self.radius * s);
    CadPoint::new(nx, ny, nz)
    }

    /// 導関数計算
    pub fn derivative(&self, t: f64) -> Vector3D { self.derivative_f64(t) }

    pub fn derivative_f64(&self, t: f64) -> Vector3D {
        let theta = t * std::f64::consts::TAU;
        let (s, c) = theta.sin_cos();
        // d/dt (radius * (c,u) + s,v) with θ=2πt => dθ/dt = 2π
        let dtheta = std::f64::consts::TAU;
        // 位置: r(c u + s v) → 微分: r * dθ * (-s u + c v)
        self.basis_u.clone() * (-self.radius * s * dtheta) + self.basis_v.clone() * (self.radius * c * dtheta)
    }

    /// 円周長
    pub fn length(&self) -> f64 { std::f64::consts::TAU * self.radius }

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
    let normal = CadDirection::from_vector_vec3(Vector3D::from_f64(0.0, 0.0, 1.0)).unwrap();
        let circle = CadCircle::new(center, 1.0, normal);

        assert_eq!(circle.radius(), 1.0);
        assert!((circle.length() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_cad_circle_evaluation() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
    let normal = CadDirection::from_vector_vec3(Vector3D::from_f64(0.0, 0.0, 1.0)).unwrap();
        let circle = CadCircle::new(center, 1.0, normal);

        let point = circle.evaluate(0.0); // t=0の点
        assert!((point.x() - 1.0).abs() < 1e-10);
        assert!(point.y().abs() < 1e-10);
    }
}