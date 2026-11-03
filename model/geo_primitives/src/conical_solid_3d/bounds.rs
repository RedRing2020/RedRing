//! ConicalSolid3D の境界ボックス

use crate::{ConicalSolid3D, Vector3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for ConicalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 軸に直交する半径 r の円の、座標軸方向への広がりは r √(1 - (軸の成分)²)
        let extent = |axis_component: T| {
            self.radius
                * (T::ONE - axis_component * axis_component)
                    .max(T::ZERO)
                    .sqrt()
        };
        let extent = Vector3D::new(
            extent(self.axis.x()),
            extent(self.axis.y()),
            extent(self.axis.z()),
        );
        let far_end = self.center + self.axis.as_vector() * self.height;
        // far_end は頂点
        Aabb3D::from_points(&[self.center - extent, self.center + extent, far_end])
    }
}
