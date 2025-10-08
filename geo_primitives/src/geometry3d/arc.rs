//! 3D Arc implementation
//!
//! 3次元円弧の基本実装

use crate::geometry3d::{Circle, Point3D, Vector3D};
use geo_foundation::abstract_types::geometry::angle::Angle;
use std::f64::consts::PI;

/// 円弧の種類を表現する列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcKind {
    /// 短弧（π未満）
    MinorArc,
    /// 長弧（πより大きい）
    MajorArc,
    /// 半円（π）
    Semicircle,
    /// 完全な円（2π）
    FullCircle,
}

/// 幾何計算用の許容誤差
const GEOMETRIC_TOLERANCE: f64 = 1e-10;

/// 3D空間上の円弧を表現する構造体
#[derive(Debug, Clone)]
pub struct Arc {
    circle: Circle,
    start_angle: Angle<f64>,
    end_angle: Angle<f64>,
}

impl Arc {
    /// 新しい円弧を作成
    pub fn new(circle: Circle, start_angle: Angle<f64>, end_angle: Angle<f64>) -> Self {
        Self {
            circle,
            start_angle,
            end_angle,
        }
    }

    /// ラジアン角度から円弧を作成（利便性メソッド）
    pub fn from_radians(circle: Circle, start_angle: f64, end_angle: f64) -> Self {
        Self::new(
            circle,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    /// 度数角度から円弧を作成（利便性メソッド）
    pub fn from_degrees(circle: Circle, start_angle: f64, end_angle: f64) -> Self {
        Self::new(
            circle,
            Angle::from_degrees(start_angle),
            Angle::from_degrees(end_angle),
        )
    }

    /// 3点から円弧を作成
    pub fn from_three_points(start: Point3D, mid: Point3D, end: Point3D) -> Option<Self> {
        let circle = Circle::from_three_points(start, mid, end)?;
        
        // 各点の角度を計算（ローカル平面での角度）
        let start_angle_rad = Self::point_to_angle_rad(&circle, start);
        let end_angle_rad = Self::point_to_angle_rad(&circle, end);
        
        Some(Self::from_radians(circle, start_angle_rad, end_angle_rad))
    }

    /// 点から角度を計算（円の平面内での角度）
    fn point_to_angle_rad(circle: &Circle, point: Point3D) -> f64 {
        let center = circle.center();
        let normal = circle.normal();
        
        // 点を円の平面に投影
        let to_point = Vector3D::new(
            point.x() - center.x(),
            point.y() - center.y(),
            point.z() - center.z(),
        );
        
        // 円の平面内での基準ベクトルを計算（x軸方向のベクトル）
        let reference = if normal.z().abs() < 0.9 {
            Vector3D::new(0.0, 0.0, 1.0).cross(&normal).normalize().unwrap_or(Vector3D::unit_x())
        } else {
            Vector3D::new(1.0, 0.0, 0.0).cross(&normal).normalize().unwrap_or(Vector3D::unit_y())
        };
        
        let y_axis = normal.cross(&reference);
        
        // 平面内での座標を計算
        let x = to_point.dot(&reference);
        let y = to_point.dot(&y_axis);
        
        y.atan2(x)
    }

    /// 基底円を取得
    pub fn circle(&self) -> &Circle {
        &self.circle
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<f64> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<f64> {
        self.end_angle
    }

    /// 指定角度での点を取得（ラジアン）
    pub fn point_at_angle(&self, angle: f64) -> Point3D {
        let center = self.circle.center();
        let normal = self.circle.normal();
        let radius = self.circle.radius();
        
        // 円の平面内での基準ベクトルを計算
        let reference = if normal.z().abs() < 0.9 {
            Vector3D::new(0.0, 0.0, 1.0).cross(&normal).normalize().unwrap_or(Vector3D::unit_x())
        } else {
            Vector3D::new(1.0, 0.0, 0.0).cross(&normal).normalize().unwrap_or(Vector3D::unit_y())
        };
        
        let y_axis = normal.cross(&reference);
        
        // 角度に基づいて点を計算
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        
        let point_vec = reference * x + y_axis * y;
        
        Point3D::new(
            center.x() + point_vec.x(),
            center.y() + point_vec.y(),
            center.z() + point_vec.z(),
        )
    }

    /// 指定角度での点を取得（Angle型）
    pub fn point_at_angle_typed(&self, angle: Angle<f64>) -> Point3D {
        self.point_at_angle(angle.to_radians())
    }

    /// 指定された角度が円弧の角度範囲内にあるかを判定
    pub fn angle_contains(&self, angle: Angle<f64>) -> bool {
        let start = self.start_angle.to_radians();
        let end = self.end_angle.to_radians();
        let test = angle.to_radians();

        if start <= end {
            test >= start && test <= end
        } else {
            test >= start || test <= end
        }
    }

    /// 円弧の角度範囲を取得
    pub fn angle_span(&self) -> Angle<f64> {
        let start = self.start_angle.to_radians();
        let end = self.end_angle.to_radians();
        let diff = if end >= start {
            end - start
        } else {
            end + 2.0 * PI - start
        };
        Angle::from_radians(diff)
    }

    /// 円弧の弧長を計算
    pub fn arc_length(&self) -> f64 {
        self.circle.radius() * self.angle_span().to_radians()
    }

    /// 円弧の開始点を取得
    pub fn start_point(&self) -> Point3D {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 円弧の終了点を取得
    pub fn end_point(&self) -> Point3D {
        self.point_at_angle(self.end_angle.to_radians())
    }

    /// 円弧の中点を取得
    pub fn midpoint(&self) -> Point3D {
        let mid_angle = (self.start_angle.to_radians() + self.end_angle.to_radians()) / 2.0;
        self.point_at_angle(mid_angle)
    }

    /// 円弧の中心を取得
    pub fn center(&self) -> Point3D {
        self.circle.center()
    }

    /// 円弧の半径を取得
    pub fn radius(&self) -> f64 {
        self.circle.radius()
    }

    /// 円弧の法線ベクトルを取得
    pub fn normal(&self) -> Vector3D {
        self.circle.normal()
    }

    /// 点が円弧上にあるかチェック
    pub fn contains_point(&self, point: Point3D) -> bool {
        // まず円上にあるかチェック
        if !self.circle.contains_point_on_boundary(&point, GEOMETRIC_TOLERANCE) {
            return false;
        }

        // 角度範囲内にあるかチェック
        let point_angle = Angle::from_radians(Self::point_to_angle_rad(&self.circle, point));
        self.angle_contains(point_angle)
    }

    /// 円弧の種類を判定
    pub fn arc_kind(&self) -> ArcKind {
        let span = self.angle_span().to_radians();
        let two_pi = 2.0 * PI;

        if (span - two_pi).abs() < GEOMETRIC_TOLERANCE {
            ArcKind::FullCircle
        } else if (span - PI).abs() < GEOMETRIC_TOLERANCE {
            ArcKind::Semicircle
        } else if span < PI {
            ArcKind::MinorArc
        } else {
            ArcKind::MajorArc
        }
    }

    /// 円弧を反転（開始と終了を入れ替え）
    pub fn reverse(&self) -> Self {
        Self::new(self.circle.clone(), self.end_angle, self.start_angle)
    }

    /// 円弧を指定した分割数で近似する点列を取得
    pub fn approximate_with_points(&self, num_segments: usize) -> Vec<Point3D> {
        if num_segments == 0 {
            return vec![];
        }

        let mut points = Vec::with_capacity(num_segments + 1);
        let span = self.angle_span().to_radians();
        
        for i in 0..=num_segments {
            let t = i as f64 / num_segments as f64;
            let angle = self.start_angle.to_radians() + span * t;
            let point = self.point_at_angle(angle);
            points.push(point);
        }
        
        points
    }

    /// 他の円弧との交差判定
    pub fn intersects_with_arc(&self, other: &Arc) -> bool {
        // 基底円同士の交差判定（簡易版）
        if !self.circle.intersects_with_circle(&other.circle) {
            return false;
        }

        // 角度範囲の重複判定（簡易版）
        let self_start = self.start_angle.to_radians();
        let self_end = self.end_angle.to_radians();
        let other_start = other.start_angle.to_radians();
        let other_end = other.end_angle.to_radians();

        !(self_end < other_start || other_end < self_start)
    }

    /// 円弧をスケール
    pub fn scale(&self, factor: f64) -> Self {
        Self::new(
            self.circle.scale(factor),
            self.start_angle,
            self.end_angle,
        )
    }

    /// 円弧を平行移動
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Self {
        let vector = Vector3D::new(dx, dy, dz);
        Self::new(
            self.circle.translate(&vector),
            self.start_angle,
            self.end_angle,
        )
    }

    /// 円弧を平行移動（Vector3D版）
    pub fn translate_by_vector(&self, vector: &Vector3D) -> Self {
        Self::new(
            self.circle.translate(vector),
            self.start_angle,
            self.end_angle,
        )
    }

    /// 円弧を回転
    pub fn rotate(&self, angle: Angle<f64>) -> Self {
        Self::new(
            self.circle.clone(), // 中心周りの回転では円は変わらない
            Angle::from_radians(self.start_angle.to_radians() + angle.to_radians()),
            Angle::from_radians(self.end_angle.to_radians() + angle.to_radians()),
        )
    }
}

impl PartialEq for Arc {
    fn eq(&self, other: &Self) -> bool {
        self.circle == other.circle
            && (self.start_angle.to_radians() - other.start_angle.to_radians()).abs() < GEOMETRIC_TOLERANCE
            && (self.end_angle.to_radians() - other.end_angle.to_radians()).abs() < GEOMETRIC_TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry3d::Direction3D;

    #[test]
    fn test_arc_creation() {
        use geo_foundation::abstract_types::geometry::Direction;
        
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 5.0, normal_dir, u_axis_dir);
        let arc = Arc::from_radians(circle, 0.0, PI);

        assert_eq!(arc.center().x(), 0.0);
        assert_eq!(arc.center().y(), 0.0);
        assert_eq!(arc.center().z(), 0.0);
        assert_eq!(arc.radius(), 5.0);
        assert_eq!(arc.start_angle().to_radians(), 0.0);
        assert_eq!(arc.end_angle().to_radians(), PI);
    }

    #[test]
    fn test_arc_length() {
        use geo_foundation::abstract_types::geometry::Direction;
        
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 3.0, normal_dir, u_axis_dir);
        let arc = Arc::from_radians(circle, 0.0, PI);

        let expected_length = 3.0 * PI;
        assert!((arc.arc_length() - expected_length).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_arc_kind() {
        use geo_foundation::abstract_types::geometry::Direction;
        
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);

        let minor_arc = Arc::from_radians(circle.clone(), 0.0, PI / 3.0);
        assert_eq!(minor_arc.arc_kind(), ArcKind::MinorArc);

        let major_arc = Arc::from_radians(circle.clone(), 0.0, 4.0 * PI / 3.0);
        assert_eq!(major_arc.arc_kind(), ArcKind::MajorArc);

        let semicircle = Arc::from_radians(circle.clone(), 0.0, PI);
        assert_eq!(semicircle.arc_kind(), ArcKind::Semicircle);

        let full_circle = Arc::from_radians(circle, 0.0, 2.0 * PI);
        assert_eq!(full_circle.arc_kind(), ArcKind::FullCircle);
    }

    #[test]
    fn test_angle_contains() {
        use geo_foundation::abstract_types::geometry::Direction;
        
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
        let arc = Arc::from_radians(circle, 0.0, PI);

        assert!(arc.angle_contains(Angle::from_radians(PI / 4.0)));
        assert!(arc.angle_contains(Angle::from_radians(PI / 2.0)));
        assert!(!arc.angle_contains(Angle::from_radians(3.0 * PI / 2.0)));
    }

    #[test]
    fn test_arc_reverse() {
        use geo_foundation::abstract_types::geometry::Direction;
        
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
        let reversed = arc.reverse();

        assert_eq!(reversed.start_angle(), arc.end_angle());
        assert_eq!(reversed.end_angle(), arc.start_angle());
    }

    #[test]
    fn test_arc_midpoint() {
        use geo_foundation::abstract_types::geometry::Direction;
        
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
        let mid = arc.midpoint();

        let expected_angle = PI / 4.0;
        let expected_x = expected_angle.cos();
        let expected_y = expected_angle.sin();

        assert!((mid.x() - expected_x).abs() < GEOMETRIC_TOLERANCE);
        assert!((mid.y() - expected_y).abs() < GEOMETRIC_TOLERANCE);
        assert!((mid.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_approximate_with_points() {
        use geo_foundation::abstract_types::geometry::Direction;
        
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_axis = Vector3D::new(1.0, 0.0, 0.0);
        let normal_dir = Direction3D::from_vector(normal).unwrap();
        let u_axis_dir = Direction3D::from_vector(u_axis).unwrap();
        let circle = Circle::new(center, 1.0, normal_dir, u_axis_dir);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);

        let points = arc.approximate_with_points(4);
        assert_eq!(points.len(), 5); // 4セグメント = 5点

        // 最初と最後の点をチェック
        let first_point = points.first().unwrap();
        let last_point = points.last().unwrap();
        
        assert!((first_point.x() - 1.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((first_point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((first_point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((last_point.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((last_point.y() - 1.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((last_point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }
}