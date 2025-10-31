//! CylindricalSolid3D のテスト（STEP準拠）

#[cfg(test)]
mod tests {
    use crate::{CylindricalSolid3D, Direction3D, Point3D, Vector3D};

    #[test]
    fn test_cylindrical_solid_creation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        assert_eq!(cylindrical_solid.center(), center);
        assert_eq!(cylindrical_solid.radius(), radius);
        assert_eq!(cylindrical_solid.height(), height);
        // 軸は Direction3D として正規化されているはず
        assert_eq!(cylindrical_solid.axis().x(), 0.0);
        assert_eq!(cylindrical_solid.axis().y(), 0.0);
        assert_eq!(cylindrical_solid.axis().z(), 1.0);
    }

    #[test]
    fn test_cylindrical_solid_invalid_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);

        // 負の半径
        assert!(CylindricalSolid3D::new(center, axis, ref_direction, -1.0, 5.0).is_none());

        // ゼロの半径
        assert!(CylindricalSolid3D::new(center, axis, ref_direction, 0.0, 5.0).is_none());

        // 負の高さ
        assert!(CylindricalSolid3D::new(center, axis, ref_direction, 5.0, -1.0).is_none());

        // ゼロの高さ
        assert!(CylindricalSolid3D::new(center, axis, ref_direction, 5.0, 0.0).is_none());

        // ゼロベクトルの軸
        assert!(CylindricalSolid3D::new(
            center,
            Vector3D::new(0.0, 0.0, 0.0),
            ref_direction,
            5.0,
            10.0
        )
        .is_none());

        // ゼロベクトルのref_direction
        assert!(
            CylindricalSolid3D::new(center, axis, Vector3D::new(0.0, 0.0, 0.0), 5.0, 10.0)
                .is_none()
        );
    }

    #[test]
    fn test_cylindrical_solid_axis_constructors() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 3.0;
        let height = 6.0;

        // Z軸円柱ソリッド（デフォルト方向）
        let cylindrical_solid_z = CylindricalSolid3D::new_z_axis(center, radius, height).unwrap();
        assert_eq!(
            cylindrical_solid_z.axis(),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap()
        );

        // Y軸円柱ソリッド
        let cylindrical_solid_y = CylindricalSolid3D::new_y_axis(center, radius, height).unwrap();
        assert_eq!(
            cylindrical_solid_y.axis(),
            Direction3D::from_vector(Vector3D::new(0.0, 1.0, 0.0)).unwrap()
        );

        // X軸円柱ソリッド
        let cylindrical_solid_x = CylindricalSolid3D::new_x_axis(center, radius, height).unwrap();
        assert_eq!(
            cylindrical_solid_x.axis(),
            Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap()
        );
    }

    #[test]
    fn test_step_compliance() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        // STEP AXIS2_PLACEMENT_3D準拠の座標系確認
        assert_eq!(cylindrical_solid.ref_direction().x(), 1.0);
        assert_eq!(cylindrical_solid.ref_direction().y(), 0.0);
        assert_eq!(cylindrical_solid.ref_direction().z(), 0.0);

        // Y軸は計算されるべき
        let y_axis = cylindrical_solid.y_axis();
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);
        assert_eq!(y_axis.z(), 0.0);
    }

    #[test]
    fn test_volume_calculation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        let expected_volume = std::f64::consts::PI * radius * radius * height;
        let actual_volume = cylindrical_solid.volume();
        assert!((actual_volume - expected_volume).abs() < 1e-10_f64);
    }

    #[test]
    fn test_surface_area_calculation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        let expected_surface_area = 2.0 * std::f64::consts::PI * radius * (radius + height);
        assert!((cylindrical_solid.surface_area() - expected_surface_area).abs() < 1e-10);
    }

    #[test]
    fn test_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        // 円柱内部の点
        let inside_point = Point3D::new(2.0, 2.0, 3.0);
        assert!(cylindrical_solid.contains_point(inside_point));

        // 円柱外部の点
        let outside_point = Point3D::new(10.0, 0.0, 0.0);
        assert!(!cylindrical_solid.contains_point(outside_point));

        // 高さ範囲外の点
        let too_high_point = Point3D::new(0.0, 0.0, 15.0);
        assert!(!cylindrical_solid.contains_point(too_high_point));
    }

    #[test]
    fn test_cylindrical_solid_f32() {
        let center = Point3D::new(0.0f32, 0.0f32, 0.0f32);
        let axis = Vector3D::new(0.0f32, 0.0f32, 1.0f32);
        let ref_direction = Vector3D::new(1.0f32, 0.0f32, 0.0f32);
        let radius = 5.0f32;
        let height = 10.0f32;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        assert_eq!(cylindrical_solid.radius(), radius);
        assert_eq!(cylindrical_solid.height(), height);
    }

    #[test]
    fn test_ref_direction_orthogonalization() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        // ref_directionとaxisが平行でない場合のテスト
        let non_orthogonal_ref = Vector3D::new(1.0, 0.0, 0.1);

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, non_orthogonal_ref, 5.0, 10.0).unwrap();

        // ref_directionはaxisに対して直交化されているはず
        let ref_dir = cylindrical_solid.ref_direction();
        let axis_dir = cylindrical_solid.axis();
        let dot_product: f64 =
            ref_dir.x() * axis_dir.x() + ref_dir.y() * axis_dir.y() + ref_dir.z() * axis_dir.z();
        assert!(
            dot_product.abs() < 1e-10,
            "ref_direction should be orthogonal to axis"
        );
    }

    #[test]
    fn test_solid_specific_properties() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 2.0;
        let height = 5.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        // ソリッド特有のプロパティ
        let volume = cylindrical_solid.volume();
        let surface_area = cylindrical_solid.surface_area();
        let bbox = cylindrical_solid.bounding_box();

        // 体積が正の値
        assert!(volume > 0.0);

        // 表面積が体積よりも大きい（このケースでは）
        assert!(surface_area > volume);

        // バウンディングボックスが適切
        assert!(bbox.min().x() <= center.x());
        assert!(bbox.max().x() >= center.x());
    }

    #[test]
    fn test_distance_to_surface() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        // 内部の点（距離は0に近い）
        let internal_point = Point3D::new(0.0, 0.0, 5.0);
        let distance = cylindrical_solid.distance_to_surface(internal_point);
        assert!(distance < 1e-10);

        // 外部の点
        let external_point = Point3D::new(8.0, 0.0, 5.0);
        let expected_distance = 3.0_f64; // 8 - 5 = 3
        assert!(
            (cylindrical_solid.distance_to_surface(external_point) - expected_distance).abs()
                < 1e-10_f64
        );
    }
}
