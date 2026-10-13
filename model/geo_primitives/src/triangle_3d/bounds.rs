//! Triangle3D の Bounds 実装

use crate::Triangle3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for Triangle3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Aabb3D::from_points(&[self.vertex_a, self.vertex_b, self.vertex_c])
    }
}
