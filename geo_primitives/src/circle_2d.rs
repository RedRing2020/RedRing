//! 2次元円（Circle2D）のCore実装
//!
//! Foundation統一システムに基づくCircle2Dの必須機能のみ

use crate::Point2D;
use geo_foundation::Scalar;

/// 2次元円
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
}

impl<T: Scalar> Circle2D<T> {
    /// 新しい円を作成
    pub fn new(center: Point2D<T>, radius: T) -> Option<Self> {
        if radius > T::ZERO {
            Some(Self { center, radius })
        } else {
            None
        }
    }

    /// 中心を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// バウンディングボックスを取得
    pub fn bounding_box(&self) -> (Point2D<T>, Point2D<T>) {
        let min_point = Point2D::new(self.center.x() - self.radius, self.center.y() - self.radius);
        let max_point = Point2D::new(self.center.x() + self.radius, self.center.y() + self.radius);
        (min_point, max_point)
    }

    /// 円周の長さ
    pub fn circumference(&self) -> T {
        T::TAU * self.radius
    }

    /// 円の面積
    pub fn area(&self) -> T {
        T::PI * self.radius * self.radius
    }

    /// 点が円内部にあるか判定
    pub fn contains_point(&self, point: Point2D<T>) -> bool {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let distance_squared = dx * dx + dy * dy;
        distance_squared < self.radius * self.radius
    }

    /// パラメータでの点を取得
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = T::TAU * t;
        Point2D::new(
            self.center.x() + self.radius * angle.cos(),
            self.center.y() + self.radius * angle.sin(),
        )
    }

    /// 点から円周への距離
    pub fn distance_to_point(&self, point: Point2D<T>) -> T {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let center_distance = (dx * dx + dy * dy).sqrt();
        (center_distance - self.radius).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_creation() {
        let center = Point2D::new(1.0, 2.0);
        let circle = Circle2D::new(center, 3.0).unwrap();
        assert_eq!(circle.center(), center);
        assert_eq!(circle.radius(), 3.0);
    }

    #[test]
    fn test_circle_invalid_radius() {
        let center = Point2D::new(0.0, 0.0);
        assert!(Circle2D::new(center, 0.0).is_none());
        assert!(Circle2D::new(center, -1.0).is_none());
    }

    #[test]
    fn test_bounding_box() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle2D::new(center, 2.0).unwrap();
        let (min, max) = circle.bounding_box();
        assert_eq!(min, Point2D::new(-2.0, -2.0));
        assert_eq!(max, Point2D::new(2.0, 2.0));
    }

    #[test]
    fn test_metrics() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle2D::new(center, 1.0).unwrap();

        assert!((circle.circumference() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
        assert!((circle.area() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_containment() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle2D::new(center, 1.0).unwrap();

        assert!(circle.contains_point(Point2D::new(0.0, 0.0))); // 中心
        assert!(circle.contains_point(Point2D::new(0.5, 0.0))); // 内部
        assert!(!circle.contains_point(Point2D::new(2.0, 0.0))); // 外部
    }

    #[test]
    fn test_parametric() {
        let center = Point2D::new(0.0, 0.0);
        let circle = Circle2D::new(center, 1.0).unwrap();

        let p0 = circle.point_at_parameter(0.0);
        let p_quarter = circle.point_at_parameter(0.25);
        let p_half = circle.point_at_parameter(0.5);

        assert!((p0.x() - 1.0).abs() < 1e-10);
        assert!((p0.y() - 0.0).abs() < 1e-10);
        assert!((p_quarter.x() - 0.0).abs() < 1e-10);
        assert!((p_quarter.y() - 1.0).abs() < 1e-10);
        assert!((p_half.x() - (-1.0)).abs() < 1e-10);
        assert!((p_half.y() - 0.0).abs() < 1e-10);
    }
}
