//! Rect3D の Bounds 実装

use crate::Rect3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for Rect3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Aabb3D::from_points(&self.corners())
    }
}
