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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_metadata_and_bounds_capabilities() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, 5.0, 10.0).unwrap();

        assert_eq!(
            cylindrical_solid.primitive_kind(),
            PrimitiveKind::CylindricalSolid
        );
        assert!(cylindrical_solid.volume_internal() > 0.0);

        let bbox = cylindrical_solid.aabb().expect("should have aabb");
        assert_eq!(bbox.min().x(), -4.0);
        assert_eq!(bbox.max().x(), 6.0);
    }
}
