//! SphericalSurface3D Bounds Implementation
//!
//! `Bounded` trait への適合を提供する。
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

use crate::SphericalSurface3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for SphericalSurface3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}
