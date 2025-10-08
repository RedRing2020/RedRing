//! 2D Arc implementation
//!
//! 2次元円弧の基本実装

use crate::geometry2d::{Circle, Point2D, Vector2D};
use geo_foundation::abstract_types::Angle;
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

/// 2D平面上の円弧を表現する構造体
#[derive(Debug, Clone)]
pub struct Arc {
    circle: Circle<f64>,
    start_angle: Angle<f64>,
    end_angle: Angle<f64>,
}

impl Arc {
    /// 新しい円弧を作成
    pub fn new(circle: Circle<f64>, start_angle: Angle<f64>, end_angle: Angle<f64>) -> Self {
        Self {
            circle,
            start_angle,
            end_angle,
        }
    }

    /// ラジアン角度から円弧を作成（利便性メソッド）
    pub fn from_radians(circle: Circle<f64>, start_angle: f64, end_angle: f64) -> Self {
        Self::new(
            circle,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    /// 度数角度から円弧を作成（利便性メソッド）
    pub fn from_degrees(circle: Circle<f64>, start_angle: f64, end_angle: f64) -> Self {
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
    fn point_to_angle_rad(circle: &Circle<f64>, point: Point2D) -> f64 {
        let center = circle.center();
        let dx = point.x() - center.x();
        let dy = point.y() - center.y();
        dy.atan2(dx)
    }

    /// 基底円を取得
    pub fn circle(&self) -> &Circle<f64> {
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
            end + 2.0 * PI - start
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

    /// 円弧の中点を取得
    pub fn midpoint(&self) -> Point2D {
        let mid_angle = (self.start_angle.to_radians() + self.end_angle.to_radians()) / 2.0;
        self.point_at_angle(mid_angle)
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
        if !self
            .circle
            .contains_point_on_boundary(&point, GEOMETRIC_TOLERANCE)
        {
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
    pub fn approximate_with_points(&self, num_segments: usize) -> Vec<Point2D> {
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

    /// 円弧をスケール
    pub fn scale(&self, factor: f64) -> Self {
        Self::new(self.circle.scale(factor), self.start_angle, self.end_angle)
    }

    /// 円弧を平行移動
    pub fn translate(&self, dx: f64, dy: f64) -> Self {
        let vector = Vector2D::new(dx, dy);
        Self::new(
            self.circle.translate(&vector),
            self.start_angle,
            self.end_angle,
        )
    }

    /// 円弧を平行移動（Vector2D版）
    pub fn translate_by_vector(&self, vector: &Vector2D) -> Self {
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
            && (self.start_angle.to_radians() - other.start_angle.to_radians()).abs()
                < GEOMETRIC_TOLERANCE
            && (self.end_angle.to_radians() - other.end_angle.to_radians()).abs()
                < GEOMETRIC_TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_arc_reverse() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
        let reversed = arc.reverse();

        assert_eq!(reversed.start_angle(), arc.end_angle());
        assert_eq!(reversed.end_angle(), arc.start_angle());
    }

    #[test]
    fn test_arc_midpoint() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
        let mid = arc.midpoint();

        let expected_angle = PI / 4.0;
        let expected_x = expected_angle.cos();
        let expected_y = expected_angle.sin();

        assert!((mid.x() - expected_x).abs() < GEOMETRIC_TOLERANCE);
        assert!((mid.y() - expected_y).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_approximate_with_points() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);

        let points = arc.approximate_with_points(4);
        assert_eq!(points.len(), 5); // 4セグメント = 5点

        // 最初と最後の点をチェック
        let first_point = points.first().unwrap();
        let last_point = points.last().unwrap();

        assert!((first_point.x() - 1.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((first_point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((last_point.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((last_point.y() - 1.0).abs() < GEOMETRIC_TOLERANCE);
    }
}
