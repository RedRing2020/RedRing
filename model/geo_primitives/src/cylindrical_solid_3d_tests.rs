//! CylindricalSolid3D のテスト（STEP準拠）
//! Core Traits経由でのAPI使用を推奨

#[cfg(test)]
mod tests {
    use crate::{CylindricalSolid3D, Direction3D, InfiniteLine3D, Point3D, Vector3D};
    use geo_contracts::{
        BasicIntersection, CylindricalSolid3DContainment, CylindricalSolid3DDerived,
        CylindricalSolid3DDistance, CylindricalSolid3DProperties,
    };

    #[test]
    fn test_cylindrical_solid_creation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylindrical_solid =
            CylindricalSolid3D::new(center, axis, ref_direction, radius, height).unwrap();

        // Core Traits経由でアクセス
        let center_tuple = CylindricalSolid3DProperties::center(&cylindrical_solid);
        assert_eq!(center_tuple, (1.0, 2.0, 3.0));
        assert_eq!(
            CylindricalSolid3DProperties::radius(&cylindrical_solid),
            radius
        );
        assert_eq!(
            CylindricalSolid3DProperties::height(&cylindrical_solid),
            height
        );
        // 軸は Direction3D として正規化されているはず
        let axis_tuple = CylindricalSolid3DProperties::axis(&cylindrical_solid);
        assert_eq!(axis_tuple.0, 0.0);
        assert_eq!(axis_tuple.1, 0.0);
        assert_eq!(axis_tuple.2, 1.0);
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
        let actual_volume = CylindricalSolid3DDerived::volume(&cylindrical_solid);
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
        let actual_area = CylindricalSolid3DDerived::surface_area(&cylindrical_solid);
        assert!((actual_area - expected_surface_area).abs() < 1e-10);
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
        let inside_point = (2.0, 2.0, 3.0);
        assert!(CylindricalSolid3DContainment::contains_point(
            &cylindrical_solid,
            inside_point
        ));

        // 円柱外部の点
        let outside_point = (10.0, 0.0, 0.0);
        assert!(!CylindricalSolid3DContainment::contains_point(
            &cylindrical_solid,
            outside_point
        ));

        // 高さ範囲外の点
        let too_high_point = (0.0, 0.0, 15.0);
        assert!(!CylindricalSolid3DContainment::contains_point(
            &cylindrical_solid,
            too_high_point
        ));
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

        assert_eq!(
            CylindricalSolid3DProperties::radius(&cylindrical_solid),
            radius
        );
        assert_eq!(
            CylindricalSolid3DProperties::height(&cylindrical_solid),
            height
        );
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
        let volume = CylindricalSolid3DDerived::volume(&cylindrical_solid);
        let surface_area = CylindricalSolid3DDerived::surface_area(&cylindrical_solid);
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
        let internal_point = (0.0, 0.0, 5.0);
        let distance =
            CylindricalSolid3DDistance::distance_to_point(&cylindrical_solid, internal_point);
        assert!(distance < 1e-10);

        // 外部の点
        let external_point = (8.0, 0.0, 5.0);
        let expected_distance = 3.0_f64; // 8 - 5 = 3
        let actual_distance =
            CylindricalSolid3DDistance::distance_to_point(&cylindrical_solid, external_point);
        assert!((actual_distance - expected_distance).abs() < 1e-10_f64);
    }

    #[test]
    fn test_cylindrical_solid_line_intersection_is_exposed_via_basic_intersection() {
        let cylindrical_solid = CylindricalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            5.0,
            10.0,
        )
        .unwrap();
        let line = InfiniteLine3D::new(Point3D::new(-10.0, 0.0, 5.0), Vector3D::unit_x()).unwrap();

        let intersection = BasicIntersection::intersection_with(&cylindrical_solid, &line, 1e-10);

        assert!(intersection.is_none());
    }
}
