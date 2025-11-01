//! CylindricalSurface3D Transform操作のテスト

#[cfg(test)]
mod tests {
    use crate::{CylindricalSurface3D, Point3D, Vector3D};
    use approx::assert_relative_eq;
    use geo_foundation::Angle;

    fn create_test_surface() -> CylindricalSurface3D<f64> {
        CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap()
    }

    // ========================================================================
    // Basic Transform Tests
    // ========================================================================

    #[test]
    fn test_translate() {
        let surface = create_test_surface();
        let translation = Vector3D::new(10.0, 20.0, 30.0);
        let translated = surface.translate(translation); // 借用を除去

        assert_eq!(translated.center(), Point3D::new(11.0, 22.0, 33.0));
        assert_eq!(translated.radius(), surface.radius());
        assert_eq!(translated.axis(), surface.axis());
        assert_eq!(translated.ref_direction(), surface.ref_direction());
    }

    #[test]
    fn test_scale_uniform() {
        let surface = create_test_surface();
        let scale_center = Point3D::origin();
        let factor = 2.0;
        let scaled = surface.scale_uniform(scale_center, factor); // 引数順序修正とunwrap()除去

        // 中心点がスケールされる
        assert_eq!(scaled.center(), Point3D::new(2.0, 4.0, 6.0));
        // 半径がスケールされる
        assert_relative_eq!(scaled.radius(), 10.0, epsilon = 1e-10);
        // 軸方向は変わらない（方向ベクトル）
        assert_eq!(scaled.axis(), surface.axis());
        assert_eq!(scaled.ref_direction(), surface.ref_direction());
    }

    #[test]
    fn test_scale_special_cases() {
        let surface = create_test_surface();
        let scale_center = Point3D::origin();

        // 負のスケール - 実際に計算される（反転スケール）
        let negative_scaled = surface.scale_uniform(scale_center, -1.0);
        // 中心が反転される
        assert_eq!(negative_scaled.center(), Point3D::new(-1.0, -2.0, -3.0));

        // 1.0スケール - 変化なし
        let same_scaled = surface.scale_uniform(scale_center, 1.0);
        assert_eq!(same_scaled.center(), surface.center());
    }

    #[test]
    fn test_non_uniform_scale() {
        let surface = create_test_surface();
        let scale_center = Point3D::origin();

        // 一様スケール
        let uniform_scaled = surface.scale_non_uniform(scale_center, 2.0, 2.0, 2.0);
        // 中心がスケールされる
        assert_eq!(uniform_scaled.center(), Point3D::new(2.0, 4.0, 6.0));

        // 非一様スケール
        let non_uniform = surface.scale_non_uniform(scale_center, 2.0, 3.0, 1.0);
        // 非一様スケールでも計算される（楕円柱的な変換）
        assert_eq!(non_uniform.center(), Point3D::new(2.0, 6.0, 3.0));
    }

    #[test]
    fn test_rotate_z() {
        let surface = create_test_surface();
        let angle = std::f64::consts::PI / 2.0; // 90度
        let rotated = surface.rotate_z(angle.into()); // Angle型に変換

        // 90度回転で (x, y) が (-y, x) になる
        assert_relative_eq!(rotated.center().x(), -2.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().z(), 3.0, epsilon = 1e-10);
        assert_eq!(rotated.radius(), surface.radius());

        // 軸がZ軸なので変化なし
        assert_relative_eq!(rotated.axis().z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rotate_around_x_axis() {
        let surface = create_test_surface();
        let angle = Angle::from_radians(std::f64::consts::PI / 2.0); // 90度
        let x_axis = Vector3D::unit_x();
        let rotation_center = Point3D::origin();

        let rotated = surface
            .rotate_around_axis(x_axis, angle, rotation_center)
            .unwrap();

        // X軸周りの90度回転で (y, z) が (-z, y) になる
        assert_relative_eq!(rotated.center().x(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), -3.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().z(), 2.0, epsilon = 1e-10);
        assert_eq!(rotated.radius(), surface.radius());
    }

    #[test]
    fn test_rotate_around_y_axis() {
        let surface = create_test_surface();
        let angle = Angle::from_radians(std::f64::consts::PI / 2.0); // 90度
        let y_axis = Vector3D::unit_y();
        let rotation_center = Point3D::origin();

        let rotated = surface
            .rotate_around_axis(y_axis, angle, rotation_center)
            .unwrap();

        // Y軸周りの90度回転で (x, z) が (z, -x) になる
        assert_relative_eq!(rotated.center().x(), 3.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 2.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().z(), -1.0, epsilon = 1e-10);
        assert_eq!(rotated.radius(), surface.radius());
    }

    #[test]
    fn test_rotate_around_axis() {
        let surface = create_test_surface();
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(1.0, 1.0, 0.0).normalize(); // 45度軸
        let angle = Angle::from_radians(std::f64::consts::PI / 4.0); // 45度

        let rotated = surface.rotate_around_axis(rotation_axis, angle, rotation_center);
        assert!(rotated.is_ok()); // Result型なのでis_ok()を使用

        let rotated = rotated.unwrap();
        assert_eq!(rotated.radius(), surface.radius());
        // 回転により中心が移動する
        assert_ne!(rotated.center(), surface.center());
    }

    #[test]
    fn test_rotate_around_zero_axis() {
        let surface = create_test_surface();
        let rotation_center = Point3D::origin();
        let zero_axis = Vector3D::zero();
        let angle = Angle::from_radians(std::f64::consts::PI / 4.0);

        let result = surface.rotate_around_axis(zero_axis, angle, rotation_center);
        assert!(result.is_err()); // ゼロ軸は無効 - Result型なのでis_err()を使用
    }

    // ========================================================================
    // BasicTransform Trait Tests
    // ========================================================================

    #[test]
    fn test_basic_transform_operations() {
        let surface = create_test_surface();

        // 基本的な変換操作のテスト
        let translation = Vector3D::new(1.0, 1.0, 1.0);
        let translated = surface.translate(translation);
        assert_eq!(translated.center(), Point3D::new(2.0, 3.0, 4.0));

        let center = Point3D::origin();
        let angle = Angle::from_radians(std::f64::consts::PI / 2.0);
        let rotated = surface.rotate_z(angle); // rotate_zを直接使用

        // Z軸周り90度回転
        assert_relative_eq!(rotated.center().x(), -2.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 1.0, epsilon = 1e-10);

        let scaled = surface.scale_uniform(center, 2.0); // scale_uniformを使用
        assert_relative_eq!(scaled.radius(), 10.0, epsilon = 1e-10);
    }

    // ========================================================================
    // Coordinate System Preservation Tests
    // ========================================================================

    #[test]
    fn test_coordinate_system_preservation() {
        let surface = create_test_surface();
        let translation = Vector3D::new(1.0, 2.0, 3.0);
        let center = Point3D::origin();

        // 平行移動後も座標系が保持される
        let translated = surface.translate(translation); // 借用除去
        assert_eq!(translated.axis(), surface.axis());
        assert_eq!(translated.ref_direction(), surface.ref_direction());

        // Y軸も正しく計算される
        let y_axis_original = surface.y_axis();
        let y_axis_translated = translated.y_axis();
        assert_eq!(y_axis_original, y_axis_translated);

        // スケール変換後も方向は保持される
        let scaled = surface.scale_uniform(center, 2.0); // 引数順序修正とunwrap()除去
        assert_eq!(scaled.axis(), surface.axis());
        assert_eq!(scaled.ref_direction(), surface.ref_direction());
    }

    #[test]
    fn test_step_compliance() {
        // STEP準拠の変換テスト
        let surface = CylindricalSurface3D::new(
            Point3D::new(10.0, 20.0, 30.0),
            Vector3D::new(0.0, 1.0, 0.0), // Y軸
            Vector3D::new(1.0, 0.0, 0.0), // X軸参照
            15.0,
        )
        .unwrap();

        // 変換後もSTEP形式の座標系が保持される
        let translated = surface.translate(Vector3D::new(5.0, 5.0, 5.0)); // 借用除去

        // AXIS2_PLACEMENT_3Dの整合性
        let axis = translated.axis().as_vector();
        let ref_dir = translated.ref_direction().as_vector();
        let y_axis = translated.y_axis().as_vector();

        // 直交性の検証
        assert_relative_eq!(axis.dot(&ref_dir), 0.0, epsilon = 1e-10);
        assert_relative_eq!(axis.dot(&y_axis), 0.0, epsilon = 1e-10);
        assert_relative_eq!(ref_dir.dot(&y_axis), 0.0, epsilon = 1e-10);

        // 正規化の検証
        assert_relative_eq!(axis.length(), 1.0, epsilon = 1e-10); // norm()をlength()に変更
        assert_relative_eq!(ref_dir.length(), 1.0, epsilon = 1e-10); // norm()をlength()に変更
        assert_relative_eq!(y_axis.length(), 1.0, epsilon = 1e-10); // norm()をlength()に変更

        // 右手系の検証
        let cross_product = axis.cross(&ref_dir);
        assert_relative_eq!(cross_product.x(), y_axis.x(), epsilon = 1e-10);
        assert_relative_eq!(cross_product.y(), y_axis.y(), epsilon = 1e-10);
        assert_relative_eq!(cross_product.z(), y_axis.z(), epsilon = 1e-10);
    }

    // ========================================================================
    // Surface Property Preservation Tests
    // ========================================================================

    #[test]
    fn test_surface_property_preservation() {
        let surface = create_test_surface();

        // 変換前のサーフェス特性
        let original_point = surface.point_at_uv(0.0, 0.0);
        let original_normal = surface.normal_at_uv(0.0, 0.0);
        let (original_k1, original_k2) = surface.curvature_at_uv(0.0, 0.0);

        // 平行移動
        let translated = surface.translate(Vector3D::new(10.0, 20.0, 30.0)); // 借用除去
        let translated_point = translated.point_at_uv(0.0, 0.0);
        let translated_normal = translated.normal_at_uv(0.0, 0.0);
        let (translated_k1, translated_k2) = translated.curvature_at_uv(0.0, 0.0);

        // 平行移動では法線と曲率は変わらない
        assert_relative_eq!(original_normal.x(), translated_normal.x(), epsilon = 1e-10);
        assert_relative_eq!(original_normal.y(), translated_normal.y(), epsilon = 1e-10);
        assert_relative_eq!(original_normal.z(), translated_normal.z(), epsilon = 1e-10);
        assert_relative_eq!(original_k1, translated_k1, epsilon = 1e-10);
        assert_relative_eq!(original_k2, translated_k2, epsilon = 1e-10);

        // 点は移動する
        assert_ne!(original_point, translated_point);

        // 一様スケール
        let scaled = surface.scale_uniform(Point3D::origin(), 2.0); // 引数順序修正とunwrap()除去
        let (scaled_k1, scaled_k2) = scaled.curvature_at_uv(0.0, 0.0);

        // スケールにより曲率は逆数倍される
        assert_relative_eq!(scaled_k1, original_k1 / 2.0, epsilon = 1e-10);
        assert_relative_eq!(scaled_k2, original_k2, epsilon = 1e-10); // 軸方向曲率は0なので変わらず
    }

    #[test]
    fn test_parametric_preservation() {
        let surface = create_test_surface();
        let u = std::f64::consts::PI / 3.0;
        let v = 7.5;

        // 変換前の点
        let original_point = surface.point_at_uv(u, v);

        // 平行移動
        let translation = Vector3D::new(1.0, 2.0, 3.0);
        let translated = surface.translate(translation); // 借用除去
        let translated_point = translated.point_at_uv(u, v);

        // 同じUVパラメータで、期待される移動を確認
        let expected_point = Point3D::new(
            original_point.x() + translation.x(),
            original_point.y() + translation.y(),
            original_point.z() + translation.z(),
        );

        assert_relative_eq!(translated_point.x(), expected_point.x(), epsilon = 1e-10);
        assert_relative_eq!(translated_point.y(), expected_point.y(), epsilon = 1e-10);
        assert_relative_eq!(translated_point.z(), expected_point.z(), epsilon = 1e-10);
    }
}
