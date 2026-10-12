//! CylindricalSolid3D の Bounds 実装

use crate::CylindricalSolid3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for CylindricalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}

/*
/// 旧Cylinder3D向けのFoundation実装は既存のcylinder_3d_foundation.rsで提供されるため
/// 重複を避けるためここでは実装しない
*/
