//! CylindricalSolid3D Transform操作のテスト

#[cfg(test)]
mod tests {
    use crate::{CylindricalSolid3D, Point3D, Vector3D};
    use approx::assert_relative_eq;
    use geo_foundation::Angle;

    fn create_test_cylindrical_solid() -> CylindricalSolid3D<f64> {
        CylindricalSolid3D::new(
            Point3D::new(1.0, 2.0, 3.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
            4.0,
        )
        .unwrap()
    }

    #[test]
    fn test_translate() {
        let cylindrical_solid = create_test_cylindrical_solid();
        let translation = Vector3D::new(5.0, -3.0, 2.0);

        let translated = cylindrical_solid.translate(&translation);

        // 中心が移動
        assert_eq!(translated.center(), Point3D::new(6.0, -1.0, 5.0));

        // 他のプロパティは不変
        assert_eq!(translated.radius(), cylindrical_solid.radius());
        assert_eq!(translated.height(), cylindrical_solid.height());
        assert_eq!(translated.axis(), cylindrical_solid.axis());
        assert_eq!(
            translated.ref_direction(),
            cylindrical_solid.ref_direction()
        );
    }

    #[test]
    fn test_scale_uniform() {
        let cylindrical_solid = create_test_cylindrical_solid();
        let scale_factor = 2.5;

        let scaled = cylindrical_solid.scale_uniform(scale_factor).unwrap();

        // 半径と高さがスケール
        assert_relative_eq!(
            scaled.radius(),
            cylindrical_solid.radius() * scale_factor,
            epsilon = 1e-10
        );
        assert_relative_eq!(
            scaled.height(),
            cylindrical_solid.height() * scale_factor,
            epsilon = 1e-10
        );

        // 中心、軸、参照方向は不変
        assert_eq!(scaled.center(), cylindrical_solid.center());
        assert_eq!(scaled.axis(), cylindrical_solid.axis());
        assert_eq!(scaled.ref_direction(), cylindrical_solid.ref_direction());

        // 無効なスケール係数
        assert!(cylindrical_solid.scale_uniform(0.0).is_none());
        assert!(cylindrical_solid.scale_uniform(-1.0).is_none());
    }

    #[test]
    fn test_rotate_z() {
        let cylindrical_solid = CylindricalSolid3D::new(
            Point3D::new(1.0, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
            Vector3D::new(0.0, 1.0, 0.0),
            2.0,
            4.0,
        )
        .unwrap();

        let angle = Angle::from_degrees(90.0);
        let rotated = cylindrical_solid.rotate_z(&angle);

        // 90度回転後の確認
        assert_relative_eq!(rotated.center().x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 1.0, epsilon = 1e-10);
        assert_eq!(rotated.center().z(), 0.0);

        // 軸も回転
        assert_relative_eq!(rotated.axis().x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.axis().y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.axis().z(), 0.0, epsilon = 1e-10);

        // 半径と高さは不変
        assert_eq!(rotated.radius(), cylindrical_solid.radius());
        assert_eq!(rotated.height(), cylindrical_solid.height());
    }

    #[test]
    fn test_basic_transform_trait() {
        let cylindrical_solid = create_test_cylindrical_solid();

        // BasicTransformトレイト経由での操作
        let translation = Vector3D::new(1.0_f64, 1.0_f64, 1.0_f64);
        let translated = cylindrical_solid.translate(&translation);
        assert_eq!(translated.center(), Point3D::new(2.0, 3.0, 4.0));

        let scaled = cylindrical_solid.scale_uniform(2.0).unwrap();
        assert_eq!(scaled.radius(), 4.0);
        assert_eq!(scaled.height(), 8.0);

        let rotation_center = Point3D::new(0.0, 0.0, 0.0);
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);
        let rotated = cylindrical_solid
            .rotate_around_axis(&rotation_center, &rotation_axis, &angle)
            .unwrap();

        assert_relative_eq!(rotated.center().x(), -2.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_coordinate_system_preservation() {
        let cylindrical_solid = create_test_cylindrical_solid();
        let angle = Angle::from_degrees(45.0);

        let rotated = cylindrical_solid.rotate_z(&angle);

        // 軸と参照方向の直交性確認
        let axis = rotated.axis().as_vector();
        let ref_dir = rotated.ref_direction().as_vector();
        let dot_product = axis.dot(&ref_dir);
        assert!(
            dot_product.abs() < 1e-10,
            "Axis and ref_direction should remain orthogonal"
        );

        // Y軸の計算確認
        let y_axis = rotated.y_axis().as_vector();
        let expected_y = axis.cross(&ref_dir);
        assert_relative_eq!(y_axis.x(), expected_y.x(), epsilon = 1e-10);
        assert_relative_eq!(y_axis.y(), expected_y.y(), epsilon = 1e-10);
        assert_relative_eq!(y_axis.z(), expected_y.z(), epsilon = 1e-10);
    }

    #[test]
    fn test_step_compliance() {
        let cylindrical_solid = create_test_cylindrical_solid();
        let translation = Vector3D::new(10.0, 20.0, 30.0);

        let transformed = cylindrical_solid.translate(&translation);

        // STEP AXIS2_PLACEMENT_3D の整合性確認
        assert_eq!(transformed.axis(), cylindrical_solid.axis());
        assert_eq!(
            transformed.ref_direction(),
            cylindrical_solid.ref_direction()
        );

        // Y軸の一貫性
        let original_y = cylindrical_solid.y_axis();
        let transformed_y = transformed.y_axis();
        assert_eq!(original_y, transformed_y);
    }

    #[test]
    fn test_non_uniform_scale() {
        let cylindrical_solid = create_test_cylindrical_solid();

        let scaled = cylindrical_solid.scale_non_uniform(3.0, 1.5).unwrap();

        assert_eq!(scaled.radius(), 6.0);
        assert_eq!(scaled.height(), 6.0);
        assert_eq!(scaled.center(), cylindrical_solid.center());

        // 無効な係数
        assert!(cylindrical_solid.scale_non_uniform(0.0, 1.0).is_none());
        assert!(cylindrical_solid.scale_non_uniform(1.0, -1.0).is_none());
    }

    #[test]
    fn test_solid_property_preservation() {
        let cylindrical_solid = create_test_cylindrical_solid();
        let original_volume = cylindrical_solid.volume();

        // 平行移動では体積不変
        let translated = cylindrical_solid.translate(&Vector3D::new(5.0, 5.0, 5.0));
        assert_relative_eq!(translated.volume(), original_volume, epsilon = 1e-10);

        // 均等スケールでは体積は3乗比例
        let scale_factor = 2.0;
        let scaled = cylindrical_solid.scale_uniform(scale_factor).unwrap();
        let expected_volume = original_volume * scale_factor * scale_factor * scale_factor;
        assert_relative_eq!(scaled.volume(), expected_volume, epsilon = 1e-10);

        // 回転では体積不変
        let angle = Angle::from_degrees(45.0);
        let rotated = cylindrical_solid.rotate_z(&angle);
        assert_relative_eq!(rotated.volume(), original_volume, epsilon = 1e-10);
    }
}
