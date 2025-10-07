//! 3D Ellipse implementation - geo_foundation Ellipse3D trait implementation
//!
//! geo_foundationのEllipse3Dトレイトを実装した3次元楕円

use crate::geometry3d::{Circle, Point3D, Vector3D, BBox3D, Direction3D};
use crate::geometry2d;
use crate::traits::Direction;
use geo_foundation::{
    geometry::{Angle, Ellipse3DImpl, Point3D as FoundationPoint3D, Vector3D as FoundationVector3D, BoundingBox3D},
    geometry::ellipse::{Ellipse as EllipseTrait, Ellipse3D as Ellipse3DTrait, EllipseError},
};
use geo_foundation::common::constants::GEOMETRIC_TOLERANCE;
use std::f64::consts::PI;

/// 3D空間上の楕円を表現する構造体（geo_foundation Ellipse3D trait実装）
#[derive(Debug, Clone)]
pub struct Ellipse {
    center: Point3D,
    major_axis: Vector3D,   // 長軸ベクトル
    minor_axis: Vector3D,   // 短軸ベクトル
    normal: Vector3D,       // 楕円平面の法線ベクトル
    foundation_ellipse: Ellipse3DImpl<f64>, // geo_foundation用の楕円
}

impl Ellipse {
    /// 新しい3D楕円を作成
    ///
    /// # Arguments
    /// * `center` - 楕円の中心点
    /// * `major_axis` - 長軸ベクトル（長さが長軸の半径）
    /// * `minor_axis` - 短軸ベクトル（長さが短軸の半径）
    pub fn new(center: Point3D, major_axis: Vector3D, minor_axis: Vector3D) -> Result<Self, EllipseError> {
        let major_length = major_axis.length();
        let minor_length = minor_axis.length();

        if major_length <= 0.0 || minor_length <= 0.0 {
            return Err(EllipseError::InvalidAxisLength);
        }
        if major_length < minor_length {
            return Err(EllipseError::InvalidAxisOrder);
        }

        // 軸が直交しているかチェック
        let dot_product = major_axis.dot(&minor_axis);
        if dot_product.abs() > GEOMETRIC_TOLERANCE {
            return Err(EllipseError::NonOrthogonalAxes);
        }

        // 法線ベクトルを計算
        let normal = major_axis.cross(&minor_axis).normalize().unwrap_or(Vector3D::unit_z());

        // geo_foundation Ellipse3DImplを作成
        let foundation_ellipse = Ellipse3DImpl::new(
            FoundationPoint3D::new(center.x(), center.y(), center.z()),
            FoundationVector3D::new(major_axis.x(), major_axis.y(), major_axis.z()),
            FoundationVector3D::new(minor_axis.x(), minor_axis.y(), minor_axis.z()),
        )?;

        Ok(Self {
            center,
            major_axis,
            minor_axis,
            normal,
            foundation_ellipse,
        })
    }

    /// XY平面上の楕円を作成
    pub fn xy_plane(center: Point3D, major_radius: f64, minor_radius: f64, rotation: f64) -> Result<Self, EllipseError> {
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        let major_axis = Vector3D::new(major_radius * cos_rot, major_radius * sin_rot, 0.0);
        let minor_axis = Vector3D::new(-minor_radius * sin_rot, minor_radius * cos_rot, 0.0);

        Self::new(center, major_axis, minor_axis)
    }

    /// XZ平面上の楕円を作成
    pub fn xz_plane(center: Point3D, major_radius: f64, minor_radius: f64, rotation: f64) -> Result<Self, EllipseError> {
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        let major_axis = Vector3D::new(major_radius * cos_rot, 0.0, major_radius * sin_rot);
        let minor_axis = Vector3D::new(-minor_radius * sin_rot, 0.0, minor_radius * cos_rot);

        Self::new(center, major_axis, minor_axis)
    }

    /// YZ平面上の楕円を作成
    pub fn yz_plane(center: Point3D, major_radius: f64, minor_radius: f64, rotation: f64) -> Result<Self, EllipseError> {
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        let major_axis = Vector3D::new(0.0, major_radius * cos_rot, major_radius * sin_rot);
        let minor_axis = Vector3D::new(0.0, -minor_radius * sin_rot, minor_radius * cos_rot);

        Self::new(center, major_axis, minor_axis)
    }

    /// 円から楕円を作成
    pub fn from_circle(circle: &Circle) -> Self {
        let center = circle.center();
        let radius = circle.radius();
        let _normal = circle.normal();
        let u_axis = circle.u_axis();

        // 長軸と短軸を円の局所座標系に沿って設定
        let major_axis = Vector3D::new(u_axis.x(), u_axis.y(), u_axis.z()) * radius;
        let v_axis = circle.v_axis();
        let minor_axis = Vector3D::new(v_axis.x(), v_axis.y(), v_axis.z()) * radius;

        Self::new(center, major_axis, minor_axis).unwrap()
    }

    /// 2D楕円から3D楕円を作成（Z=0平面）
    pub fn from_2d_ellipse(ellipse_2d: &geometry2d::Ellipse) -> Self {
        let center_2d = ellipse_2d.center();
        let center = Point3D::new(center_2d.x(), center_2d.y(), 0.0);

        let major_radius = ellipse_2d.major_radius();
        let minor_radius = ellipse_2d.minor_radius();
        let rotation = ellipse_2d.rotation_radians();

        Self::xy_plane(center, major_radius, minor_radius, rotation).unwrap()
    }
}

// 手動でPartialEqを実装（foundation_ellipseは比較対象外）
impl PartialEq for Ellipse {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && self.major_axis == other.major_axis
            && self.minor_axis == other.minor_axis
            && self.normal == other.normal
    }
}

// geo_foundation Ellipse トレイトの実装
impl EllipseTrait<f64> for Ellipse {
    type Point = Point3D;
    type Vector = Vector3D;
    type BoundingBox = BBox3D;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn major_radius(&self) -> f64 {
        self.major_axis.length()
    }

    fn minor_radius(&self) -> f64 {
        self.minor_axis.length()
    }

    fn point_at_angle(&self, angle: Angle<f64>) -> Self::Point {
        let t = angle.to_radians();
        let cos_t = t.cos();
        let sin_t = t.sin();

        // 楕円上の点を局所座標系で計算
        let major_normalized = self.major_axis.normalize().unwrap_or(Vector3D::unit_x());
        let minor_normalized = self.minor_axis.normalize().unwrap_or(Vector3D::unit_y());

        let point_vec = major_normalized * (self.major_radius() * cos_t) + 
                       minor_normalized * (self.minor_radius() * sin_t);

        Point3D::new(
            self.center.x() + point_vec.x(),
            self.center.y() + point_vec.y(),
            self.center.z() + point_vec.z(),
        )
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        // 点を楕円の局所座標系に変換
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 点が楕円の平面上にあるかチェック
        let distance_to_plane = to_point.dot(&self.normal).abs();
        if distance_to_plane > GEOMETRIC_TOLERANCE {
            return false;
        }

        // 楕円の局所座標系での位置を計算
        let major_normalized = self.major_axis.normalize().unwrap_or(Vector3D::unit_x());
        let minor_normalized = self.minor_axis.normalize().unwrap_or(Vector3D::unit_y());

        let u = to_point.dot(&major_normalized);
        let v = to_point.dot(&minor_normalized);

        // 楕円の方程式で内部判定
        let normalized = (u / self.major_radius()).powi(2) + (v / self.minor_radius()).powi(2);
        normalized <= 1.0
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: f64) -> bool {
        // 点を楕円の局所座標系に変換
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 点が楕円の平面上にあるかチェック
        let distance_to_plane = to_point.dot(&self.normal).abs();
        if distance_to_plane > tolerance {
            return false;
        }

        // 楕円の局所座標系での位置を計算
        let major_normalized = self.major_axis.normalize().unwrap_or(Vector3D::unit_x());
        let minor_normalized = self.minor_axis.normalize().unwrap_or(Vector3D::unit_y());

        let u = to_point.dot(&major_normalized);
        let v = to_point.dot(&minor_normalized);

        // 楕円の方程式で境界判定
        let normalized = (u / self.major_radius()).powi(2) + (v / self.minor_radius()).powi(2);
        (normalized - 1.0).abs() <= tolerance
    }

    fn bounding_box(&self) -> Self::BoundingBox {
        // 楕円の軸端点を計算
        let major_endpoint1 = Vector3D::new(
            self.center.x() + self.major_axis.x(),
            self.center.y() + self.major_axis.y(),
            self.center.z() + self.major_axis.z(),
        );
        let major_endpoint2 = Vector3D::new(
            self.center.x() - self.major_axis.x(),
            self.center.y() - self.major_axis.y(),
            self.center.z() - self.major_axis.z(),
        );
        let minor_endpoint1 = Vector3D::new(
            self.center.x() + self.minor_axis.x(),
            self.center.y() + self.minor_axis.y(),
            self.center.z() + self.minor_axis.z(),
        );
        let minor_endpoint2 = Vector3D::new(
            self.center.x() - self.minor_axis.x(),
            self.center.y() - self.minor_axis.y(),
            self.center.z() - self.minor_axis.z(),
        );

        let endpoints = [
            Point3D::new(major_endpoint1.x(), major_endpoint1.y(), major_endpoint1.z()),
            Point3D::new(major_endpoint2.x(), major_endpoint2.y(), major_endpoint2.z()),
            Point3D::new(minor_endpoint1.x(), minor_endpoint1.y(), minor_endpoint1.z()),
            Point3D::new(minor_endpoint2.x(), minor_endpoint2.y(), minor_endpoint2.z()),
        ];

        BBox3D::from_point_array(&endpoints).unwrap()
    }

    fn scale(&self, factor: f64) -> Self {
        Self::new(
            self.center,
            self.major_axis * factor,
            self.minor_axis * factor,
        ).unwrap()
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        let new_center = Point3D::new(
            self.center.x() + vector.x(),
            self.center.y() + vector.y(),
            self.center.z() + vector.z(),
        );
        Self::new(new_center, self.major_axis, self.minor_axis).unwrap()
    }
}

// geo_foundation Ellipse3D トレイトの実装
impl Ellipse3DTrait<f64> for Ellipse {
    fn normal(&self) -> Vector3D {
        self.normal
    }

    fn local_coordinate_system(&self) -> (Vector3D, Vector3D, Vector3D) {
        let major_normalized = self.major_axis.normalize().unwrap_or(Vector3D::unit_x());
        let minor_normalized = self.minor_axis.normalize().unwrap_or(Vector3D::unit_y());
        (major_normalized, minor_normalized, self.normal)
    }

    fn point_on_plane(&self, point: &Point3D, tolerance: f64) -> bool {
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );
        to_point.dot(&self.normal).abs() <= tolerance
    }

    #[allow(refining_impl_trait_reachable)]
    fn project_to_2d(&self) -> geometry2d::Ellipse {
        // 楕円を2D平面に投影（XY平面）
        let center_2d = geometry2d::Point2D::new(self.center.x(), self.center.y());
        
        // 軸をXY平面に投影
        let major_2d_length = (self.major_axis.x().powi(2) + self.major_axis.y().powi(2)).sqrt();
        let minor_2d_length = (self.minor_axis.x().powi(2) + self.minor_axis.y().powi(2)).sqrt();
        
        // 回転角度を計算
        let rotation = self.major_axis.y().atan2(self.major_axis.x());
        
        geometry2d::Ellipse::new(center_2d, major_2d_length, minor_2d_length, rotation).unwrap()
    }
}

// 追加のメソッド（geo_primitives独自）
impl Ellipse {
    /// geo_foundation Ellipse3DImplを取得
    pub fn foundation_ellipse(&self) -> &Ellipse3DImpl<f64> {
        &self.foundation_ellipse
    }

    /// 長軸ベクトルを取得
    pub fn major_axis(&self) -> Vector3D {
        self.major_axis
    }

    /// 短軸ベクトルを取得
    pub fn minor_axis(&self) -> Vector3D {
        self.minor_axis
    }

    /// 楕円の面積を計算
    pub fn area(&self) -> f64 {
        PI * self.major_radius() * self.minor_radius()
    }

    /// 楕円の周長を概算計算（ラマヌジャンの近似式）
    pub fn circumference(&self) -> f64 {
        let a = self.major_radius();
        let b = self.minor_radius();
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    /// 楕円の離心率を計算
    pub fn eccentricity(&self) -> f64 {
        let major = self.major_radius();
        let minor = self.minor_radius();
        if major <= minor {
            0.0
        } else {
            (1.0 - (minor * minor) / (major * major)).sqrt()
        }
    }

    /// 楕円の焦点距離を計算
    pub fn focal_distance(&self) -> f64 {
        let major = self.major_radius();
        let minor = self.minor_radius();
        if major <= minor {
            0.0
        } else {
            (major * major - minor * minor).sqrt()
        }
    }

    /// 楕円の焦点を取得
    pub fn foci(&self) -> (Point3D, Point3D) {
        let focal_dist = self.focal_distance();
        let major_normalized = self.major_axis.normalize().unwrap_or(Vector3D::unit_x());
        
        let focal_vector = major_normalized * focal_dist;
        let f1 = Point3D::new(
            self.center.x() + focal_vector.x(),
            self.center.y() + focal_vector.y(),
            self.center.z() + focal_vector.z(),
        );
        let f2 = Point3D::new(
            self.center.x() - focal_vector.x(),
            self.center.y() - focal_vector.y(),
            self.center.z() - focal_vector.z(),
        );

        (f1, f2)
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self) -> bool {
        (self.major_radius() - self.minor_radius()).abs() <= GEOMETRIC_TOLERANCE
    }

    /// 楕円が退化している（面積がほぼゼロ）かを判定
    pub fn is_degenerate(&self) -> bool {
        self.minor_radius() <= GEOMETRIC_TOLERANCE
    }

    /// 楕円を円に変換（長軸の半径を使用）
    pub fn to_circle(&self) -> Circle {
        let normal_dir = Direction3D::from_vector(self.normal).unwrap_or(Direction3D::positive_z());
        let u_axis_dir = Direction3D::from_vector(self.major_axis.normalize().unwrap_or(Vector3D::unit_x())).unwrap_or(Direction3D::positive_x());
        
        Circle::new(self.center, self.major_radius(), normal_dir, u_axis_dir)
    }

    /// 楕円を最小外接円に変換
    pub fn bounding_circle(&self) -> Circle {
        self.to_circle()
    }

    /// 楕円を最大内接円に変換
    pub fn inscribed_circle(&self) -> Circle {
        let normal_dir = Direction3D::from_vector(self.normal).unwrap_or(Direction3D::positive_z());
        let u_axis_dir = Direction3D::from_vector(self.major_axis.normalize().unwrap_or(Vector3D::unit_x())).unwrap_or(Direction3D::positive_x());
        
        Circle::new(self.center, self.minor_radius(), normal_dir, u_axis_dir)
    }

    /// 指定された点から楕円境界への最短距離を計算（近似）
    pub fn distance_to_point(&self, point: &Point3D) -> f64 {
        if self.contains_point(point) {
            0.0
        } else {
            // 簡易実装：楕円境界上の複数点との距離を計算し最小値を返す
            let mut min_dist = f64::INFINITY;
            for i in 0..36 {
                let angle = Angle::from_degrees(i as f64 * 10.0);
                let boundary_point = self.point_at_angle(angle);
                let dist = point.distance_to(&boundary_point);
                if dist < min_dist {
                    min_dist = dist;
                }
            }
            min_dist
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    use crate::traits::Direction;

    #[test]
    fn test_ellipse_3d_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let major_axis = Vector3D::new(3.0, 0.0, 0.0);
        let minor_axis = Vector3D::new(0.0, 2.0, 0.0);
        let ellipse = Ellipse::new(center, major_axis, minor_axis).unwrap();

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.major_radius(), 3.0);
        assert_eq!(ellipse.minor_radius(), 2.0);
    }

    #[test]
    fn test_ellipse_xy_plane() {
        let center = Point3D::new(1.0, 1.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 4.0, 2.0, 0.0).unwrap();

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.major_radius(), 4.0);
        assert_eq!(ellipse.minor_radius(), 2.0);
    }

    #[test]
    fn test_ellipse_invalid_parameters() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        
        // 平行な軸（直交していない）
        let major_axis = Vector3D::new(3.0, 0.0, 0.0);
        let parallel_minor = Vector3D::new(2.0, 0.0, 0.0);
        assert!(Ellipse::new(center, major_axis, parallel_minor).is_err());
    }

    #[test]
    fn test_ellipse_area() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0, 0.0).unwrap();
        
        let expected_area = PI * 3.0 * 2.0;
        assert!((ellipse.area() - expected_area).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_ellipse_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0, 0.0).unwrap();

        // 中心点
        assert!(ellipse.contains_point(&center));
        
        // 楕円内部の点
        assert!(ellipse.contains_point(&Point3D::new(1.0, 1.0, 0.0)));
        
        // 楕円外部の点
        assert!(!ellipse.contains_point(&Point3D::new(4.0, 0.0, 0.0)));
        
        // 楕円平面外の点
        assert!(!ellipse.contains_point(&Point3D::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn test_ellipse_on_boundary() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0, 0.0).unwrap();

        // 長軸の端点
        assert!(ellipse.on_boundary(&Point3D::new(3.0, 0.0, 0.0), GEOMETRIC_TOLERANCE));
        
        // 短軸の端点
        assert!(ellipse.on_boundary(&Point3D::new(0.0, 2.0, 0.0), GEOMETRIC_TOLERANCE));
    }

    #[test]
    fn test_ellipse_point_at_angle() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0, 0.0).unwrap();

        // 0度の点（長軸上）
        let point_0 = ellipse.point_at_angle(Angle::from_degrees(0.0));
        assert!((point_0.x() - 3.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_0.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_0.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);

        // 90度の点（短軸上）
        let point_90 = ellipse.point_at_angle(Angle::from_degrees(90.0));
        assert!((point_90.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_90.y() - 2.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point_90.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_ellipse_scale() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 3.0, 2.0, 0.0).unwrap();
        let scaled = ellipse.scale(2.0);

        assert_eq!(scaled.major_radius(), 6.0);
        assert_eq!(scaled.minor_radius(), 4.0);
    }

    #[test]
    fn test_ellipse_from_circle() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 5.0, normal_dir, u_axis_dir);
        let ellipse = Ellipse::from_circle(&circle);

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.major_radius(), 5.0);
        assert_eq!(ellipse.minor_radius(), 5.0);
        assert!(ellipse.is_circle());
    }

    #[test]
    fn test_ellipse_project_to_2d() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let ellipse = Ellipse::xy_plane(center, 4.0, 2.0, PI / 4.0).unwrap();
        let projected = ellipse.project_to_2d();

        assert_eq!(projected.center(), geometry2d::Point2D::new(1.0, 2.0));
        assert!((projected.major_radius() - 4.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((projected.minor_radius() - 2.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_ellipse_foci() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipse = Ellipse::xy_plane(center, 5.0, 3.0, 0.0).unwrap();
        let (f1, f2) = ellipse.foci();

        let focal_distance = ellipse.focal_distance();
        assert!((f1.x() - focal_distance).abs() < GEOMETRIC_TOLERANCE);
        assert!((f2.x() + focal_distance).abs() < GEOMETRIC_TOLERANCE);
    }
}