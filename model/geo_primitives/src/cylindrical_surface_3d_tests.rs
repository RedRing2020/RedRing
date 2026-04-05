//! CylindricalSurface3D のテストスイート
//!
//! Core機能、パラメータ化、曲面解析、STEP準拠性の包括的テスト
//! Core Traits経由でのAPI使用を推奨

#[cfg(test)]
mod tests {
    use crate::{CylindricalSurface3D, Point3D, Vector3D};
    use analysis::test_constants::TOLERANCE_F32;
    use approx::assert_relative_eq;
    use geo_contracts::{CylindricalSurface3DDistance, CylindricalSurface3DProperties, Scalar};

    fn create_test_surface() -> CylindricalSurface3D<f64> {
        CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap()
    }

    #[test]
    fn test_cylindrical_surface_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;

        let surface = CylindricalSurface3D::new(center, axis, ref_direction, radius);
        assert!(surface.is_some());

        let surface = surface.unwrap();
        let center_tuple = CylindricalSurface3DProperties::center(&surface);
        assert_eq!(center_tuple, (0.0, 0.0, 0.0));
        assert_eq!(CylindricalSurface3DProperties::radius(&surface), radius);
    }

    #[test]
    fn test_cylindrical_surface_invalid_creation() {
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);

        assert!(CylindricalSurface3D::new(center, axis, ref_direction, -1.0).is_none());
        assert!(CylindricalSurface3D::new(center, axis, ref_direction, 0.0).is_none());
        assert!(CylindricalSurface3D::new(center, Vector3D::zero(), ref_direction, 5.0).is_none());
        assert!(CylindricalSurface3D::new(center, axis, Vector3D::zero(), 5.0).is_none());
    }

    #[test]
    fn test_cylindrical_surface_axis_constructors() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let radius = 5.0;

        let z_surface = CylindricalSurface3D::new_z_axis(center, radius).unwrap();
        assert_eq!(z_surface.axis().as_vector(), Vector3D::new(0.0, 0.0, 1.0));

        let y_surface = CylindricalSurface3D::new_y_axis(center, radius).unwrap();
        assert_eq!(y_surface.axis().as_vector(), Vector3D::new(0.0, 1.0, 0.0));

        let x_surface = CylindricalSurface3D::new_x_axis(center, radius).unwrap();
        assert_eq!(x_surface.axis().as_vector(), Vector3D::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_cylindrical_surface_f32() {
        let surface: CylindricalSurface3D<f32> =
            CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();

        assert_relative_eq!(
            CylindricalSurface3DProperties::radius(&surface),
            5.0f32,
            epsilon = TOLERANCE_F32
        );
        let center_tuple = CylindricalSurface3DProperties::center(&surface);
        assert_eq!(center_tuple, (1.0f32, 2.0f32, 3.0f32));
    }

    #[test]
    fn test_ref_direction_orthogonalization() {
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let non_orthogonal_ref = Vector3D::new(1.0, 0.0, 0.5);

        let surface = CylindricalSurface3D::new(center, axis, non_orthogonal_ref, 5.0).unwrap();

        let dot_product = surface
            .ref_direction()
            .as_vector()
            .dot(&surface.axis().as_vector());
        assert_relative_eq!(dot_product, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_point_at_uv() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let point = surface.point_at_uv(0.0, 0.0);
        assert_relative_eq!(point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 0.0, epsilon = 1e-10);

        let point = surface.point_at_uv(std::f64::consts::PI / 2.0, 0.0);
        assert_relative_eq!(point.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 0.0, epsilon = 1e-10);

        let point = surface.point_at_uv(0.0, 10.0);
        assert_relative_eq!(point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_normal_at_uv() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let normal = surface.normal_at_uv(0.0, 0.0);
        assert_relative_eq!(normal.x(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), 0.0, epsilon = 1e-10);

        let normal = surface.normal_at_uv(std::f64::consts::PI / 2.0, 5.0);
        assert_relative_eq!(normal.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_tangent_vectors() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let tangent_u = surface.tangent_u_at_uv(0.0, 0.0);
        assert_relative_eq!(tangent_u.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_u.y(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_u.z(), 0.0, epsilon = 1e-10);

        let tangent_v = surface.tangent_v_at_uv(0.0, 0.0);
        assert_relative_eq!(tangent_v.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_v.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_v.z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_curvature_analysis() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let (k1, k2) = surface.curvature_at_uv(0.0, 0.0);
        assert_relative_eq!(k1, 1.0 / 5.0, epsilon = 1e-10);
        assert_relative_eq!(k2, 0.0, epsilon = 1e-10);

        let mean_curvature = surface.mean_curvature_at_uv(0.0, 0.0);
        assert_relative_eq!(mean_curvature, 1.0 / 10.0, epsilon = 1e-10);

        let gaussian_curvature = surface.gaussian_curvature_at_uv(0.0, 0.0);
        assert_relative_eq!(gaussian_curvature, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_distance_to_surface() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let surface_point = (5.0, 0.0, 0.0);
        let distance = CylindricalSurface3DDistance::distance_to_point(&surface, surface_point);
        assert_relative_eq!(distance, 0.0, epsilon = 1e-10);

        let internal_point = (3.0, 0.0, 0.0);
        let distance = CylindricalSurface3DDistance::distance_to_point(&surface, internal_point);
        assert_relative_eq!(distance, 2.0, epsilon = 1e-10);

        let external_point = (8.0, 0.0, 0.0);
        let distance = CylindricalSurface3DDistance::distance_to_point(&surface, external_point);
        assert_relative_eq!(distance, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_closest_point_on_surface() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let query_point = Point3D::new(8.0, 0.0, 10.0);
        let (closest_point, u, v) = surface.closest_point_on_surface(query_point);

        assert_relative_eq!(closest_point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(closest_point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(closest_point.z(), 10.0, epsilon = 1e-10);

        assert_relative_eq!(u, 0.0, epsilon = 1e-10);
        assert_relative_eq!(v, 10.0, epsilon = 1e-10);

        let reconstructed = surface.point_at_uv(u, v);
        assert_relative_eq!(reconstructed.x(), closest_point.x(), epsilon = 1e-10);
        assert_relative_eq!(reconstructed.y(), closest_point.y(), epsilon = 1e-10);
        assert_relative_eq!(reconstructed.z(), closest_point.z(), epsilon = 1e-10);
    }

    #[test]
    fn test_step_compliance() {
        let center = Point3D::new(10.0, 20.0, 30.0);
        let axis = Vector3D::new(0.0, 1.0, 0.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 15.0;

        let surface = CylindricalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        let center_tuple = CylindricalSurface3DProperties::center(&surface);
        assert_eq!(center_tuple, (10.0, 20.0, 30.0));

        let axis_tuple = CylindricalSurface3DProperties::axis(&surface);
        assert_eq!(axis_tuple, (0.0, 1.0, 0.0));

        let ref_dir_tuple = CylindricalSurface3DProperties::ref_direction(&surface);
        assert_eq!(ref_dir_tuple, (1.0, 0.0, 0.0));

        let radius_value = CylindricalSurface3DProperties::radius(&surface);
        assert_eq!(radius_value, radius);

        let y_axis = surface.y_axis();
        let expected_y = axis.cross(&ref_direction);
        assert_relative_eq!(y_axis.as_vector().x(), expected_y.x(), epsilon = 1e-10);
        assert_relative_eq!(y_axis.as_vector().y(), expected_y.y(), epsilon = 1e-10);
        assert_relative_eq!(y_axis.as_vector().z(), expected_y.z(), epsilon = 1e-10);
    }

    #[test]
    fn test_bounding_box_radial() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();
        let bbox = surface.bounding_box_radial();

        assert_relative_eq!(bbox.min().x(), -5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().y(), -5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().y(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().z(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_surface_display() {
        let surface = create_test_surface();
        let display_string = format!("{}", surface);
        assert!(display_string.contains("CylindricalSurface3D"));
        assert!(display_string.contains("center"));
        assert!(display_string.contains("axis"));
        assert!(display_string.contains("radius"));
    }

    #[test]
    fn test_point_on_axis_projection() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let axis_point = Point3D::new(0.0, 0.0, 10.0);
        let (closest_point, u, v) = surface.closest_point_on_surface(axis_point);

        assert_relative_eq!(v, 10.0, epsilon = 1e-10);
        assert_relative_eq!(u, 0.0, epsilon = 1e-10);

        let distance_from_axis =
            (closest_point.x() * closest_point.x() + closest_point.y() * closest_point.y()).sqrt();
        assert_relative_eq!(distance_from_axis, 5.0, epsilon = 1e-10);
    }

    #[test]
    fn test_parametric_continuity() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let point_0 = surface.point_at_uv(0.0, 5.0);
        let point_2pi = surface.point_at_uv(2.0 * std::f64::consts::PI, 5.0);

        assert_relative_eq!(point_0.x(), point_2pi.x(), epsilon = 1e-10);
        assert_relative_eq!(point_0.y(), point_2pi.y(), epsilon = 1e-10);
        assert_relative_eq!(point_0.z(), point_2pi.z(), epsilon = 1e-10);
    }
}
