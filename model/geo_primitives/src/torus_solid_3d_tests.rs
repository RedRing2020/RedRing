// torus_solid_3d_tests.rs
// TorusSolid3D の包括的テストスイート
//
// 基本機能、Foundation実装、Extensions、Transform操作の全般的なテストを提供します。
// 3D CAM固体加工計算での使用を想定したテストケースを含みます。

#[cfg(test)]
mod tests {
    use crate::{Point3D, TorusSolid3D};
    use geo_foundation::{ExtensionFoundation, PrimitiveKind};
    use std::f64::consts::PI;

    #[test]
    fn test_torus_solid_creation() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        assert_eq!(torus.major_radius(), 3.0);
        assert_eq!(torus.minor_radius(), 1.0);
        assert_eq!(torus.origin(), &Point3D::origin());
    }

    #[test]
    fn test_torus_solid_creation_invalid() {
        // 負の半径
        assert!(TorusSolid3D::standard(-1.0_f64, 1.0_f64).is_none());
        assert!(TorusSolid3D::standard(3.0_f64, -1.0_f64).is_none());

        // ゼロ半径
        assert!(TorusSolid3D::standard(0.0_f64, 1.0_f64).is_none());
        assert!(TorusSolid3D::standard(3.0_f64, 0.0_f64).is_none());

        // 主半径 <= 副半径 (固体の制約)
        assert!(TorusSolid3D::standard(1.0_f64, 1.0_f64).is_none());
        assert!(TorusSolid3D::standard(1.0_f64, 2.0_f64).is_none());
    }

    #[test]
    fn test_minimal_creation() {
        let torus: TorusSolid3D<f64> = TorusSolid3D::minimal().unwrap();
        assert!(torus.major_radius() > torus.minor_radius());
        assert!(torus.major_radius() > 0.0);
        assert!(torus.minor_radius() > 0.0);
    }

    #[test]
    fn test_volume_calculation() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let volume = torus.volume();

        // 期待される体積: 2π²R²r = 2π² × 3² × 1
        let expected = 2.0_f64 * PI.powi(2) * 3.0_f64.powi(2) * 1.0_f64;
        assert!((volume - expected).abs() < 1e-10);
    }

    #[test]
    fn test_surface_area_calculation() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let surface_area = torus.surface_area();

        // 期待される表面積: 4π²Rr = 4π² × 3 × 1
        let expected = 4.0_f64 * PI.powi(2) * 3.0_f64 * 1.0_f64;
        assert!((surface_area - expected).abs() < 1e-10);
    }

    #[test]
    fn test_foundation_trait() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();

        assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSolid);
        assert_eq!(torus.measure().unwrap(), torus.volume());

        let bbox = torus.bounding_box();
        assert!(bbox.width() > 0.0);
        assert!(bbox.height() > 0.0);
        assert!(bbox.depth() > 0.0);
    }

    #[test]
    fn test_point_containment() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();

        // 中心点は内部にない（ドーナツ型の穴）
        assert!(!torus.contains_point(&Point3D::origin()));

        // 主半径上の点（トーラス表面上）
        let surface_point = Point3D::new(3.0, 0.0, 0.0);
        assert!(torus.contains_point(&surface_point));

        // 明らかに外部の点
        let far_point = Point3D::new(10.0, 10.0, 10.0);
        assert!(!torus.contains_point(&far_point));
    }

    #[test]
    fn test_rotate_z() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let rotated = torus.rotate_z(PI / 2.0);

        // 90度回転後、X軸(1,0,0)はY方向(0,1,0)を向く
        assert!((rotated.x_axis().x() - 0.0).abs() < 1e-10);
        assert!((rotated.x_axis().y() - 1.0).abs() < 1e-10);
        assert!((rotated.x_axis().z() - 0.0).abs() < 1e-10);
    }
}
