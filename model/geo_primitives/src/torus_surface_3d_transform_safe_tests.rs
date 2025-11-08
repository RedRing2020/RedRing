//! TorusSurface3D Safe Transform Test Suite - Comprehensive error handling tests
//!
//! SafeTransform実装のエラーハンドリングと境界値テスト

#[cfg(test)]
mod tests {
    use crate::{Direction3D, Point3D, TorusSurface3D, Vector3D};
    use analysis::Angle;
    use geo_foundation::TransformError;
    use std::f64::consts::PI;

    // テスト用のf64型別名
    type TestScalar = f64;

    // テスト用の標準トーラス面を作成
    fn create_test_torus() -> TorusSurface3D<TestScalar> {
        let origin = Point3D::origin();
        let z_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let x_axis = Direction3D::from_vector(Vector3D::unit_x()).unwrap();
        let major_radius = 5.0;
        let minor_radius = 2.0;

        TorusSurface3D::new(origin, z_axis, x_axis, major_radius, minor_radius).unwrap()
    }

    // =================================================================
    // 安全な平行移動テスト
    // =================================================================

    #[test]
    fn test_safe_translate_valid() {
        let torus = create_test_torus();
        let translation = Vector3D::new(10.0, 20.0, 30.0);

        let result = torus.safe_translate(translation);
        assert!(result.is_ok());

        let translated = result.unwrap();
        let expected_origin = torus.origin() + translation;
        assert_eq!(translated.origin(), expected_origin);
        assert_eq!(translated.major_radius(), torus.major_radius());
        assert_eq!(translated.minor_radius(), torus.minor_radius());
    }

    #[test]
    fn test_safe_translate_infinite_vector() {
        let torus = create_test_torus();
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);

        let result = torus.safe_translate(invalid_translation);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("無効な移動ベクトル"));
        } else {
            panic!("Expected InvalidGeometry error for infinite translation vector");
        }
    }

    #[test]
    fn test_safe_translate_nan_vector() {
        let torus = create_test_torus();
        let invalid_translation = Vector3D::new(0.0, f64::NAN, 0.0);

        let result = torus.safe_translate(invalid_translation);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("無効な移動ベクトル"));
        } else {
            panic!("Expected InvalidGeometry error for NaN translation vector");
        }
    }

    #[test]
    fn test_safe_translate_very_large_values() {
        let torus = create_test_torus();
        let large_translation = Vector3D::new(1e100, 1e100, 1e100);

        let result = torus.safe_translate(large_translation);
        // 有限値だが非常に大きい値での計算結果のチェック
        if let Ok(translated) = result {
            assert!(translated.origin().x().is_finite());
            assert!(translated.origin().y().is_finite());
            assert!(translated.origin().z().is_finite());
        }
    }

    // =================================================================
    // 安全なスケール変換テスト
    // =================================================================

    #[test]
    fn test_safe_scale_valid() {
        let torus = create_test_torus();
        let center = Point3D::new(1.0, 2.0, 3.0);
        let scale_factor = 2.0;

        let result = torus.safe_scale(center, scale_factor, scale_factor, scale_factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();
        assert_eq!(scaled.major_radius(), torus.major_radius() * scale_factor);
        assert_eq!(scaled.minor_radius(), torus.minor_radius() * scale_factor);
    }

    #[test]
    fn test_safe_scale_zero_factor() {
        let torus = create_test_torus();
        let center = Point3D::origin();

        let result = torus.safe_scale(center, 0.0, 1.0, 1.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("ゼロスケール倍率は未対応"));
        } else {
            panic!("Expected InvalidScaleFactor error for zero scale factor");
        }
    }

    #[test]
    fn test_safe_scale_negative_factor() {
        let torus = create_test_torus();
        let center = Point3D::origin();

        let result = torus.safe_scale(center, -1.0, 1.0, 1.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("負のスケール倍率は未対応"));
        } else {
            panic!("Expected InvalidScaleFactor error for negative scale factor");
        }
    }

    #[test]
    fn test_safe_scale_infinite_factor() {
        let torus = create_test_torus();
        let center = Point3D::origin();

        let result = torus.safe_scale(center, f64::INFINITY, 1.0, 1.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("無効なスケール倍率"));
        } else {
            panic!("Expected InvalidScaleFactor error for infinite scale factor");
        }
    }

    #[test]
    fn test_safe_scale_non_uniform() {
        let torus = create_test_torus();
        let center = Point3D::origin();

        // 非等方性スケール（X, Y, Z軸で異なる倍率）
        let result = torus.safe_scale(center, 2.0, 3.0, 4.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidScaleFactor(msg)) = result {
            assert!(msg.contains("等方性スケールのみサポート"));
        } else {
            panic!("Expected InvalidScaleFactor error for non-uniform scaling");
        }
    }

    #[test]
    fn test_safe_scale_invalid_radius_result() {
        let torus = create_test_torus();
        let center = Point3D::origin();
        let tiny_scale = 1e-20; // 非常に小さなスケール

        let result = torus.safe_scale(center, tiny_scale, tiny_scale, tiny_scale);
        // 結果として半径が無効になる可能性をチェック
        if result.is_err() {
            if let Err(TransformError::InvalidGeometry(msg)) = result {
                assert!(msg.contains("半径") || msg.contains("無効"));
            }
        }
    }

    #[test]
    fn test_safe_scale_geometric_constraint_violation() {
        // 元々主半径と副半径が近いトーラス
        let close_radii_torus = TorusSurface3D::new(
            Point3D::origin(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            2.1, // 主半径
            2.0, // 副半径（非常に近い）
        )
        .unwrap();

        let center = Point3D::origin();
        let scale_factor = 0.9; // 小さくスケール

        let result = close_radii_torus.safe_scale(center, scale_factor, scale_factor, scale_factor);

        if result.is_err() {
            if let Err(TransformError::InvalidGeometry(msg)) = result {
                assert!(msg.contains("主半径") && msg.contains("副半径"));
            }
        }
    }

    // =================================================================
    // 安全な回転変換テスト
    // =================================================================

    #[test]
    fn test_safe_rotate_z_valid() {
        let torus = create_test_torus();
        let angle = Angle::from_radians(PI / 4.0);

        let result = torus.safe_rotate_z(angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        assert_eq!(rotated.z_axis(), torus.z_axis()); // Z軸は変わらない
        assert_eq!(rotated.major_radius(), torus.major_radius());
        assert_eq!(rotated.minor_radius(), torus.minor_radius());
    }

    #[test]
    fn test_safe_rotate_z_infinite_angle() {
        let torus = create_test_torus();
        let invalid_angle = Angle::from_radians(f64::INFINITY);

        let result = torus.safe_rotate_z(invalid_angle);
        assert!(result.is_err());

        if let Err(TransformError::InvalidRotation(msg)) = result {
            assert!(msg.contains("無効な回転角度"));
        } else {
            panic!("Expected InvalidRotation error for infinite angle");
        }
    }

    #[test]
    fn test_safe_rotate_z_nan_angle() {
        let torus = create_test_torus();
        let invalid_angle = Angle::from_radians(f64::NAN);

        let result = torus.safe_rotate_z(invalid_angle);
        assert!(result.is_err());

        if let Err(TransformError::InvalidRotation(msg)) = result {
            assert!(msg.contains("無効な回転角度"));
        } else {
            panic!("Expected InvalidRotation error for NaN angle");
        }
    }

    #[test]
    fn test_safe_rotate_around_axis_valid() {
        let torus = create_test_torus();
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Direction3D::from_vector(Vector3D::unit_y()).unwrap();
        let angle = Angle::from_radians(PI / 6.0);

        let result = torus.safe_rotate_around_axis(center, axis, angle);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        assert_eq!(rotated.major_radius(), torus.major_radius());
        assert_eq!(rotated.minor_radius(), torus.minor_radius());
    }

    #[test]
    fn test_safe_rotate_around_axis_invalid_center() {
        let torus = create_test_torus();
        let invalid_center = Point3D::new(f64::INFINITY, 0.0, 0.0);
        let axis = Direction3D::from_vector(Vector3D::unit_y()).unwrap();
        let angle = Angle::from_radians(PI / 6.0);

        let result = torus.safe_rotate_around_axis(invalid_center, axis, angle);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("無効な回転中心"));
        } else {
            panic!("Expected InvalidGeometry error for invalid rotation center");
        }
    }

    // =================================================================
    // 安全な半径変更テスト
    // =================================================================

    #[test]
    fn test_safe_resize_valid() {
        let torus = create_test_torus();
        let new_major_radius = 10.0;
        let new_minor_radius = 3.0;

        let result = torus.safe_resize(new_major_radius, new_minor_radius);
        assert!(result.is_ok());

        let resized = result.unwrap();
        assert_eq!(resized.major_radius(), new_major_radius);
        assert_eq!(resized.minor_radius(), new_minor_radius);
        assert_eq!(resized.origin(), torus.origin()); // 他は変わらない
    }

    #[test]
    fn test_safe_resize_zero_radius() {
        let torus = create_test_torus();

        let result = torus.safe_resize(0.0, 2.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("半径は正の値である必要があります"));
        } else {
            panic!("Expected InvalidGeometry error for zero radius");
        }
    }

    #[test]
    fn test_safe_resize_negative_radius() {
        let torus = create_test_torus();

        let result = torus.safe_resize(5.0, -1.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("半径は正の値である必要があります"));
        } else {
            panic!("Expected InvalidGeometry error for negative radius");
        }
    }

    #[test]
    fn test_safe_resize_geometric_constraint() {
        let torus = create_test_torus();

        // 主半径 <= 副半径の無効な組み合わせ
        let result = torus.safe_resize(2.0, 5.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("主半径は副半径より大きい必要があります"));
        } else {
            panic!("Expected InvalidGeometry error for major <= minor radius");
        }
    }

    #[test]
    fn test_safe_resize_infinite_radius() {
        let torus = create_test_torus();

        let result = torus.safe_resize(f64::INFINITY, 2.0);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("無効な半径値"));
        } else {
            panic!("Expected InvalidGeometry error for infinite radius");
        }
    }

    // =================================================================
    // 安全な軸方向変更テスト
    // =================================================================

    #[test]
    fn test_safe_reorient_valid() {
        let torus = create_test_torus();
        let new_z_axis = Direction3D::from_vector(Vector3D::unit_y()).unwrap();
        let new_x_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();

        let result = torus.safe_reorient(new_z_axis, new_x_axis);
        assert!(result.is_ok());

        let reoriented = result.unwrap();
        assert_eq!(reoriented.z_axis(), new_z_axis);
        assert_eq!(reoriented.x_axis(), new_x_axis);
        assert_eq!(reoriented.major_radius(), torus.major_radius());
        assert_eq!(reoriented.minor_radius(), torus.minor_radius());
    }

    #[test]
    fn test_safe_reorient_non_orthogonal() {
        let torus = create_test_torus();
        let z_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let non_orthogonal_x = Direction3D::from_vector(Vector3D::new(0.0, 1.0, 1.0)).unwrap();

        let result = torus.safe_reorient(z_axis, non_orthogonal_x);
        assert!(result.is_err());

        if let Err(TransformError::InvalidGeometry(msg)) = result {
            assert!(msg.contains("Z軸とX軸は直交している必要があります"));
        } else {
            panic!("Expected InvalidGeometry error for non-orthogonal axes");
        }
    }

    // =================================================================
    // 複合変換テスト
    // =================================================================

    #[test]
    fn test_safe_transform_valid() {
        let torus = create_test_torus();
        let translation = Vector3D::new(5.0, 10.0, 15.0);
        let rotation_center = Point3D::new(2.0, 3.0, 4.0);
        let rotation_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let rotation_angle = Angle::from_radians(PI / 3.0);
        let scale_factor = 1.5;

        let result = torus.safe_transform(
            translation,
            rotation_center,
            rotation_axis,
            rotation_angle,
            scale_factor,
        );

        assert!(result.is_ok());
        let transformed = result.unwrap();

        // スケールされた半径の確認
        assert!((transformed.major_radius() - torus.major_radius() * scale_factor).abs() < 1e-10);
        assert!((transformed.minor_radius() - torus.minor_radius() * scale_factor).abs() < 1e-10);
    }

    #[test]
    fn test_safe_transform_invalid_scale() {
        let torus = create_test_torus();
        let translation = Vector3D::new(1.0, 2.0, 3.0);
        let rotation_center = Point3D::origin();
        let rotation_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let rotation_angle = Angle::from_radians(0.0);
        let invalid_scale = 0.0; // ゼロスケール

        let result = torus.safe_transform(
            translation,
            rotation_center,
            rotation_axis,
            rotation_angle,
            invalid_scale,
        );

        assert!(result.is_err());
        // スケール段階でエラーが発生するはず
    }

    #[test]
    fn test_safe_transform_invalid_rotation() {
        let torus = create_test_torus();
        let translation = Vector3D::new(1.0, 2.0, 3.0);
        let rotation_center = Point3D::origin();
        let rotation_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let invalid_angle = Angle::from_radians(f64::NAN);
        let scale_factor = 1.0;

        let result = torus.safe_transform(
            translation,
            rotation_center,
            rotation_axis,
            invalid_angle,
            scale_factor,
        );

        assert!(result.is_err());
        // 回転段階でエラーが発生するはず
    }

    // =================================================================
    // 境界値・エッジケーステスト
    // =================================================================

    #[test]
    fn test_edge_case_very_small_torus() {
        let tiny_major = 1e-10;
        let tiny_minor = 1e-11;

        let tiny_torus = TorusSurface3D::new(
            Point3D::origin(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            tiny_major,
            tiny_minor,
        )
        .unwrap();

        // 小さなトーラスでの変換操作
        let translation = Vector3D::new(1.0, 0.0, 0.0);
        let result = tiny_torus.safe_translate(translation);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case_large_scale_factor() {
        let torus = create_test_torus();
        let center = Point3D::origin();
        let large_scale = 1e10;

        let result = torus.safe_scale(center, large_scale, large_scale, large_scale);

        if let Ok(scaled) = result {
            // 計算結果が有限であることを確認
            assert!(scaled.major_radius().is_finite());
            assert!(scaled.minor_radius().is_finite());
        } else {
            // エラーが発生する場合は、適切なエラー種別であることを確認
            assert!(matches!(
                result.unwrap_err(),
                TransformError::InvalidGeometry(_) | TransformError::InvalidScaleFactor(_)
            ));
        }
    }

    #[test]
    fn test_numerical_precision_edge_cases() {
        let torus = create_test_torus();

        // 数値精度の限界近くでの計算
        let tiny_translation = Vector3D::new(f64::EPSILON, f64::EPSILON, f64::EPSILON);
        let result = torus.safe_translate(tiny_translation);
        assert!(result.is_ok());

        // 微小回転
        let tiny_angle = Angle::from_radians(f64::EPSILON);
        let result = torus.safe_rotate_z(tiny_angle);
        assert!(result.is_ok());
    }
}
