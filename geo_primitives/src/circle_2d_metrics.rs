//! Circle2D 計量関連機能（ジェネリック版）

use crate::{Circle2D, Point2D};
use geo_foundation::Scalar;

impl<T: Scalar> Circle2D<T> {
    /// 直径を取得
    pub fn diameter(&self) -> T {
        self.radius() + self.radius() // 2.0 * self.radius()と同等、但しT型対応
    }

    /// 点から円の中心までの距離
    pub fn distance_to_center(&self, point: Point2D<T>) -> T {
        let dx = point.x() - self.center().x();
        let dy = point.y() - self.center().y();
        (dx * dx + dy * dy).sqrt()
    }

    /// 点から円周までの符号付き距離
    pub fn signed_distance_to_circle(&self, point: Point2D<T>) -> T {
        self.distance_to_center(point) - self.radius()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diameter() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0).unwrap();
        assert_eq!(circle.diameter(), 10.0);
    }

    #[test]
    fn test_distance_to_center() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let point = Point2D::new(3.0, 4.0);
        assert_eq!(circle.distance_to_center(point), 5.0);
    }

    #[test]
    fn test_signed_distance() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let inside = Point2D::new(0.5, 0.0);
        assert!(circle.signed_distance_to_circle(inside) < 0.0);

        let outside = Point2D::new(2.0, 0.0);
        assert!(circle.signed_distance_to_circle(outside) > 0.0);
    }
}
