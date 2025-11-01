//! SphericalSolid3D Foundation Implementation
//!
//! ExtensionFoundation トレイトによる統一インターフェースの実装
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

use crate::{BBox3D, SphericalSolid3D};
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for SphericalSolid3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::SphericalSolid
    }

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
    }

    fn measure(&self) -> Option<T> {
        Some(self.volume())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_spherical_solid_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let solid = SphericalSolid3D::new_standard(center, 2.0).unwrap();

        // Foundation トレイトのテスト
        assert_eq!(solid.primitive_kind(), PrimitiveKind::SphericalSolid);

        let bbox = solid.bounding_box();
        assert_eq!(bbox.min(), Point3D::new(-1.0, 0.0, 1.0));
        assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 5.0));

        let volume = solid.measure().unwrap();
        let expected_volume = 4.0 * std::f64::consts::PI * 8.0 / 3.0; // 4/3 * π * r³
        assert!((volume - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_spherical_solid_degenerate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let solid = SphericalSolid3D::new_standard(center, f64::EPSILON / 2.0).unwrap();

        assert!(solid.is_degenerate());

        let bbox = solid.bounding_box();
        assert!(!bbox.is_empty());
    }
}
