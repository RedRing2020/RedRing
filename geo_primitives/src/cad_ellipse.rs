/// CAD Ellipse - modelからの楕円構造体移植
///
/// Scalar基礎計算を使用した高精度楕円演算を提供

use geo_core::Scalar;
use crate::{CadPoint, CadVector};

/// CAD Ellipse（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadEllipse {
    center: CadPoint,
    major_axis: CadVector,
    minor_axis: CadVector,
    major_radius: Scalar,
    minor_radius: Scalar,
}

impl CadEllipse {
    /// 楕円を構築（直交チェック付き）
    pub fn new(
        center: CadPoint,
        major_axis: CadVector,
        minor_axis: CadVector,
        major_radius: f64,
        minor_radius: f64,
    ) -> Option<Self> {
        let dot = major_axis.dot(&minor_axis);

        if dot.abs() > geo_core::GEOMETRIC_TOLERANCE {
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
    pub fn major_axis(&self) -> CadVector {
        self.major_axis.clone()
    }

    /// 短軸ベクトルを取得
    pub fn minor_axis(&self) -> CadVector {
        self.minor_axis.clone()
    }

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

        self.center.clone() + self.major_axis.clone() * x + self.minor_axis.clone() * y
    }

    /// 導関数計算
    pub fn derivative(&self, t: f64) -> CadVector {
        let angle = t * 2.0 * std::f64::consts::PI;
        let two_pi = Scalar::new(2.0 * std::f64::consts::PI);
        let dx = (-self.major_radius * Scalar::new(angle.sin()) * two_pi).value();
        let dy = (self.minor_radius * Scalar::new(angle.cos()) * two_pi).value();
        self.major_axis.clone() * dx + self.minor_axis.clone() * dy
    }

    /// 楕円の周長（数値積分による近似）
    pub fn length(&self) -> f64 {
        let major = self.major_axis.clone();
        let minor = self.minor_axis.clone();
        let steps = 360;

        // 速度ベクトル関数
        let evaluate = |theta: f64| {
            let dx = -theta.sin();
            let dy = theta.cos();
            major.clone() * dx + minor.clone() * dy
        };

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
        let major = CadVector::new(2.0, 0.0, 0.0);
        let minor = CadVector::new(0.0, 1.0, 0.0);

        let ellipse = CadEllipse::new(center, major, minor, 2.0, 1.0).unwrap();

        assert_eq!(ellipse.major_radius(), 2.0);
        assert_eq!(ellipse.minor_radius(), 1.0);
    }

    #[test]
    fn test_cad_ellipse_orthogonal_check() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let major = CadVector::new(1.0, 0.0, 0.0);
        let minor = CadVector::new(1.0, 1.0, 0.0); // 非直交

        let ellipse = CadEllipse::new(center, major, minor, 1.0, 1.0);
        assert!(ellipse.is_none()); // 軸が直交していないため作成失敗
    }
}
