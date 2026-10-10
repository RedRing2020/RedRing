//! Circle3D Test Suite - Comprehensive testing for all Circle3D functionality
//!
//! 3次元円の全機能テスト：作成、アクセサ、拡張機能、変換操作

#[cfg(test)]
mod tests {
    use crate::{Circle3D, Direction3D, Point3D, Vector3D};

    use geo_contracts::Circle3DEvaluation;

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
        assert_eq!(circle.center_internal(), center);
        assert_eq!(circle.normal_internal(), normal);
        assert_eq!(circle.radius_internal(), radius);

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
            xy_circle.normal_internal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );

        // XZ平面の円
        let xz_circle = Circle3D::new_xz_plane(center, radius).unwrap();
        assert_eq!(
            xz_circle.normal_internal(),
            Direction3D::from_vector(Vector3D::unit_y()).unwrap()
        );

        // YZ平面の円
        let yz_circle = Circle3D::new_yz_plane(center, radius).unwrap();
        assert_eq!(
            yz_circle.normal_internal(),
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
        let ref_direction = Direction3D::from_vector(Vector3D::unit_x()).unwrap();
        let radius = 2.0;
        let circle =
            Circle3D::new_with_ref_direction(center, normal, ref_direction, radius).unwrap();

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

        let point_90_from_parameter =
            <Circle3D<f64> as Circle3DEvaluation<f64>>::point_at_parameter(
                &circle,
                std::f64::consts::PI / 2.0,
            );
        assert_approx_eq(point_90.x(), point_90_from_parameter.0, 1e-10);
        assert_approx_eq(point_90.y(), point_90_from_parameter.1, 1e-10);
        assert_approx_eq(point_90.z(), point_90_from_parameter.2, 1e-10);
    }

    #[test]
    fn test_point_at_parameter_uses_local_angle_parameter() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let ref_direction = Direction3D::from_vector(Vector3D::unit_x()).unwrap();
        let radius = 2.0;
        let circle =
            Circle3D::new_with_ref_direction(center, normal, ref_direction, radius).unwrap();

        let point_0 = <Circle3D<f64> as Circle3DEvaluation<f64>>::point_at_parameter(&circle, 0.0);
        assert_approx_eq(point_0.0, 2.0, 1e-10);
        assert_approx_eq(point_0.1, 0.0, 1e-10);
        assert_approx_eq(point_0.2, 0.0, 1e-10);

        let point_pi_2 = <Circle3D<f64> as Circle3DEvaluation<f64>>::point_at_parameter(
            &circle,
            std::f64::consts::PI / 2.0,
        );
        assert_approx_eq(point_pi_2.0, 0.0, 1e-10);
        assert_approx_eq(point_pi_2.1, 2.0, 1e-10);
        assert_approx_eq(point_pi_2.2, 0.0, 1e-10);

        let point_tau = <Circle3D<f64> as Circle3DEvaluation<f64>>::point_at_parameter(
            &circle,
            std::f64::consts::TAU,
        );
        assert_approx_eq(point_tau.0, 2.0, 1e-10);
        assert_approx_eq(point_tau.1, 0.0, 1e-10);
        assert_approx_eq(point_tau.2, 0.0, 1e-10);
    }

    #[test]
    fn test_tangent_at_angle_matches_parameter_semantics() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let ref_direction = Direction3D::from_vector(Vector3D::unit_x()).unwrap();
        let radius = 2.0;
        let circle =
            Circle3D::new_with_ref_direction(center, normal, ref_direction, radius).unwrap();

        let tangent_0 = circle.tangent_at_parameter(0.0);
        assert_approx_eq(tangent_0.x(), 0.0, 1e-10);
        assert_approx_eq(tangent_0.y(), 1.0, 1e-10);
        assert_approx_eq(tangent_0.z(), 0.0, 1e-10);

        let tangent_pi_2 = circle.tangent_at_parameter(std::f64::consts::PI / 2.0);
        assert_approx_eq(tangent_pi_2.x(), -1.0, 1e-10);
        assert_approx_eq(tangent_pi_2.y(), 0.0, 1e-10);
        assert_approx_eq(tangent_pi_2.z(), 0.0, 1e-10);
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
            let distance = center.distance_to(point);
            assert_approx_eq(distance, radius, 1e-10);
        }
    }

    #[test]
    fn test_contains_and_classify_point() {
        use geo_contracts::{Circle3DContainment, PointClassification};

        // XY 平面上、中心 (1, 2, 3)・半径 2 の円
        let circle = Circle3D::new(
            Point3D::new(1.0_f64, 2.0, 3.0),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            2.0,
        )
        .unwrap();
        let tolerance = 1e-3;

        // contains_point は円周上にあるかを判定する
        assert!(circle.contains_point(&Point3D::new(3.0, 2.0, 3.0), tolerance));
        assert!(!circle.contains_point(&Point3D::new(1.0, 2.0, 3.0), tolerance));

        let classify =
            |x: f64, y: f64, z: f64| circle.classify_point(&Point3D::new(x, y, z), tolerance);
        assert_eq!(classify(1.0, 2.0, 3.0), PointClassification::Inside);
        assert_eq!(classify(3.0, 2.0, 3.0), PointClassification::OnBoundary);
        assert_eq!(classify(4.0, 2.0, 3.0), PointClassification::Outside);

        // 円周までの距離が許容誤差以内なら、平面外でも OnBoundary
        assert_eq!(
            classify(3.0, 2.0, 3.0 + 0.5 * tolerance),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(3.0 - 0.5 * tolerance, 2.0, 3.0),
            PointClassification::OnBoundary
        );

        // 平面からの距離が許容誤差を超える点は、円の内側の上方でも Outside
        assert_eq!(
            classify(1.0, 2.0, 3.0 + 2.0 * tolerance),
            PointClassification::Outside
        );
        assert_eq!(
            classify(1.0, 2.0, 3.0 + 0.5 * tolerance),
            PointClassification::Inside
        );

        // trait定義は既定の距離トレランスで判定する
        assert!(Circle3DContainment::contains_point(
            &circle,
            (1.0, 4.0, 3.0)
        ));
        assert_eq!(
            Circle3DContainment::classify_point(&circle, (1.0, 2.0, 3.0)),
            PointClassification::Inside
        );
    }

    #[test]
    fn test_from_three_points_passes_through_all_points() {
        use geo_contracts::Circle3DConstructor;

        // 中心 (0, 0, 0)・半径 1 の XY 平面上の円
        let circle = <Circle3D<f64> as Circle3DConstructor<f64>>::from_three_points(
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (-1.0, 0.0, 0.0),
        )
        .unwrap();
        assert!(
            circle
                .center_internal()
                .distance_to(&Point3D::new(0.0, 0.0, 0.0))
                < 1e-12
        );
        assert!((circle.radius_internal() - 1.0).abs() < 1e-12);

        // 傾いた平面上の 3 点のいずれも円上にある
        let points = [
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(4.0, -1.0, 2.0),
            Point3D::new(0.0, 5.0, -1.0),
        ];
        let circle = <Circle3D<f64> as Circle3DConstructor<f64>>::from_three_points(
            (points[0].x(), points[0].y(), points[0].z()),
            (points[1].x(), points[1].y(), points[1].z()),
            (points[2].x(), points[2].y(), points[2].z()),
        )
        .unwrap();
        for point in points {
            assert!(circle.contains_point(&point, 1e-9));
        }

        // 一直線上の 3 点は円を作れない
        assert!(
            <Circle3D<f64> as Circle3DConstructor<f64>>::from_three_points(
                (0.0, 0.0, 0.0),
                (1.0, 1.0, 1.0),
                (2.0, 2.0, 2.0),
            )
            .is_none()
        );
    }
}
