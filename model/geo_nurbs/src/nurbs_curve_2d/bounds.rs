//! `NurbsCurve2D` の境界ボックス

use crate::{NurbsCurve2D, Scalar};
use geo_contracts::{Bounded, NurbsCurve2DProperties};
use geo_core::{Aabb2D, Point2D};

impl<T: Scalar> Bounded<T> for NurbsCurve2D<T> {
    type Aabb = Aabb2D<T>;

    /// 曲線は制御点の凸包に含まれるため、制御点を包む境界ボックスを返す
    fn aabb(&self) -> Option<Self::Aabb> {
        let points: Vec<Point2D<T>> = (0..self.control_point_count())
            .filter_map(|i| self.control_point(i))
            .collect();
        Aabb2D::from_points(&points)
    }
}
