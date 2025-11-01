//! Plane3DCoordinateSystem Transform Tests
//!
//! Transform操作の単体テスト
//!
//! **作成日: 2025年10月29日**

#[cfg(test)]
mod tests {
    use super::super::{Plane3DCoordinateSystem, Point3D, Vector3D};
    use approx::assert_relative_eq;
    use geo_foundation::{Angle, Scalar};

    fn create_test_coordinate_system<T: Scalar>() -> Plane3DCoordinateSystem<T> {
        let origin = Point3D::new(T::ZERO, T::ZERO, T::ZERO);
        let normal = Vector3D::new(T::ZERO, T::ZERO, T::ONE); // Z軸
        let u_direction = Vector3D::new(T::ONE, T::ZERO, T::ZERO); // X軸

        Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap()
    }

    #[test]
    fn test_translate() {
        let coord_system = create_test_coordinate_system::<f64>();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let translated = coord_system.translate(&translation);

        // origin が移動している
        assert_relative_eq!(translated.origin().x(), 1.0);
        assert_relative_eq!(translated.origin().y(), 2.0);
        assert_relative_eq!(translated.origin().z(), 3.0);

        // 座標軸は変化していない
        assert_relative_eq!(translated.normal().x(), coord_system.normal().x());
        assert_relative_eq!(translated.normal().y(), coord_system.normal().y());
        assert_relative_eq!(translated.normal().z(), coord_system.normal().z());
    }

    #[test]
    fn test_scale_original() {
        let coord_system = create_test_coordinate_system::<f64>();
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scale_factor = 2.0;

        let scaled = coord_system
            .scale_uniform(scale_factor, scale_center)
            .unwrap();

        // スケール中心が原点なので、origin は変化なし
        assert_relative_eq!(scaled.origin().x(), 0.0);
        assert_relative_eq!(scaled.origin().y(), 0.0);
        assert_relative_eq!(scaled.origin().z(), 0.0);

        // 座標軸の方向は変化なし（単位ベクトルのまま）
        assert_relative_eq!(scaled.normal().as_vector().magnitude(), 1.0);
        assert_relative_eq!(scaled.u_axis().as_vector().magnitude(), 1.0);
        assert_relative_eq!(scaled.v_axis().as_vector().magnitude(), 1.0);
    }

    #[test]
    fn test_rotate_z() {
        let coord_system = create_test_coordinate_system::<f64>();
        let rotation_center = Point3D::new(0.0, 0.0, 0.0);
        let angle = std::f64::consts::PI / 2.0; // 90度

        let rotated = coord_system.rotate_z(rotation_center, angle).unwrap();

        // Z軸周りの90度回転：X軸 → Y軸、Y軸 → -X軸
        assert_relative_eq!(rotated.u_axis().x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.u_axis().y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.u_axis().z(), 0.0, epsilon = 1e-10);

        assert_relative_eq!(rotated.v_axis().x(), -1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.v_axis().y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.v_axis().z(), 0.0, epsilon = 1e-10);

        // Z軸（法線）は変化なし
        assert_relative_eq!(rotated.normal().z(), 1.0);
    }

    #[test]
    fn test_basic_transform_trait() {
        use geo_foundation::extensions::BasicTransform;

        let coord_system = create_test_coordinate_system::<f64>();
        let translation = Vector3D::new(1.0, 0.0, 0.0);
        let center = Point3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(std::f64::consts::PI / 4.0); // 45度
        let scale_factor = 1.5;

        // BasicTransform トレイト経由でテスト
        let translated = coord_system.translate(&translation);
        let rotated = coord_system.rotate(center, angle);
        let scaled = coord_system.scale(center, scale_factor); // BasicTransform::scale

        // 変換が適用されている（詳細な検証は個別テストで行う）
        assert!(translated.origin().x() > 0.0); // 平行移動の影響
        assert!(rotated.u_axis().x() != 1.0); // 回転の影響
        assert!(scaled.origin().x() == 0.0); // スケール中心が原点なので座標は変化なし
    }

    #[test]
    fn test_custom_scale_method() {
        let coord_system = create_test_coordinate_system::<f64>();
        let scale_center = Point3D::new(1.0, 1.0, 1.0);
        let scale_factor = 2.0;

        // 独自の scale_uniform メソッドをテスト
        let scaled = coord_system
            .scale_uniform(scale_factor, scale_center)
            .unwrap();

        // origin がスケール中心を基準にスケーリングされている
        assert_relative_eq!(scaled.origin().x(), -1.0); // (0-1)*2 + 1 = -1
        assert_relative_eq!(scaled.origin().y(), -1.0);
        assert_relative_eq!(scaled.origin().z(), -1.0);
    }

    #[test]
    fn test_mirror() {
        let coord_system = create_test_coordinate_system::<f64>();
        let mirror_origin = Point3D::new(0.0, 0.0, 0.0);
        let mirror_normal = Vector3D::new(1.0, 0.0, 0.0); // YZ平面

        let mirrored = coord_system.mirror(mirror_origin, mirror_normal).unwrap();

        // X成分が反転している
        assert_relative_eq!(mirrored.u_axis().x(), -1.0, epsilon = 1e-10);
        assert_relative_eq!(mirrored.u_axis().y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(mirrored.u_axis().z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_coordinate_system_preservation() {
        let coord_system = create_test_coordinate_system::<f64>();
        let translation = Vector3D::new(1.0, 2.0, 3.0);
        let center = Point3D::new(0.0, 0.0, 0.0);
        let angle = std::f64::consts::PI / 3.0; // 60度

        let transformed = coord_system
            .translate(&translation)
            .rotate_z(center, angle)
            .unwrap()
            .scale_uniform(2.0, center)
            .unwrap();

        // 変換後も直交性が保たれている
        let dot_nu = transformed
            .normal()
            .as_vector()
            .dot(&transformed.u_axis().as_vector());
        let dot_nv = transformed
            .normal()
            .as_vector()
            .dot(&transformed.v_axis().as_vector());
        let dot_uv = transformed
            .u_axis()
            .as_vector()
            .dot(&transformed.v_axis().as_vector());

        assert_relative_eq!(dot_nu, 0.0, epsilon = 1e-10);
        assert_relative_eq!(dot_nv, 0.0, epsilon = 1e-10);
        assert_relative_eq!(dot_uv, 0.0, epsilon = 1e-10);

        // 全ての軸が単位ベクトル
        assert_relative_eq!(
            transformed.normal().as_vector().magnitude(),
            1.0,
            epsilon = 1e-10
        );
        assert_relative_eq!(
            transformed.u_axis().as_vector().magnitude(),
            1.0,
            epsilon = 1e-10
        );
        assert_relative_eq!(
            transformed.v_axis().as_vector().magnitude(),
            1.0,
            epsilon = 1e-10
        );
    }
}
