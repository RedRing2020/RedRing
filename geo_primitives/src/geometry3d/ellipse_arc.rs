//! 3D Ellipse Arc implementation
//!
//! 3次元楕円弧の基本実装

use crate::geometry3d::{BBox3D, Ellipse, Point3D, Vector3D};
use geo_foundation::abstract_types::geometry::common::{
    AnalyticalCurve, CurveAnalysis3D, CurveType, DifferentialGeometry,
};
use geo_foundation::abstract_types::geometry::Direction;
use geo_foundation::abstract_types::Angle;
use geo_foundation::constants::precision::{GEOMETRIC_TOLERANCE, PI};

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

/// 3D空間上の楕円弧を表現する構造体
#[derive(Debug, Clone)]
pub struct EllipseArc {
    ellipse: Ellipse,
    start_angle: geo_foundation::abstract_types::Angle<f64>,
    end_angle: geo_foundation::abstract_types::Angle<f64>,
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
        center: Point3D,
        major_radius: f64,
        minor_radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<Self, EllipseArcError> {
        let ellipse = Ellipse::xy_plane(center, major_radius, minor_radius)
            .map_err(|e| EllipseArcError::EllipseError(format!("{}", e)))?;
        Self::new(
            ellipse,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
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
    pub fn center(&self) -> Point3D {
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

    /// 指定された角度での楕円弧上の点を取得
    pub fn point_at_angle(&self, angle: f64) -> Point3D {
        self.ellipse.point_at_angle(angle)
    }

    /// 指定されたパラメータ（0.0-1.0）での楕円弧上の点を取得
    pub fn point_at_parameter(&self, t: f64) -> Point3D {
        let angle = self.start_angle + t * self.calculate_angle_range();
        self.point_at_angle(angle)
    }

    /// 楕円弧の開始点を取得
    pub fn start_point(&self) -> Point3D {
        self.point_at_angle(self.start_angle)
    }

    /// 楕円弧の終了点を取得
    pub fn end_point(&self) -> Point3D {
        self.point_at_angle(self.end_angle)
    }

    /// 楕円弧の中間点を取得
    pub fn mid_point(&self) -> Point3D {
        self.point_at_parameter(0.5)
    }

    /// 指定された角度での楕円弧の接線ベクトルを取得
    pub fn tangent_at_angle(&self, angle: f64) -> Vector3D {
        // 楕円の微分を計算
        let u_vec = self.ellipse.u_axis().to_vector();
        let v_vec = self.ellipse.v_axis().to_vector();

        let cos_t = angle.cos();
        let sin_t = angle.sin();

        u_vec * (-self.ellipse.major_radius() * sin_t)
            + v_vec * (self.ellipse.minor_radius() * cos_t)
    }

    /// 指定されたパラメータでの楕円弧の接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: f64) -> Vector3D {
        let angle = self.start_angle + t * self.calculate_angle_range();
        self.tangent_at_angle(angle)
    }

    /// 楕円弧の開始点での接線ベクトルを取得
    pub fn start_tangent(&self) -> Vector3D {
        self.tangent_at_angle(self.start_angle)
    }

    /// 楕円弧の終了点での接線ベクトルを取得
    pub fn end_tangent(&self) -> Vector3D {
        self.tangent_at_angle(self.end_angle)
    }

    /// 楕円弧のバウンディングボックスを計算
    pub fn bounding_box(&self) -> BBox3D {
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

        BBox3D::from_point_array(&points)
            .unwrap_or_else(|| BBox3D::from_two_points(self.center(), self.center()))
    }

    /// 点が楕円弧上にあるかを判定
    pub fn on_arc(&self, point: &Point3D) -> bool {
        // まず楕円境界上にあるかをチェック
        if !self.ellipse.on_boundary(point) {
            return false;
        }

        // 角度を計算して範囲内かをチェック
        let to_point = Vector3D::new(
            point.x() - self.center().x(),
            point.y() - self.center().y(),
            point.z() - self.center().z(),
        );

        let u_coord = to_point.dot(&self.ellipse.u_axis().to_vector());
        let v_coord = to_point.dot(&self.ellipse.v_axis().to_vector());

        let angle = v_coord.atan2(u_coord);
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
    pub fn translate(&self, vector: &Vector3D) -> Self {
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
    pub fn generate_points(&self, num_points: usize) -> Vec<Point3D> {
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
        let center = Point3D::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, PI).unwrap();

        assert_eq!(arc.center(), center);
        assert_eq!(arc.start_angle(), 0.0);
        assert_eq!(arc.end_angle(), PI);
    }

    #[test]
    fn test_ellipse_arc_points() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, PI).unwrap();

        let start_point = arc.start_point();
        assert!((start_point.x() - 3.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((start_point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);

        let end_point = arc.end_point();
        assert!((end_point.x() - (-3.0)).abs() < GEOMETRIC_TOLERANCE);
        assert!((end_point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_ellipse_arc_length() {
        let center = Point3D::new(0.0, 0.0, 0.0);

        // 小さな楕円弧（π/4ラジアン = 45度）
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, PI / 4.0).unwrap();
        let arc_length = arc.arc_length();

        // 長さは正の値である必要があります
        assert!(arc_length > 0.0, "楕円弧の長さは正の値である必要があります");
    }

    #[test]
    fn test_angle_in_range() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, PI / 4.0, 3.0 * PI / 4.0).unwrap();

        assert!(arc.angle_in_range(PI / 2.0));
        assert!(!arc.angle_in_range(0.0));
        assert!(!arc.angle_in_range(PI));
    }

    #[test]
    fn test_on_arc() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, PI).unwrap();

        // 開始点
        assert!(arc.on_arc(&arc.start_point()));

        // 終了点
        assert!(arc.on_arc(&arc.end_point()));

        // 弧の範囲外の点（楕円上だが角度範囲外）
        let point_outside_range = Point3D::new(0.0, -2.0, 0.0); // 270度の位置
        assert!(!arc.on_arc(&point_outside_range));
    }

    #[test]
    fn test_tangent_vectors() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, PI / 2.0).unwrap();

        let start_tangent = arc.start_tangent();
        let end_tangent = arc.end_tangent();

        // 開始点（0度）での接線は上向き
        assert!(start_tangent.y() > 0.0);
        assert!((start_tangent.x()).abs() < GEOMETRIC_TOLERANCE);

        // 終了点（90度）での接線は左向き
        assert!(end_tangent.x() < 0.0);
        assert!((end_tangent.y()).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_scale_and_translate() {
        let center = Point3D::new(1.0, 1.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, PI).unwrap();

        // スケール
        let scaled = arc.scale(2.0);
        assert_eq!(scaled.ellipse().major_radius(), 6.0);
        assert_eq!(scaled.ellipse().minor_radius(), 4.0);

        // 平行移動
        let vector = Vector3D::new(2.0, 3.0, 1.0);
        let translated = arc.translate(&vector);
        assert_eq!(translated.center(), Point3D::new(3.0, 4.0, 1.0));
    }

    #[test]
    fn test_generate_points() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let arc = EllipseArc::xy_plane(center, 3.0, 2.0, 0.0, PI).unwrap();

        let points = arc.generate_points(5);
        assert_eq!(points.len(), 5);

        // 最初の点は開始点
        assert!((points[0].distance_to(&arc.start_point())).abs() < GEOMETRIC_TOLERANCE);

        // 最後の点は終了点
        assert!((points[4].distance_to(&arc.end_point())).abs() < GEOMETRIC_TOLERANCE);
    }
}

// =============================================================================
// 統一曲線解析インターフェイスの実装
// =============================================================================

/// EllipseArcに統一曲線解析インターフェイスを実装
/// 楕円弧は基底楕円の部分弧として基底楕円の計算を活用
impl CurveAnalysis3D<f64> for EllipseArc {
    type Point = Point3D;
    type Vector = Vector3D;
    type Direction = crate::geometry3d::Direction3D;

    /// 指定されたパラメータ位置での点を取得
    /// t: 0.0=開始点、1.0=終了点
    fn point_at_parameter(&self, t: f64) -> Self::Point {
        let angle = self.interpolate_angle(t);
        self.ellipse.point_at_angle(angle)
    }

    /// 指定されたパラメータ位置での接線ベクトルを取得（正規化済み）
    fn tangent_at_parameter(&self, t: f64) -> Self::Vector {
        let angle = self.interpolate_angle(t);
        // 基底楕円の接線を取得し、パラメータt位置での実際の角度で計算
        let ellipse_param = angle / (2.0 * PI); // 楕円の0-1パラメータに変換
        self.ellipse.tangent_at_parameter(ellipse_param)
    }

    /// 指定されたパラメータ位置での主法線ベクトルを取得（正規化済み）
    fn normal_at_parameter(&self, t: f64) -> Self::Vector {
        let angle = self.interpolate_angle(t);
        let ellipse_param = angle / (2.0 * PI);
        self.ellipse.normal_at_parameter(ellipse_param)
    }

    /// 指定されたパラメータ位置での双法線ベクトルを取得（正規化済み）
    fn binormal_at_parameter(&self, t: f64) -> Self::Vector {
        // 楕円弧の双法線は基底楕円と同じ（平面の法線）
        self.ellipse.binormal_at_parameter(0.0) // 位置に依存しない
    }

    /// 指定されたパラメータ位置での曲率を取得
    fn curvature_at_parameter(&self, t: f64) -> f64 {
        let angle = self.interpolate_angle(t);
        let ellipse_param = angle / (2.0 * PI);
        self.ellipse.curvature_at_parameter(ellipse_param)
    }

    /// 指定されたパラメータ位置での捩率（ねじれ）を取得
    fn torsion_at_parameter(&self, _t: f64) -> f64 {
        // 平面曲線（楕円弧）の捩率は常にゼロ
        0.0
    }

    /// 指定されたパラメータ位置での微分幾何学的情報を一括取得（最も効率的）
    fn differential_geometry_at_parameter(
        &self,
        t: f64,
    ) -> DifferentialGeometry<f64, Self::Vector> {
        let angle = self.interpolate_angle(t);
        let ellipse_param = angle / (2.0 * PI);

        // 基底楕円の微分幾何学情報を取得
        self.ellipse
            .differential_geometry_at_parameter(ellipse_param)
    }

    /// 最大曲率の位置と値を取得（楕円弧では弧範囲内での最大値）
    fn max_curvature(&self) -> Option<(f64, f64)> {
        // 基底楕円の最大曲率位置が弧範囲内にあるかチェック
        if let Some((_, max_curv)) = self.ellipse.max_curvature() {
            // 短軸端点の角度（π/2, 3π/2）が弧範囲内にあるかチェック
            let max_curv_angles = [PI / 2.0, 3.0 * PI / 2.0];
            for angle in &max_curv_angles {
                if self.contains_angle(*angle) {
                    let t = self.angle_to_parameter(*angle);
                    return Some((t, max_curv));
                }
            }
        }

        // 弧の端点での曲率を比較
        let start_curv = self.curvature_at_parameter(0.0);
        let end_curv = self.curvature_at_parameter(1.0);

        if start_curv >= end_curv {
            Some((0.0, start_curv))
        } else {
            Some((1.0, end_curv))
        }
    }

    /// 最小曲率の位置と値を取得（楕円弧では弧範囲内での最小値）
    fn min_curvature(&self) -> Option<(f64, f64)> {
        // 基底楕円の最小曲率位置が弧範囲内にあるかチェック
        if let Some((_, min_curv)) = self.ellipse.min_curvature() {
            // 長軸端点の角度（0, π）が弧範囲内にあるかチェック
            let min_curv_angles = [0.0, PI];
            for angle in &min_curv_angles {
                if self.contains_angle(*angle) {
                    let t = self.angle_to_parameter(*angle);
                    return Some((t, min_curv));
                }
            }
        }

        // 弧の端点での曲率を比較
        let start_curv = self.curvature_at_parameter(0.0);
        let end_curv = self.curvature_at_parameter(1.0);

        if start_curv <= end_curv {
            Some((0.0, start_curv))
        } else {
            Some((1.0, end_curv))
        }
    }

    /// 曲率がゼロになる位置を取得（楕円弧では存在しない）
    fn inflection_points(&self) -> Vec<f64> {
        Vec::new() // 楕円弧に変曲点は存在しない
    }

    /// 曲線が平面曲線かどうかを判定（楕円弧は常に平面曲線）
    fn is_planar(&self) -> bool {
        true
    }
}

/// EllipseArcに解析的曲線インターフェイスを実装
impl AnalyticalCurve<f64> for EllipseArc {
    /// 曲線の種類（楕円弧）
    fn curve_type(&self) -> CurveType {
        CurveType::EllipseArc
    }

    /// 一定曲率かどうか（楕円弧は基底楕円が円の場合のみ一定曲率）
    fn has_constant_curvature(&self) -> bool {
        self.ellipse.has_constant_curvature()
    }

    /// 解析的に計算可能な曲率の定数値（基底楕円が円の場合のみ）
    fn constant_curvature(&self) -> Option<f64> {
        self.ellipse.constant_curvature()
    }

    /// 解析的に計算可能な曲率半径の定数値（基底楕円が円の場合のみ）
    fn constant_curvature_radius(&self) -> Option<f64> {
        self.ellipse.constant_curvature_radius()
    }
}

impl EllipseArc {
    /// パラメータtから実際の角度への変換
    fn interpolate_angle(&self, t: f64) -> f64 {
        let span = self.angle_span();
        self.start_angle + span * t
    }

    /// 角度からパラメータtへの変換
    fn angle_to_parameter(&self, angle: f64) -> f64 {
        let span = self.angle_span();
        if span.abs() < GEOMETRIC_TOLERANCE {
            0.0
        } else {
            (angle - self.start_angle) / span
        }
    }

    /// 指定角度が弧範囲内にあるかチェック
    fn contains_angle(&self, angle: f64) -> bool {
        let normalized_angle = Self::normalize_angle(angle);
        let start = self.start_angle;
        let end = self.end_angle;

        if start <= end {
            normalized_angle >= start && normalized_angle <= end
        } else {
            // 角度が2π境界をまたぐ場合
            normalized_angle >= start || normalized_angle <= end
        }
    }
}
