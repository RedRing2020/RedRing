//! SphericalSurface3D のテストスイート
//! 作成日: 2025年1月27日

use crate::{Point3D, SphericalSurface3D, Vector3D};
use approx::assert_relative_eq;
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_surface_3d_basic_creation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 5.0;

        let surface = SphericalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        assert_eq!(surface.center(), center);
        assert_eq!(surface.radius(), radius);
    }

    #[test]
    fn test_spherical_surface_3d_parametric_point() {
        let center = Point3D::origin();
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 2.0;

        let surface = SphericalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        // 極の点 (u=0, v=0)
        let point = surface.point_at(0.0, 0.0);
        assert_relative_eq!(point.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), radius, epsilon = 1e-10);

        // 赤道上の点 (u=π/2, v=0)
        let point = surface.point_at(std::f64::consts::PI / 2.0, 0.0);
        assert_relative_eq!(point.x(), radius, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_spherical_surface_3d_normal() {
        let center = Point3D::origin();
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 3.0;

        let surface = SphericalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        // 極での法線
        let normal = surface.normal_at(0.0, 0.0);
        assert_relative_eq!(normal.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), 1.0, epsilon = 1e-10);

        // 赤道での法線
        let normal = surface.normal_at(std::f64::consts::PI / 2.0, 0.0);
        assert_relative_eq!(normal.x(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_spherical_surface_3d_curvature() {
        let center = Point3D::origin();
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 4.0;

        let surface = SphericalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        let expected_curvature = 1.0 / radius;

        // 極での曲率
        let (k1, k2) = surface.principal_curvatures_at(0.0, 0.0);
        assert_relative_eq!(k1, expected_curvature, epsilon = 1e-10);
        assert_relative_eq!(k2, expected_curvature, epsilon = 1e-10);

        // 赤道での曲率
        let (k1, k2) = surface.principal_curvatures_at(std::f64::consts::PI / 2.0, 0.0);
        assert_relative_eq!(k1, expected_curvature, epsilon = 1e-10);
        assert_relative_eq!(k2, expected_curvature, epsilon = 1e-10);
    }

    #[test]
    fn test_spherical_surface_3d_foundation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 5.0;

        let surface = SphericalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        assert_eq!(surface.primitive_kind(), PrimitiveKind::SphericalSurface);

        let bbox = surface.bounding_box();
        assert_relative_eq!(bbox.min().x(), -radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().x(), radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().y(), -radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().y(), radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().z(), -radius, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().z(), radius, epsilon = 1e-10);

        let surface_area = 4.0 * std::f64::consts::PI * radius.powi(2);
        assert_eq!(surface.measure(), Some(surface_area));
    }

    #[test]
    fn test_spherical_surface_3d_area() {
        let center = Point3D::origin();
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 2.5;

        let surface = SphericalSurface3D::new(center, axis, ref_direction, radius).unwrap();
        let expected_area = 4.0 * std::f64::consts::PI * radius.powi(2);

        assert_relative_eq!(surface.area(), expected_area, epsilon = 1e-10);
    }

    #[test]
    fn test_spherical_surface_3d_step_compliance() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::unit_z();
        let ref_direction = Vector3D::unit_x();
        let radius = 1.8;

        let surface = SphericalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        // STEP AP214 準拠の確認
        assert_eq!(surface.center(), center);
        assert_eq!(surface.radius(), radius);

        // 座標系の確認
        let position = surface.position();
        assert_eq!(position.origin(), center);
    }
}
