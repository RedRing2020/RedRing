//! CylindricalSurface3D のテストスイート
//!
//! Core機能、パラメータ化、曲面解析、STEP準拠性の包括的テスト

#[cfg(test)]
mod tests {
    use crate::{CylindricalSurface3D, Point3D, Vector3D};
    use approx::assert_relative_eq;
    use geo_foundation::Scalar;

    fn create_test_surface() -> CylindricalSurface3D<f64> {
        CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap()
    }

    // ========================================================================
    // Core Creation Tests
    // ========================================================================

    #[test]
    fn test_cylindrical_surface_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;

        let surface = CylindricalSurface3D::new(center, axis, ref_direction, radius);
        assert!(surface.is_some());

        let surface = surface.unwrap();
        assert_eq!(surface.center(), center);
        assert_eq!(surface.radius(), radius);
    }

    #[test]
    fn test_cylindrical_surface_invalid_creation() {
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);

        // 負の半径
        assert!(CylindricalSurface3D::new(center, axis, ref_direction, -1.0).is_none());
        // ゼロ半径
        assert!(CylindricalSurface3D::new(center, axis, ref_direction, 0.0).is_none());
        // ゼロ軸ベクトル
        assert!(CylindricalSurface3D::new(center, Vector3D::zero(), ref_direction, 5.0).is_none());
        // ゼロ参照方向
        assert!(CylindricalSurface3D::new(center, axis, Vector3D::zero(), 5.0).is_none());
    }

    #[test]
    fn test_cylindrical_surface_axis_constructors() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let radius = 5.0;

        // Z軸円柱
        let z_surface = CylindricalSurface3D::new_z_axis(center, radius).unwrap();
        assert_eq!(z_surface.axis().as_vector(), Vector3D::new(0.0, 0.0, 1.0));

        // Y軸円柱
        let y_surface = CylindricalSurface3D::new_y_axis(center, radius).unwrap();
        assert_eq!(y_surface.axis().as_vector(), Vector3D::new(0.0, 1.0, 0.0));

        // X軸円柱
        let x_surface = CylindricalSurface3D::new_x_axis(center, radius).unwrap();
        assert_eq!(x_surface.axis().as_vector(), Vector3D::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_cylindrical_surface_f32() {
        let surface: CylindricalSurface3D<f32> =
            CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();

        assert_relative_eq!(surface.radius(), 5.0f32, epsilon = 1e-6);
        assert_eq!(surface.center(), Point3D::new(1.0f32, 2.0f32, 3.0f32));
    }

    #[test]
    fn test_ref_direction_orthogonalization() {
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let non_orthogonal_ref = Vector3D::new(1.0, 0.0, 0.5); // 軸と非直交

        let surface = CylindricalSurface3D::new(center, axis, non_orthogonal_ref, 5.0).unwrap();

        // 参照方向が軸と直交しているかチェック
        let dot_product = surface
            .ref_direction()
            .as_vector()
            .dot(&surface.axis().as_vector());
        assert_relative_eq!(dot_product, 0.0, epsilon = 1e-10);
    }

    // ========================================================================
    // Parametric Surface Tests
    // ========================================================================

    #[test]
    fn test_point_at_uv() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        // U=0 (X軸方向)、V=0 (基準面)
        let point = surface.point_at_uv(0.0, 0.0);
        assert_relative_eq!(point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 0.0, epsilon = 1e-10);

        // U=π/2 (Y軸方向)、V=0
        let point = surface.point_at_uv(std::f64::consts::PI / 2.0, 0.0);
        assert_relative_eq!(point.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 0.0, epsilon = 1e-10);

        // U=0、V=10 (軸方向移動)
        let point = surface.point_at_uv(0.0, 10.0);
        assert_relative_eq!(point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_normal_at_uv() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        // 法線は径方向を向く
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

        // U方向接線（円周方向）
        let tangent_u = surface.tangent_u_at_uv(0.0, 0.0);
        assert_relative_eq!(tangent_u.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_u.y(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_u.z(), 0.0, epsilon = 1e-10);

        // V方向接線（軸方向）
        let tangent_v = surface.tangent_v_at_uv(0.0, 0.0);
        assert_relative_eq!(tangent_v.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_v.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(tangent_v.z(), 1.0, epsilon = 1e-10);
    }

    // ========================================================================
    // Surface Analysis Tests
    // ========================================================================

    #[test]
    fn test_curvature_analysis() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        // 主曲率
        let (k1, k2) = surface.curvature_at_uv(0.0, 0.0);
        assert_relative_eq!(k1, 1.0 / 5.0, epsilon = 1e-10); // 円周方向曲率
        assert_relative_eq!(k2, 0.0, epsilon = 1e-10); // 軸方向曲率

        // 平均曲率
        let mean_curvature = surface.mean_curvature_at_uv(0.0, 0.0);
        assert_relative_eq!(mean_curvature, 1.0 / 10.0, epsilon = 1e-10);

        // ガウス曲率
        let gaussian_curvature = surface.gaussian_curvature_at_uv(0.0, 0.0);
        assert_relative_eq!(gaussian_curvature, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_distance_to_surface() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        // サーフェス上の点（距離0）
        let surface_point = Point3D::new(5.0, 0.0, 0.0);
        let distance = surface.distance_to_surface(surface_point);
        assert_relative_eq!(distance, 0.0, epsilon = 1e-10);

        // 内部の点
        let internal_point = Point3D::new(3.0, 0.0, 0.0);
        let distance = surface.distance_to_surface(internal_point);
        assert_relative_eq!(distance, 2.0, epsilon = 1e-10);

        // 外部の点
        let external_point = Point3D::new(8.0, 0.0, 0.0);
        let distance = surface.distance_to_surface(external_point);
        assert_relative_eq!(distance, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_closest_point_on_surface() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        let query_point = Point3D::new(8.0, 0.0, 10.0);
        let (closest_point, u, v) = surface.closest_point_on_surface(query_point);

        // 最近点はサーフェス上にある
        assert_relative_eq!(closest_point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(closest_point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(closest_point.z(), 10.0, epsilon = 1e-10);

        // UVパラメータの検証
        assert_relative_eq!(u, 0.0, epsilon = 1e-10);
        assert_relative_eq!(v, 10.0, epsilon = 1e-10);

        // パラメータから点を再計算して一致するか確認
        let reconstructed = surface.point_at_uv(u, v);
        assert_relative_eq!(reconstructed.x(), closest_point.x(), epsilon = 1e-10);
        assert_relative_eq!(reconstructed.y(), closest_point.y(), epsilon = 1e-10);
        assert_relative_eq!(reconstructed.z(), closest_point.z(), epsilon = 1e-10);
    }

    // ========================================================================
    // STEP Compliance Tests
    // ========================================================================

    #[test]
    fn test_step_compliance() {
        // STEP CYLINDRICAL_SURFACE準拠テスト
        let center = Point3D::new(10.0, 20.0, 30.0);
        let axis = Vector3D::new(0.0, 1.0, 0.0); // Y軸
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0); // X軸
        let radius = 15.0;

        let surface = CylindricalSurface3D::new(center, axis, ref_direction, radius).unwrap();

        // STEP座標系の検証
        assert_eq!(surface.center(), center);
        assert_eq!(surface.axis().as_vector(), axis);
        assert_eq!(surface.ref_direction().as_vector(), ref_direction);
        assert_eq!(surface.radius(), radius);

        // Y軸（派生軸）の検証
        let y_axis = surface.y_axis();
        let expected_y = axis.cross(&ref_direction);
        assert_relative_eq!(y_axis.as_vector().x(), expected_y.x(), epsilon = 1e-10);
        assert_relative_eq!(y_axis.as_vector().y(), expected_y.y(), epsilon = 1e-10);
        assert_relative_eq!(y_axis.as_vector().z(), expected_y.z(), epsilon = 1e-10);
    }

    // ========================================================================
    // Surface Properties Tests
    // ========================================================================

    #[test]
    fn test_bounding_box_radial() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();
        let bbox = surface.bounding_box_radial();

        // 径方向の境界ボックス
        assert_relative_eq!(bbox.min().x(), -5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().y(), -5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().y(), 5.0, epsilon = 1e-10);

        // Z方向は軸に平行なので影響なし
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

    // ========================================================================
    // Edge Cases and Error Handling
    // ========================================================================

    #[test]
    fn test_point_on_axis_projection() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        // 軸上の点の投影
        let axis_point = Point3D::new(0.0, 0.0, 10.0);
        let (closest_point, u, v) = surface.closest_point_on_surface(axis_point);

        // V座標は正しい
        assert_relative_eq!(v, 10.0, epsilon = 1e-10);

        // 軸上の点なので任意のU座標（通常0）
        assert_relative_eq!(u, 0.0, epsilon = 1e-10);

        // 最近点はサーフェス上
        let distance_from_axis =
            (closest_point.x() * closest_point.x() + closest_point.y() * closest_point.y()).sqrt();
        assert_relative_eq!(distance_from_axis, 5.0, epsilon = 1e-10);
    }

    #[test]
    fn test_parametric_continuity() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap();

        // U=0とU=2πは同じ点を示す（周期性）
        let point_0 = surface.point_at_uv(0.0, 5.0);
        let point_2pi = surface.point_at_uv(2.0 * std::f64::consts::PI, 5.0);

        assert_relative_eq!(point_0.x(), point_2pi.x(), epsilon = 1e-10);
        assert_relative_eq!(point_0.y(), point_2pi.y(), epsilon = 1e-10);
        assert_relative_eq!(point_0.z(), point_2pi.z(), epsilon = 1e-10);
    }
}
