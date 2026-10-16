//! `NurbsCurve3D` の境界ボックス

use crate::NurbsCurve3D;
use geo_contracts::{Bounded, NurbsCurve3DProperties, Scalar};
use geo_core::{Aabb3D, Point3D};

impl<T: Scalar> Bounded<T> for NurbsCurve3D<T> {
    type Aabb = Aabb3D<T>;

    /// 曲線は制御点の凸包に含まれるため、制御点を包む境界ボックスを返す
    fn aabb(&self) -> Option<Self::Aabb> {
        let points: Vec<Point3D<T>> = (0..self.control_point_count())
            .filter_map(|i| self.control_point(i))
            .collect();
        Aabb3D::from_points(&points)
    }
}
