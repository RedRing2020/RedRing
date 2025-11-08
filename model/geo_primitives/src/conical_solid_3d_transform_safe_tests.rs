//! ConicalSolid3D の安全な変換操作のテストスイート
//!
//! エラーハンドリング版の変換操作の包括的テスト

#[cfg(test)]
mod tests {
    use crate::{ConicalSolid3D, Point3D, Vector3D};
    use geo_foundation::{SafeTransform, TransformError};

    fn create_test_cone() -> ConicalSolid3D<f64> {
        ConicalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            2.0,
        )
        .unwrap()
    }

    #[test]
    fn test_safe_translate_success() {
        let cone = create_test_cone();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let result = cone.translate_safe(&translation);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.center(), Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(transformed.radius(), 1.0);
        assert_eq!(transformed.height(), 2.0);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let cone = create_test_cone();
        let invalid_translation = Vector3D::new(f64::NAN, 0.0, 0.0);

        let result = cone.translate_safe(&invalid_translation);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }

    #[test]
    fn test_safe_rotate_z_success() {
        let cone = create_test_cone();
        let angle = std::f64::consts::PI / 2.0; // 90度

        let result = cone.rotate_z_safe(angle);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        // 90度回転後、参照方向 (1,0,0) は (0,1,0) になる
        let expected_ref = Vector3D::new(0.0, 1.0, 0.0);
        let actual_ref = transformed.ref_direction().as_vector();

        assert!((actual_ref.x() - expected_ref.x()).abs() < 1e-10);
        assert!((actual_ref.y() - expected_ref.y()).abs() < 1e-10);
        assert!((actual_ref.z() - expected_ref.z()).abs() < 1e-10);
    }

    #[test]
    fn test_safe_rotate_z_invalid_angle() {
        let cone = create_test_cone();
        let invalid_angle = f64::NAN;

        let result = cone.rotate_z_safe(invalid_angle);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidRotation(_)
        ));
    }

    #[test]
    fn test_safe_rotate_axis_success() {
        let cone = create_test_cone();
        let axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸
        let angle = std::f64::consts::PI / 2.0; // 90度

        let result = cone.rotate_axis_safe(&axis, angle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_rotate_axis_zero_axis_error() {
        let cone = create_test_cone();
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = std::f64::consts::PI / 2.0;

        let result = cone.rotate_axis_safe(&zero_axis, angle);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }

    #[test]
    fn test_safe_scale_uniform_success() {
        let cone = create_test_cone();
        let factor = 2.0;

        let result = cone.scale_uniform_safe(factor);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.radius(), 2.0);
        assert_eq!(transformed.height(), 4.0);
        assert_eq!(transformed.center(), cone.center()); // 中心は変化しない
    }

    #[test]
    fn test_safe_scale_uniform_zero_factor_error() {
        let cone = create_test_cone();
        let zero_factor = 0.0;

        let result = cone.scale_uniform_safe(zero_factor);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_safe_scale_uniform_negative_factor_error() {
        let cone = create_test_cone();
        let negative_factor = -1.0;

        let result = cone.scale_uniform_safe(negative_factor);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_safe_reflect_success() {
        let cone = create_test_cone();
        let normal = Vector3D::new(1.0, 0.0, 0.0); // X軸法線

        let result = cone.reflect_safe(&normal);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        // X軸での反射により、参照方向 (1,0,0) は (-1,0,0) になる
        let expected_ref = Vector3D::new(-1.0, 0.0, 0.0);
        let actual_ref = transformed.ref_direction().as_vector();

        assert!((actual_ref.x() - expected_ref.x()).abs() < 1e-10);
        assert!((actual_ref.y() - expected_ref.y()).abs() < 1e-10);
        assert!((actual_ref.z() - expected_ref.z()).abs() < 1e-10);
    }

    #[test]
    fn test_safe_reflect_zero_normal_error() {
        let cone = create_test_cone();
        let zero_normal = Vector3D::new(0.0, 0.0, 0.0);

        let result = cone.reflect_safe(&zero_normal);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }

    #[test]
    fn test_safe_transform_trait() {
        let cone = create_test_cone();
        let offset = 1.0; // Z軸方向の移動

        let result = cone.safe_translate(offset);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.center(), Point3D::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_rodrigues_rotation_x_axis() {
        let cone = create_test_cone();
        let x_axis = Vector3D::new(1.0, 0.0, 0.0);
        let angle = std::f64::consts::PI / 2.0; // 90度

        let result = cone.rotate_axis_safe(&x_axis, angle);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        // X軸周りの90度回転後、Z軸(0,0,1)は(0,1,0)になる（右手系）
        let rotated_axis = transformed.axis().as_vector();
        let expected_axis = Vector3D::new(0.0, 1.0, 0.0);

        assert!(
            (rotated_axis.x() - expected_axis.x()).abs() < 1e-10,
            "Expected x: {}, got: {}",
            expected_axis.x(),
            rotated_axis.x()
        );
        assert!(
            (rotated_axis.y() - expected_axis.y()).abs() < 1e-10,
            "Expected y: {}, got: {}",
            expected_axis.y(),
            rotated_axis.y()
        );
        assert!(
            (rotated_axis.z() - expected_axis.z()).abs() < 1e-10,
            "Expected z: {}, got: {}",
            expected_axis.z(),
            rotated_axis.z()
        );
    }

    #[test]
    fn test_error_propagation() {
        let cone = create_test_cone();

        // 連続した変換でのエラーハンドリング
        let translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);
        let translate_result = cone.translate_safe(&translation);
        assert!(translate_result.is_err());

        // 無効なスケール
        let scale_result = cone.scale_uniform_safe(-1.0);
        assert!(scale_result.is_err());

        // 無効な回転軸
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let rotate_result = cone.rotate_axis_safe(&zero_axis, 1.0);
        assert!(rotate_result.is_err());
    }

    #[test]
    fn test_boundary_values() {
        let cone = create_test_cone();

        // 極小スケール
        let tiny_scale = f64::EPSILON;
        let result = cone.scale_uniform_safe(tiny_scale);
        assert!(result.is_ok());

        // 極大角度
        let large_angle = 1000.0 * std::f64::consts::PI;
        let result = cone.rotate_z_safe(large_angle);
        assert!(result.is_ok());

        // 極小ベクトル
        let tiny_vector = Vector3D::new(f64::EPSILON, f64::EPSILON, f64::EPSILON);
        let result = cone.translate_safe(&tiny_vector);
        assert!(result.is_ok());
    }
}
