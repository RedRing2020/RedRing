//! 3D Ellipse Arc implementation (simplified)
//!
//! 3次元楕円弧の基本実装（2D Ellipseベース）

use crate::geometry2d::Ellipse; // 2D Ellipse を使用（現在はf64固定実装）
use crate::geometry3d::Point;
use analysis::abstract_types::angle::Angle;

/// 楕円弧関連のエラー
#[derive(Debug, Clone, PartialEq)]
pub enum EllipseArcError {
    /// 楕円作成エラー
    EllipseError(String),
    /// 角度が無効
    InvalidAngle,
}

impl std::fmt::Display for EllipseArcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EllipseArcError::EllipseError(msg) => write!(f, "Ellipse error: {}", msg),
            EllipseArcError::InvalidAngle => write!(f, "Invalid angle"),
        }
    }
}

impl std::error::Error for EllipseArcError {}

/// 3D空間上の楕円弧を表現する構造体（現在は2D Ellipseベース）
#[derive(Debug, Clone)]
pub struct EllipseArc {
    ellipse: Ellipse, // f64固定の2D Ellipse
    start_angle: Angle<f64>,
    end_angle: Angle<f64>,
}

impl EllipseArc {
    /// 新しい楕円弧を作成
    pub fn new(
        ellipse: Ellipse,
        start_angle: Angle<f64>,
        end_angle: Angle<f64>,
    ) -> Result<Self, EllipseArcError> {
        let normalized_start = Self::normalize_angle(start_angle.to_radians());
        let normalized_end = Self::normalize_angle(end_angle.to_radians());

        Ok(Self {
            ellipse,
            start_angle: Angle::from_radians(normalized_start),
            end_angle: Angle::from_radians(normalized_end),
        })
    }

    /// XY平面上の楕円弧を作成
    pub fn xy_plane(
        center: Point<f64>,
        major_radius: f64,
        minor_radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<Self, EllipseArcError> {
        // 3D Point から 2D Point に変換（Z座標を無視）
        let center_2d = crate::geometry2d::Point2DF64::new(center.x(), center.y());
        let ellipse = Ellipse::new(
            center_2d,
            major_radius,
            minor_radius,
            Angle::from_radians(0.0),
        )
        .map_err(|e| EllipseArcError::EllipseError(format!("{}", e)))?;
        Self::new(
            ellipse,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    /// 角度を正規化（0 ～ 2π）
    fn normalize_angle(angle: f64) -> f64 {
        let two_pi = 2.0 * std::f64::consts::PI;
        let mut normalized = angle % two_pi;
        if normalized < 0.0 {
            normalized += two_pi;
        }
        normalized
    }

    /// 角度範囲を計算
    fn calculate_angle_range(&self) -> f64 {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();
        if end_rad >= start_rad {
            end_rad - start_rad
        } else {
            (2.0 * std::f64::consts::PI - start_rad) + end_rad
        }
    }

    /// 楕円弧の長さを計算（近似）
    pub fn arc_length(&self) -> f64 {
        let angle_range = self.calculate_angle_range();
        let ellipse_circumference = self.ellipse.circumference();
        ellipse_circumference * (angle_range / (2.0 * std::f64::consts::PI))
    }

    /// 楕円弧の中心を取得（2D→3D変換）
    pub fn center(&self) -> Point<f64> {
        let center_2d = self.ellipse.center();
        Point::new(center_2d.x(), center_2d.y(), 0.0)
    }

    /// 楕円弧の開始角度を取得
    pub fn start_angle(&self) -> f64 {
        self.start_angle.to_radians()
    }

    /// 楕円弧の終了角度を取得
    pub fn end_angle(&self) -> f64 {
        self.end_angle.to_radians()
    }

    /// 指定された角度での楕円弧上の点を取得（2D→3D変換）
    pub fn point_at_angle(&self, angle: f64) -> Point<f64> {
        let point_2d = self.ellipse.point_at_angle(angle);
        Point::new(point_2d.x(), point_2d.y(), 0.0)
    }

    /// 指定されたパラメータ（0.0-1.0）での楕円弧上の点を取得
    pub fn point_at_parameter(&self, t: f64) -> Point<f64> {
        let angle = self.start_angle.to_radians() + t * self.calculate_angle_range();
        self.point_at_angle(angle)
    }

    /// 楕円弧の開始点を取得
    pub fn start_point(&self) -> Point<f64> {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 楕円弧の終了点を取得
    pub fn end_point(&self) -> Point<f64> {
        self.point_at_angle(self.end_angle.to_radians())
    }

    /// 基本となる楕円を取得
    pub fn ellipse(&self) -> &Ellipse {
        &self.ellipse
    }
}

// 手動でPartialEqを実装
impl PartialEq for EllipseArc {
    fn eq(&self, other: &Self) -> bool {
        let tolerance = 1e-10;
        self.ellipse == other.ellipse
            && (self.start_angle.to_radians() - other.start_angle.to_radians()).abs() < tolerance
            && (self.end_angle.to_radians() - other.end_angle.to_radians()).abs() < tolerance
    }
}

// 型エイリアス
pub type EllipseArcF64 = EllipseArc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_arc_creation() {
        let center = Point::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, std::f64::consts::PI).unwrap();

        assert_eq!(arc.center().x(), center.x());
        assert_eq!(arc.center().y(), center.y());
        assert_eq!(arc.start_angle(), 0.0);
    }

    #[test]
    fn test_ellipse_arc_points() {
        let center = Point::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, std::f64::consts::PI).unwrap();

        let start_point = arc.start_point();
        let tolerance = 1e-10;
        assert!((start_point.x() - 3.0).abs() < tolerance);
        assert!((start_point.y() - 0.0).abs() < tolerance);
    }
}
