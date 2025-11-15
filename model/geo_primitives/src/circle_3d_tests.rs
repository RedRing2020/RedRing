//! Circle3D Test Suite - Comprehensive testing for all Circle3D functionality
//!
//! 3次元円の全機能テスト：作成、アクセサ、拡張機能、変換操作

#[cfg(test)]
mod tests {
    use crate::{Circle3D, Direction3D, Point3D, Vector3D};
    use geo_foundation::tolerance_migration::DefaultTolerances;

    // テスト用のf64型別名
    type TestScalar = f64;

    // 近似等価性チェック用のヘルパー関数
    fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!((a - b).abs() < epsilon, "Expected {}, got {}", b, a);
    }

    #[test]
    fn test_circle_creation() {
        // 基本的な円の作成
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let radius = 5.0;

        let circle = Circle3D::new(center, normal, radius).unwrap();
        assert_eq!(circle.center(), center);
        assert_eq!(circle.normal(), normal);
        assert_eq!(circle.radius(), radius);

        // 無効な半径での作成失敗
        assert!(Circle3D::new(center, normal, 0.0).is_none());
        assert!(Circle3D::new(center, normal, -1.0).is_none());
    }

    #[test]
    fn test_plane_constructors() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let radius = 4.0;

        // XY平面の円
        let xy_circle = Circle3D::new_xy_plane(center, radius).unwrap();
        assert_eq!(
            xy_circle.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );

        // XZ平面の円
        let xz_circle = Circle3D::new_xz_plane(center, radius).unwrap();
        assert_eq!(
            xz_circle.normal(),
            Direction3D::from_vector(Vector3D::unit_y()).unwrap()
        );

        // YZ平面の円
        let yz_circle = Circle3D::new_yz_plane(center, radius).unwrap();
        assert_eq!(
            yz_circle.normal(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap()
        );
    }

    #[test]
    fn test_geometric_properties() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let radius = 3.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        // 幾何学的性質
        assert_approx_eq(circle.diameter(), 6.0, 1e-10);
        assert_approx_eq(
            circle.circumference(),
            2.0 * std::f64::consts::PI * 3.0,
            1e-10,
        );
        assert_approx_eq(circle.area(), std::f64::consts::PI * 9.0, 1e-10);
    }

    #[test]
    fn test_point_at_angle() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let radius = 2.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        // 角度0での点（X軸正方向）
        let point_0 = circle.point_at_angle(0.0);
        assert_approx_eq(point_0.x(), 2.0, 1e-10);
        assert_approx_eq(point_0.y(), 0.0, 1e-10);
        assert_approx_eq(point_0.z(), 0.0, 1e-10);

        // 角度π/2での点（Y軸正方向）
        let point_90 = circle.point_at_angle(std::f64::consts::PI / 2.0);
        assert_approx_eq(point_90.x(), 0.0, 1e-10);
        assert_approx_eq(point_90.y(), 2.0, 1e-10);
        assert_approx_eq(point_90.z(), 0.0, 1e-10);
    }

    #[test]
    fn test_distance_calculations() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let radius = 3.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        // 中心への距離
        let test_point = Point3D::new(4.0, 0.0, 0.0);
        assert_approx_eq(circle.distance_to_center(&test_point), 4.0, 1e-10);

        // 円周上の点への距離（XY平面上）
        let point_on_plane = Point3D::new(5.0, 0.0, 0.0); // 半径3の円の外側
        assert_approx_eq(circle.distance_to_circle(&point_on_plane), 2.0, 1e-10);

        // 平面外の点への距離
        let point_above = Point3D::new(3.0, 0.0, 4.0); // 円周上のx=3の点の真上4単位
        assert_approx_eq(circle.distance_to_circle(&point_above), 4.0, 1e-10);
    }

    #[test]
    fn test_point_on_plane() {
        let center = Point3D::new(0.0, 0.0, 5.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let radius = 2.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        let tolerance = DefaultTolerances::distance::<TestScalar>();

        // 平面上の点
        let point_on_plane = Point3D::new(1.0, 1.0, 5.0);
        assert!(circle.point_on_plane(&point_on_plane, tolerance));

        // 平面外の点
        let point_off_plane = Point3D::new(1.0, 1.0, 6.0);
        assert!(!circle.point_on_plane(&point_off_plane, tolerance));
    }

    #[test]
    fn test_extensions() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let radius = 1.0;
        let circle = Circle3D::new(center, normal, radius).unwrap();

        // 軸の取得
        let u_axis = circle.u_axis();
        let v_axis = circle.v_axis();

        // 軸は正規化されている
        assert_approx_eq(u_axis.as_vector().length(), 1.0, 1e-10);
        assert_approx_eq(v_axis.as_vector().length(), 1.0, 1e-10);

        // 軸は互いに直交
        assert_approx_eq(u_axis.as_vector().dot(&v_axis.as_vector()), 0.0, 1e-10);

        // サンプルポイントの生成
        let points = circle.sample_points(4);
        assert_eq!(points.len(), 4);

        for point in &points {
            let distance = circle.distance_to_center(point);
            assert_approx_eq(distance, radius, 1e-10);
        }
    }
}
