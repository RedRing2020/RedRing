//! TriangleMesh3D の Bounds 実装

use crate::TriangleMesh3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

/// 頂点を包む境界ボックス（頂点がない場合は `None`）
impl<T: Scalar> Bounded<T> for TriangleMesh3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Aabb3D::from_points(&self.vertices)
    }
}
