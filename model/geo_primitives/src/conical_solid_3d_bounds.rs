//! ConicalSolid3D Foundation Implementation
//!
//! ExtensionFoundation トレイトによる統一インターフェースの実装

use crate::ConicalSolid3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for ConicalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};
    use approx::assert_relative_eq;
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_extension_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let conical_solid = ConicalSolid3D::new(center, axis, ref_direction, 5.0, 10.0).unwrap();

        // primitive_kind のテスト
        assert_eq!(conical_solid.primitive_kind(), PrimitiveKind::ConicalSolid);

        let volume = conical_solid.volume_internal();
        let expected_volume = std::f64::consts::PI * 25.0 * 10.0 / 3.0; // π * r² * h / 3
        assert_relative_eq!(volume, expected_volume, epsilon = 1e-10);

        // bounding_box のテスト
        let bbox = conical_solid.aabb().expect("should have aabb");
        // 底面: center ± radius in x,y directions
        // 頂点: center + axis * height
        assert_relative_eq!(bbox.min().x(), -4.0, epsilon = 1e-10); // 1 - 5
        assert_relative_eq!(bbox.max().x(), 6.0, epsilon = 1e-10); // 1 + 5
        assert_relative_eq!(bbox.min().y(), -3.0, epsilon = 1e-10); // 2 - 5
        assert_relative_eq!(bbox.max().y(), 7.0, epsilon = 1e-10); // 2 + 5
        assert_relative_eq!(bbox.min().z(), 3.0, epsilon = 1e-10); // min(3, 13) = 3
        assert_relative_eq!(bbox.max().z(), 13.0, epsilon = 1e-10); // max(3, 13) = 13
    }

    #[test]
    fn test_conical_solid_degenerate() {
        // ゼロ半径の円錐
        let zero_radius = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 0.0, 5.0);
        assert!(zero_radius.is_none());

        // ゼロ高さの円錐
        let zero_height = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, 0.0);
        assert!(zero_height.is_none());

        // 負の半径
        let negative_radius = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), -1.0, 5.0);
        assert!(negative_radius.is_none());

        // 負の高さ
        let negative_height = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, -1.0);
        assert!(negative_height.is_none());
    }
}
