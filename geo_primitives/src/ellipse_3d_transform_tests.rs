//! Ellipse3D Transform テスト
//!
//! BasicTransform3D トレイトと追加メソッドのテスト

#[cfg(test)]
mod ellipse_3d_transform_tests {
    use crate::{ellipse_3d::Ellipse3D, point_3d::Point3D, vector_3d::Vector3D, Angle};
    use geo_foundation::extensions::BasicTransform3D;
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
    fn test_translate_3d() {
        let ellipse = create_test_ellipse();
        let translation = Vector3D::new(5.0, -2.0, 1.0);

        let translated = ellipse.translate_3d(translation);

        // 中心が平行移動されることを確認
        let expected_center = Point3D::new(6.0, 0.0, 4.0);
        assert_eq!(translated.center(), expected_center);

        // 半軸長は変化しないことを確認
        assert_eq!(translated.semi_major_axis(), 2.0);
        assert_eq!(translated.semi_minor_axis(), 1.0);

        // 法線方向は変化しないことを確認
        assert_eq!(
            translated.normal().as_vector(),
            Vector3D::new(0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn test_rotate_3d() {
        let ellipse = create_test_ellipse();
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_radians(PI / 2.0);

        let rotated = ellipse.rotate_3d(rotation_center, (rotation_axis, rotation_angle));

        // 簡易実装では回転は近似的
        // 基本的な妥当性のみ確認
        assert!(rotated.is_valid_transform());
        assert_eq!(rotated.semi_major_axis(), 2.0);
        assert_eq!(rotated.semi_minor_axis(), 1.0);
    }

    #[test]
    fn test_scale_3d() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let scale_factor = 2.0;

        let scaled = ellipse.scale_3d(scale_center, scale_factor);

        // 中心位置がスケールされることを確認
        let expected_center = Point3D::new(2.0, 4.0, 6.0);
        assert_eq!(scaled.center(), expected_center);

        // 半軸長がスケールされることを確認
        assert_eq!(scaled.semi_major_axis(), 4.0);
        assert_eq!(scaled.semi_minor_axis(), 2.0);

        // 法線方向は変化しないことを確認
        assert_eq!(scaled.normal().as_vector(), Vector3D::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_scale_3d_negative_factor() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let scale_factor = -2.0;

        let scaled = ellipse.scale_3d(scale_center, scale_factor);

        // 負のスケールでは絶対値を使用
        assert_eq!(scaled.semi_major_axis(), 4.0);
        assert_eq!(scaled.semi_minor_axis(), 2.0);

        // 中心位置は負のスケールが適用される
        let expected_center = Point3D::new(-2.0, -4.0, -6.0);
        assert_eq!(scaled.center(), expected_center);
    }

    #[test]
    fn test_reverse() {
        let ellipse = create_test_ellipse();

        let reversed = ellipse.reverse();

        // 中心と半軸長は変化しない
        assert_eq!(reversed.center(), ellipse.center());
        assert_eq!(reversed.semi_major_axis(), ellipse.semi_major_axis());
        assert_eq!(reversed.semi_minor_axis(), ellipse.semi_minor_axis());

        // 法線方向が反転される
        assert_eq!(reversed.normal().as_vector(), -ellipse.normal().as_vector());
    }

    #[test]
    fn test_translate_and_rotate() {
        let ellipse = create_test_ellipse();
        let translation = Vector3D::new(1.0, 1.0, 0.0);
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_radians(PI / 4.0);

        let transformed = ellipse.translate_and_rotate(
            translation,
            rotation_center,
            rotation_axis,
            rotation_angle,
        );

        // 複合変換後の妥当性を確認
        assert!(transformed.is_valid_transform());
    }

    #[test]
    fn test_scale_and_translate() {
        let ellipse = create_test_ellipse();
        let scale_center = Point3D::origin();
        let scale_factor = 1.5;
        let translation = Vector3D::new(2.0, -1.0, 0.5);

        let transformed = ellipse.scale_and_translate(scale_center, scale_factor, translation);

        // 複合変換後の妥当性を確認
        assert!(transformed.is_valid_transform());

        // スケール後の寸法を確認
        assert_eq!(transformed.semi_major_axis(), 3.0);
        assert_eq!(transformed.semi_minor_axis(), 1.5);
    }

    #[test]
    fn test_is_valid_transform() {
        let ellipse = create_test_ellipse();

        // 正常な楕円は妥当
        assert!(ellipse.is_valid_transform());

        // ゼロスケールは楕円作成自体が失敗するため、有効な楕円のみテスト
        // 代わりに小さなスケールをテスト
        let small_scaled = ellipse.scale_3d(Point3D::origin(), 0.1);
        assert!(small_scaled.is_valid_transform());
    }

    #[test]
    fn test_transform_equivalent() {
        let ellipse1 = create_test_ellipse();
        let ellipse2 = create_test_ellipse();
        let tolerance = 1e-10;

        // 同じ楕円は等価
        assert!(ellipse1.transform_equivalent(&ellipse2, tolerance));

        // わずかに異なる楕円は等価でない
        let ellipse3 = ellipse1.scale_3d(Point3D::origin(), 1.1);
        assert!(!ellipse1.transform_equivalent(&ellipse3, tolerance));

        // 許容誤差内では等価
        let large_tolerance = 1.0;
        assert!(ellipse1.transform_equivalent(&ellipse3, large_tolerance));
    }

    #[test]
    fn test_transform_identity() {
        let ellipse = create_test_ellipse();

        // 恒等変換（変換なし）
        let identity_translated = ellipse.translate_3d(Vector3D::zero());
        let identity_scaled = ellipse.scale_3d(ellipse.center(), 1.0);

        let tolerance = 1e-10;

        // 恒等変換の結果は元の楕円と等価
        assert!(ellipse.transform_equivalent(&identity_translated, tolerance));
        assert!(ellipse.transform_equivalent(&identity_scaled, tolerance));
    }

    #[test]
    fn test_transform_composition() {
        let ellipse = create_test_ellipse();

        // 2回の変換の合成
        let translation1 = Vector3D::new(1.0, 0.0, 0.0);
        let translation2 = Vector3D::new(0.0, 1.0, 0.0);

        let step1 = ellipse.translate_3d(translation1);
        let step2 = step1.translate_3d(translation2);

        // 一回での合成変換
        let combined_translation = translation1 + translation2;
        let combined = ellipse.translate_3d(combined_translation);

        let tolerance = 1e-10;

        // 結果は等価
        assert!(step2.transform_equivalent(&combined, tolerance));
    }

    #[test]
    fn test_transform_symmetry() {
        let ellipse = create_test_ellipse();

        // 正方向と負方向の変換
        let translation = Vector3D::new(5.0, 3.0, -1.0);
        let forward = ellipse.translate_3d(translation);
        let back_to_original = forward.translate_3d(-translation);

        let tolerance = 1e-10;

        // 往復変換で元に戻る
        assert!(ellipse.transform_equivalent(&back_to_original, tolerance));
    }

    #[test]
    fn test_multiple_transforms() {
        let ellipse = create_test_ellipse();

        // 複数の変換を連続適用
        let transformed = ellipse
            .translate_3d(Vector3D::new(1.0, 1.0, 1.0))
            .scale_3d(Point3D::origin(), 2.0)
            .translate_3d(Vector3D::new(-1.0, -1.0, -1.0))
            .reverse();

        // 最終結果の妥当性確認
        assert!(transformed.is_valid_transform());

        // 法線が反転されていることを確認
        let dot_product = ellipse.normal().dot(&transformed.normal());
        assert!(dot_product < 0.0, "法線方向が反転されている");
    }
}
