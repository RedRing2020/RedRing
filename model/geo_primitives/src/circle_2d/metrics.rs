//! Circle2D 計量関連機能（ジェネリック版）

use crate::Circle2D;
use geo_contracts::Scalar;

impl<T: Scalar> Circle2D<T> {
    /// 直径を取得
    pub fn diameter(&self) -> T {
        self.radius + self.radius // 2.0 * self.radiusと同等、但しT型対応
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point2D;

    #[test]
    fn test_diameter() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0).unwrap();
        assert_eq!(circle.diameter(), 10.0);
    }
}
