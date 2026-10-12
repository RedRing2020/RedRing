//! CylindricalSurface3D の Bounds 実装
//!
//! PrimitiveMetadata と Bounded に関わる補助実装を保持する。

use crate::CylindricalSurface3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for CylindricalSurface3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // サーフェスは無限軸方向のため、径方向の境界のみ
        // 実際の用途では境界制約が必要
        Some(self.bounding_box_radial())
    }
}
