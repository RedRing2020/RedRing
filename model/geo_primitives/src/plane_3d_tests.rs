//! Plane3D テストファイル

use crate::{Direction3D, Plane3D, Point3D, Vector3D};

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // ========================================================================
    // Construction Tests
    // ========================================================================

    #[test]
    fn test_from_point_and_normal() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);

        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        assert_eq!(plane.point(), point);
        assert_relative_eq!(plane.normal().x(), 0.0);
        assert_relative_eq!(plane.normal().y(), 0.0);
        assert_relative_eq!(plane.normal().z(), 1.0);
    }

    #[test]
    fn test_from_point_and_normal_normalization() {
        let point = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(3.0, 4.0, 0.0); // length = 5.0

        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        // 法線ベクトルが正規化されているかチェック
        assert_relative_eq!(plane.normal().length(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(plane.normal().x(), 0.6); // 3/5
        assert_relative_eq!(plane.normal().y(), 0.8); // 4/5
        assert_relative_eq!(plane.normal().z(), 0.0);
    }

    #[test]
    fn test_from_point_and_normal_zero_normal() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::zero();

        let result = Plane3D::from_point_and_normal(point, normal);

        assert!(result.is_none());
    }

    #[test]
    fn test_from_three_points() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(0.0, 1.0, 0.0);

        let plane = Plane3D::from_three_points(p1, p2, p3).unwrap();

        // XY平面になるはず
        assert_relative_eq!(plane.normal().x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(plane.normal().y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(plane.normal().z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_from_three_points_collinear() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(2.0, 0.0, 0.0); // 一直線上

        let result = Plane3D::from_three_points(p1, p2, p3);

        assert!(result.is_none());
    }

    #[test]
    fn test_coordinate_planes() {
        // XY平面
        let xy_plane = Plane3D::xy_plane(5.0);
        assert_eq!(xy_plane.origin(), Point3D::new(0.0, 0.0, 5.0));
        assert_eq!(
            xy_plane.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );

        // XZ平面
        let xz_plane = Plane3D::xz_plane(3.0);
        assert_eq!(xz_plane.origin(), Point3D::new(0.0, 3.0, 0.0));
        assert_eq!(
            xz_plane.normal(),
            Direction3D::from_vector(Vector3D::unit_y()).unwrap()
        );

        // YZ平面
        let yz_plane = Plane3D::yz_plane(2.0);
        assert_eq!(yz_plane.origin(), Point3D::new(2.0, 0.0, 0.0));
        assert_eq!(
            yz_plane.normal(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap()
        );
    }

    // ========================================================================
    // Geometric Operation Tests
    // ========================================================================

    #[test]
    fn test_contains_point() {
        let plane = Plane3D::xy_plane(5.0);

        // 平面上の点
        assert!(plane.contains_point(Point3D::new(1.0, 2.0, 5.0), 1e-10));
        assert!(plane.contains_point(Point3D::new(-3.0, 7.0, 5.0), 1e-10));

        // 平面外の点
        assert!(!plane.contains_point(Point3D::new(1.0, 2.0, 5.1), 1e-10));
        assert!(!plane.contains_point(Point3D::new(1.0, 2.0, 4.9), 1e-10));

        // 許容誤差内の点
        assert!(plane.contains_point(Point3D::new(1.0, 2.0, 5.05), 0.1));
    }

    #[test]
    fn test_distance_to_point() {
        let plane = Plane3D::xy_plane(3.0);

        // 平面上の点
        assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 3.0)), 0.0);

        // 法線方向の点（正の距離）
        assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 5.0)), 2.0);

        // 法線逆方向の点（負の距離）
        assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 1.0)), -2.0);
    }

    #[test]
    fn test_project_point() {
        let plane = Plane3D::xy_plane(2.0);

        let point = Point3D::new(3.0, 4.0, 7.0);
        let projected = plane.project_point(point);

        assert_relative_eq!(projected.x(), 3.0);
        assert_relative_eq!(projected.y(), 4.0);
        assert_relative_eq!(projected.z(), 2.0);

        // 投影点は平面上にあるはず
        assert!(plane.contains_point(projected, 1e-10));
    }

    #[test]
    fn test_equation_coefficients() {
        let plane = Plane3D::xy_plane(3.0);
        let (a, b, c, d) = plane.equation_coefficients();

        // z = 3 なので、0x + 0y + 1z - 3 = 0
        assert_relative_eq!(a, 0.0);
        assert_relative_eq!(b, 0.0);
        assert_relative_eq!(c, 1.0);
        assert_relative_eq!(d, -3.0);

        // 平面上の点で方程式をチェック
        let test_point = Point3D::new(5.0, 7.0, 3.0);
        let result = a * test_point.x() + b * test_point.y() + c * test_point.z() + d;
        assert_relative_eq!(result, 0.0, epsilon = 1e-10);
    }

    // ========================================================================
    // Validation Tests
    // ========================================================================

    #[test]
    fn test_is_valid() {
        // 正常な平面
        let valid_plane = Plane3D::xy_plane(0.0);
        assert!(valid_plane.is_valid());

        // 正規化されていない法線で作成された平面は、from_point_and_normal で自動正規化されるため、
        // 直接的に不正な平面を作ることは難しい。代わりに長さが非常に短い法線をテスト
        let point = Point3D::new(0.0, 0.0, 0.0);
        let very_small_normal = Vector3D::new(1e-15, 0.0, 0.0);

        // 非常に小さい法線ベクトルでも正規化は成功するが...
        if let Some(plane) = Plane3D::from_point_and_normal(point, very_small_normal) {
            // 作成された平面の法線は正規化されているはず
            assert!(plane.is_valid());
        }
    }

    // ========================================================================
    // Default and Constants Tests
    // ========================================================================

    #[test]
    fn test_default() {
        let plane = Plane3D::<f64>::default();
        assert_eq!(plane.origin(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(
            plane.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );
    }

    #[test]
    fn test_xy_constant() {
        let plane = Plane3D::<f64>::xy();
        assert_eq!(plane.origin(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(
            plane.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );
    }

    // ========================================================================
    // Display Tests
    // ========================================================================

    #[test]
    fn test_display() {
        let plane = Plane3D::xy_plane(1.0);
        let display_str = format!("{}", plane);

        assert!(display_str.contains("Plane3D"));
        assert!(display_str.contains("origin"));
        assert!(display_str.contains("normal"));
    }
}
