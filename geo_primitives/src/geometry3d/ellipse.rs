//! 3D Ellipse implementation
//!
//! 3次元楕円の基本実装

use crate::geometry3d::{Circle, Point3D, Vector3D, BBox3D, Direction3D};
use geo_foundation::abstract_types::geometry::angle::Angle;
use geo_foundation::abstract_types::geometry::Direction;
use std::f64::consts::PI;

/// 楕円関連のエラー
#[derive(Debug, Clone, PartialEq)]
pub enum EllipseError {
    /// 軸の長さが無効（負または0）
    InvalidAxisLength,
    /// 軸の順序が無効（短軸が長軸より長い）
    InvalidAxisOrder,
}

impl std::fmt::Display for EllipseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EllipseError::InvalidAxisLength => write!(f, "Invalid axis length: must be positive"),
            EllipseError::InvalidAxisOrder => write!(
                f,
                "Invalid axis order: major radius must be >= minor radius"
            ),
        }
    }
}

impl std::error::Error for EllipseError {}

/// 幾何計算用の許容誤差
const GEOMETRIC_TOLERANCE: f64 = 1e-10;

/// 3D空間上の楕円を表現する構造体
#[derive(Debug, Clone)]
pub struct Ellipse {
    center: Point3D,
    major_radius: f64,
    minor_radius: f64,
    normal: Direction3D,
    u_axis: Direction3D, // 長軸方向
}

impl Ellipse {
    /// 新しい楕円を作成
    ///
    /// # Arguments
    /// * `center` - 楕円の中心点
    /// * `major_radius` - 長軸の半径
    /// * `minor_radius` - 短軸の半径
    /// * `normal` - 楕円平面の法線方向
    /// * `u_axis` - 長軸の方向
    pub fn new(
        center: Point3D,
        major_radius: f64,
        minor_radius: f64,
        normal: Direction3D,
        u_axis: Direction3D,
    ) -> Result<Self, EllipseError> {
        if major_radius <= 0.0 || minor_radius <= 0.0 {
            return Err(EllipseError::InvalidAxisLength);
        }
        if major_radius < minor_radius {
            return Err(EllipseError::InvalidAxisOrder);
        }

        Ok(Self {
            center,
            major_radius,
            minor_radius,
            normal,
            u_axis,
        })
    }

    /// XY平面上の楕円を作成
    pub fn xy_plane(
        center: Point3D,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, EllipseError> {
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let u_axis = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        Self::new(center, major_radius, minor_radius, normal, u_axis)
    }

    /// 円から楕円を作成
    pub fn from_circle(circle: &Circle) -> Self {
        let center = circle.center();
        let radius = circle.radius();
        let normal = Direction3D::from_vector(circle.normal()).unwrap();
        let u_axis = circle.u_axis();
        Self::new(center, radius, radius, normal, u_axis).unwrap()
    }

    /// 楕円の中心座標を取得
    pub fn center(&self) -> Point3D {
        self.center
    }

    /// 楕円の長軸半径を取得
    pub fn major_radius(&self) -> f64 {
        self.major_radius
    }

    /// 楕円の短軸半径を取得
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius
    }

    /// 楕円の法線方向を取得
    pub fn normal(&self) -> Direction3D {
        self.normal
    }

    /// 楕円の長軸方向を取得
    pub fn u_axis(&self) -> Direction3D {
        self.u_axis
    }

    /// 楕円の短軸方向を取得（長軸と法線の外積）
    pub fn v_axis(&self) -> Direction3D {
        let v = self.normal.to_vector().cross(&self.u_axis.to_vector());
        Direction3D::from_vector(v).unwrap()
    }

    /// 楕円の面積を計算
    pub fn area(&self) -> f64 {
        PI * self.major_radius * self.minor_radius
    }

    /// 楕円の周長を概算計算（ラマヌジャンの近似式）
    pub fn circumference(&self) -> f64 {
        let a = self.major_radius;
        let b = self.minor_radius;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    /// 楕円の離心率を計算
    pub fn eccentricity(&self) -> f64 {
        if self.major_radius <= self.minor_radius {
            0.0
        } else {
            (1.0 - (self.minor_radius * self.minor_radius)
                / (self.major_radius * self.major_radius))
                .sqrt()
        }
    }

    /// 楕円の焦点距離を計算
    pub fn focal_distance(&self) -> f64 {
        if self.major_radius <= self.minor_radius {
            0.0
        } else {
            (self.major_radius * self.major_radius - self.minor_radius * self.minor_radius).sqrt()
        }
    }

    /// 楕円の焦点を取得
    pub fn foci(&self) -> (Point3D, Point3D) {
        let focal_dist = self.focal_distance();
        let u_vec = self.u_axis.to_vector() * focal_dist;

        let f1 = Point3D::new(
            self.center.x() + u_vec.x(),
            self.center.y() + u_vec.y(),
            self.center.z() + u_vec.z(),
        );
        let f2 = Point3D::new(
            self.center.x() - u_vec.x(),
            self.center.y() - u_vec.y(),
            self.center.z() - u_vec.z(),
        );

        (f1, f2)
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self) -> bool {
        (self.major_radius - self.minor_radius).abs() <= GEOMETRIC_TOLERANCE
    }

    /// 指定された角度での楕円周上の点を取得
    pub fn point_at_angle(&self, angle: f64) -> Point3D {
        let u_vec = self.u_axis.to_vector();
        let v_vec = self.v_axis().to_vector();
        
        let cos_t = angle.cos();
        let sin_t = angle.sin();

        let local_point = u_vec * (self.major_radius * cos_t) + v_vec * (self.minor_radius * sin_t);

        Point3D::new(
            self.center.x() + local_point.x(),
            self.center.y() + local_point.y(),
            self.center.z() + local_point.z(),
        )
    }

    /// 点が楕円内部にあるかを判定
    pub fn contains_point(&self, point: &Point3D) -> bool {
        // 楕円の中心を原点とした座標系に変換
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 楕円のローカル座標系での座標を計算
        let u_coord = to_point.dot(&self.u_axis.to_vector());
        let v_coord = to_point.dot(&self.v_axis().to_vector());

        // 楕円の方程式で内部判定
        let normalized = (u_coord / self.major_radius).powi(2) + (v_coord / self.minor_radius).powi(2);
        normalized <= 1.0
    }

    /// 点が楕円境界上にあるかを判定
    pub fn on_boundary(&self, point: &Point3D) -> bool {
        // 楕円の中心を原点とした座標系に変換
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 楕円のローカル座標系での座標を計算
        let u_coord = to_point.dot(&self.u_axis.to_vector());
        let v_coord = to_point.dot(&self.v_axis().to_vector());

        // 楕円の方程式で境界判定
        let normalized = (u_coord / self.major_radius).powi(2) + (v_coord / self.minor_radius).powi(2);
        (normalized - 1.0).abs() <= GEOMETRIC_TOLERANCE
    }

    /// 楕円のバウンディングボックスを計算
    pub fn bounding_box(&self) -> BBox3D {
        // 楕円の軸方向での最大範囲を計算
        let u_vec = self.u_axis.to_vector() * self.major_radius;
        let v_vec = self.v_axis().to_vector() * self.minor_radius;

        // 楕円上の8つの主要な点を計算（簡易版）
        let mut points = Vec::new();
        for i in 0..8 {
            let angle = (i as f64) * PI / 4.0;
            points.push(self.point_at_angle(angle));
        }

        BBox3D::from_point_array(&points).unwrap_or_else(|| {
            BBox3D::from_3d_points(self.center, self.center)
        })
    }

    /// 楕円をスケール
    pub fn scale(&self, factor: f64) -> Self {
        Self::new(
            self.center,
            self.major_radius * factor,
            self.minor_radius * factor,
            self.normal,
            self.u_axis,
        )
        .unwrap()
    }

    /// 楕円を平行移動
    pub fn translate(&self, vector: &Vector3D) -> Self {
        let new_center = Point3D::new(
            self.center.x() + vector.x(),
            self.center.y() + vector.y(),
            self.center.z() + vector.z(),
        );
        Self::new(
            new_center,
            self.major_radius,
            self.minor_radius,
            self.normal,
            self.u_axis,
        )
        .unwrap()
    }

    /// 指定された点から楕円境界への最短距離を計算（近似）
    pub fn distance_to_point(&self, point: &Point3D) -> f64 {
        if self.contains_point(point) {
            0.0
        } else {
            // 簡易実装：楕円境界上の複数点との距離を計算し最小値を返す
            let mut min_dist = f64::INFINITY;
            for i in 0..36 {
                let angle = (i as f64 * 10.0).to_radians();
                let boundary_point = self.point_at_angle(angle);
                let dist = point.distance_to(&boundary_point);
                if dist < min_dist {
                    min_dist = dist;
                }
            }
            min_dist
        }
    }

    /// 楕円を円に変換（長軸の半径を使用）
    pub fn to_circle(&self) -> Circle {
        Circle::new(self.center, self.major_radius, self.normal, self.u_axis)
    }

    /// 楕円を最小外接円に変換
    pub fn bounding_circle(&self) -> Circle {
        Circle::new(self.center, self.major_radius, self.normal, self.u_axis)
    }

    /// 楕円を最大内接円に変換
    pub fn inscribed_circle(&self) -> Circle {
        Circle::new(self.center, self.minor_radius, self.normal, self.u_axis)
    }
}

// 手動でPartialEqを実装
impl PartialEq for Ellipse {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && (self.major_radius - other.major_radius).abs() < GEOMETRIC_TOLERANCE
            && (self.minor_radius - other.minor_radius).abs() < GEOMETRIC_TOLERANCE
            && self.normal == other.normal
            && self.u_axis == other.u_axis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.major_radius(), 3.0);
        assert_eq!(ellipse.minor_radius(), 2.0);
    }

    #[test]
    fn test_ellipse_invalid_parameters() {
        let center = Point3D::new(0.0, 0.0, 0.0);

        // 負の半径
        assert!(Ellipse::xy_plane(center, -1.0, 2.0).is_err());
        assert!(Ellipse::xy_plane(center, 2.0, -1.0).is_err());

        // 短軸が長軸より長い
        assert!(Ellipse::xy_plane(center, 2.0, 3.0).is_err());
    }

    #[test]
    fn test_ellipse_area() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

        let expected_area = PI * 3.0 * 2.0;
        assert!((ellipse.area() - expected_area).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_ellipse_eccentricity() {
        let center = Point3D::new(0.0, 0.0, 0.0);

        // 円の場合
        let circle = Ellipse::xy_plane(center, 2.0, 2.0).unwrap();
        assert!((circle.eccentricity() - 0.0).abs() < GEOMETRIC_TOLERANCE);

        // 楕円の場合
        let ellipse = Ellipse::xy_plane(center, 5.0, 3.0).unwrap();
        let expected_eccentricity = (1.0f64 - (3.0 * 3.0) / (5.0 * 5.0)).sqrt();
        assert!((ellipse.eccentricity() - expected_eccentricity).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_ellipse_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

        // 中心点
        assert!(ellipse.contains_point(&center));

        // 楕円内部の点
        assert!(ellipse.contains_point(&Point3D::new(1.0, 1.0, 0.0)));

        // 楕円外部の点
        assert!(!ellipse.contains_point(&Point3D::new(4.0, 0.0, 0.0)));
        assert!(!ellipse.contains_point(&Point3D::new(0.0, 3.0, 0.0)));
    }

    #[test]
    fn test_ellipse_on_boundary() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

        // 長軸の端点
        assert!(ellipse.on_boundary(&Point3D::new(3.0, 0.0, 0.0)));
        assert!(ellipse.on_boundary(&Point3D::new(-3.0, 0.0, 0.0)));

        // 短軸の端点
        assert!(ellipse.on_boundary(&Point3D::new(0.0, 2.0, 0.0)));
        assert!(ellipse.on_boundary(&Point3D::new(0.0, -2.0, 0.0)));
    }

    #[test]
    fn test_ellipse_scale() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();
        let scaled = ellipse.scale(2.0);

        assert_eq!(scaled.major_radius(), 6.0);
        assert_eq!(scaled.minor_radius(), 4.0);
        assert_eq!(scaled.center(), center);
    }

    #[test]
    fn test_ellipse_translate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();
        let vector = Vector3D::new(2.0, 3.0, 1.0);
        let translated = ellipse.translate(&vector);

        assert_eq!(translated.center(), Point3D::new(2.0, 3.0, 1.0));
        assert_eq!(translated.major_radius(), 3.0);
        assert_eq!(translated.minor_radius(), 2.0);
    }

    #[test]
    fn test_ellipse_from_circle() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let u_axis = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let circle = Circle::new(center, 5.0, normal, u_axis);
        let ellipse = Ellipse::from_circle(&circle);

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.major_radius(), 5.0);
        assert_eq!(ellipse.minor_radius(), 5.0);
        assert!(ellipse.is_circle());
    }

    #[test]
    fn test_ellipse_foci() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 5.0, 3.0).unwrap();
        let (f1, f2) = ellipse.foci();

        let focal_distance = ellipse.focal_distance();
        assert_eq!(f1, Point3D::new(focal_distance, 0.0, 0.0));
        assert_eq!(f2, Point3D::new(-focal_distance, 0.0, 0.0));
    }

    #[test]
    fn test_ellipse_is_circle() {
        let center = Point3D::new(0.0, 0.0, 0.0);

        // 円
        let circle = Ellipse::xy_plane(center, 2.0, 2.0).unwrap();
        assert!(circle.is_circle());

        // 楕円
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();
        assert!(!ellipse.is_circle());
    }

    #[test]
    fn test_ellipse_point_at_angle() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0).unwrap();

        // 0度の点（長軸上）
        let point_0 = ellipse.point_at_angle(0.0);
        assert!((point_0.x() - 3.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_0.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_0.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);

        // 90度の点（短軸上）
        let point_90 = ellipse.point_at_angle(PI / 2.0);
        assert!((point_90.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_90.y() - 2.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_90.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }
}