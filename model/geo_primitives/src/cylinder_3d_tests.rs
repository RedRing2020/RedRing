//! Cylinder3D のテスト（STEP準拠）

#[cfg(test)]
mod tests {
    use crate::{Cylinder3D, Direction3D, Point3D, Vector3D};

    #[test]
    fn test_cylinder_creation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylinder = Cylinder3D::new(center, axis, ref_direction, radius, height).unwrap();

        assert_eq!(cylinder.center(), center);
        assert_eq!(cylinder.radius(), radius);
        assert_eq!(cylinder.height(), height);
        // 軸は Direction3D として正規化されているはず
        assert_eq!(cylinder.axis().x(), 0.0);
        assert_eq!(cylinder.axis().y(), 0.0);
        assert_eq!(cylinder.axis().z(), 1.0);
    }

    #[test]
    fn test_cylinder_invalid_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);

        // 負の半径
        assert!(Cylinder3D::new(center, axis, ref_direction, -1.0, 5.0).is_none());

        // ゼロの半径
        assert!(Cylinder3D::new(center, axis, ref_direction, 0.0, 5.0).is_none());

        // 負の高さ
        assert!(Cylinder3D::new(center, axis, ref_direction, 5.0, -1.0).is_none());

        // ゼロの高さ
        assert!(Cylinder3D::new(center, axis, ref_direction, 5.0, 0.0).is_none());

        // ゼロベクトルの軸
        assert!(Cylinder3D::new(
            center,
            Vector3D::new(0.0, 0.0, 0.0),
            ref_direction,
            5.0,
            10.0
        )
        .is_none());

        // ゼロベクトルのref_direction
        assert!(Cylinder3D::new(center, axis, Vector3D::new(0.0, 0.0, 0.0), 5.0, 10.0).is_none());
    }

    #[test]
    fn test_cylinder_axis_constructors() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 3.0;
        let height = 6.0;

        // Z軸円柱（デフォルト方向）
        let cylinder_z = Cylinder3D::new_z_axis(center, radius, height).unwrap();
        assert_eq!(
            cylinder_z.axis(),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap()
        );

        // Y軸円柱
        let cylinder_y = Cylinder3D::new_y_axis(center, radius, height).unwrap();
        assert_eq!(
            cylinder_y.axis(),
            Direction3D::from_vector(Vector3D::new(0.0, 1.0, 0.0)).unwrap()
        );

        // X軸円柱
        let cylinder_x = Cylinder3D::new_x_axis(center, radius, height).unwrap();
        assert_eq!(
            cylinder_x.axis(),
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

        let cylinder = Cylinder3D::new(center, axis, ref_direction, radius, height).unwrap();

        // STEP AXIS2_PLACEMENT_3D準拠の座標系確認
        assert_eq!(cylinder.ref_direction().x(), 1.0);
        assert_eq!(cylinder.ref_direction().y(), 0.0);
        assert_eq!(cylinder.ref_direction().z(), 0.0);

        // Y軸は計算されるべき
        let y_axis = cylinder.y_axis();
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

        let cylinder = Cylinder3D::new(center, axis, ref_direction, radius, height).unwrap();

        let expected_volume = std::f64::consts::PI * radius * radius * height;
        assert!((cylinder.volume() - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_surface_area_calculation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylinder = Cylinder3D::new(center, axis, ref_direction, radius, height).unwrap();

        let expected_surface_area = 2.0 * std::f64::consts::PI * radius * (radius + height);
        assert!((cylinder.surface_area() - expected_surface_area).abs() < 1e-10);
    }

    #[test]
    fn test_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 5.0;
        let height = 10.0;

        let cylinder = Cylinder3D::new(center, axis, ref_direction, radius, height).unwrap();

        // 円柱内部の点
        let inside_point = Point3D::new(2.0, 2.0, 3.0);
        assert!(cylinder.contains_point(inside_point));

        // 円柱外部の点
        let outside_point = Point3D::new(10.0, 0.0, 0.0);
        assert!(!cylinder.contains_point(outside_point));

        // 高さ範囲外の点
        let too_high_point = Point3D::new(0.0, 0.0, 15.0);
        assert!(!cylinder.contains_point(too_high_point));
    }

    #[test]
    fn test_cylinder_f32() {
        let center = Point3D::new(0.0f32, 0.0f32, 0.0f32);
        let axis = Vector3D::new(0.0f32, 0.0f32, 1.0f32);
        let ref_direction = Vector3D::new(1.0f32, 0.0f32, 0.0f32);
        let radius = 5.0f32;
        let height = 10.0f32;

        let cylinder = Cylinder3D::new(center, axis, ref_direction, radius, height).unwrap();

        assert_eq!(cylinder.radius(), radius);
        assert_eq!(cylinder.height(), height);
    }

    #[test]
    fn test_ref_direction_orthogonalization() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        // ref_directionとaxisが平行でない場合のテスト
        let non_orthogonal_ref = Vector3D::new(1.0, 0.0, 0.1);

        let cylinder = Cylinder3D::new(center, axis, non_orthogonal_ref, 5.0, 10.0).unwrap();

        // ref_directionはaxisに対して直交化されているはず
        let ref_dir = cylinder.ref_direction();
        let axis_dir = cylinder.axis();
        let dot_product: f64 =
            ref_dir.x() * axis_dir.x() + ref_dir.y() * axis_dir.y() + ref_dir.z() * axis_dir.z();
        assert!(
            dot_product.abs() < 1e-10,
            "ref_direction should be orthogonal to axis"
        );
    }
}
