/// Arc（円弧）構造体とトレイト定義（基本版）
///
/// Circle構造体を基盤とした円弧の統一インターフェース

use crate::abstract_types::Scalar;
use crate::geometry::{Angle, Point2D, Point3D, Vector3D, BoundingBox3D};
use crate::geometry::circle::{Circle, Circle2D, Circle2DImpl, Circle3D, Circle3DImpl};
use std::fmt;

/// 円弧の構築エラー
#[derive(Debug, Clone, PartialEq)]
pub enum ArcError {
    /// 無効な角度範囲
    InvalidAngleRange,
    /// 角度の順序が不正
    InvalidAngleOrder,
    /// 無効な半径
    InvalidRadius,
    /// 3点が一直線上にある
    CollinearPoints,
}

impl fmt::Display for ArcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArcError::InvalidAngleRange => write!(f, "無効な角度範囲です"),
            ArcError::InvalidAngleOrder => write!(f, "角度の順序が不正です"),
            ArcError::InvalidRadius => write!(f, "無効な半径です"),
            ArcError::CollinearPoints => write!(f, "3点が一直線上にあります"),
        }
    }
}

impl std::error::Error for ArcError {}

/// 円弧の種類
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArcKind {
    /// 劣弧（180度未満）
    MinorArc,
    /// 優弧（180度以上）
    MajorArc,
    /// 半円（180度）
    Semicircle,
    /// 完全円（360度）
    FullCircle,
}

/// 2D円弧の基本トレイト
pub trait Arc2D<T: Scalar> {
    /// 基底の円を取得
    fn circle(&self) -> &Circle2DImpl<T>;

    /// 円弧の開始角度を取得
    fn start_angle(&self) -> Angle<T>;

    /// 円弧の終了角度を取得
    fn end_angle(&self) -> Angle<T>;

    /// 円弧の中心座標を取得
    fn center(&self) -> Point2D<T> {
        self.circle().center()
    }

    /// 円弧の半径を取得
    fn radius(&self) -> T {
        self.circle().radius()
    }

    /// 円弧の角度範囲を取得
    fn angle_span(&self) -> Angle<T> {
        let start = self.start_angle().to_radians();
        let end = self.end_angle().to_radians();
        let diff = if end >= start {
            end - start
        } else {
            end + T::TAU - start
        };
        Angle::from_radians(diff)
    }

    /// 円弧の弧長を計算
    fn arc_length(&self) -> T {
        self.radius() * self.angle_span().to_radians()
    }

    /// 円弧の開始点を取得
    fn start_point(&self) -> Point2D<T> {
        self.circle().point_at_angle(self.start_angle())
    }

    /// 円弧の終了点を取得
    fn end_point(&self) -> Point2D<T> {
        self.circle().point_at_angle(self.end_angle())
    }

    /// 円弧の中点を取得
    fn midpoint(&self) -> Point2D<T> {
        let start_rad = self.start_angle().to_radians();
        let end_rad = self.end_angle().to_radians();
        let mid_rad = if end_rad >= start_rad {
            (start_rad + end_rad) / (T::ONE + T::ONE)
        } else {
            (start_rad + end_rad + T::TAU) / (T::ONE + T::ONE)
        };
        let mid_angle = Angle::from_radians(mid_rad);
        self.circle().point_at_angle(mid_angle)
    }

    /// 指定された点が円弧上にあるかを判定
    fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 円周からの距離をチェック
        let distance = self.circle().distance_to_point(point);
        if distance > tolerance {
            return false;
        }

        // 点の角度を計算
        let center = self.center();
        let relative_x = point.x() - center.x();
        let relative_y = point.y() - center.y();
        let point_angle = Angle::from_radians(relative_y.atan2(relative_x));
        
        self.angle_contains(point_angle)
    }

    /// 指定された角度が円弧の角度範囲内にあるかを判定
    fn angle_contains(&self, angle: Angle<T>) -> bool {
        let start = self.start_angle().normalize_0_2pi().to_radians();
        let end = self.end_angle().normalize_0_2pi().to_radians();
        let test = angle.normalize_0_2pi().to_radians();

        if start <= end {
            // 角度が跨がっていない場合
            test >= start && test <= end
        } else {
            // 角度が0度を跨いでいる場合
            test >= start || test <= end
        }
    }

    /// 点から円弧への最短距離を計算
    fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let center = self.center();
        let radius = self.radius();
        let point_to_center = center.distance_to(*point);

        // 点が円弧の角度範囲内にあるかチェック
        let relative_x = point.x() - center.x();
        let relative_y = point.y() - center.y();
        let point_angle = Angle::from_radians(relative_y.atan2(relative_x));
        
        if self.angle_contains(point_angle) {
            // 角度範囲内の場合、円周からの距離
            (point_to_center - radius).abs()
        } else {
            // 角度範囲外の場合、端点からの距離
            let start_point = self.start_point();
            let end_point = self.end_point();
            let dist_to_start = point.distance_to(start_point);
            let dist_to_end = point.distance_to(end_point);
            if dist_to_start < dist_to_end { dist_to_start } else { dist_to_end }
        }
    }

    /// 円弧と円の交差判定
    fn intersects_with_circle(&self, other_circle: &Circle2DImpl<T>) -> bool {
        // 基底の円同士の交差をチェック
        let center1 = self.center();
        let radius1 = self.radius();
        let center2 = other_circle.center();
        let radius2 = other_circle.radius();
        
        let distance = center1.distance_to(center2);
        let radii_sum = radius1 + radius2;
        let radii_diff = (radius1 - radius2).abs();
        
        // 円同士が交差または包含している場合
        if distance <= radii_sum && distance >= radii_diff {
            // 円弧の端点が他の円内部または境界にあるかチェック
            let start_point = self.start_point();
            let end_point = self.end_point();
            
            let start_dist = start_point.distance_to(center2);
            let end_dist = end_point.distance_to(center2);
            
            start_dist <= radius2 || end_dist <= radius2 ||
            other_circle.contains_point(&start_point) || other_circle.contains_point(&end_point)
        } else {
            false
        }
    }

    /// 円弧同士の交差判定
    fn intersects_with_arc(&self, other: &impl Arc2D<T>) -> bool {
        // 基底の円同士の交差をチェック
        let center1 = self.center();
        let radius1 = self.radius();
        let center2 = other.center();
        let radius2 = other.radius();
        
        let distance = center1.distance_to(center2);
        let radii_sum = radius1 + radius2;
        let radii_diff = (radius1 - radius2).abs();
        
        if distance > radii_sum || distance < radii_diff {
            return false;
        }

        // 簡略化：端点が相手の円弧に含まれるかチェック
        let tolerance = T::from_f64(1e-10);
        
        other.contains_point(&self.start_point(), tolerance) ||
        other.contains_point(&self.end_point(), tolerance) ||
        self.contains_point(&other.start_point(), tolerance) ||
        self.contains_point(&other.end_point(), tolerance)
    }

    /// 円弧の種類を判定
    fn arc_kind(&self) -> ArcKind {
        let span = self.angle_span().to_radians();
        let pi = T::PI;
        let tau = T::TAU;
        let tolerance = T::from_f64(1e-10);

        if (span - tau).abs() <= tolerance {
            ArcKind::FullCircle
        } else if (span - pi).abs() <= tolerance {
            ArcKind::Semicircle
        } else if span < pi {
            ArcKind::MinorArc
        } else {
            ArcKind::MajorArc
        }
    }
}

/// 2D円弧の具象実装
#[derive(Debug, Clone)]
pub struct Arc2DImpl<T: Scalar> {
    circle: Circle2DImpl<T>,
    start_angle: Angle<T>,
    end_angle: Angle<T>,
}

impl<T: Scalar> Arc2DImpl<T> {
    /// 新しい2D円弧を作成
    pub fn new(
        circle: Circle2DImpl<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Result<Self, ArcError> {
        if circle.radius() <= T::ZERO {
            return Err(ArcError::InvalidRadius);
        }

        Ok(Arc2DImpl {
            circle,
            start_angle,
            end_angle,
        })
    }

    /// 中心、半径、角度範囲から2D円弧を作成
    pub fn from_center_radius_angles(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Result<Self, ArcError> {
        let circle = Circle2DImpl::new(center, radius);
        Self::new(circle, start_angle, end_angle)
    }
}

impl<T: Scalar> Arc2D<T> for Arc2DImpl<T> {
    fn circle(&self) -> &Circle2DImpl<T> {
        &self.circle
    }

    fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }
}

/// 型エイリアス
pub type Arc2DType<T> = Arc2DImpl<T>;

/// 3D円弧の基本トレイト
pub trait Arc3D<T: Scalar> {
    /// 基底の円を取得
    fn circle(&self) -> &Circle3DImpl<T>;

    /// 円弧の開始角度を取得
    fn start_angle(&self) -> Angle<T>;

    /// 円弧の終了角度を取得
    fn end_angle(&self) -> Angle<T>;

    /// 円弧の中心座標を取得
    fn center(&self) -> Point3D<T> {
        self.circle().center()
    }

    /// 円弧の半径を取得
    fn radius(&self) -> T {
        self.circle().radius()
    }

    /// 円弧の法線ベクトルを取得
    fn normal(&self) -> Vector3D<T> {
        self.circle().normal()
    }

    /// 円弧の角度範囲を取得
    fn angle_span(&self) -> Angle<T> {
        let start = self.start_angle().to_radians();
        let end = self.end_angle().to_radians();
        let diff = if end >= start {
            end - start
        } else {
            end + T::TAU - start
        };
        Angle::from_radians(diff)
    }

    /// 円弧の弧長を計算
    fn arc_length(&self) -> T {
        self.radius() * self.angle_span().to_radians()
    }

    /// 円弧の開始点を取得
    fn start_point(&self) -> Point3D<T> {
        self.circle().point_at_angle(self.start_angle())
    }

    /// 円弧の終了点を取得
    fn end_point(&self) -> Point3D<T> {
        self.circle().point_at_angle(self.end_angle())
    }

    /// 円弧の中点を取得
    fn midpoint(&self) -> Point3D<T> {
        let start_rad = self.start_angle().to_radians();
        let end_rad = self.end_angle().to_radians();
        let mid_rad = if end_rad >= start_rad {
            (start_rad + end_rad) / (T::ONE + T::ONE)
        } else {
            (start_rad + end_rad + T::TAU) / (T::ONE + T::ONE)
        };
        let mid_angle = Angle::from_radians(mid_rad);
        self.circle().point_at_angle(mid_angle)
    }

    /// 指定された点が円弧の平面上にあるかを判定
    fn point_on_plane(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.circle().point_on_plane(point, tolerance)
    }

    /// 指定された点が円弧上にあるかを判定
    fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // まず平面上にあるかチェック
        if !self.point_on_plane(point, tolerance) {
            return false;
        }

        // 円周からの距離をチェック
        let distance = self.circle().distance_to_point_3d(point);
        if distance > tolerance {
            return false;
        }

        // 点の角度を計算（ローカル座標系で）
        let center = self.center();
        let relative = Vector3D::new(
            point.x() - center.x(),
            point.y() - center.y(),
            point.z() - center.z(),
        );
        
        // 簡略化：XY平面での角度計算
        let point_angle = Angle::from_radians(relative.y().atan2(relative.x()));
        self.angle_contains(point_angle)
    }

    /// 指定された角度が円弧の角度範囲内にあるかを判定
    fn angle_contains(&self, angle: Angle<T>) -> bool {
        let start = self.start_angle().normalize_0_2pi().to_radians();
        let end = self.end_angle().normalize_0_2pi().to_radians();
        let test = angle.normalize_0_2pi().to_radians();

        if start <= end {
            // 角度が跨がっていない場合
            test >= start && test <= end
        } else {
            // 角度が0度を跨いでいる場合
            test >= start || test <= end
        }
    }

    /// 点から円弧への最短距離を計算
    fn distance_to_point(&self, point: &Point3D<T>) -> T {
        // 平面外の場合は平面への距離も考慮
        let center = self.center();
        let radius = self.radius();
        
        // 平面上への投影点での距離計算
        let relative = Vector3D::new(
            point.x() - center.x(),
            point.y() - center.y(),
            point.z() - center.z(),
        );
        
        // 簡略化：3D空間での直接距離計算
        let point_to_center = center.distance_to(*point);
        let point_angle = Angle::from_radians(relative.y().atan2(relative.x()));
        
        if self.angle_contains(point_angle) {
            // 角度範囲内の場合、円周からの距離
            (point_to_center - radius).abs()
        } else {
            // 角度範囲外の場合、端点からの距離
            let start_point = self.start_point();
            let end_point = self.end_point();
            let dist_to_start = point.distance_to(start_point);
            let dist_to_end = point.distance_to(end_point);
            if dist_to_start < dist_to_end { dist_to_start } else { dist_to_end }
        }
    }

    /// 円弧と円の交差判定
    fn intersects_with_circle(&self, other_circle: &Circle3DImpl<T>) -> bool {
        // 平面が同じかチェック
        let normal1 = self.normal();
        let normal2 = other_circle.normal();
        let dot = normal1.dot(&normal2).abs();
        let tolerance = T::from_f64(1e-10);
        
        if (dot - T::ONE).abs() > tolerance {
            // 異なる平面の場合、より複雑な3D交差判定が必要
            return false;
        }

        // 同一平面での交差判定
        let center1 = self.center();
        let radius1 = self.radius();
        let center2 = other_circle.center();
        let radius2 = other_circle.radius();
        
        let distance = center1.distance_to(center2);
        let radii_sum = radius1 + radius2;
        let radii_diff = (radius1 - radius2).abs();
        
        // 円同士が交差または包含している場合
        if distance <= radii_sum && distance >= radii_diff {
            // 円弧の端点が他の円内部または境界にあるかチェック
            let start_point = self.start_point();
            let end_point = self.end_point();
            
            let start_dist = start_point.distance_to(center2);
            let end_dist = end_point.distance_to(center2);
            
            start_dist <= radius2 || end_dist <= radius2 ||
            other_circle.contains_point(&start_point) || other_circle.contains_point(&end_point)
        } else {
            false
        }
    }

    /// 円弧同士の交差判定
    fn intersects_with_arc(&self, other: &impl Arc3D<T>) -> bool {
        // 平面が同じかチェック
        let normal1 = self.normal();
        let normal2 = other.normal();
        let dot = normal1.dot(&normal2).abs();
        let tolerance = T::from_f64(1e-10);
        
        if (dot - T::ONE).abs() > tolerance {
            // 異なる平面の場合、3D交差は複雑
            return false;
        }

        // 同一平面での交差判定
        let center1 = self.center();
        let radius1 = self.radius();
        let center2 = other.center();
        let radius2 = other.radius();
        
        let distance = center1.distance_to(center2);
        let radii_sum = radius1 + radius2;
        let radii_diff = (radius1 - radius2).abs();
        
        if distance > radii_sum || distance < radii_diff {
            return false;
        }

        // 簡略化：端点が相手の円弧に含まれるかチェック
        let tolerance = T::from_f64(1e-10);
        
        other.contains_point(&self.start_point(), tolerance) ||
        other.contains_point(&self.end_point(), tolerance) ||
        self.contains_point(&other.start_point(), tolerance) ||
        self.contains_point(&other.end_point(), tolerance)
    }

    /// 円弧の種類を判定
    fn arc_kind(&self) -> ArcKind {
        let span = self.angle_span().to_radians();
        let pi = T::PI;
        let tau = T::TAU;
        let tolerance = T::from_f64(1e-10);

        if (span - tau).abs() <= tolerance {
            ArcKind::FullCircle
        } else if (span - pi).abs() <= tolerance {
            ArcKind::Semicircle
        } else if span < pi {
            ArcKind::MinorArc
        } else {
            ArcKind::MajorArc
        }
    }
}

/// 3D円弧の具象実装
#[derive(Debug, Clone)]
pub struct Arc3DImpl<T: Scalar> {
    circle: Circle3DImpl<T>,
    start_angle: Angle<T>,
    end_angle: Angle<T>,
}

impl<T: Scalar> Arc3DImpl<T> {
    /// 新しい3D円弧を作成
    pub fn new(
        circle: Circle3DImpl<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Result<Self, ArcError> {
        if circle.radius() <= T::ZERO {
            return Err(ArcError::InvalidRadius);
        }

        Ok(Arc3DImpl {
            circle,
            start_angle,
            end_angle,
        })
    }

    /// 中心、半径、法線、角度範囲から3D円弧を作成
    pub fn from_center_radius_normal_angles(
        center: Point3D<T>,
        radius: T,
        normal: Vector3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Result<Self, ArcError> {
        let circle = Circle3DImpl::new(center, radius, normal);
        Self::new(circle, start_angle, end_angle)
    }
}

impl<T: Scalar> Arc3D<T> for Arc3DImpl<T> {
    fn circle(&self) -> &Circle3DImpl<T> {
        &self.circle
    }

    fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }
}

/// 型エイリアス
pub type Arc3DType<T> = Arc3DImpl<T>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts;

    #[test]
    fn test_arc2d_creation() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 1.0;
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc2DImpl::from_center_radius_angles(center, radius, start_angle, end_angle)
            .expect("円弧作成に失敗");

        assert_eq!(arc.center(), center);
        assert_eq!(arc.radius(), radius);
        assert_eq!(arc.start_angle().to_radians(), 0.0);
        assert!((arc.end_angle().to_radians() - consts::PI / 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_arc2d_arc_length() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 2.0;
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc2DImpl::from_center_radius_angles(center, radius, start_angle, end_angle)
            .expect("円弧作成に失敗");

        let expected_length = radius * consts::PI / 2.0; // 90度の弧長
        assert!((arc.arc_length() - expected_length).abs() < f64::EPSILON);
    }

    #[test]
    fn test_arc2d_points() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 1.0;
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc2DImpl::from_center_radius_angles(center, radius, start_angle, end_angle)
            .expect("円弧作成に失敗");

        let start_point = arc.start_point();
        let end_point = arc.end_point();
        let mid_point = arc.midpoint();

        // 開始点は (1, 0)
        assert!((start_point.x() - 1.0).abs() < f64::EPSILON);
        assert!(start_point.y().abs() < f64::EPSILON);

        // 終了点は (0, 1)
        assert!(end_point.x().abs() < f64::EPSILON);
        assert!((end_point.y() - 1.0).abs() < f64::EPSILON);

        // 中点は (√2/2, √2/2)
        let expected_mid = 2_f64.sqrt() / 2.0;
        assert!((mid_point.x() - expected_mid).abs() < f64::EPSILON);
        assert!((mid_point.y() - expected_mid).abs() < f64::EPSILON);
    }

    #[test]
    fn test_arc2d_invalid_radius() {
        let center = Point2D::new(0.0, 0.0);
        let radius = -1.0; // 無効な半径
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let result = Arc2DImpl::from_center_radius_angles(center, radius, start_angle, end_angle);
        assert!(matches!(result, Err(ArcError::InvalidRadius)));
    }

    #[test]
    fn test_arc3d_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let normal = Vector3D::new(0.0, 0.0, 1.0); // Z軸方向
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc3DImpl::from_center_radius_normal_angles(
            center, radius, normal, start_angle, end_angle
        ).expect("3D円弧作成に失敗");

        assert_eq!(arc.center(), center);
        assert_eq!(arc.radius(), radius);
        assert_eq!(arc.start_angle().to_radians(), 0.0);
        assert!((arc.end_angle().to_radians() - consts::PI / 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_arc3d_arc_length() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 2.0;
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc3DImpl::from_center_radius_normal_angles(
            center, radius, normal, start_angle, end_angle
        ).expect("3D円弧作成に失敗");

        let expected_length = radius * consts::PI / 2.0; // 90度の弧長
        assert!((arc.arc_length() - expected_length).abs() < f64::EPSILON);
    }

    #[test]
    fn test_arc3d_points() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc3DImpl::from_center_radius_normal_angles(
            center, radius, normal, start_angle, end_angle
        ).expect("3D円弧作成に失敗");

        let start_point = arc.start_point();
        let end_point = arc.end_point();
        let mid_point = arc.midpoint();

        // 開始点は (1, 0, 0)
        assert!((start_point.x() - 1.0).abs() < f64::EPSILON);
        assert!(start_point.y().abs() < f64::EPSILON);
        assert!(start_point.z().abs() < f64::EPSILON);

        // 終了点は (0, 1, 0)
        assert!(end_point.x().abs() < f64::EPSILON);
        assert!((end_point.y() - 1.0).abs() < f64::EPSILON);
        assert!(end_point.z().abs() < f64::EPSILON);

        // 中点は (√2/2, √2/2, 0)
        let expected_mid = 2_f64.sqrt() / 2.0;
        assert!((mid_point.x() - expected_mid).abs() < f64::EPSILON);
        assert!((mid_point.y() - expected_mid).abs() < f64::EPSILON);
        assert!(mid_point.z().abs() < f64::EPSILON);
    }

    #[test]
    fn test_arc2d_contains_point() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 2.0;
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc2DImpl::from_center_radius_angles(center, radius, start_angle, end_angle)
            .expect("円弧作成に失敗");

        let tolerance = 1e-10;

        // 円弧上の点（開始点、終了点、中点）
        assert!(arc.contains_point(&Point2D::new(2.0, 0.0), tolerance)); // 開始点
        assert!(arc.contains_point(&Point2D::new(0.0, 2.0), tolerance)); // 終了点
        
        let mid_coord = 2.0 * 2_f64.sqrt() / 2.0; // r * sqrt(2)/2
        assert!(arc.contains_point(&Point2D::new(mid_coord, mid_coord), tolerance)); // 中点

        // 円弧外の点（角度範囲外）
        assert!(!arc.contains_point(&Point2D::new(-2.0, 0.0), tolerance)); // 180度
        assert!(!arc.contains_point(&Point2D::new(0.0, -2.0), tolerance)); // 270度

        // 円周外の点
        assert!(!arc.contains_point(&Point2D::new(3.0, 0.0), tolerance)); // 半径外
        assert!(!arc.contains_point(&Point2D::new(1.0, 0.0), tolerance)); // 半径内
    }

    #[test]
    fn test_arc2d_angle_contains() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 1.0;
        let start_angle = Angle::from_degrees(30.0);
        let end_angle = Angle::from_degrees(120.0);

        let arc = Arc2DImpl::from_center_radius_angles(center, radius, start_angle, end_angle)
            .expect("円弧作成に失敗");

        // 角度範囲内
        assert!(arc.angle_contains(Angle::from_degrees(45.0)));
        assert!(arc.angle_contains(Angle::from_degrees(90.0)));
        assert!(arc.angle_contains(Angle::from_degrees(30.0))); // 境界
        assert!(arc.angle_contains(Angle::from_degrees(120.0))); // 境界

        // 角度範囲外
        assert!(!arc.angle_contains(Angle::from_degrees(0.0)));
        assert!(!arc.angle_contains(Angle::from_degrees(150.0)));
        assert!(!arc.angle_contains(Angle::from_degrees(270.0)));
    }

    #[test]
    fn test_arc2d_intersects_with_circle() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 2.0;
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc2DImpl::from_center_radius_angles(center, radius, start_angle, end_angle)
            .expect("円弧作成に失敗");

        // 交差する円
        let intersecting_circle = Circle2DImpl::new(Point2D::new(1.0, 1.0), 1.5);
        assert!(arc.intersects_with_circle(&intersecting_circle));

        // 交差しない円（遠い）
        let far_circle = Circle2DImpl::new(Point2D::new(10.0, 10.0), 1.0);
        assert!(!arc.intersects_with_circle(&far_circle));

        // 交差しない円（同心で小さい）
        let small_circle = Circle2DImpl::new(Point2D::new(0.0, 0.0), 0.5);
        assert!(!arc.intersects_with_circle(&small_circle));
    }

    #[test]
    fn test_arc2d_intersects_with_arc() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 2.0;
        let start_angle1 = Angle::from_radians(0.0);
        let end_angle1 = Angle::from_radians(consts::PI / 2.0);

        let arc1 = Arc2DImpl::from_center_radius_angles(center, radius, start_angle1, end_angle1)
            .expect("円弧1作成に失敗");

        // 重複する円弧
        let start_angle2 = Angle::from_radians(consts::PI / 4.0);
        let end_angle2 = Angle::from_radians(3.0 * consts::PI / 4.0);
        let arc2 = Arc2DImpl::from_center_radius_angles(center, radius, start_angle2, end_angle2)
            .expect("円弧2作成に失敗");

        assert!(arc1.intersects_with_arc(&arc2));

        // 非重複の円弧
        let start_angle3 = Angle::from_radians(consts::PI);
        let end_angle3 = Angle::from_radians(3.0 * consts::PI / 2.0);
        let arc3 = Arc2DImpl::from_center_radius_angles(center, radius, start_angle3, end_angle3)
            .expect("円弧3作成に失敗");

        assert!(!arc1.intersects_with_arc(&arc3));
    }

    #[test]
    fn test_arc2d_arc_kind() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 1.0;

        // 劣弧（90度）
        let minor_arc = Arc2DImpl::from_center_radius_angles(
            center, radius, 
            Angle::from_radians(0.0), 
            Angle::from_radians(consts::PI / 2.0)
        ).expect("劣弧作成に失敗");
        assert_eq!(minor_arc.arc_kind(), ArcKind::MinorArc);

        // 半円（180度）
        let semicircle = Arc2DImpl::from_center_radius_angles(
            center, radius,
            Angle::from_radians(0.0),
            Angle::from_radians(consts::PI)
        ).expect("半円作成に失敗");
        assert_eq!(semicircle.arc_kind(), ArcKind::Semicircle);

        // 優弧（270度）
        let major_arc = Arc2DImpl::from_center_radius_angles(
            center, radius,
            Angle::from_radians(0.0),
            Angle::from_radians(3.0 * consts::PI / 2.0)
        ).expect("優弧作成に失敗");
        assert_eq!(major_arc.arc_kind(), ArcKind::MajorArc);

        // 完全円（360度）
        let full_circle = Arc2DImpl::from_center_radius_angles(
            center, radius,
            Angle::from_radians(0.0),
            Angle::from_radians(2.0 * consts::PI)
        ).expect("完全円作成に失敗");
        assert_eq!(full_circle.arc_kind(), ArcKind::FullCircle);
    }

    #[test]
    fn test_arc3d_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 2.0;
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let start_angle = Angle::from_radians(0.0);
        let end_angle = Angle::from_radians(consts::PI / 2.0);

        let arc = Arc3DImpl::from_center_radius_normal_angles(
            center, radius, normal, start_angle, end_angle
        ).expect("3D円弧作成に失敗");

        let tolerance = 1e-10;

        // 円弧上の点
        assert!(arc.contains_point(&Point3D::new(2.0, 0.0, 0.0), tolerance)); // 開始点
        assert!(arc.contains_point(&Point3D::new(0.0, 2.0, 0.0), tolerance)); // 終了点

        // 平面外の点
        assert!(!arc.contains_point(&Point3D::new(2.0, 0.0, 1.0), tolerance));

        // 角度範囲外の点
        assert!(!arc.contains_point(&Point3D::new(-2.0, 0.0, 0.0), tolerance));
    }
}