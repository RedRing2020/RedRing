/// Ellipse（楕円）構造体とトレイト定義
///
/// 中心点と長軸・短軸による楕円の統一インターフェース

use crate::abstract_types::Scalar;
use crate::geometry::{Angle, Point2D, Point3D, Vector2D, Vector3D, BoundingBox2D, BoundingBox3D};
use std::fmt;

/// 楕円の構築エラー
#[derive(Debug, Clone, PartialEq)]
pub enum EllipseError {
    /// 無効な軸長（ゼロまたは負の値）
    InvalidAxisLength,
    /// 長軸が短軸より短い
    InvalidAxisOrder,
    /// 軸が直交していない
    NonOrthogonalAxes,
}

impl fmt::Display for EllipseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EllipseError::InvalidAxisLength => write!(f, "無効な軸長です"),
            EllipseError::InvalidAxisOrder => write!(f, "長軸は短軸より長くする必要があります"),
            EllipseError::NonOrthogonalAxes => write!(f, "軸が直交していません"),
        }
    }
}

impl std::error::Error for EllipseError {}

/// 楕円の基本トレイト
pub trait Ellipse<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// バウンディングボックスの型
    type BoundingBox;

    /// 楕円の中心点を取得
    fn center(&self) -> Self::Point;

    /// 長軸の長さを取得（半径）
    fn major_radius(&self) -> T;

    /// 短軸の長さを取得（半径）
    fn minor_radius(&self) -> T;

    /// 長軸の長さを取得（直径）
    fn major_axis_length(&self) -> T {
        self.major_radius() * (T::ONE + T::ONE)
    }

    /// 短軸の長さを取得（直径）
    fn minor_axis_length(&self) -> T {
        self.minor_radius() * (T::ONE + T::ONE)
    }

    /// 楕円の面積を計算
    fn area(&self) -> T {
        T::PI * self.major_radius() * self.minor_radius()
    }

    /// 楕円の周長を概算計算（ラマヌジャンの近似式）
    fn circumference(&self) -> T {
        let a = self.major_radius();
        let b = self.minor_radius();
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        T::PI * (a + b) * (T::ONE + (h * T::from_f64(3.0)) / (T::from_f64(10.0) + (T::from_f64(4.0) - h * T::from_f64(3.0)).sqrt()))
    }

    /// 楕円の離心率を計算
    fn eccentricity(&self) -> T {
        let a = self.major_radius();
        let b = self.minor_radius();
        if a <= b {
            T::ZERO
        } else {
            (T::ONE - (b * b) / (a * a)).sqrt()
        }
    }

    /// 指定された角度での楕円上の点を取得
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;

    /// 指定された点が楕円内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が楕円の境界上にあるかを判定
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 楕円の境界ボックスを取得
    fn bounding_box(&self) -> Self::BoundingBox;

    /// 楕円を指定倍率で拡大縮小
    fn scale(&self, factor: T) -> Self;

    /// 楕円を指定ベクトルで平行移動
    fn translate(&self, vector: &Self::Vector) -> Self;

    /// 楕円が円かどうかを判定
    fn is_circle(&self, tolerance: T) -> bool {
        (self.major_radius() - self.minor_radius()).abs() <= tolerance
    }

    /// 楕円が退化している（面積がほぼゼロ）かを判定
    fn is_degenerate(&self, tolerance: T) -> bool {
        self.minor_radius() <= tolerance
    }
}

/// 2D楕円専用の追加機能
pub trait Ellipse2D<T: Scalar>: Ellipse<T> {
    /// 楕円の回転角度を取得
    fn rotation(&self) -> Angle<T>;

    /// 指定角度で回転した楕円を取得
    fn rotated(&self, angle: Angle<T>) -> Self;

    /// 楕円の焦点を取得
    fn foci(&self) -> (Self::Point, Self::Point);

    /// 指定点から楕円への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;
}

/// 3D楕円専用の追加機能
pub trait Ellipse3D<T: Scalar>: Ellipse<T> {
    /// 楕円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 楕円の局所座標系を取得
    fn local_coordinate_system(&self) -> (Self::Vector, Self::Vector, Self::Vector);

    /// 指定された点が楕円の平面上にあるかを判定
    fn point_on_plane(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 2D楕円に投影
    fn project_to_2d(&self) -> impl Ellipse2D<T>;
}

/// 3D楕円の具象実装
#[derive(Debug, Clone)]
pub struct Ellipse3DImpl<T: Scalar> {
    /// 中心点
    center: Point3D<T>,
    /// 長軸ベクトル（方向と長さを含む）
    major_axis: Vector3D<T>,
    /// 短軸ベクトル（方向と長さを含む、major_axisに直交）
    minor_axis: Vector3D<T>,
    /// 平面の法線ベクトル（正規化済み）
    normal: Vector3D<T>,
}

impl<T: Scalar> Ellipse3DImpl<T> {
    /// 3D楕円を作成
    pub fn new(
        center: Point3D<T>,
        major_axis: Vector3D<T>,
        minor_axis: Vector3D<T>,
    ) -> Result<Self, EllipseError> {
        let major_length = major_axis.length();
        let minor_length = minor_axis.length();

        // 軸の長さをチェック
        if major_length <= T::ZERO || minor_length <= T::ZERO {
            return Err(EllipseError::InvalidAxisLength);
        }

        // 長軸が短軸より長いかチェック
        if major_length < minor_length {
            return Err(EllipseError::InvalidAxisOrder);
        }

        // 軸が直交しているかチェック
        let dot_product = major_axis.dot(&minor_axis).abs();
        let tolerance = T::from_f64(1e-10);
        if dot_product > tolerance {
            return Err(EllipseError::NonOrthogonalAxes);
        }

        // 法線ベクトルを計算（外積）
        let normal = major_axis.cross(&minor_axis).normalized();

        Ok(Ellipse3DImpl {
            center,
            major_axis,
            minor_axis,
            normal,
        })
    }

    /// 半径と角度から3D楕円を作成（XY平面上）
    pub fn from_radii_xy(
        center: Point3D<T>,
        major_radius: T,
        minor_radius: T,
        rotation: Angle<T>,
    ) -> Result<Self, EllipseError> {
        if major_radius <= T::ZERO || minor_radius <= T::ZERO {
            return Err(EllipseError::InvalidAxisLength);
        }
        if major_radius < minor_radius {
            return Err(EllipseError::InvalidAxisOrder);
        }

        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        let major_axis = Vector3D::new(
            major_radius * cos_rot,
            major_radius * sin_rot,
            T::ZERO,
        );
        let minor_axis = Vector3D::new(
            -minor_radius * sin_rot,
            minor_radius * cos_rot,
            T::ZERO,
        );

        Self::new(center, major_axis, minor_axis)
    }

    /// ローカル座標系から3D楕円を作成
    pub fn from_local_system(
        center: Point3D<T>,
        local_x: Vector3D<T>,
        local_y: Vector3D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Result<Self, EllipseError> {
        if major_radius <= T::ZERO || minor_radius <= T::ZERO {
            return Err(EllipseError::InvalidAxisLength);
        }
        if major_radius < minor_radius {
            return Err(EllipseError::InvalidAxisOrder);
        }

        let major_axis = local_x.normalized() * major_radius;
        let minor_axis = local_y.normalized() * minor_radius;

        Self::new(center, major_axis, minor_axis)
    }

    /// ローカル座標系に変換
    fn to_local_coords(&self, point: &Point3D<T>) -> (T, T) {
        // 点から中心への相対ベクトルを計算
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let dz = point.z() - self.center.z();
        let relative = Vector3D::new(dx, dy, dz);
        
        let major_radius = self.major_axis.length();
        let minor_radius = self.minor_axis.length();
        let x = relative.dot(&self.major_axis.normalized()) / major_radius;
        let y = relative.dot(&self.minor_axis.normalized()) / minor_radius;
        (x, y)
    }

    /// グローバル座標系に変換
    fn to_global_coords(&self, local_x: T, local_y: T) -> Point3D<T> {
        let major_unit = self.major_axis.normalized();
        let minor_unit = self.minor_axis.normalized();
        let major_radius = self.major_axis.length();
        let minor_radius = self.minor_axis.length();
        let offset = major_unit * (local_x * major_radius) + minor_unit * (local_y * minor_radius);
        Point3D::new(
            self.center.x() + offset.x(),
            self.center.y() + offset.y(),
            self.center.z() + offset.z(),
        )
    }
}

/// 2D楕円の具象実装
#[derive(Debug, Clone)]
pub struct Ellipse2DImpl<T: Scalar> {
    /// 中心点
    center: Point2D<T>,
    /// 長軸の半径
    major_radius: T,
    /// 短軸の半径
    minor_radius: T,
    /// 回転角度
    rotation: Angle<T>,
}

impl<T: Scalar> Ellipse2DImpl<T> {
    /// 2D楕円を作成
    pub fn new(
        center: Point2D<T>,
        major_radius: T,
        minor_radius: T,
        rotation: Angle<T>,
    ) -> Result<Self, EllipseError> {
        if major_radius <= T::ZERO || minor_radius <= T::ZERO {
            return Err(EllipseError::InvalidAxisLength);
        }
        if major_radius < minor_radius {
            return Err(EllipseError::InvalidAxisOrder);
        }

        Ok(Ellipse2DImpl {
            center,
            major_radius,
            minor_radius,
            rotation,
        })
    }

    /// 軸平行楕円を作成（回転なし）
    pub fn axis_aligned(
        center: Point2D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Result<Self, EllipseError> {
        Self::new(center, major_radius, minor_radius, Angle::from_radians(T::ZERO))
    }

    /// 単位円を作成
    pub fn unit_circle() -> Self {
        Self::axis_aligned(Point2D::new(T::ZERO, T::ZERO), T::ONE, T::ONE)
            .expect("単位円作成に失敗")
    }

    /// ローカル座標系に変換
    fn to_local_coords(&self, point: &Point2D<T>) -> (T, T) {
        let relative = *point - self.center;
        let cos_rot = (-self.rotation).cos();
        let sin_rot = (-self.rotation).sin();
        
        let rotated_x = relative.x() * cos_rot - relative.y() * sin_rot;
        let rotated_y = relative.x() * sin_rot + relative.y() * cos_rot;
        
        (rotated_x / self.major_radius, rotated_y / self.minor_radius)
    }

    /// グローバル座標系に変換
    fn to_global_coords(&self, local_x: T, local_y: T) -> Point2D<T> {
        let local_global_x = local_x * self.major_radius;
        let local_global_y = local_y * self.minor_radius;
        
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        
        let rotated_x = local_global_x * cos_rot - local_global_y * sin_rot;
        let rotated_y = local_global_x * sin_rot + local_global_y * cos_rot;
        
        Point2D::new(
            self.center.x() + rotated_x,
            self.center.y() + rotated_y,
        )
    }
}

// 3D楕円のトレイト実装
impl<T: Scalar> Ellipse<T> for Ellipse3DImpl<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;
    type BoundingBox = BoundingBox3D<T>;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn major_radius(&self) -> T {
        self.major_axis.length()
    }

    fn minor_radius(&self) -> T {
        self.minor_axis.length()
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point {
        let cos_t = angle.cos();
        let sin_t = angle.sin();
        self.to_global_coords(cos_t, sin_t)
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        let (x, y) = self.to_local_coords(point);
        (x * x + y * y) <= T::ONE
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        let (x, y) = self.to_local_coords(point);
        let dist_from_boundary = ((x * x + y * y).sqrt() - T::ONE).abs();
        dist_from_boundary <= tolerance
    }

    fn bounding_box(&self) -> Self::BoundingBox {
        // 簡略化された実装：軸に平行な境界ボックス
        let major_extent = self.major_axis.length();
        let minor_extent = self.minor_axis.length();
        let extent = if major_extent > minor_extent { major_extent } else { minor_extent };
        
        BoundingBox3D::new(
            Point3D::new(
                self.center.x() - extent,
                self.center.y() - extent,
                self.center.z() - extent,
            ),
            Point3D::new(
                self.center.x() + extent,
                self.center.y() + extent,
                self.center.z() + extent,
            ),
        )
    }

    fn scale(&self, factor: T) -> Self {
        Ellipse3DImpl {
            center: self.center,
            major_axis: self.major_axis * factor,
            minor_axis: self.minor_axis * factor,
            normal: self.normal,
        }
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        Ellipse3DImpl {
            center: Point3D::new(
                self.center.x() + vector.x(),
                self.center.y() + vector.y(),
                self.center.z() + vector.z(),
            ),
            major_axis: self.major_axis,
            minor_axis: self.minor_axis,
            normal: self.normal,
        }
    }
}

impl<T: Scalar> Ellipse3D<T> for Ellipse3DImpl<T> {
    fn normal(&self) -> Self::Vector {
        self.normal
    }

    fn local_coordinate_system(&self) -> (Self::Vector, Self::Vector, Self::Vector) {
        let x_axis = self.major_axis.normalized();
        let y_axis = self.minor_axis.normalized();
        let z_axis = self.normal;
        (x_axis, y_axis, z_axis)
    }

    fn point_on_plane(&self, point: &Self::Point, tolerance: T) -> bool {
        let relative = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );
        let distance_to_plane = relative.dot(&self.normal).abs();
        distance_to_plane <= tolerance
    }

    fn project_to_2d(&self) -> impl Ellipse2D<T> {
        // 簡略化：XY平面への投影
        Ellipse2DImpl::axis_aligned(
            Point2D::new(self.center.x(), self.center.y()),
            self.major_axis.length(),
            self.minor_axis.length(),
        ).expect("2D投影楕円作成に失敗")
    }
}

// 2D楕円のトレイト実装
impl<T: Scalar> Ellipse<T> for Ellipse2DImpl<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BoundingBox = BoundingBox2D<T>;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn major_radius(&self) -> T {
        self.major_radius
    }

    fn minor_radius(&self) -> T {
        self.minor_radius
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point {
        let cos_t = angle.cos();
        let sin_t = angle.sin();
        self.to_global_coords(cos_t, sin_t)
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        let (x, y) = self.to_local_coords(point);
        (x * x + y * y) <= T::ONE
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        let (x, y) = self.to_local_coords(point);
        let dist_from_boundary = ((x * x + y * y).sqrt() - T::ONE).abs();
        dist_from_boundary <= tolerance
    }

    fn bounding_box(&self) -> Self::BoundingBox {
        // 回転楕円の境界ボックス計算（簡略化）
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        
        let a_cos = self.major_radius * cos_rot.abs();
        let a_sin = self.major_radius * sin_rot.abs();
        let b_cos = self.minor_radius * cos_rot.abs();
        let b_sin = self.minor_radius * sin_rot.abs();
        
        let width = a_cos + b_sin;
        let height = a_sin + b_cos;
        
        BoundingBox2D::new(
            Point2D::new(self.center.x() - width, self.center.y() - height),
            Point2D::new(self.center.x() + width, self.center.y() + height),
        )
    }

    fn scale(&self, factor: T) -> Self {
        Ellipse2DImpl {
            center: self.center,
            major_radius: self.major_radius * factor,
            minor_radius: self.minor_radius * factor,
            rotation: self.rotation,
        }
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        Ellipse2DImpl {
            center: Point2D::new(
                self.center.x() + vector.x(),
                self.center.y() + vector.y(),
            ),
            major_radius: self.major_radius,
            minor_radius: self.minor_radius,
            rotation: self.rotation,
        }
    }
}

impl<T: Scalar> Ellipse2D<T> for Ellipse2DImpl<T> {
    fn rotation(&self) -> Angle<T> {
        self.rotation
    }

    fn rotated(&self, angle: Angle<T>) -> Self {
        Ellipse2DImpl {
            center: self.center,
            major_radius: self.major_radius,
            minor_radius: self.minor_radius,
            rotation: self.rotation + angle,
        }
    }

    fn foci(&self) -> (Self::Point, Self::Point) {
        let c = (self.major_radius * self.major_radius - self.minor_radius * self.minor_radius).sqrt();
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        
        let f1 = Point2D::new(
            self.center.x() + c * cos_rot,
            self.center.y() + c * sin_rot,
        );
        let f2 = Point2D::new(
            self.center.x() - c * cos_rot,
            self.center.y() - c * sin_rot,
        );
        
        (f1, f2)
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        let (x, y) = self.to_local_coords(point);
        let ellipse_distance = (x * x + y * y).sqrt();
        
        if ellipse_distance <= T::ONE {
            T::ZERO
        } else {
            // 楕円外部の点からの近似距離
            (ellipse_distance - T::ONE) * self.minor_radius
        }
    }
}

/// 型エイリアス
pub type Ellipse2DType<T> = Ellipse2DImpl<T>;
pub type Ellipse3DType<T> = Ellipse3DImpl<T>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts;

    #[test]
    fn test_ellipse3d_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let major_axis = Vector3D::new(3.0, 0.0, 0.0);
        let minor_axis = Vector3D::new(0.0, 2.0, 0.0);
        
        let ellipse = Ellipse3DImpl::new(center, major_axis, minor_axis)
            .expect("Ellipse3D作成に失敗");
        
        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.major_radius(), 3.0);
        assert_eq!(ellipse.minor_radius(), 2.0);
        
        let expected_area = consts::PI * 3.0 * 2.0;
        assert!((ellipse.area() - expected_area).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse3d_xy_plane() {
        let center = Point3D::new(1.0, 1.0, 0.0);
        let rotation = Angle::from_degrees(45.0);
        
        let ellipse = Ellipse3DImpl::from_radii_xy(center, 3.0, 2.0, rotation)
            .expect("XY平面楕円作成に失敗");
        
        assert_eq!(ellipse.center(), center);
        assert!((ellipse.major_radius() - 3.0).abs() < 1e-10);
        assert!((ellipse.minor_radius() - 2.0).abs() < 1e-10);
        
        // Z軸が法線
        let normal = ellipse.normal();
        assert!((normal.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ellipse3d_point_on_plane() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let major_axis = Vector3D::new(2.0, 0.0, 0.0);
        let minor_axis = Vector3D::new(0.0, 1.0, 0.0);
        
        let ellipse = Ellipse3DImpl::new(center, major_axis, minor_axis)
            .expect("楕円作成に失敗");
        
        // 平面上の点
        let point_on_plane = Point3D::new(1.0, 0.5, 0.0);
        assert!(ellipse.point_on_plane(&point_on_plane, 1e-10));
        
        // 平面外の点
        let point_off_plane = Point3D::new(1.0, 0.5, 1.0);
        assert!(!ellipse.point_on_plane(&point_off_plane, 1e-10));
    }

    #[test]
    fn test_ellipse3d_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let major_axis = Vector3D::new(2.0, 0.0, 0.0);
        let minor_axis = Vector3D::new(0.0, 1.0, 0.0);
        
        let ellipse = Ellipse3DImpl::new(center, major_axis, minor_axis)
            .expect("楕円作成に失敗");
        
        // 楕円内部の点
        assert!(ellipse.contains_point(&Point3D::new(1.0, 0.0, 0.0)));
        assert!(ellipse.contains_point(&Point3D::new(0.0, 0.5, 0.0)));
        
        // 楕円外部の点  
        assert!(!ellipse.contains_point(&Point3D::new(2.1, 0.0, 0.0)));
        assert!(!ellipse.contains_point(&Point3D::new(0.0, 1.1, 0.0)));
    }

    #[test]
    fn test_ellipse3d_local_coordinate_system() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let major_axis = Vector3D::new(3.0, 0.0, 0.0);
        let minor_axis = Vector3D::new(0.0, 2.0, 0.0);
        
        let ellipse = Ellipse3DImpl::new(center, major_axis, minor_axis)
            .expect("楕円作成に失敗");
        
        let (x_axis, y_axis, z_axis) = ellipse.local_coordinate_system();
        
        // X軸は正規化された長軸方向
        assert!((x_axis.x() - 1.0).abs() < f64::EPSILON);
        assert!(x_axis.y().abs() < f64::EPSILON);
        assert!(x_axis.z().abs() < f64::EPSILON);
        
        // Y軸は正規化された短軸方向
        assert!(y_axis.x().abs() < f64::EPSILON);
        assert!((y_axis.y() - 1.0).abs() < f64::EPSILON);
        assert!(y_axis.z().abs() < f64::EPSILON);
        
        // Z軸は法線ベクトル
        assert!(z_axis.x().abs() < f64::EPSILON);
        assert!(z_axis.y().abs() < f64::EPSILON);
        assert!((z_axis.z() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse3d_invalid_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        
        // 無効な軸長（ゼロベクトル）
        let zero_major = Vector3D::new(0.0, 0.0, 0.0);
        let valid_minor = Vector3D::new(0.0, 1.0, 0.0);
        let result1 = Ellipse3DImpl::new(center, zero_major, valid_minor);
        assert!(matches!(result1, Err(EllipseError::InvalidAxisLength)));
        
        // 長軸 < 短軸
        let short_major = Vector3D::new(1.0, 0.0, 0.0);
        let long_minor = Vector3D::new(0.0, 2.0, 0.0);
        let result2 = Ellipse3DImpl::new(center, short_major, long_minor);
        assert!(matches!(result2, Err(EllipseError::InvalidAxisOrder)));
        
        // 非直交軸
        let major_axis = Vector3D::new(2.0, 0.0, 0.0);
        let non_orthogonal = Vector3D::new(1.0, 1.0, 0.0);
        let result3 = Ellipse3DImpl::new(center, major_axis, non_orthogonal);
        assert!(matches!(result3, Err(EllipseError::NonOrthogonalAxes)));
    }

    // 2D楕円テスト（既存）
    #[test]
    fn test_ellipse2d_creation() {
        let center = Point2D::new(0.0, 0.0);
        let major_radius = 3.0;
        let minor_radius = 2.0;
        let rotation = Angle::from_radians(0.0);

        let ellipse = Ellipse2DImpl::new(center, major_radius, minor_radius, rotation)
            .expect("楕円作成に失敗");

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.major_radius(), major_radius);
        assert_eq!(ellipse.minor_radius(), minor_radius);
        assert_eq!(ellipse.rotation(), rotation);

        let expected_area = consts::PI * major_radius * minor_radius;
        assert!((ellipse.area() - expected_area).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse2d_area() {
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            5.0,
            3.0,
        ).expect("楕円作成に失敗");

        let expected_area = consts::PI * 5.0 * 3.0;
        assert!((ellipse.area() - expected_area).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse2d_eccentricity() {
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            5.0,
            3.0,
        ).expect("楕円作成に失敗");

        let expected_eccentricity = (1.0 - (3.0 * 3.0) / (5.0 * 5.0)).sqrt();
        assert!((ellipse.eccentricity() - expected_eccentricity).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse2d_point_at_angle() {
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            2.0,
            1.0,
        ).expect("楕円作成に失敗");

        let point_0 = ellipse.point_at_angle(Angle::from_radians(0.0));
        assert!((point_0.x() - 2.0).abs() < f64::EPSILON);
        assert!(point_0.y().abs() < f64::EPSILON);

        let point_90 = ellipse.point_at_angle(Angle::from_radians(consts::PI / 2.0));
        assert!(point_90.x().abs() < f64::EPSILON);
        assert!((point_90.y() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse2d_contains_point() {
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            2.0,
            1.0,
        ).expect("楕円作成に失敗");

        // 楕円内部の点
        assert!(ellipse.contains_point(&Point2D::new(1.0, 0.0))); // X軸上
        assert!(ellipse.contains_point(&Point2D::new(0.0, 0.5))); // Y軸上
        assert!(ellipse.contains_point(&Point2D::new(0.0, 0.0))); // 中心点

        // 楕円境界上の点
        assert!(ellipse.contains_point(&Point2D::new(2.0, 0.0))); // 右端
        assert!(ellipse.contains_point(&Point2D::new(-2.0, 0.0))); // 左端
        assert!(ellipse.contains_point(&Point2D::new(0.0, 1.0))); // 上端

        // 楕円外の点
        assert!(!ellipse.contains_point(&Point2D::new(3.0, 0.0))); // X軸上外部
        assert!(!ellipse.contains_point(&Point2D::new(0.0, 2.0))); // Y軸上外部
    }

    #[test]
    fn test_ellipse2d_unit_circle() {
        let circle = Ellipse2DImpl::unit_circle();
        
        assert_eq!(circle.center(), Point2D::new(0.0, 0.0));
        assert_eq!(circle.major_radius(), 1.0);
        assert_eq!(circle.minor_radius(), 1.0);
        assert!(circle.is_circle(f64::EPSILON));
        
        let expected_area = consts::PI;
        assert!((circle.area() - expected_area).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse2d_foci() {
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            5.0,
            3.0,
        ).expect("楕円作成に失敗");

        let (f1, f2) = ellipse.foci();
        let c = 4.0; // sqrt(25 - 9) = 4
        
        assert!((f1.x() - c).abs() < f64::EPSILON);
        assert!(f1.y().abs() < f64::EPSILON);
        assert!((f2.x() + c).abs() < f64::EPSILON);
        assert!(f2.y().abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipse2d_invalid_creation() {
        let center = Point2D::new(0.0, 0.0);
        let rotation = Angle::from_radians(0.0);

        // 無効な軸長
        let result1 = Ellipse2DImpl::new(center, -1.0, 1.0, rotation);
        assert!(matches!(result1, Err(EllipseError::InvalidAxisLength)));

        // 長軸 < 短軸
        let result2 = Ellipse2DImpl::new(center, 1.0, 2.0, rotation);
        assert!(matches!(result2, Err(EllipseError::InvalidAxisOrder)));
    }
}