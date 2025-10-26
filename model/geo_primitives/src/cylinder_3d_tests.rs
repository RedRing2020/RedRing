//! Cylinder3D のテスト

#[cfg(test)]
mod tests {
    use crate::{Cylinder3D, Point3D, Vector3D};

    #[test]
    fn test_cylinder_creation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let radius = 5.0;
        let height = 10.0;

        let cylinder = Cylinder3D::new(center, axis, radius, height).unwrap();

        assert_eq!(cylinder.center(), center);
        assert_eq!(cylinder.radius(), radius);
        assert_eq!(cylinder.height(), height);
        // 軸は正規化されているはず
        assert!((cylinder.axis().magnitude() - 1.0_f64).abs() < 1e-10);
    }

    #[test]
    fn test_cylinder_invalid_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);

        // 負の半径
        assert!(Cylinder3D::new(center, axis, -1.0, 5.0).is_none());

        // ゼロの半径
        assert!(Cylinder3D::new(center, axis, 0.0, 5.0).is_none());

        // 負の高さ
        assert!(Cylinder3D::new(center, axis, 5.0, -1.0).is_none());

        // ゼロの高さ
        assert!(Cylinder3D::new(center, axis, 5.0, 0.0).is_none());

        // ゼロベクトルの軸
        assert!(Cylinder3D::new(center, Vector3D::new(0.0, 0.0, 0.0), 5.0, 10.0).is_none());
    }

    #[test]
    fn test_cylinder_axis_constructors() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 3.0;
        let height = 6.0;

        // Z軸円柱
        let cylinder_z = Cylinder3D::new_z_axis(center, radius, height).unwrap();
        assert_eq!(cylinder_z.axis(), Vector3D::new(0.0, 0.0, 1.0));

        // Y軸円柱
        let cylinder_y = Cylinder3D::new_y_axis(center, radius, height).unwrap();
        assert_eq!(cylinder_y.axis(), Vector3D::new(0.0, 1.0, 0.0));

        // X軸円柱
        let cylinder_x = Cylinder3D::new_x_axis(center, radius, height).unwrap();
        assert_eq!(cylinder_x.axis(), Vector3D::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_geometric_properties() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let radius = 2.0;
        let height = 5.0;

        let cylinder = Cylinder3D::new(center, axis, radius, height).unwrap();

        // 体積 = π × r² × h = π × 4 × 5 = 20π
        let expected_volume = std::f64::consts::PI * 4.0 * 5.0;
        assert!((cylinder.volume() - expected_volume).abs() < 1e-10);

        // 表面積 = 2π × r² + 2π × r × h = 2π × 4 + 2π × 2 × 5 = 8π + 20π = 28π
        let expected_surface_area = std::f64::consts::PI * (2.0 * 4.0 + 2.0 * 2.0 * 5.0);
        assert!((cylinder.surface_area() - expected_surface_area).abs() < 1e-10);
    }

    #[test]
    fn test_top_center() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let height = 10.0;

        let cylinder = Cylinder3D::new(center, axis, 5.0, height).unwrap();
        let top_center = cylinder.top_center();

        assert_eq!(top_center, Point3D::new(1.0, 2.0, 13.0));
    }

    #[test]
    fn test_point_containment() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let radius = 5.0;
        let height = 10.0;

        let cylinder = Cylinder3D::new(center, axis, radius, height).unwrap();

        // 内部の点
        assert!(cylinder.contains_point(Point3D::new(0.0, 0.0, 5.0))); // 中心軸上
        assert!(cylinder.contains_point(Point3D::new(3.0, 0.0, 5.0))); // 半径内
        assert!(cylinder.contains_point(Point3D::new(0.0, 4.0, 2.0))); // 半径内

        // 外部の点
        assert!(!cylinder.contains_point(Point3D::new(6.0, 0.0, 5.0))); // 半径外
        assert!(!cylinder.contains_point(Point3D::new(0.0, 0.0, -1.0))); // 高さ範囲外（下）
        assert!(!cylinder.contains_point(Point3D::new(0.0, 0.0, 11.0))); // 高さ範囲外（上）
        assert!(!cylinder.contains_point(Point3D::new(4.0, 4.0, 5.0))); // 半径外（対角）
    }

    #[test]
    fn test_bounding_box() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let radius = 3.0;
        let height = 6.0;

        let cylinder = Cylinder3D::new(center, axis, radius, height).unwrap();
        let bbox = cylinder.bounding_box();

        // Z軸に平行な円柱の場合
        assert_eq!(bbox.min().x(), -3.0);
        assert_eq!(bbox.max().x(), 3.0);
        assert_eq!(bbox.min().y(), -3.0);
        assert_eq!(bbox.max().y(), 3.0);
        assert_eq!(bbox.min().z(), 0.0);
        assert_eq!(bbox.max().z(), 6.0);
    }

    #[test]
    fn test_distance_to_surface() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let radius = 5.0;
        let height = 10.0;

        let cylinder = Cylinder3D::new(center, axis, radius, height).unwrap();

        // 内部の点（距離は0）
        let internal_point = Point3D::new(0.0, 0.0, 5.0);
        assert!(cylinder.distance_to_surface(internal_point) < 1e-10);

        // 側面に近い外部の点
        let side_point = Point3D::new(8.0, 0.0, 5.0);
        assert!((cylinder.distance_to_surface(side_point) - 3.0_f64).abs() < 1e-10);

        // 上面に近い外部の点
        let top_point = Point3D::new(0.0, 0.0, 15.0);
        assert!((cylinder.distance_to_surface(top_point) - 5.0_f64).abs() < 1e-10);
    }

    #[test]
    fn test_cylinder_display() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let cylinder = Cylinder3D::new(center, axis, 5.0, 10.0).unwrap();

        let display_str = format!("{}", cylinder);
        assert!(display_str.contains("Cylinder3D"));
        assert!(display_str.contains("center"));
        assert!(display_str.contains("axis"));
        assert!(display_str.contains("radius"));
        assert!(display_str.contains("height"));
    }

    #[test]
    fn test_angled_cylinder() {
        // 斜めの円柱をテスト
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(1.0, 1.0, 0.0); // 45度傾斜
        let radius = 2.0;
        let height = 4.0;

        let cylinder = Cylinder3D::new(center, axis, radius, height).unwrap();

        // 軸が正規化されていることを確認
        assert!((cylinder.axis().magnitude() - 1.0_f64).abs() < 1e-10);

        // 上面の中心が正しく計算されることを確認
        let expected_top = Point3D::new(
            4.0 / std::f64::consts::SQRT_2,
            4.0 / std::f64::consts::SQRT_2,
            0.0,
        );
        let top_center = cylinder.top_center();
        assert!((top_center.x() - expected_top.x()).abs() < 1e-10);
        assert!((top_center.y() - expected_top.y()).abs() < 1e-10);
        assert!((top_center.z() - expected_top.z()).abs() < 1e-10);
    }

    #[test]
    fn test_f32_compatibility() {
        let center = Point3D::new(1.0f32, 2.0f32, 3.0f32);
        let axis = Vector3D::new(0.0f32, 0.0f32, 1.0f32);
        let cylinder = Cylinder3D::new(center, axis, 5.0f32, 10.0f32).unwrap();

        assert_eq!(cylinder.radius(), 5.0f32);
        assert_eq!(cylinder.height(), 10.0f32);

        // f32での体積計算
        let volume = cylinder.volume();
        assert!(volume > 0.0f32);
    }
}
