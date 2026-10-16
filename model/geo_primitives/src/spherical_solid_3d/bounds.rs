//! SphericalSolid3D の境界ボックス

use crate::{SphericalSolid3D, Vector3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for SphericalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let extent = Vector3D::new(self.radius, self.radius, self.radius);
        Some(Aabb3D::new(self.center - extent, self.center + extent))
    }
}
