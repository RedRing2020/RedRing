//! SphericalSolid3D のテストスイート
//! 作成日: 2025年1月27日

use crate::{Point3D, SphericalSolid3D, Vector3D};
use approx::assert_relative_eq;
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_solid_3d_basic_creation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 5.0;

        let sphere = SphericalSolid3D::new(center, axis, ref_direction, radius).unwrap();

        assert_eq!(sphere.center(), center);
        assert_eq!(sphere.radius(), radius);
    }

    #[test]
    fn test_spherical_solid_3d_volume() {
        let center = Point3D::origin();
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 3.0;

        let sphere = SphericalSolid3D::new(center, axis, ref_direction, radius).unwrap();
        let expected_volume = (4.0 / 3.0) * std::f64::consts::PI * radius.powi(3);

        assert_relative_eq!(sphere.volume(), expected_volume, epsilon = 1e-10);
    }

    #[test]
    fn test_spherical_solid_3d_surface_area() {
        let center = Point3D::origin();
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 2.0;

        let sphere = SphericalSolid3D::new(center, axis, ref_direction, radius).unwrap();
        let expected_area = 4.0 * std::f64::consts::PI * radius.powi(2);

        assert_relative_eq!(sphere.surface_area(), expected_area, epsilon = 1e-10);
    }

    #[test]
    fn test_spherical_solid_3d_foundation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 4.0;

        let sphere = SphericalSolid3D::new(center, axis, ref_direction, radius).unwrap();

        assert_eq!(sphere.primitive_kind(), PrimitiveKind::SphericalSolid);

        let bbox = sphere.bounding_box();
        assert_relative_eq!(bbox.min().x(), -radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().x(), radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().y(), -radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().y(), radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().z(), -radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().z(), radius, epsilon = 1e-10);

        assert_eq!(sphere.measure(), Some(sphere.volume()));
    }

    #[test]
    fn test_spherical_solid_3d_contains_point() {
        let center = Point3D::new(1.0, 1.0, 1.0);
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 2.0;

        let sphere = SphericalSolid3D::new(center, axis, ref_direction, radius).unwrap();

        // 中心点は含まれる
        assert!(sphere.contains_point(center));

        // 球面上の点は含まれる
        let surface_point = Point3D::new(1.0 + radius, 1.0, 1.0);
        assert!(sphere.contains_point(surface_point));

        // 内部の点は含まれる
        let inner_point = Point3D::new(1.5, 1.5, 1.5);
        assert!(sphere.contains_point(inner_point));

        // 外部の点は含まれない
        let outer_point = Point3D::new(1.0 + radius + 1.0, 1.0, 1.0);
        assert!(!sphere.contains_point(outer_point));
    }

    #[test]
    fn test_spherical_solid_3d_step_compliance() {
        let center = Point3D::new(2.0, 3.0, 4.0);
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 1.5;

        let sphere = SphericalSolid3D::new(center, axis, ref_direction, radius).unwrap();

        // STEP AP214 準拠の確認
        assert_eq!(sphere.center(), center);
        assert_eq!(sphere.radius(), radius);

        // 座標系の確認
        let position = sphere.position();
        assert_eq!(position.origin(), center);
    }
}
