//! EllipsoidalSolid3D の境界ボックス

use crate::{EllipsoidalSolid3D, Vector3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for EllipsoidalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 各座標軸方向への広がりは、3つの主軸の成分の二乗和の平方根
        let x_axis = self.ref_direction.as_vector() * self.a_radius;
        let y_axis = self.y_axis() * self.b_radius;
        let z_axis = self.axis.as_vector() * self.c_radius;
        let extent = |x: T, y: T, z: T| (x * x + y * y + z * z).sqrt();
        let extent = Vector3D::new(
            extent(x_axis.x(), y_axis.x(), z_axis.x()),
            extent(x_axis.y(), y_axis.y(), z_axis.y()),
            extent(x_axis.z(), y_axis.z(), z_axis.z()),
        );
        Some(Aabb3D::new(self.center - extent, self.center + extent))
    }
}
