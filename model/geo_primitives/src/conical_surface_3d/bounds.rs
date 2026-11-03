//! ConicalSurface3D の境界ボックス

use crate::ConicalSurface3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for ConicalSurface3D<T> {
    type Aabb = Aabb3D<T>;

    /// 軸方向に無限に続く面のため、有限の境界ボックスを持たない
    fn aabb(&self) -> Option<Self::Aabb> {
        None
    }
}
