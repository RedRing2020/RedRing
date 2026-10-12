//! Ellipse2D の Bounds 実装

use crate::{Ellipse2D, Point2D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb2D;

/// 回転を考慮した楕円の境界ボックス
impl<T: Scalar> Bounded<T> for Ellipse2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let rotation = self.rotation.to_radians();
        let (cos_theta, sin_theta) = (rotation.cos(), rotation.sin());
        let (a, b) = (self.semi_major, self.semi_minor);

        // 回転楕円の幅と高さ
        let width = ((a * cos_theta) * (a * cos_theta) + (b * sin_theta) * (b * sin_theta)).sqrt();
        let height = ((a * sin_theta) * (a * sin_theta) + (b * cos_theta) * (b * cos_theta)).sqrt();

        Some(Aabb2D::new(
            Point2D::new(self.center.x() - width, self.center.y() - height),
            Point2D::new(self.center.x() + width, self.center.y() + height),
        ))
    }
}
