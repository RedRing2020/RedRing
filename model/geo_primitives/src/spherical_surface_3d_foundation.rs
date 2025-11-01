//! SphericalSurface3D Foundation Implementation
//!
//! ExtensionFoundation トレイトによる統一インターフェースの実装
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

use crate::{BBox3D, SphericalSurface3D};
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for SphericalSurface3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::SphericalSurface
    }

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
    }

    fn measure(&self) -> Option<T> {
        Some(self.surface_area())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_spherical_surface_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let surface = SphericalSurface3D::new_standard(center, 2.0).unwrap();

        // Foundation トレイトのテスト
        assert_eq!(surface.primitive_kind(), PrimitiveKind::SphericalSurface);

        let bbox = surface.bounding_box();
        assert_eq!(bbox.min(), Point3D::new(-1.0, 0.0, 1.0));
        assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 5.0));

        let area = surface.measure().unwrap();
        let expected_area = 4.0 * std::f64::consts::PI * 4.0; // 4π * r²
        assert!((area - expected_area).abs() < 1e-10);
    }

    #[test]
    fn test_spherical_surface_degenerate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let surface = SphericalSurface3D::new_standard(center, f64::EPSILON / 2.0).unwrap();

        assert!(surface.is_degenerate());

        let bbox = surface.bounding_box();
        assert!(!bbox.is_empty());
    }

    #[test]
    fn test_spherical_surface_normal() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let surface = SphericalSurface3D::new_standard(center, 1.0).unwrap();

        // 表面上の点での法線テスト
        let point = Point3D::new(1.0, 0.0, 0.0);
        let normal = surface.normal_at_point(point).unwrap();

        // 法線は外向きで正規化されている
        assert!((normal.x() - 1.0).abs() < 1e-10);
        assert!((normal.y() - 0.0).abs() < 1e-10);
        assert!((normal.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_spherical_surface_curvature() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let surface = SphericalSurface3D::new_standard(center, 2.0).unwrap();

        let (k1, k2) = surface.principal_curvatures();
        assert!((k1 - 0.5).abs() < 1e-10);
        assert!((k2 - 0.5).abs() < 1e-10);

        let mean_curvature = surface.mean_curvature();
        assert!((mean_curvature - 0.5).abs() < 1e-10);

        let gaussian_curvature = surface.gaussian_curvature();
        assert!((gaussian_curvature - 0.25).abs() < 1e-10);
    }
}
