//! SphericalSolid3D の Bounds 実装
//!
//! `Bounded` trait への適合を提供する。

use crate::SphericalSolid3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for SphericalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_spherical_solid_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let solid = SphericalSolid3D::new_standard(center, 2.0).unwrap();

        assert_eq!(solid.primitive_kind(), PrimitiveKind::SphericalSolid);

        let bbox = solid.aabb().expect("should have aabb");
        assert_eq!(bbox.min(), Point3D::new(-1.0, 0.0, 1.0));
        assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 5.0));

        let volume = solid.volume();
        let expected_volume = 4.0 * std::f64::consts::PI * 8.0 / 3.0;
        assert!((volume - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_spherical_solid_degenerate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let solid = SphericalSolid3D::new_standard(center, f64::EPSILON / 2.0).unwrap();

        assert!(solid.is_degenerate());

        let bbox = solid.aabb().expect("should have aabb");
        assert!(!bbox.is_empty());
    }
}
