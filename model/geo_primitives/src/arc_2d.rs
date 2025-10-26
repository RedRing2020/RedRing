//! 2次元円弧（Arc2D）の Core 実装
//!
//! Foundation統一システムに基づく Arc2D の必須機能のみ
//! 拡張機能は arc_2d_extensions.rs を参照

use crate::{Circle2D, Direction2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::Scalar;

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
