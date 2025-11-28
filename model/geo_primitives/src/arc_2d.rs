//! 2次元円弧（Arc2D）の Core 実装
//!
//! Foundation統一システムに基づく Arc2D の必須機能のみ
//! 拡張機能は arc_2d_extensions.rs を参照

use crate::{Circle2D, Direction2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::{
    core::arc_core_traits::{Arc2DConstructor, Arc2DMeasure, Arc2DProperties},
    Scalar,
};

/// 2次元円弧
///
/// 基底円と角度範囲による円弧の定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc2D<T: Scalar> {
    /// 基底となる円
    circle: Circle2D<T>,
    /// 開始角度
    start_angle: Angle<T>,
    /// 終了角度
    end_angle: Angle<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい円弧を作成
    ///
    /// # 引数
    /// * `circle` - 基底となる円
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn new(circle: Circle2D<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Option<Self> {
        // 基底円の有効性チェック
        if circle.radius() <= T::ZERO {
            return None;
        }

        Some(Self {
            circle,
            start_angle,
            end_angle,
        })
    }

    /// 中心点・半径・角度から円弧を作成
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `radius` - 半径（正の値）
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn from_center_radius(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let circle = Circle2D::new(center, radius)?;
        Self::new(circle, start_angle, end_angle)
    }

    /// XY平面円弧の便利な作成メソッド（テスト用）
    ///
    /// `from_center_radius` のエイリアス
    pub fn xy_arc(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Self::from_center_radius(center, radius, start_angle, end_angle)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 基底円を取得
    pub fn circle(&self) -> &Circle2D<T> {
        &self.circle
    }

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        self.circle.center()
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.circle.radius()
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    // ========================================================================
    // Core Geometric Methods
    // ========================================================================

    /// 指定角度における点を取得
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let x = self.radius() * angle.cos();
        let y = self.radius() * angle.sin();
        self.center() + Vector2D::new(x, y)
    }

    /// 開始点を取得
    pub fn start_point(&self) -> Point2D<T> {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point2D<T> {
        self.point_at_angle(self.end_angle.to_radians())
    }

    /// 開始方向ベクトルを取得
    pub fn start_direction(&self) -> Direction2D<T> {
        let angle = self.start_angle.to_radians();
        // 円の接線方向（時計回り）
        let direction_vector = Vector2D::new(-angle.sin(), angle.cos());
        Direction2D::from_vector(direction_vector).expect("Direction vector should be valid")
    }

    /// 円弧の角度範囲を取得
    pub fn angular_span(&self) -> T {
        let start = self.start_angle.to_radians();
        let end = self.end_angle.to_radians();
        if end >= start {
            end - start
        } else {
            // 角度が逆転している場合（例：350度から10度まで）
            T::TAU - (start - end)
        }
    }

    /// 円弧の長さを計算
    pub fn arc_length(&self) -> T {
        self.radius() * self.angular_span()
    }

    /// 完全な円かどうかを判定
    pub fn is_full_circle(&self) -> bool {
        let span = self.angular_span();
        (span - T::TAU).abs() < T::EPSILON
    }
}

// ============================================================================
// geo_foundation abstracts trait implementations
// ============================================================================

/// geo_foundation::core::Arc2D<T> トレイト実装
impl<T: Scalar> geo_foundation::core::arc_traits::Arc2D<T> for Arc2D<T> {
    type Circle = Circle2D<T>;
    type Point = Point2D<T>;
    type Angle = analysis::Angle<T>;

    fn circle(&self) -> &Self::Circle {
        &self.circle
    }

    fn start_angle(&self) -> Self::Angle {
        self.start_angle
    }

    fn end_angle(&self) -> Self::Angle {
        self.end_angle
    }

    fn is_full_circle(&self) -> bool {
        self.is_full_circle()
    }

    fn start_point(&self) -> Self::Point {
        self.start_point()
    }

    fn end_point(&self) -> Self::Point {
        self.end_point()
    }
}

/// ArcMetrics トレイト実装
impl<T: Scalar> geo_foundation::core::arc_traits::ArcMetrics<T> for Arc2D<T> {
    fn arc_length(&self) -> T {
        self.arc_length()
    }

    fn sector_area(&self) -> T {
        let half_radius_squared = self.radius() * self.radius() / (T::ONE + T::ONE);
        half_radius_squared * self.angular_span()
    }

    fn central_angle(&self) -> Self::Angle {
        analysis::Angle::from_radians(self.angular_span())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_creation() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 1.0;
        let start = Angle::from_degrees(0.0);
        let end = Angle::from_degrees(90.0);

        let arc = Arc2D::from_center_radius(center, radius, start, end).unwrap();
        assert_eq!(arc.center(), center);
        assert_eq!(arc.radius(), radius);
        assert_eq!(arc.start_angle(), start);
        assert_eq!(arc.end_angle(), end);
    }

    #[test]
    fn test_arc_points() {
        let center = Point2D::new(1.0, 1.0);
        let radius = 2.0;
        let start = Angle::from_degrees(0.0);
        let end = Angle::from_degrees(90.0);

        let arc = Arc2D::from_center_radius(center, radius, start, end).unwrap();

        let start_pt = arc.start_point();
        let end_pt = arc.end_point();

        // 開始点：(center_x + radius, center_y) = (3.0, 1.0)
        assert!((start_pt.x() - 3.0).abs() < 1e-10);
        assert!((start_pt.y() - 1.0).abs() < 1e-10);

        // 終了点：(center_x, center_y + radius) = (1.0, 3.0)
        assert!((end_pt.x() - 1.0).abs() < 1e-10);
        assert!((end_pt.y() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_arc_length() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 1.0;
        let start = Angle::from_degrees(0.0);
        let end = Angle::from_degrees(90.0);

        let arc = Arc2D::from_center_radius(center, radius, start, end).unwrap();

        // 90度円弧の長さ = π/2
        let expected_length = std::f64::consts::PI / 2.0;
        assert!((arc.arc_length() - expected_length).abs() < 1e-10);
    }

    #[test]
    fn test_full_circle() {
        let center = Point2D::new(0.0, 0.0);
        let radius = 1.0;
        let start = Angle::from_degrees(0.0);
        let end = Angle::from_degrees(360.0);

        let arc = Arc2D::from_center_radius(center, radius, start, end).unwrap();
        assert!(arc.is_full_circle());
    }
}

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

impl<T: Scalar> Arc2DConstructor<T> for Arc2D<T> {
    fn new(center: (T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self> {
        let center_point = Point2D::new(center.0, center.1);
        let circle = Circle2D::new(center_point, radius)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Self::new(circle, start, end)
    }

    fn from_three_points(start: (T, T), mid: (T, T), end: (T, T)) -> Option<Self> {
        // 3点から円を求める
        let p1 = Point2D::new(start.0, start.1);
        let p2 = Point2D::new(mid.0, mid.1);
        let p3 = Point2D::new(end.0, end.1);

        // 外接円の中心と半径を計算
        let center = Self::circumcenter(p1, p2, p3)?;
        let radius = center.distance_to(&p1);

        // 各点の角度を計算
        let start_angle = Self::angle_from_center(center, p1);
        let mid_angle = Self::angle_from_center(center, p2);
        let end_angle = Self::angle_from_center(center, p3);

        // 角度の順序を確認（start→mid→endの順になるようにする）
        let (start_rad, end_rad) = if Self::is_angle_between(start_angle, mid_angle, end_angle) {
            (start_angle, end_angle)
        } else {
            (end_angle, start_angle)
        };

        let circle = Circle2D::new(center, radius)?;
        Self::new(circle, Angle::from_radians(start_rad), Angle::from_radians(end_rad))
    }

    fn semicircle(center: (T, T), radius: T) -> Self {
        let center_point = Point2D::new(center.0, center.1);
        let circle = Circle2D::new(center_point, radius).unwrap();
        let start = Angle::from_radians(T::ZERO);
        let end = Angle::from_radians(T::PI);
        Self::new(circle, start, end).unwrap()
    }
}

impl<T: Scalar> Arc2DProperties<T> for Arc2D<T> {
    fn center(&self) -> (T, T) {
        let c = self.center();
        (c.x(), c.y())
    }

    fn radius(&self) -> T {
        self.radius()
    }

    fn start_angle(&self) -> T {
        self.start_angle.to_radians()
    }

    fn end_angle(&self) -> T {
        self.end_angle.to_radians()
    }

    fn dimension(&self) -> u32 {
        2
    }
}

impl<T: Scalar> Arc2DMeasure<T> for Arc2D<T> {
    fn measure(&self) -> T {
        self.arc_length()
    }

    fn start_point(&self) -> (T, T) {
        let p = self.start_point();
        (p.x(), p.y())
    }

    fn end_point(&self) -> (T, T) {
        let p = self.end_point();
        (p.x(), p.y())
    }

    fn point_at_parameter(&self, t: T) -> (T, T) {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();
        let angle = start_rad + t * (end_rad - start_rad);
        let p = self.point_at_angle(angle);
        (p.x(), p.y())
    }
}

// ============================================================================
// Helper methods for Arc2D
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    /// 3点の外接円の中心を計算
    fn circumcenter(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Point2D<T>> {
        let x1 = p1.x();
        let y1 = p1.y();
        let x2 = p2.x();
        let y2 = p2.y();
        let x3 = p3.x();
        let y3 = p3.y();

        let d = (x1 - x2) * (y2 - y3) - (x2 - x3) * (y1 - y2);
        if d.abs() < T::EPSILON {
            return None; // 3点が一直線上にある
        }

        let two = T::ONE + T::ONE;
        let ux = ((x1 * x1 + y1 * y1) * (y2 - y3) + (x2 * x2 + y2 * y2) * (y3 - y1)
            + (x3 * x3 + y3 * y3) * (y1 - y2))
            / (two * d);
        let uy = ((x1 * x1 + y1 * y1) * (x3 - x2) + (x2 * x2 + y2 * y2) * (x1 - x3)
            + (x3 * x3 + y3 * y3) * (x2 - x1))
            / (two * d);

        Some(Point2D::new(ux, uy))
    }

    /// 中心からの点の角度を計算
    fn angle_from_center(center: Point2D<T>, point: Point2D<T>) -> T {
        let dx = point.x() - center.x();
        let dy = point.y() - center.y();
        dy.atan2(dx)
    }

    /// 角度がstart→end範囲内にあるかチェック
    fn is_angle_between(start: T, mid: T, end: T) -> bool {
        let normalize = |angle: T| {
            let mut a = angle;
            while a < T::ZERO {
                a = a + T::TAU;
            }
            while a >= T::TAU {
                a = a - T::TAU;
            }
            a
        };

        let s = normalize(start);
        let m = normalize(mid);
        let e = normalize(end);

        if s <= e {
            s <= m && m <= e
        } else {
            s <= m || m <= e
        }
    }
}
