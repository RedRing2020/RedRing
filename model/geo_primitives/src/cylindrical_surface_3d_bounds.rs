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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_metadata_and_bounds_capabilities() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();

        assert_eq!(surface.primitive_kind(), PrimitiveKind::CylindricalSurface);

        let bbox = surface.aabb().expect("should have aabb");
        assert!(!bbox.is_empty());
    }
}
