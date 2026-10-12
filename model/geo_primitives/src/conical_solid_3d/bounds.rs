//! ConicalSolid3D の Bounds 実装
//!
//! PrimitiveMetadata と Bounded に関わる補助実装を保持する。

use crate::ConicalSolid3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for ConicalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}
