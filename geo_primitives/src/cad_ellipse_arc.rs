/// CAD EllipseArc - modelからの楕円弧構造体移植
/// 
/// Scalar基礎計算を使用した高精度楕円弧演算を提供

use geo_core::Scalar;
use crate::{CadPoint, CadVector};

/// CAD EllipseArc（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadEllipseArc {
    center: CadPoint,
    major_axis: CadVector,
    minor_axis: CadVector,
    major_radius: Scalar,
    minor_radius: Scalar,
    start_angle: f64, // in radians
    end_angle: f64,   // in radians
}

impl CadEllipseArc {
    pub fn new(
        center: CadPoint,
        major_axis: CadVector,
        minor_axis: CadVector,
        major_radius: f64,
        minor_radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Option<Self> {
        if major_axis.dot(&minor_axis).abs() > geo_core::GEOMETRIC_TOLERANCE {
            return None; // 軸が直交していない
        }
        if end_angle <= start_angle {
            return None; // 角度範囲が不正
        }
        Some(Self {
            center,
            major_axis,
            minor_axis,
            major_radius: Scalar::new(major_radius),
            minor_radius: Scalar::new(minor_radius),
            start_angle,
            end_angle,
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

    /// 開始角度を取得
    pub fn start_angle(&self) -> f64 {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> f64 {
        self.end_angle
    }

    /// パラメトリック評価
    pub fn evaluate(&self, t: f64) -> CadPoint {
        let theta = self.start_angle + t * (self.end_angle - self.start_angle);
        let x = theta.cos();
        let y = theta.sin();

        self.center.clone() + self.major_axis.clone() * x + self.minor_axis.clone() * y
    }

    /// 導関数計算
    pub fn derivative(&self, t: f64) -> CadVector {
        let angle = self.start_angle + t * (self.end_angle - self.start_angle);
        let d_angle = self.end_angle - self.start_angle;

        let dx = (-self.major_radius * Scalar::new(angle.sin()) * Scalar::new(d_angle)).value();
        let dy = (self.minor_radius * Scalar::new(angle.cos()) * Scalar::new(d_angle)).value();
        self.major_axis.clone() * dx + self.minor_axis.clone() * dy
    }

    /// 3D楕円弧の長さを数値積分で近似
    pub fn length(&self) -> f64 {
        let major = self.major_axis.clone();
        let minor = self.minor_axis.clone();
        let start = self.start_angle;
        let end = self.end_angle;
        let steps = 360; // 内部変数として分割数を固定

        // 速度ベクトル関数
        let evaluate = |theta: f64| {
            let dx = -theta.sin();
            let dy = theta.cos();
            major.clone() * dx + minor.clone() * dy
        };

        analysis::newton_arc_length(evaluate, start, end, steps)
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
    fn test_cad_ellipse_arc_basic() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let major = CadVector::new(1.0, 0.0, 0.0);
        let minor = CadVector::new(0.0, 1.0, 0.0);

        let arc = CadEllipseArc::new(
            center, major, minor,
            1.0, 1.0,
            0.0, std::f64::consts::PI / 2.0
        ).unwrap();

        assert_eq!(arc.start_angle(), 0.0);
        assert!((arc.end_angle() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_cad_ellipse_arc_invalid_angles() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let major = CadVector::new(1.0, 0.0, 0.0);
        let minor = CadVector::new(0.0, 1.0, 0.0);

        // 終了角度が開始角度より小さい場合
        let arc = CadEllipseArc::new(
            center, major, minor,
            1.0, 1.0,
            std::f64::consts::PI, 0.0
        );

        assert!(arc.is_none()); // 不正な角度範囲のため作成失敗
    }
}