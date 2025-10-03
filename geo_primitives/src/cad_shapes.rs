/// CAD高次構造体 - modelからの幾何形状移植
/// 
/// Phase 2: 高次構造体をgeo_primitivesに移動
/// Scalar基礎計算を維持しつつ、CAD機能を提供

use geo_core::Scalar;
use crate::cad_primitives::{CadPoint, CadVector, CadDirection};

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
        const EPSILON: f64 = 1e-10;
        if major_axis.dot(&minor_axis).abs() > EPSILON {
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
    fn test_cad_circle_basic() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let normal = CadDirection::from_vector(CadVector::new(0.0, 0.0, 1.0)).unwrap();
        let circle = CadCircle::new(center, 1.0, normal);

        assert_eq!(circle.radius(), 1.0);
        assert!((circle.length() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
    }

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
}