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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_spherical_surface_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let surface = SphericalSurface3D::new_standard(center, 2.0).unwrap();

        // Foundation トレイトのテスト
        assert_eq!(surface.primitive_kind(), PrimitiveKind::SphericalSurface);

        let bbox = surface.aabb().expect("should have aabb");
        assert_eq!(bbox.min(), Point3D::new(-1.0, 0.0, 1.0));
        assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 5.0));

        let area = surface.surface_area();
        let expected_area = 4.0 * std::f64::consts::PI * 4.0; // 4π * r²
        assert!((area - expected_area).abs() < 1e-10);
    }
}
