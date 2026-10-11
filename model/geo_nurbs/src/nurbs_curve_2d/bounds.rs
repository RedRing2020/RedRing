//! `NurbsCurve2D` の Bounds 実装
//!
//! `Bounded` trait を提供する。

use crate::{NurbsCurve2D, Scalar};
use geo_contracts::Bounded;
use geo_contracts::NurbsCurve2DProperties;

impl<T: Scalar> Bounded<T> for NurbsCurve2D<T> {
    type Aabb = geo_core::Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 制御点ベースの境界ボックスを計算
        let mut min_x = T::from_f64(f64::INFINITY);
        let mut min_y = T::from_f64(f64::INFINITY);
        let mut max_x = T::from_f64(f64::NEG_INFINITY);
        let mut max_y = T::from_f64(f64::NEG_INFINITY);

        for i in 0..self.num_control_points() {
            let point = self.control_point(i);
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
        }

        Some(geo_core::Aabb2D::new(
            geo_core::Point2D::new(min_x, min_y),
            geo_core::Point2D::new(max_x, max_y),
        ))
    }
}
