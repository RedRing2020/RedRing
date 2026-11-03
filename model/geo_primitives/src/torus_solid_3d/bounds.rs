//! TorusSolid3D の境界ボックス

use crate::{TorusSolid3D, Vector3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for TorusSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 座標軸方向への広がりは、軸に直交する面内の外径の円の広がりと、軸方向の小半径の広がりの和
        let outer_radius = self.major_radius + self.minor_radius;
        let extent = |axis_component: T| {
            outer_radius
                * (T::ONE - axis_component * axis_component)
                    .max(T::ZERO)
                    .sqrt()
                + self.minor_radius * axis_component.abs()
        };
        let extent = Vector3D::new(
            extent(self.z_axis.x()),
            extent(self.z_axis.y()),
            extent(self.z_axis.z()),
        );
        Some(Aabb3D::new(self.origin - extent, self.origin + extent))
    }
}
