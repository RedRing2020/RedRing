//! 2D Arc implementation - geo_foundation Arc2D trait implementation
//!
//! geo_foundationのArc2Dトレイトを実装した2次元円弧

use crate::geometry2d::{Circle, Point2D};
use geo_foundation::{
    geometry::{Angle, Circle2DImpl, Point2D as FoundationPoint2D},
    geometry::arc::{Arc2D as Arc2DTrait, ArcKind},
};
use geo_foundation::common::constants::GEOMETRIC_TOLERANCE;

/// 2D平面上の円弧を表現する構造体（geo_foundation Arc2D trait実装）
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    circle: Circle, // geo_primitives Circle
    foundation_circle: Circle2DImpl<f64>, // geo_foundation用の円
    start_angle: Angle<f64>,
    end_angle: Angle<f64>,
}

impl Arc {
    /// 新しい円弧を作成
    pub fn new(circle: Circle, start_angle: Angle<f64>, end_angle: Angle<f64>) -> Self {
        // geo_primitives CircleをCircle2DImplに変換
        let foundation_circle = Circle2DImpl::new(
            FoundationPoint2D::new(circle.center().x(), circle.center().y()),
            circle.radius(),
        );

        Self {
            circle,
            foundation_circle,
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
    pub fn from_three_points(start: Point2D, mid: Point2D, end: Point2D) -> Option<Self> {
        let circle = Circle::from_three_points(start, mid, end)?;
        
        // 各点の角度を計算（ラジアン）
        let start_angle_rad = Self::point_to_angle_rad(&circle, start);
        let end_angle_rad = Self::point_to_angle_rad(&circle, end);
        
        Some(Self::from_radians(circle, start_angle_rad, end_angle_rad))
    }

    /// 点から角度を計算（ラジアン）
    fn point_to_angle_rad(circle: &Circle, point: Point2D) -> f64 {
        let center = circle.center();
        let dx = point.x() - center.x();
        let dy = point.y() - center.y();
        dy.atan2(dx)
    }
}

// geo_foundation Arc2D トレイトの実装
impl Arc2DTrait<f64> for Arc {
    fn circle(&self) -> &Circle2DImpl<f64> {
        &self.foundation_circle
    }

    fn start_angle(&self) -> Angle<f64> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<f64> {
        self.end_angle
    }
}

// 追加のメソッド（geo_primitives独自 + Arc2Dトレイトのデフォルト実装補完）
impl Arc {
    /// geo_primitives Circleを取得
    pub fn primitives_circle(&self) -> &Circle {
        &self.circle
    }

    /// 指定角度での点を取得（ラジアン）
    pub fn point_at_angle(&self, angle: f64) -> Point2D {
        let center = self.circle.center();
        Point2D::new(
            center.x() + self.circle.radius() * angle.cos(),
            center.y() + self.circle.radius() * angle.sin(),
        )
    }

    /// 指定角度での点を取得（Angle型）
    pub fn point_at_angle_typed(&self, angle: Angle<f64>) -> Point2D {
        self.point_at_angle(angle.to_radians())
    }

    /// 指定された角度が円弧の角度範囲内にあるかを判定
    pub fn angle_contains(&self, angle: Angle<f64>) -> bool {
        let start = self.start_angle.to_radians();
        let end = self.end_angle.to_radians();
        let test = angle.to_radians();

        if start <= end {
            // 角度が跨がっていない場合
            test >= start && test <= end
        } else {
            // 角度が0度を跨いでいる場合
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
            end + 2.0 * std::f64::consts::PI - start
        };
        Angle::from_radians(diff)
    }

    /// 円弧の弧長を計算
    pub fn arc_length(&self) -> f64 {
        self.circle.radius() * self.angle_span().to_radians()
    }

    /// 円弧の開始点を取得
    pub fn start_point(&self) -> Point2D {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 円弧の終了点を取得
    pub fn end_point(&self) -> Point2D {
        self.point_at_angle(self.end_angle.to_radians())
    }

    /// 円弧の中心を取得
    pub fn center(&self) -> Point2D {
        self.circle.center()
    }

    /// 円弧の半径を取得
    pub fn radius(&self) -> f64 {
        self.circle.radius()
    }

    /// 点が円弧上にあるかチェック
    pub fn contains_point(&self, point: Point2D) -> bool {
        // まず円上にあるかチェック
        if !self.circle.contains_point_on_boundary(&point, GEOMETRIC_TOLERANCE) {
            return false;
        }

        // 角度範囲内にあるかチェック
        let point_angle = Angle::from_radians(Self::point_to_angle_rad(&self.circle, point));
        self.angle_contains(point_angle)
    }

    /// 他の円弧との交差判定
    pub fn intersects_with_arc(&self, other: &Arc) -> bool {
        // 基底円同士の交差判定
        if !self.circle.intersects_with_circle(&other.circle) {
            return false;
        }

        // 角度範囲の重複判定（簡易版）
        let self_start = self.start_angle.to_radians();
        let self_end = self.end_angle.to_radians();
        let other_start = other.start_angle.to_radians();
        let other_end = other.end_angle.to_radians();

        // 角度範囲の正規化と重複チェック
        // TODO: より正確な角度重複判定の実装
        !(self_end < other_start || other_end < self_start)
    }

    /// 円弧の種類を判定
    pub fn arc_kind(&self) -> ArcKind {
        let span = self.angle_span().to_radians();
        let pi = std::f64::consts::PI;
        let two_pi = 2.0 * pi;

        if (span - two_pi).abs() < GEOMETRIC_TOLERANCE {
            ArcKind::FullCircle
        } else if (span - pi).abs() < GEOMETRIC_TOLERANCE {
            ArcKind::Semicircle
        } else if span < pi {
            ArcKind::MinorArc
        } else {
            ArcKind::MajorArc
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_arc_creation() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 5.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        assert_eq!(arc.center().x(), 0.0);
        assert_eq!(arc.center().y(), 0.0);
        assert_eq!(arc.radius(), 5.0);
        assert_eq!(arc.start_angle().to_radians(), 0.0);
        assert_eq!(arc.end_angle().to_radians(), PI);
    }

    #[test]
    fn test_arc_points() {
        let center = Point2D::new(2.0, 3.0);
        let circle = Circle::new(center, 4.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);

        let start = arc.start_point();
        let end = arc.end_point();

        assert!((start.x() - 6.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((start.y() - 3.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((end.x() - 2.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((end.y() - 7.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_arc_length() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 3.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        let expected_length = 3.0 * PI;
        assert!((arc.arc_length() - expected_length).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_arc_kind() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 1.0);

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
    fn test_from_three_points() {
        let p1 = Point2D::new(1.0, 0.0);
        let p2 = Point2D::new(0.0, 1.0);
        let p3 = Point2D::new(-1.0, 0.0);

        let arc = Arc::from_three_points(p1, p2, p3).expect("円弧作成に失敗");
        
        assert!((arc.center().x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((arc.center().y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((arc.radius() - 1.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_angle_contains() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        assert!(arc.angle_contains(Angle::from_radians(PI / 4.0)));
        assert!(arc.angle_contains(Angle::from_radians(PI / 2.0)));
        assert!(!arc.angle_contains(Angle::from_radians(3.0 * PI / 2.0)));
    }

    #[test]
    fn test_contains_point() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        // 円弧上の点（開始点）
        let point_on_arc_start = Point2D::new(1.0, 0.0);
        assert!(arc.contains_point(point_on_arc_start));

        // 円弧上の点（終了点）
        let point_on_arc_end = Point2D::new(-1.0, 0.0);
        assert!(arc.contains_point(point_on_arc_end));

        // 円上だが角度範囲外の点（270度の位置）
        let point_off_arc = Point2D::new(0.0, -1.0);
        assert!(!arc.contains_point(point_off_arc));

        // 円外の点
        let point_outside = Point2D::new(2.0, 0.0);
        assert!(!arc.contains_point(point_outside));
    }
}