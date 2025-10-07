/// CAD EllipseArc - modelからの楕円弧構造体移植
///
/// Scalar基礎計算を使用した高精度楕円弧演算を提供

// geo_core参照を削除 - f64を直接使用
use crate::{CadPoint, CadVector};

/// CAD EllipseArc（modelからの移植）
#[derive(Debug, Clone)]
pub struct CadEllipseArc {
    center: CadPoint,
    major_axis: CadVector,
    minor_axis: CadVector,
    major_radius: f64,
    minor_radius: f64,
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
        const GEOMETRIC_TOLERANCE: f64 = 1e-6;
        if major_axis.dot(&minor_axis).abs() > GEOMETRIC_TOLERANCE {
            return None; // 軸が直交していない
        }
        if end_angle <= start_angle {
            return None; // 角度範囲が不正
        }
        Some(Self {
            center,
            major_axis,
            minor_axis,
            major_radius,
            minor_radius,
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
        self.major_radius
    }

    /// 短軸半径を取得
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius
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

        let dx = -self.major_radius * angle.sin() * d_angle;
        let dy = self.minor_radius * angle.cos() * d_angle;
        self.major_axis.clone() * dx + self.minor_axis.clone() * dy
    }

    /// 3D楕円弧の長さを数値積分で近似
    pub fn length(&self) -> f64 {
        // Analysis dependency removed - using direct Ramanujan approximation
        let start = self.start_angle;
        let end = self.end_angle;
        // Analysis dependency removed - using direct Ramanujan approximation

        // TODO: arc length calculation needs to be moved to geo_algorithms
        // For now, use Ramanujan's approximation for ellipse perimeter
        let arc_fraction = (end - start) / (2.0 * std::f64::consts::PI);
        let three = 3.0;
        let pi = std::f64::consts::PI;
        let full_perimeter = pi * (three * (self.major_radius + self.minor_radius) -
            ((three * self.major_radius + self.minor_radius) * (self.major_radius + three * self.minor_radius)).sqrt());
        full_perimeter * arc_fraction
    }

    /// ドメイン
    pub fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}


