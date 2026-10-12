//! SphericalSolid3D の Bounds 実装
//!
//! `Bounded` trait への適合を提供する。

use crate::SphericalSolid3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for SphericalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}
