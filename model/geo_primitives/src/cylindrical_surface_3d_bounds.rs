//! CylindricalSurface3D の Foundation Pattern 実装
//!
//! ExtensionFoundation と TolerantEq トレイトの実装
//! ハイブリッドモデラーの分類システムとの統合

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
    fn test_extension_foundation() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();

        assert_eq!(surface.primitive_kind(), PrimitiveKind::CylindricalSurface);

        let bbox = surface.aabb().expect("should have aabb");
        assert!(!bbox.is_empty());
    }
}
