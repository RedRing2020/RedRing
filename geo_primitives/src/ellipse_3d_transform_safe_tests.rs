//! Ellipse3D Safe Transform テスト
//!
//! Result型を使用した安全な変換機能のテスト

#[cfg(test)]
mod ellipse_3d_safe_transform_tests {
    use crate::{ellipse_3d::Ellipse3D, point_3d::Point3D, vector_3d::Vector3D, Angle};
    use geo_foundation::TransformError;
    use std::f64::consts::PI;

    /// テスト用の基準楕円を作成
    fn create_test_ellipse() -> Ellipse3D<f64> {
        Ellipse3D::new(
            Point3D::new(1.0, 2.0, 3.0),
            2.0,                          // 長軸
            1.0,                          // 短軸
            Vector3D::new(0.0, 0.0, 1.0), // Z軸に垂直な楕円
            Vector3D::new(1.0, 0.0, 0.0), // X軸方向が長軸
        )
        .expect("テスト楕円の作成に成功")
    }

    #[test]
    fn test_safe_translate_success() {
        let ellipse = create_test_ellipse();
        let translation = Vector3D::new(5.0, -2.0, 1.0);

        let result = ellipse.safe_translate(translation);

        assert!(result.is_ok());
        let translated = result.unwrap();

        // 中心が平行移動されることを確認
        let expected_center = Point3D::new(6.0, 0.0, 4.0);
        assert_eq!(translated.center(), expected_center);
    }

    #[test]
    fn test_safe_scale_success() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let scale_factor = 2.0;

        let result = ellipse.safe_scale(scale_center, scale_factor);

        assert!(result.is_ok());
        let scaled = result.unwrap();

        // 半軸長がスケールされることを確認
        assert_eq!(scaled.semi_major_axis(), 4.0);
        assert_eq!(scaled.semi_minor_axis(), 2.0);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let scale_factor = 0.0;

        let result = ellipse.safe_scale(scale_center, scale_factor);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_safe_scale_negative_factor() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let scale_factor = -2.0;

        let result = ellipse.safe_scale(scale_center, scale_factor);

        assert!(result.is_ok());
        let scaled = result.unwrap();

        // 負のスケールでは絶対値を使用
        assert_eq!(scaled.semi_major_axis(), 4.0);
        assert_eq!(scaled.semi_minor_axis(), 2.0);
    }

    #[test]
    fn test_safe_rotate_success() {
        let ellipse = create_test_ellipse();
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_radians(PI / 2.0);

        let result = ellipse.safe_rotate(rotation_center, rotation_axis, rotation_angle);

        assert!(result.is_ok());
        let rotated = result.unwrap();

        // 基本的な妥当性を確認
        assert!(rotated.detailed_validation().is_ok());
    }

    #[test]
    fn test_safe_rotate_zero_axis_error() {
        let ellipse = create_test_ellipse();
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::zero(); // ゼロベクトル
        let rotation_angle = Angle::from_radians(PI / 2.0);

        let result = ellipse.safe_rotate(rotation_center, rotation_axis, rotation_angle);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::ZeroVector(_) => (),
            _ => panic!("Expected ZeroVector error"),
        }
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();

        let result = ellipse.safe_scale_non_uniform(scale_center, 2.0, 1.5, 3.0);

        assert!(result.is_ok());
        let scaled = result.unwrap();

        // 平均スケール倍率での近似
        let expected_avg_scale = (2.0 + 1.5 + 3.0) / 3.0;
        let expected_major = 2.0 * expected_avg_scale;
        let expected_minor = 1.0 * expected_avg_scale;

        assert!((scaled.semi_major_axis() - expected_major).abs() < 1e-10);
        assert!((scaled.semi_minor_axis() - expected_minor).abs() < 1e-10);
    }

    #[test]
    fn test_safe_scale_non_uniform_zero_scale_error() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();

        // X軸でゼロスケール
        let result = ellipse.safe_scale_non_uniform(scale_center, 0.0, 1.5, 3.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }

        // Y軸でゼロスケール
        let result = ellipse.safe_scale_non_uniform(scale_center, 2.0, 0.0, 3.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }

        // Z軸でゼロスケール
        let result = ellipse.safe_scale_non_uniform(scale_center, 2.0, 1.5, 0.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_safe_reverse_success() {
        let ellipse = create_test_ellipse();

        let result = ellipse.safe_reverse();

        assert!(result.is_ok());
        let reversed = result.unwrap();

        // 法線方向が反転されていることを確認
        assert_eq!(reversed.normal().as_vector(), -ellipse.normal().as_vector());

        // 他のプロパティは変化しない
        assert_eq!(reversed.center(), ellipse.center());
        assert_eq!(reversed.semi_major_axis(), ellipse.semi_major_axis());
        assert_eq!(reversed.semi_minor_axis(), ellipse.semi_minor_axis());
    }

    #[test]
    fn test_safe_translate_and_rotate() {
        let ellipse = create_test_ellipse();
        let translation = Vector3D::new(1.0, 1.0, 0.0);
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_radians(PI / 4.0);

        let result = ellipse.safe_translate_and_rotate(
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
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let scale_factor = 1.5;
        let translation = Vector3D::new(2.0, -1.0, 0.5);

        let result = ellipse.safe_scale_and_translate(scale_center, scale_factor, translation);

        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert!(transformed.detailed_validation().is_ok());
    }

    #[test]
    fn test_safe_scale_non_uniform_and_translate() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let result =
            ellipse.safe_scale_non_uniform_and_translate(scale_center, 2.0, 1.5, 3.0, translation);

        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert!(transformed.detailed_validation().is_ok());
    }

    #[test]
    fn test_validate_scale_factor() {
        assert!(Ellipse3D::<f64>::validate_scale_factor(1.0));
        assert!(Ellipse3D::<f64>::validate_scale_factor(-1.0));
        assert!(Ellipse3D::<f64>::validate_scale_factor(0.1));

        assert!(!Ellipse3D::<f64>::validate_scale_factor(0.0));
        assert!(!Ellipse3D::<f64>::validate_scale_factor(f64::INFINITY));
        assert!(!Ellipse3D::<f64>::validate_scale_factor(f64::NAN));
    }

    #[test]
    fn test_validate_rotation_axis() {
        assert!(Ellipse3D::<f64>::validate_rotation_axis(Vector3D::new(
            1.0, 0.0, 0.0
        )));
        assert!(Ellipse3D::<f64>::validate_rotation_axis(Vector3D::new(
            0.0, 1.0, 0.0
        )));
        assert!(Ellipse3D::<f64>::validate_rotation_axis(Vector3D::new(
            1.0, 1.0, 1.0
        )));

        assert!(!Ellipse3D::<f64>::validate_rotation_axis(Vector3D::zero()));
        assert!(!Ellipse3D::<f64>::validate_rotation_axis(Vector3D::new(
            f64::NAN,
            0.0,
            0.0
        )));
    }

    #[test]
    fn test_detailed_validation() {
        let ellipse = create_test_ellipse();

        // 正常な楕円
        assert!(ellipse.detailed_validation().is_ok());

        // 不正な楕円は作成時点で弾かれるため、
        // ここでは既存の楕円の検証のみテスト
    }

    #[test]
    fn test_error_propagation() {
        let ellipse = create_test_ellipse();

        // 複合変換でエラーが適切に伝播することを確認
        let result = ellipse.safe_scale_and_translate(
            Point3D::origin(),
            0.0, // 無効なスケール倍率
            Vector3D::new(1.0, 1.0, 1.0),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_transform_chain_with_error_handling() {
        let ellipse = create_test_ellipse();

        // 正常な変換チェーン
        let result = ellipse
            .safe_translate(Vector3D::new(1.0, 1.0, 1.0))
            .and_then(|e| e.safe_scale(Point3D::origin(), 2.0))
            .and_then(|e| e.safe_reverse());

        assert!(result.is_ok());

        // エラーを含む変換チェーン
        let result = ellipse
            .safe_translate(Vector3D::new(1.0, 1.0, 1.0))
            .and_then(|e| e.safe_scale(Point3D::origin(), 0.0)) // エラー発生点
            .and_then(|e| e.safe_reverse());

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => (),
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }
}
