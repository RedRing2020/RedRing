//! Plane3D Safe Transform テスト
//!
//! Result型を使用した安全な変換機能のテスト

#[cfg(test)]
mod plane_3d_safe_transform_tests {
    use crate::{Plane3D, Point3D, Vector3D};
    use analysis::Angle;
    use geo_foundation::TransformError;
    use std::f64::consts::PI;

    fn create_test_plane() -> Plane3D<f64> {
        let point = Point3D::new(0.0, 0.0, 1.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        Plane3D::from_point_and_normal(point, normal).unwrap()
    }

    // ============================================================================
    // 安全な平行移動テスト
    // ============================================================================

    #[test]
    fn test_safe_translate_success() {
        let plane = create_test_plane();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let result = plane.safe_translate(translation);
        assert!(result.is_ok());

        let translated = result.unwrap();
        assert!((translated.point().x() - 1.0).abs() < 1e-10);
        assert!((translated.point().y() - 2.0).abs() < 1e-10);
        assert!((translated.point().z() - 4.0).abs() < 1e-10);

        // 法線ベクトルは変わらない
        assert!((translated.normal().x() - 0.0).abs() < 1e-10);
        assert!((translated.normal().y() - 0.0).abs() < 1e-10);
        assert!((translated.normal().z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let plane = create_test_plane();
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);

        let result = plane.safe_translate(invalid_translation);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("無効な移動ベクトル"));
        } else {
            panic!("Expected InvalidGeometry error");
        }
    }

    #[test]
    fn test_safe_translate_nan_vector() {
        let plane = create_test_plane();
        let nan_translation = Vector3D::new(0.0, f64::NAN, 0.0);

        let result = plane.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_translate_result_overflow() {
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(f64::MAX * 0.9, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let large_translation = Vector3D::new(f64::MAX * 0.9, 0.0, 0.0);

        let result = plane.safe_translate(large_translation);
        assert!(result.is_err());
    }

    // ============================================================================
    // 安全な回転テスト
    // ============================================================================

    #[test]
    fn test_safe_rotate_success() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 4.0); // 45度

        let result = plane.safe_rotate(center, angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();

        // Z軸回りの回転なので、Z=1の平面の法線(0,0,1)は変わらない
        // 平面上の点(0,0,1)も変わらない
        assert!((rotated.point().x() - 0.0).abs() < 1e-10);
        assert!((rotated.point().y() - 0.0).abs() < 1e-10);
        assert!((rotated.point().z() - 1.0).abs() < 1e-10);

        // 法線ベクトルもZ軸方向なので変わらない
        assert!((rotated.normal().x() - 0.0).abs() < 1e-10);
        assert!((rotated.normal().y() - 0.0).abs() < 1e-10);
        assert!((rotated.normal().z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_rotate_invalid_center() {
        let plane = create_test_plane();
        let invalid_center = Point3D::new(f64::INFINITY, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0);

        let result = plane.safe_rotate(invalid_center, angle);
        assert!(result.is_err());

        if let Err(TransformError::InvalidRotation(msg)) = result {
            assert!(msg.contains("回転中心が無効"));
        } else {
            panic!("Expected InvalidRotation error");
        }
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let invalid_angle = Angle::from_radians(f64::NAN);

        let result = plane.safe_rotate(center, invalid_angle);
        assert!(result.is_err());

        if let Err(TransformError::InvalidRotation(msg)) = result {
            assert!(msg.contains("回転角度が無効"));
        } else {
            panic!("Expected InvalidRotation error");
        }
    }

    #[test]
    fn test_safe_rotate_infinite_angle() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let infinite_angle = Angle::from_radians(f64::INFINITY);

        let result = plane.safe_rotate(center, infinite_angle);
        assert!(result.is_err());
    }

    // ============================================================================
    // 安全なスケールテスト
    // ============================================================================

    #[test]
    fn test_safe_scale_success() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let factor = 2.0;

        let result = plane.safe_scale(center, factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();

        // 参照点がスケールされる
        assert!((scaled.point().x() - 0.0).abs() < 1e-10);
        assert!((scaled.point().y() - 0.0).abs() < 1e-10);
        assert!((scaled.point().z() - 2.0).abs() < 1e-10); // 1.0 * 2.0

        // 法線ベクトルは正規化された状態を保持
        assert!((scaled.normal().x() - 0.0).abs() < 1e-10);
        assert!((scaled.normal().y() - 0.0).abs() < 1e-10);
        assert!((scaled.normal().z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_safe_scale_zero_factor() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let zero_factor = 0.0;

        let result = plane.safe_scale(center, zero_factor);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("スケール係数がゼロ"));
        } else {
            panic!("Expected InvalidScaleFactor error");
        }
    }

    #[test]
    fn test_safe_scale_negative_factor() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let negative_factor = -1.5;

        let result = plane.safe_scale(center, negative_factor);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("負のスケール係数"));
        } else {
            panic!("Expected InvalidScaleFactor error");
        }
    }

    #[test]
    fn test_safe_scale_infinite_factor() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let infinite_factor = f64::INFINITY;

        let result = plane.safe_scale(center, infinite_factor);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("スケール係数が無効"));
        } else {
            panic!("Expected InvalidScaleFactor error");
        }
    }

    #[test]
    fn test_safe_scale_invalid_center() {
        let plane = create_test_plane();
        let invalid_center = Point3D::new(f64::NAN, 0.0, 0.0);
        let factor = 2.0;

        let result = plane.safe_scale(invalid_center, factor);
        assert!(result.is_err());
    }

    // ============================================================================
    // 安全な非一様スケールテスト
    // ============================================================================

    #[test]
    fn test_safe_non_uniform_scale_success() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let factors = Vector3D::new(2.0, 3.0, 1.5);

        let result = plane.safe_non_uniform_scale(center, factors);
        assert!(result.is_ok());

        let scaled = result.unwrap();

        // 参照点がスケールされる
        assert!((scaled.point().x() - 0.0).abs() < 1e-10);
        assert!((scaled.point().y() - 0.0).abs() < 1e-10);
        assert!((scaled.point().z() - 1.5).abs() < 1e-10); // 1.0 * 1.5

        // 非一様スケールでは法線ベクトルも変形される
        let normal = scaled.normal();
        let length =
            (normal.x() * normal.x() + normal.y() * normal.y() + normal.z() * normal.z()).sqrt();
        assert!((length - 1.0).abs() < 1e-10); // 正規化されている
    }

    #[test]
    fn test_safe_non_uniform_scale_zero_factor() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let factors = Vector3D::new(2.0, 0.0, 1.5); // Y軸がゼロ

        let result = plane.safe_non_uniform_scale(center, factors);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("スケール係数にゼロが含まれています"));
        } else {
            panic!("Expected InvalidScaleFactor error");
        }
    }

    #[test]
    fn test_safe_non_uniform_scale_negative_factor() {
        let plane = create_test_plane();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let factors = Vector3D::new(2.0, -1.0, 1.5); // Y軸が負

        let result = plane.safe_non_uniform_scale(center, factors);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("負のスケール係数"));
        } else {
            panic!("Expected InvalidScaleFactor error");
        }
    }

    // ============================================================================
    // 安全な複合変換テスト
    // ============================================================================

    #[test]
    fn test_safe_transform_composite_success() {
        let plane = create_test_plane();
        let translation = Vector3D::new(1.0, 0.0, 0.0);
        let rotation_center = Point3D::new(0.0, 0.0, 0.0);
        let rotation_angle = Angle::from_radians(PI / 2.0);
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scale_factor = 2.0;

        let result = plane.safe_transform_composite(
            translation,
            rotation_center,
            rotation_angle,
            scale_center,
            scale_factor,
        );

        assert!(result.is_ok());
        let transformed = result.unwrap();

        // 複合変換後の結果を検証
        // (平行移動 -> 回転 -> スケール の順で適用)
        assert!(transformed.point().x().is_finite());
        assert!(transformed.point().y().is_finite());
        assert!(transformed.point().z().is_finite());
        assert!(transformed.normal().x().is_finite());
        assert!(transformed.normal().y().is_finite());
        assert!(transformed.normal().z().is_finite());
    }

    #[test]
    fn test_safe_transform_composite_fails_at_translation() {
        let plane = create_test_plane();
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);
        let rotation_center = Point3D::new(0.0, 0.0, 0.0);
        let rotation_angle = Angle::from_radians(PI / 2.0);
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scale_factor = 2.0;

        let result = plane.safe_transform_composite(
            invalid_translation,
            rotation_center,
            rotation_angle,
            scale_center,
            scale_factor,
        );

        assert!(result.is_err());
        // 最初のステップ（平行移動）で失敗するはず
    }

    #[test]
    fn test_safe_transform_composite_fails_at_rotation() {
        let plane = create_test_plane();
        let translation = Vector3D::new(1.0, 0.0, 0.0);
        let invalid_rotation_center = Point3D::new(f64::NAN, 0.0, 0.0);
        let rotation_angle = Angle::from_radians(PI / 2.0);
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scale_factor = 2.0;

        let result = plane.safe_transform_composite(
            translation,
            invalid_rotation_center,
            rotation_angle,
            scale_center,
            scale_factor,
        );

        assert!(result.is_err());
        // 回転ステップで失敗するはず
    }

    #[test]
    fn test_safe_transform_composite_fails_at_scale() {
        let plane = create_test_plane();
        let translation = Vector3D::new(1.0, 0.0, 0.0);
        let rotation_center = Point3D::new(0.0, 0.0, 0.0);
        let rotation_angle = Angle::from_radians(PI / 2.0);
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let invalid_scale_factor = 0.0; // ゼロスケール

        let result = plane.safe_transform_composite(
            translation,
            rotation_center,
            rotation_angle,
            scale_center,
            invalid_scale_factor,
        );

        assert!(result.is_err());
        // スケールステップで失敗するはず
    }
}
