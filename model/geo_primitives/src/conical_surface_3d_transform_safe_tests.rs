//! ConicalSurface3D の安全な変換操作（SafeTransform）のテストスイート

#[cfg(test)]
mod tests {
    use crate::conical_surface_3d_transform_safe::ConicalSurfaceSafeTransform;
    use crate::{ConicalSurface3D, Point3D, Vector3D};
    use geo_foundation::TransformError;

    /// 標準的な円錐サーフェスを作成するヘルパー関数
    fn create_test_cone() -> ConicalSurface3D<f64> {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 2.0;
        let semi_angle = std::f64::consts::PI / 6.0; // 30度

        ConicalSurface3D::new(center, axis, ref_direction, radius, semi_angle)
            .expect("有効な円錐サーフェスを作成できませんでした")
    }

    #[test]
    fn test_translate_safe_valid() {
        let cone = create_test_cone();
        let translation = Vector3D::new(5.0, -3.0, 2.0);

        let result = cone.translate_safe(translation);
        assert!(result.is_ok());

        let translated = result.unwrap();
        let expected_center = Point3D::new(6.0, -1.0, 5.0);

        assert!((translated.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((translated.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((translated.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 他のパラメータは変更されない
        assert!((translated.radius() - cone.radius()).abs() < f64::EPSILON);
        assert!((translated.semi_angle() - cone.semi_angle()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scale_uniform_safe_valid() {
        let cone = create_test_cone();
        let factor = 2.5;

        let result = cone.scale_uniform_safe(factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();

        // スケール後の半径を確認
        let expected_radius = cone.radius() * factor;
        assert!((scaled.radius() - expected_radius).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scale_uniform_safe_zero_factor() {
        let cone = create_test_cone();
        let factor = 0.0;

        let result = cone.scale_uniform_safe(factor);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(_)) = result {
            // 期待通り
        } else {
            panic!("期待されたエラー型と一致しません");
        }
    }

    #[test]
    fn test_with_radius_safe_valid() {
        let cone = create_test_cone();
        let new_radius = 5.0;

        let result = cone.with_radius_safe(new_radius);
        assert!(result.is_ok());

        let modified = result.unwrap();
        assert!((modified.radius() - new_radius).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_radius_safe_negative_radius() {
        let cone = create_test_cone();
        let new_radius = -2.0;

        let result = cone.with_radius_safe(new_radius);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(_)) = result {
            // 期待通り
        } else {
            panic!("期待されたエラー型と一致しません");
        }
    }
}
