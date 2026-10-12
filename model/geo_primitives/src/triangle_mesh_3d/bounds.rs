//! TriangleMesh3D の Bounds 実装

use crate::TriangleMesh3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for TriangleMesh3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        if let Some((min_point, max_point)) = self.bounding_box() {
            Some(Aabb3D::new(min_point, max_point))
        } else {
            // 空のメッシュの場合は None
            None
        }
    }
}
