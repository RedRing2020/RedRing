//! `NurbsSurface3D` の境界ボックス

use crate::NurbsSurface3D;
use geo_contracts::{Bounded, NurbsSurface3DProperties, Scalar};
use geo_core::{Aabb3D, Point3D};

impl<T: Scalar> Bounded<T> for NurbsSurface3D<T> {
    type Aabb = Aabb3D<T>;

    /// 曲面は制御点の凸包に含まれるため、制御点を包む境界ボックスを返す
    fn aabb(&self) -> Option<Self::Aabb> {
        let points: Vec<Point3D<T>> = (0..self.u_count())
            .flat_map(|u| (0..self.v_count()).filter_map(move |v| self.control_point(u, v)))
            .collect();
        Aabb3D::from_points(&points)
    }
}
