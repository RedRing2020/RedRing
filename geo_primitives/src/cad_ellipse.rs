/// CAD Ellipse - modelからの楕円構造体移植
/// 
/// Scalar基礎計算を使用した高精度楕円演算を提供

use geo_core::{Scalar, Vector3D, Vector as VectorTrait};
use crate::CadPoint;

/// CAD Ellipse（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadEllipse {
    center: CadPoint,
    major_axis: Vector3D,
    minor_axis: Vector3D,
    major_radius: Scalar,
    minor_radius: Scalar,
}

impl CadEllipse {
    /// 楕円を構築（直交チェック付き）
    pub fn new(
        center: CadPoint,
    major_axis: Vector3D,
    minor_axis: Vector3D,
        major_radius: f64,
        minor_radius: f64,
    ) -> Option<Self> {
    let dot = VectorTrait::dot(&major_axis, &minor_axis);
        const EPSILON: f64 = 1e-10;

        if dot.abs() > EPSILON {
            return None; // 軸が直交していない
        }

        Some(Self {
            center,
            major_axis,
            minor_axis,
            major_radius: Scalar::new(major_radius),
            minor_radius: Scalar::new(minor_radius),
        })
    }

    /// 中心点を取得
    pub fn center(&self) -> CadPoint {
        self.center.clone()
    }

    /// 長軸ベクトルを取得
    pub fn major_axis(&self) -> Vector3D { self.major_axis.clone() }

    /// 短軸ベクトルを取得
    pub fn minor_axis(&self) -> Vector3D { self.minor_axis.clone() }

    /// 長軸半径を取得
    pub fn major_radius(&self) -> f64 {
        self.major_radius.value()
    }

    /// 短軸半径を取得
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius.value()
    }

    /// パラメトリック評価
    pub fn evaluate(&self, t: f64) -> CadPoint {
        let theta = t * 2.0 * std::f64::consts::PI;
        let x = theta.cos();
        let y = theta.sin();

    let p = self.center.clone();
    let nx = p.x() + self.major_axis.x_val() * x + self.minor_axis.x_val() * y;
    let ny = p.y() + self.major_axis.y_val() * x + self.minor_axis.y_val() * y;
    let nz = p.z() + self.major_axis.z_val() * x + self.minor_axis.z_val() * y;
    CadPoint::new(nx, ny, nz)
    }

    /// 導関数計算
    pub fn derivative(&self, t: f64) -> Vector3D {
        let angle = t * 2.0 * std::f64::consts::PI;
        let two_pi = 2.0 * std::f64::consts::PI;
        let dx = -self.major_radius.value() * angle.sin() * two_pi;
        let dy =  self.minor_radius.value() * angle.cos() * two_pi;
        self.major_axis.clone() * dx + self.minor_axis.clone() * dy
    }

    /// 楕円の周長（数値積分による近似）
    pub fn length(&self) -> f64 {
    let major = self.major_axis.clone();
    let minor = self.minor_axis.clone();
        let steps = 360;

        // 速度ベクトル関数
        let evaluate = |theta: f64| { let dx = -theta.sin(); let dy = theta.cos(); major.clone() * dx + minor.clone() * dy };

        analysis::newton_arc_length(evaluate, 0.0, 2.0 * std::f64::consts::PI, steps)
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
    fn test_cad_ellipse_basic() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
    let major = Vector3D::from_f64(2.0, 0.0, 0.0);
    let minor = Vector3D::from_f64(0.0, 1.0, 0.0);

        let ellipse = CadEllipse::new(center, major, minor, 2.0, 1.0).unwrap();

        assert_eq!(ellipse.major_radius(), 2.0);
        assert_eq!(ellipse.minor_radius(), 1.0);
    }

    #[test]
    fn test_cad_ellipse_orthogonal_check() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
    let major = Vector3D::from_f64(1.0, 0.0, 0.0);
    let minor = Vector3D::from_f64(1.0, 1.0, 0.0); // 非直交

        let ellipse = CadEllipse::new(center, major, minor, 1.0, 1.0);
        assert!(ellipse.is_none()); // 軸が直交していないため作成失敗
    }
}