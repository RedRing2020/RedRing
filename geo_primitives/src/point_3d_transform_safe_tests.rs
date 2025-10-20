//! Point3D Safe Transform テスト
//!
//! Result型を使用した安全な変換機能のテスト

#[cfg(test)]
mod point_3d_safe_transform_tests {
    use crate::{point_3d::Point3D, vector_3d::Vector3D, Angle};
    use geo_foundation::TransformError;
    use std::f64::{INFINITY, NAN};

    const PI: f64 = std::f64::consts::PI;

    fn create_test_point() -> Point3D<f64> {
        Point3D::new(1.0, 2.0, 3.0)
    }

    // ============================================================================
    // 平行移動テスト
    // ============================================================================

    #[test]
    fn test_safe_translate_success() {
        let point = create_test_point();
        let translation = Vector3D::new(1.0, -0.5, 2.0);

        let result = point.safe_translate(translation);
        assert!(result.is_ok());

        let translated = result.unwrap();
        assert!((translated.x() - 2.0).abs() < 1e-10);
        assert!((translated.y() - 1.5).abs() < 1e-10);
        assert!((translated.z() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_translate_infinite_vector() {
        let point = create_test_point();
        let translation = Vector3D::new(INFINITY, 1.0, 1.0);

        let result = point.safe_translate(translation);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    #[test]
    fn test_safe_translate_nan_vector() {
        let point = create_test_point();
        let translation = Vector3D::new(1.0, NAN, 1.0);

        let result = point.safe_translate(translation);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    // ============================================================================
    // スケールテスト
    // ============================================================================

    #[test]
    fn test_safe_scale_success() {
        let point = create_test_point();
        let center = Point3D::origin();
        let factor = 2.0;

        let result = point.safe_scale(center, factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();
        assert!((scaled.x() - 2.0).abs() < 1e-10);
        assert!((scaled.y() - 4.0).abs() < 1e-10);
        assert!((scaled.z() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let point = create_test_point();
        let center = Point3D::origin();

        let result = point.safe_scale(center, 0.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_safe_scale_infinite_factor_error() {
        let point = create_test_point();
        let center = Point3D::origin();

        let result = point.safe_scale(center, INFINITY);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_safe_scale_infinite_center_error() {
        let point = create_test_point();
        let center = Point3D::new(INFINITY, 0.0, 0.0);

        let result = point.safe_scale(center, 2.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    #[test]
    fn test_safe_scale_negative_factor() {
        let point = create_test_point();
        let center = Point3D::origin();
        let factor = -1.0;

        let result = point.safe_scale(center, factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();
        assert!((scaled.x() - (-1.0)).abs() < 1e-10);
        assert!((scaled.y() - (-2.0)).abs() < 1e-10);
        assert!((scaled.z() - (-3.0)).abs() < 1e-10);
    }

    // ============================================================================
    // 回転テスト
    // ============================================================================

    #[test]
    fn test_safe_rotate_success() {
        let point = Point3D::new(1.0, 0.0, 0.0);
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸周り
        let angle = Angle::from_radians(PI / 2.0);

        let result = point.safe_rotate(center, axis, angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        // 90度回転で (1,0,0) -> (0,1,0)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_rotate_zero_axis_error() {
        let point = create_test_point();
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 0.0); // ゼロベクトル
        let angle = Angle::from_radians(PI / 4.0);

        let result = point.safe_rotate(center, axis, angle);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::ZeroVector(_) => (),
            _ => panic!("Expected ZeroVector error"),
        }
    }

    #[test]
    fn test_safe_rotate_infinite_center_error() {
        let point = create_test_point();
        let center = Point3D::new(INFINITY, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_radians(PI / 4.0);

        let result = point.safe_rotate(center, axis, angle);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => (),
            _ => panic!("Expected InvalidGeometry error"),
        }
    }

    #[test]
    fn test_safe_rotate_infinite_angle_error() {
        let point = create_test_point();
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_radians(INFINITY);

        let result = point.safe_rotate(center, axis, angle);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidRotation(_) => (),
            _ => panic!("Expected InvalidRotation error"),
        }
    }

    // ============================================================================
    // 複合変換テスト
    // ============================================================================

    #[test]
    fn test_safe_translate_and_rotate() {
        let point = create_test_point();
        let translation = Vector3D::new(1.0, 1.0, 0.0);
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_radians(PI / 4.0);

        let result = point.safe_translate_and_rotate(
            translation,
            rotation_center,
            rotation_axis,
            rotation_angle,
        );

        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert!(transformed.detailed_validation().is_ok());
    }

    #[test]
    fn test_safe_scale_and_translate() {
        let point = create_test_point();
        let scale_center = Point3D::origin();
        let scale_factor = 2.0;
        let translation = Vector3D::new(1.0, -1.0, 0.5);

        let result = point.safe_scale_and_translate(scale_center, scale_factor, translation);

        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert!(transformed.detailed_validation().is_ok());
    }

    // ============================================================================
    // 検証テスト
    // ============================================================================

    #[test]
    fn test_detailed_validation() {
        let valid_point = create_test_point();
        assert!(valid_point.detailed_validation().is_ok());

        let invalid_point = Point3D::new(NAN, 2.0, 3.0);
        assert!(invalid_point.detailed_validation().is_err());
    }

    #[test]
    fn test_error_propagation() {
        let point = create_test_point();
        let invalid_translation = Vector3D::new(NAN, 1.0, 1.0);

        let result = point.safe_translate(invalid_translation);
        assert!(result.is_err());

        // エラーがチェーン変換でも適切に伝播することを確認
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_radians(PI / 4.0);

        let chain_result = point.safe_translate_and_rotate(
            invalid_translation,
            rotation_center,
            rotation_axis,
            rotation_angle,
        );
        assert!(chain_result.is_err());
    }

    #[test]
    fn test_transform_chain_with_error_handling() {
        let point = create_test_point();

        // 正常なチェーン
        let result = point
            .safe_translate(Vector3D::new(1.0, 0.0, 0.0))
            .and_then(|p| p.safe_scale(Point3D::origin(), 2.0))
            .and_then(|p| {
                p.safe_rotate(
                    Point3D::origin(),
                    Vector3D::new(0.0, 0.0, 1.0),
                    Angle::from_radians(PI / 6.0),
                )
            });

        assert!(result.is_ok());
        let final_point = result.unwrap();
        assert!(final_point.detailed_validation().is_ok());
    }
}
