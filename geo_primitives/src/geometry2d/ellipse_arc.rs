//! 2D Ellipse Arc implementation
//!
//! 2次元楕円弧の基本実装

use crate::geometry2d::{BBox2D, Ellipse, Point2D, Vector2D};
use geo_foundation::abstract_types::Angle;
use std::f64::consts::PI;

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

/// 幾何計算用の許容誤差
const GEOMETRIC_TOLERANCE: f64 = 1e-10;

/// 2D空間上の楕円弧を表現する構造体
#[derive(Debug, Clone)]
pub struct EllipseArc {
    ellipse: Ellipse,
    start_angle: f64,
    end_angle: f64,
}

impl EllipseArc {
    /// 新しい楕円弧を作成
    pub fn new(
        ellipse: Ellipse,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<Self, EllipseArcError> {
        let normalized_start = Self::normalize_angle(start_angle);
        let normalized_end = Self::normalize_angle(end_angle);

        Ok(Self {
            ellipse,
            start_angle: normalized_start,
            end_angle: normalized_end,
        })
    }

    /// 楕円弧を作成（center, radii, angles）
    pub fn new_from_params(
        center: Point2D,
        major_radius: f64,
        minor_radius: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<Self, EllipseArcError> {
        let ellipse = Ellipse::new(
            center,
            major_radius,
            minor_radius,
            Angle::from_radians(rotation),
        )
        .map_err(|e| EllipseArcError::EllipseError(format!("{}", e)))?;
        Self::new(ellipse, start_angle, end_angle)
    }

    /// 角度を正規化（0 ～ 2π）
    fn normalize_angle(angle: f64) -> f64 {
        let mut normalized = angle % (2.0 * PI);
        if normalized < 0.0 {
            normalized += 2.0 * PI;
        }
        normalized
    }

    /// 角度範囲を計算
    fn calculate_angle_range(&self) -> f64 {
        if self.end_angle >= self.start_angle {
            self.end_angle - self.start_angle
        } else {
            (2.0 * PI - self.start_angle) + self.end_angle
        }
    }

    /// 楕円弧の長さを計算（近似）
    pub fn arc_length(&self) -> f64 {
        let angle_range = self.calculate_angle_range();
        let ellipse_circumference = self.ellipse.circumference();
        ellipse_circumference * (angle_range / (2.0 * PI))
    }

    /// 楕円弧の中心を取得
    pub fn center(&self) -> Point2D {
        self.ellipse.center()
    }

    /// 楕円弧の開始角度を取得
    pub fn start_angle(&self) -> f64 {
        self.start_angle
    }

    /// 楕円弧の終了角度を取得
    pub fn end_angle(&self) -> f64 {
        self.end_angle
    }

    /// 楕円弧の角度範囲を取得
    pub fn angle_range(&self) -> f64 {
        self.calculate_angle_range()
    }

    /// 長軸の半径を取得
    pub fn semi_major_axis(&self) -> f64 {
        self.ellipse.major_radius()
    }

    /// 短軸の半径を取得
    pub fn semi_minor_axis(&self) -> f64 {
        self.ellipse.minor_radius()
    }

    /// 指定された角度での楕円弧上の点を取得
    pub fn point_at_angle(&self, angle: f64) -> Point2D {
        self.ellipse.point_at_angle(angle)
    }

    /// 指定されたパラメータ（0.0-1.0）での楕円弧上の点を取得
    pub fn point_at_parameter(&self, t: f64) -> Point2D {
        let angle = self.start_angle + t * self.calculate_angle_range();
        self.point_at_angle(angle)
    }

    /// 楕円弧の開始点を取得
    pub fn start_point(&self) -> Point2D {
        self.point_at_angle(self.start_angle)
    }

    /// 楕円弧の終了点を取得
    pub fn end_point(&self) -> Point2D {
        self.point_at_angle(self.end_angle)
    }

    /// 楕円弧の中間点を取得
    pub fn mid_point(&self) -> Point2D {
        self.point_at_parameter(0.5)
    }

    /// 指定された角度での楕円弧の接線ベクトルを取得
    pub fn tangent_at_angle(&self, angle: f64) -> Vector2D {
        // 楕円の微分を計算
        let a = self.semi_major_axis();
        let b = self.semi_minor_axis();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        let rotation = self.ellipse.rotation().to_radians();
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        // ローカル座標での接線
        let local_tangent_x = -a * sin_theta;
        let local_tangent_y = b * cos_theta;

        // グローバル座標に変換
        let tangent_x = local_tangent_x * cos_rot - local_tangent_y * sin_rot;
        let tangent_y = local_tangent_x * sin_rot + local_tangent_y * cos_rot;

        Vector2D::new(tangent_x, tangent_y)
            .normalize()
            .unwrap_or(Vector2D::new(1.0, 0.0))
    }

    /// 指定されたパラメータでの楕円弧の接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: f64) -> Vector2D {
        let angle = self.start_angle + t * self.calculate_angle_range();
        self.tangent_at_angle(angle)
    }

    /// 楕円弧の開始点での接線ベクトルを取得
    pub fn start_tangent(&self) -> Vector2D {
        self.tangent_at_angle(self.start_angle)
    }

    /// 楕円弧の終了点での接線ベクトルを取得
    pub fn end_tangent(&self) -> Vector2D {
        self.tangent_at_angle(self.end_angle)
    }

    /// 楕円弧のバウンディングボックスを計算
    pub fn bounding_box(&self) -> BBox2D {
        // 楕円弧上の複数の点をサンプリングしてバウンディングボックスを計算
        let mut points = Vec::new();

        // 開始点と終了点
        points.push(self.start_point());
        points.push(self.end_point());

        // 中間点を追加
        let num_samples = 16;
        for i in 1..num_samples {
            let t = i as f64 / num_samples as f64;
            points.push(self.point_at_parameter(t));
        }

        BBox2D::from_point_array(&points).unwrap_or_else(|| {
            let center = self.center();
            BBox2D::new((center.x(), center.y()), (center.x(), center.y()))
        })
    }

    /// 点が楕円弧上にあるかを判定
    pub fn on_arc(&self, point: &Point2D) -> bool {
        // まず楕円境界上にあるかをチェック
        if !self.ellipse.on_boundary(point) {
            return false;
        }

        // 角度を計算して範囲内かをチェック
        let to_point = Vector2D::new(point.x() - self.center().x(), point.y() - self.center().y());

        let angle = to_point.y().atan2(to_point.x());
        let normalized_angle = Self::normalize_angle(angle);

        self.angle_in_range(normalized_angle)
    }

    /// 角度が楕円弧の範囲内にあるかを判定
    pub fn angle_in_range(&self, angle: f64) -> bool {
        let normalized = Self::normalize_angle(angle);

        if self.end_angle >= self.start_angle {
            normalized >= self.start_angle && normalized <= self.end_angle
        } else {
            normalized >= self.start_angle || normalized <= self.end_angle
        }
    }

    /// 楕円弧を平行移動
    pub fn translate(&self, vector: &Vector2D) -> Self {
        let translated_ellipse = self.ellipse.translate(vector);
        Self::new(translated_ellipse, self.start_angle, self.end_angle).unwrap()
    }

    /// 楕円弧をスケール
    pub fn scale(&self, factor: f64) -> Self {
        let scaled_ellipse = self.ellipse.scale(factor);
        Self::new(scaled_ellipse, self.start_angle, self.end_angle).unwrap()
    }

    /// 楕円弧が完全な楕円（360度）かを判定
    pub fn is_full_ellipse(&self) -> bool {
        (self.calculate_angle_range() - 2.0 * PI).abs() <= GEOMETRIC_TOLERANCE
    }

    /// 楕円弧が半楕円（180度）かを判定
    pub fn is_semi_ellipse(&self) -> bool {
        (self.calculate_angle_range() - PI).abs() <= GEOMETRIC_TOLERANCE
    }

    /// 楕円弧上の点列を生成
    pub fn generate_points(&self, num_points: usize) -> Vec<Point2D> {
        let mut points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let t = if num_points == 1 {
                0.5
            } else {
                i as f64 / (num_points - 1) as f64
            };
            points.push(self.point_at_parameter(t));
        }

        points
    }

    /// 基本となる楕円を取得
    pub fn ellipse(&self) -> &Ellipse {
        &self.ellipse
    }
}

// 手動でPartialEqを実装
impl PartialEq for EllipseArc {
    fn eq(&self, other: &Self) -> bool {
        self.ellipse == other.ellipse
            && (self.start_angle - other.start_angle).abs() < GEOMETRIC_TOLERANCE
            && (self.end_angle - other.end_angle).abs() < GEOMETRIC_TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_arc_creation() {
        let center = Point2D::new(0.0, 0.0);
        let arc = EllipseArc::new_from_params(center, 3.0, 2.0, 0.0, 0.0, PI).unwrap();

        assert_eq!(arc.center(), center);
        assert_eq!(arc.start_angle(), 0.0);
        assert_eq!(arc.end_angle(), PI);
    }

    #[test]
    fn test_ellipse_arc_points() {
        let center = Point2D::new(0.0, 0.0);
        let arc = EllipseArc::new_from_params(center, 3.0, 2.0, 0.0, 0.0, PI).unwrap();

        let start_point = arc.start_point();
        assert!((start_point.x() - 3.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((start_point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);

        let end_point = arc.end_point();
        assert!((end_point.x() - (-3.0)).abs() < GEOMETRIC_TOLERANCE);
        assert!((end_point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_ellipse_arc_length() {
        let center = Point2D::new(0.0, 0.0);

        // 小さな楕円弧（π/4ラジアン = 45度）
        let arc = EllipseArc::new_from_params(center, 3.0, 2.0, 0.0, 0.0, PI / 4.0).unwrap();
        let arc_length = arc.arc_length();

        // 長さは正の値である必要があります
        assert!(arc_length > 0.0, "楕円弧の長さは正の値である必要があります");
    }

    #[test]
    fn test_angle_in_range() {
        let center = Point2D::new(0.0, 0.0);
        let arc =
            EllipseArc::new_from_params(center, 3.0, 2.0, 0.0, PI / 4.0, 3.0 * PI / 4.0).unwrap();

        assert!(arc.angle_in_range(PI / 2.0));
        assert!(!arc.angle_in_range(0.0));
        assert!(!arc.angle_in_range(PI));
    }

    #[test]
    fn test_generate_points() {
        let center = Point2D::new(0.0, 0.0);
        let arc = EllipseArc::new_from_params(center, 3.0, 2.0, 0.0, 0.0, PI).unwrap();

        let points = arc.generate_points(5);
        assert_eq!(points.len(), 5);

        // 最初の点は開始点
        assert!((points[0].distance_to(&arc.start_point())).abs() < GEOMETRIC_TOLERANCE);

        // 最後の点は終了点
        assert!((points[4].distance_to(&arc.end_point())).abs() < GEOMETRIC_TOLERANCE);
    }
}
