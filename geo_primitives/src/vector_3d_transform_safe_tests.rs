//! Vector3D Safe Transform テスト
//!
//! Result型を使用した安全な変換機能のテスト

#[cfg(test)]
mod vector_3d_safe_transform_tests {
    use crate::{Angle, Point3D, Vector3D};
    use geo_foundation::TransformError;
    use std::f64::consts::PI;

    fn create_test_vector() -> Vector3D<f64> {
        Vector3D::new(1.0, 2.0, 3.0)
    }

    fn create_unit_vector_x() -> Vector3D<f64> {
        Vector3D::new(1.0, 0.0, 0.0)
    }

    fn create_unit_vector_z() -> Vector3D<f64> {
        Vector3D::new(0.0, 0.0, 1.0)
    }

    // ============================================================================
    // 平行移動テスト
    // ============================================================================

    #[test]
    fn test_safe_translate_success() {
        let vector = create_test_vector();
        let translation = Vector3D::new(1.5, -0.5, 2.0);

        let result = vector.safe_translate(translation);
        assert!(result.is_ok());

        let translated = result.unwrap();
        assert!((translated.x() - 2.5).abs() < 1e-10);
        assert!((translated.y() - 1.5).abs() < 1e-10);
        assert!((translated.z() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_translate_infinite_vector() {
        let vector = create_test_vector();
        let translation = Vector3D::new(f64::INFINITY, 1.0, 1.0);

        let result = vector.safe_translate(translation);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    #[test]
    fn test_safe_translate_nan_vector() {
        let vector = create_test_vector();
        let translation = Vector3D::new(1.0, f64::NAN, 1.0);

        let result = vector.safe_translate(translation);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    #[test]
    fn test_safe_translate_invalid_source() {
        let vector = Vector3D::new(f64::NAN, 2.0, 3.0);
        let translation = Vector3D::new(1.0, 1.0, 1.0);

        let result = vector.safe_translate(translation);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    // ============================================================================
    // 原点スケールテスト
    // ============================================================================

    #[test]
    fn test_safe_scale_origin_success() {
        let vector = create_test_vector();
        let factor = 2.0;

        let result = vector.safe_scale_origin(factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();
        assert!((scaled.x() - 2.0).abs() < 1e-10);
        assert!((scaled.y() - 4.0).abs() < 1e-10);
        assert!((scaled.z() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_scale_origin_zero_factor_error() {
        let vector = create_test_vector();

        let result = vector.safe_scale_origin(0.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_safe_scale_origin_infinite_factor_error() {
        let vector = create_test_vector();

        let result = vector.safe_scale_origin(f64::INFINITY);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_safe_scale_origin_negative_factor() {
        let vector = create_test_vector();
        let factor = -1.5;

        let result = vector.safe_scale_origin(factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();
        assert!((scaled.x() - (-1.5)).abs() < 1e-10);
        assert!((scaled.y() - (-3.0)).abs() < 1e-10);
        assert!((scaled.z() - (-4.5)).abs() < 1e-10);
    }

    // ============================================================================
    // 軸周り回転テスト
    // ============================================================================

    #[test]
    fn test_safe_rotate_axis_origin_z_axis() {
        let vector = create_unit_vector_x(); // (1, 0, 0)
        let z_axis = create_unit_vector_z(); // (0, 0, 1)
        let angle = Angle::from_radians(PI / 2.0); // 90度

        let result = vector.safe_rotate_axis_origin(z_axis, angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        // Z軸周りの90度回転で (1,0,0) -> (0,1,0)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_rotate_axis_origin_zero_axis_error() {
        let vector = create_test_vector();
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 4.0);

        let result = vector.safe_rotate_axis_origin(zero_axis, angle);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::ZeroVector(_) => (),
            _ => panic!("Expected ZeroVector error"),
        }
    }

    #[test]
    fn test_safe_rotate_axis_origin_infinite_axis_error() {
        let vector = create_test_vector();
        let infinite_axis = Vector3D::new(f64::INFINITY, 0.0, 1.0);
        let angle = Angle::from_radians(PI / 4.0);

        let result = vector.safe_rotate_axis_origin(infinite_axis, angle);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    #[test]
    fn test_safe_rotate_axis_origin_infinite_angle_error() {
        let vector = create_test_vector();
        let z_axis = create_unit_vector_z();
        let angle = Angle::from_radians(f64::INFINITY);

        let result = vector.safe_rotate_axis_origin(z_axis, angle);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidRotation(_) => (),
            _ => panic!("Expected InvalidRotation error"),
        }
    }

    // ============================================================================
    // X/Y/Z軸周り回転テスト
    // ============================================================================

    #[test]
    fn test_safe_rotate_x_origin() {
        let vector = Vector3D::new(0.0, 1.0, 0.0); // Y軸方向
        let angle = Angle::from_radians(PI / 2.0); // 90度

        let result = vector.safe_rotate_x_origin(angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        // X軸周りの90度回転で (0,1,0) -> (0,0,1)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_rotate_y_origin() {
        let vector = Vector3D::new(1.0, 0.0, 0.0); // X軸方向
        let angle = Angle::from_radians(PI / 2.0); // 90度

        let result = vector.safe_rotate_y_origin(angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        // Y軸周りの90度回転で (1,0,0) -> (0,0,-1)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_safe_rotate_z_origin() {
        let vector = create_unit_vector_x(); // (1, 0, 0)
        let angle = Angle::from_radians(PI / 2.0); // 90度

        let result = vector.safe_rotate_z_origin(angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        // Z軸周りの90度回転で (1,0,0) -> (0,1,0)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    // ============================================================================
    // 任意点周り回転テスト
    // ============================================================================

    #[test]
    fn test_safe_rotate_around_point() {
        let vector = create_test_vector();
        let center = Point3D::new(1.0, 1.0, 1.0);
        let axis = create_unit_vector_z();
        let angle = Angle::from_radians(PI / 2.0);

        let result = vector.safe_rotate(center, axis, angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        assert!(rotated.detailed_validation().is_ok());
    }

    // ============================================================================
    // 正規化テスト
    // ============================================================================

    #[test]
    fn test_safe_normalize_success() {
        let vector = Vector3D::new(3.0, 4.0, 0.0); // 長さ5

        let result = vector.safe_normalize();
        assert!(result.is_ok());

        let normalized = result.unwrap();
        assert!((normalized.x() - 0.6_f64).abs() < 1e-10);
        assert!((normalized.y() - 0.8_f64).abs() < 1e-10);
        assert!((normalized.z() - 0.0_f64).abs() < 1e-10);

        // 長さが1に近いことを確認
        let length = normalized.magnitude();
        assert!((length - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_normalize_zero_vector_error() {
        let vector = Vector3D::new(0.0, 0.0, 0.0);

        let result = vector.safe_normalize();
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::ZeroVector(_) => (),
            _ => panic!("Expected ZeroVector error"),
        }
    }

    #[test]
    fn test_safe_normalize_infinite_vector_error() {
        let vector = Vector3D::new(f64::INFINITY, 4.0, 2.0);

        let result = vector.safe_normalize();
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    // ============================================================================
    // 複合変換テスト
    // ============================================================================

    #[test]
    fn test_safe_translate_and_rotate_z_origin() {
        let vector = create_test_vector();
        let translation = Vector3D::new(1.0, 1.0, 0.0);
        let rotation_angle = Angle::from_radians(PI / 4.0);

        let result = vector.safe_translate_and_rotate_z_origin(translation, rotation_angle);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.detailed_validation().is_ok());
    }

    #[test]
    fn test_safe_scale_and_translate_origin() {
        let vector = create_test_vector();
        let scale_factor = 2.0;
        let translation = Vector3D::new(0.5, -1.0, 1.5);

        let result = vector.safe_scale_and_translate_origin(scale_factor, translation);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.detailed_validation().is_ok());
    }

    // ============================================================================
    // 検証テスト
    // ============================================================================

    #[test]
    fn test_detailed_validation() {
        let valid_vector = create_test_vector();
        assert!(valid_vector.detailed_validation().is_ok());

        let invalid_vector = Vector3D::new(f64::NAN, 2.0, 3.0);
        assert!(invalid_vector.detailed_validation().is_err());
    }

    #[test]
    fn test_error_propagation() {
        let vector = create_test_vector();
        let invalid_translation = Vector3D::new(f64::NAN, 1.0, 1.0);

        let result = vector.safe_translate(invalid_translation);
        assert!(result.is_err());

        // エラーがチェーン変換でも適切に伝播することを確認
        let rotation_angle = Angle::from_radians(PI / 4.0);

        let chain_result =
            vector.safe_translate_and_rotate_z_origin(invalid_translation, rotation_angle);
        assert!(chain_result.is_err());
    }

    #[test]
    fn test_transform_chain_with_error_handling() {
        let vector = create_test_vector();

        // 正常なチェーン
        let result = vector
            .safe_translate(Vector3D::new(1.0, 0.0, 0.0))
            .and_then(|v| v.safe_scale_origin(2.0))
            .and_then(|v| v.safe_rotate_z_origin(Angle::from_radians(PI / 6.0)));

        assert!(result.is_ok());
        let final_vector = result.unwrap();
        assert!(final_vector.detailed_validation().is_ok());
    }

    #[test]
    fn test_vector_specific_transforms() {
        let vector = Vector3D::new(2.0, 3.0, 1.0);

        // 原点スケール + 正規化の組み合わせ
        let result = vector
            .safe_scale_origin(3.0)
            .and_then(|v| v.safe_normalize());

        assert!(result.is_ok());
        let final_vector = result.unwrap();

        // 正規化されているので長さが1
        let length = final_vector.magnitude();
        assert!((length - 1.0_f64).abs() < 1e-10);
    }

    #[test]
    fn test_rodrigues_rotation_identity() {
        let vector = create_test_vector();
        let axis = create_unit_vector_z();
        let zero_angle = Angle::from_radians(0.0);

        let result = vector.safe_rotate_axis_origin(axis, zero_angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        // 0度回転は元のベクトルと同じ
        assert!((rotated.x() - vector.x()).abs() < 1e-10);
        assert!((rotated.y() - vector.y()).abs() < 1e-10);
        assert!((rotated.z() - vector.z()).abs() < 1e-10);
    }

    #[test]
    fn test_helper_functions() {
        let valid_vector = create_test_vector();
        // Vector creation succeeded, so it's valid
        assert_eq!(valid_vector.x(), 1.0);

        let valid_axis = Vector3D::new(1.0, 0.0, 0.0);
        assert_eq!(valid_axis.x(), 1.0);
    }
}
