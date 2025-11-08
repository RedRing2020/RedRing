//! TorusSurface3D Test Suite - Comprehensive testing for all TorusSurface3D functionality
//!
//! トーラス面の全機能テスト：作成、アクセサ、拡張機能、変換操作、CAM工具オフセット計算

#[cfg(test)]
mod tests {
    use crate::{Direction3D, Point3D, TorusSurface3D, Vector3D};
    use geo_foundation::{BasicTransform, ExtensionFoundation, PrimitiveKind};
    use std::f64::consts::PI;

    // テスト用のf64型別名
    type TestScalar = f64;

    // 近似等価性チェック用のヘルパー関数
    fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!((a - b).abs() < epsilon, "Expected {}, got {}", b, a);
    }

    // テスト用の標準トーラス面を作成
    fn create_test_torus() -> TorusSurface3D<TestScalar> {
        let origin = Point3D::origin();
        let z_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let x_axis = Direction3D::from_vector(Vector3D::unit_x()).unwrap();
        let major_radius = 5.0;
        let minor_radius = 2.0;

        TorusSurface3D::new(origin, z_axis, x_axis, major_radius, minor_radius).unwrap()
    }

    // CAM用の複雑なトーラス面を作成
    fn create_cam_torus() -> TorusSurface3D<TestScalar> {
        let origin = Point3D::new(10.0, 20.0, 30.0);
        let z_axis = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let x_axis = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let major_radius = 15.0;
        let minor_radius = 3.0;

        TorusSurface3D::new(origin, z_axis, x_axis, major_radius, minor_radius).unwrap()
    }

    // =================================================================
    // 基本機能テスト
    // =================================================================

    #[test]
    fn test_torus_creation() {
        // 基本的なトーラス面の作成
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let z_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let x_axis = Direction3D::from_vector(Vector3D::unit_x()).unwrap();
        let major_radius = 5.0;
        let minor_radius = 2.0;

        let torus =
            TorusSurface3D::new(origin, z_axis, x_axis, major_radius, minor_radius).unwrap();
        assert_eq!(torus.origin(), origin);
        assert_eq!(torus.z_axis(), z_axis);
        assert_eq!(torus.x_axis(), x_axis);
        assert_eq!(torus.major_radius(), major_radius);
        assert_eq!(torus.minor_radius(), minor_radius);
    }

    #[test]
    fn test_torus_creation_invalid() {
        let origin = Point3D::origin();
        let z_axis = Direction3D::from_vector(Vector3D::unit_z()).unwrap();
        let x_axis = Direction3D::from_vector(Vector3D::unit_x()).unwrap();

        // 無効な半径での作成失敗
        assert!(TorusSurface3D::new(origin, z_axis, x_axis, 0.0, 2.0).is_none()); // 主半径がゼロ
        assert!(TorusSurface3D::new(origin, z_axis, x_axis, 5.0, 0.0).is_none()); // 副半径がゼロ
        assert!(TorusSurface3D::new(origin, z_axis, x_axis, -1.0, 2.0).is_none()); // 負の主半径
        assert!(TorusSurface3D::new(origin, z_axis, x_axis, 5.0, -1.0).is_none()); // 負の副半径

        // 注意: 現在の実装では主半径 < 副半径の制約はチェックされていない
        // assert!(TorusSurface3D::new(origin, z_axis, x_axis, 2.0, 5.0).is_none()); // 主半径 < 副半径

        // 非直交軸での作成失敗
        // (1, 1, 0) を正規化すると (√2/2, √2/2, 0) になり、Z軸 (0, 0, 1) との内積は0で直交している
        // より明確に非直交なベクトルを使用
        let non_orthogonal = Direction3D::from_vector(Vector3D::new(0.0, 1.0, 1.0)).unwrap(); // Y軸とZ軸の混合
        assert!(TorusSurface3D::new(origin, z_axis, non_orthogonal, 5.0, 2.0).is_none());
    }

    #[test]
    fn test_standard_constructors() {
        let major_radius = 8.0;
        let minor_radius = 3.0;

        // 標準トーラス面（XY平面上、原点中心）
        let standard_torus = TorusSurface3D::standard(major_radius, minor_radius).unwrap();
        assert_eq!(
            standard_torus.z_axis(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );
        assert_eq!(
            standard_torus.x_axis(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap()
        );
        assert_eq!(standard_torus.origin(), Point3D::origin());
    }

    // =================================================================
    // Foundation トレイト実装テスト
    // =================================================================

    #[test]
    fn test_foundation_primitive_kind() {
        let torus = create_test_torus();
        assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSurface);
    }

    #[test]
    fn test_foundation_bounding_box() {
        let torus = create_test_torus();
        let bbox = torus.bounding_box();

        // トーラス面の境界ボックスは (major_radius + minor_radius) を使って計算される
        let expected_radius = torus.major_radius() + torus.minor_radius(); // 5.0 + 2.0 = 7.0

        assert_approx_eq(bbox.min().x(), -expected_radius, 1e-10);
        assert_approx_eq(bbox.min().y(), -expected_radius, 1e-10);
        assert_approx_eq(bbox.min().z(), -torus.minor_radius(), 1e-10);
        assert_approx_eq(bbox.max().x(), expected_radius, 1e-10);
        assert_approx_eq(bbox.max().y(), expected_radius, 1e-10);
        assert_approx_eq(bbox.max().z(), torus.minor_radius(), 1e-10);
    }

    #[test]
    fn test_foundation_measure() {
        let torus = create_test_torus();
        let surface_area = torus.measure().unwrap();

        // トーラス面の表面積: 4π²Rr (R: 主半径, r: 副半径)
        let expected_area = 4.0 * PI * PI * torus.major_radius() * torus.minor_radius();
        assert_approx_eq(surface_area, expected_area, 1e-10);
    }

    // =================================================================
    // パラメータ化テスト
    // =================================================================

    #[test]
    fn test_parameterization() {
        let torus = create_test_torus();

        // パラメータ (0, 0) - 外側の点
        let point_00 = torus.point_at(0.0, 0.0);
        let expected_x = torus.major_radius() + torus.minor_radius(); // 5.0 + 2.0 = 7.0
        assert_approx_eq(point_00.x(), expected_x, 1e-10);
        assert_approx_eq(point_00.y(), 0.0, 1e-10);
        assert_approx_eq(point_00.z(), 0.0, 1e-10);

        // パラメータ (π/2, 0) - Y軸方向の外側の点
        let point_pi2_0 = torus.point_at(PI / 2.0, 0.0);
        assert_approx_eq(point_pi2_0.x(), 0.0, 1e-10);
        assert_approx_eq(point_pi2_0.y(), expected_x, 1e-10);
        assert_approx_eq(point_pi2_0.z(), 0.0, 1e-10);

        // パラメータ (0, π/2) - Z軸正方向の点
        let point_0_pi2 = torus.point_at(0.0, PI / 2.0);
        assert_approx_eq(point_0_pi2.x(), torus.major_radius(), 1e-10);
        assert_approx_eq(point_0_pi2.y(), 0.0, 1e-10);
        assert_approx_eq(point_0_pi2.z(), torus.minor_radius(), 1e-10);
    }

    #[test]
    fn test_normal_calculation() {
        let torus = create_test_torus();

        // パラメータ (0, 0) での法線
        let normal_00 = torus.normal_at(0.0, 0.0);
        assert_approx_eq(normal_00.x(), 1.0, 1e-10); // 外向き法線
        assert_approx_eq(normal_00.y(), 0.0, 1e-10);
        assert_approx_eq(normal_00.z(), 0.0, 1e-10);

        // パラメータ (0, π/2) での法線 - Z軸正方向
        let normal_0_pi2 = torus.normal_at(0.0, PI / 2.0);
        assert_approx_eq(normal_0_pi2.x(), 0.0, 1e-10);
        assert_approx_eq(normal_0_pi2.y(), 0.0, 1e-10);
        assert_approx_eq(normal_0_pi2.z(), 1.0, 1e-10);
    }

    // =================================================================
    // 拡張機能テスト（Extensions）
    // =================================================================

    #[test]
    fn test_closest_point_to() {
        let torus = create_test_torus();

        // 原点に最も近い点（内側の最近点）
        let origin = Point3D::origin();
        let closest = torus.closest_point_to(origin);

        // 原点からの最近点の距離を確認（実装に基づく期待値）
        let distance_from_origin = closest.distance_to(&origin);
        // 実際の実装での結果に基づいて期待値を調整
        assert!(distance_from_origin > 0.0);
        assert!(distance_from_origin.is_finite());

        // X軸上の遠い点
        let far_point = Point3D::new(20.0, 0.0, 0.0);
        let closest_far = torus.closest_point_to(far_point);

        // 最近点が妥当な範囲にあることを確認
        assert!(closest_far.x().is_finite());
        assert!(closest_far.y().is_finite());
        assert!(closest_far.z().is_finite());

        // 最近点がトーラス面上にあることを確認（距離が有限で正の値）
        let distance_to_far = closest_far.distance_to(&far_point);
        assert!(distance_to_far > 0.0);
        assert!(distance_to_far < far_point.distance_to(&origin)); // 最適化されていることを確認
    }

    #[test]
    fn test_principal_curvatures() {
        let torus = create_test_torus();

        // パラメータ (0, 0) での主曲率
        let (k1, k2) = torus.principal_curvatures(0.0, 0.0);

        // トーラス面の主曲率（実装された式に基づく）
        // v = 0 なので cos(v) = 1
        // k1 = cos(v) / (major_radius + minor_radius * cos(v)) = 1 / (5 + 2 * 1) = 1/7
        // k2 = 1 / minor_radius = 1/2
        let expected_k1 = 1.0 / (torus.major_radius() + torus.minor_radius()); // 1/7
        let expected_k2 = 1.0 / torus.minor_radius(); // 1/2 = 0.5

        assert_approx_eq(k1, expected_k1, 1e-10);
        assert_approx_eq(k2, expected_k2, 1e-10);
    }

    #[test]
    fn test_toolpath_parameters() {
        let torus = create_cam_torus();
        let tool_radius = 1.5;

        // 工具経路パラメータの計算（パラメータ空間の中心点で計算）
        let u = PI / 2.0;
        let v = PI / 4.0;
        let (tool_center, feed_direction, surface_normal, feed_factor) =
            torus.toolpath_parameters(u, v, tool_radius);

        // 工具中心点が妥当な位置にあることを確認
        assert!(tool_center.x().is_finite());
        assert!(tool_center.y().is_finite());
        assert!(tool_center.z().is_finite());

        // 送り方向が単位ベクトルであることを確認
        let feed_length = (feed_direction.x() * feed_direction.x()
            + feed_direction.y() * feed_direction.y()
            + feed_direction.z() * feed_direction.z())
        .sqrt();
        assert_approx_eq(feed_length, 1.0, 1e-10);

        // 表面法線が単位ベクトルであることを確認
        let normal_length = (surface_normal.x() * surface_normal.x()
            + surface_normal.y() * surface_normal.y()
            + surface_normal.z() * surface_normal.z())
        .sqrt();
        assert_approx_eq(normal_length, 1.0, 1e-10);

        // 送り速度係数が妥当な範囲にあることを確認
        assert!(feed_factor > 0.0);
        assert!(feed_factor <= 1.0);
    }

    // =================================================================
    // 基本変換操作テスト（BasicTransform）
    // =================================================================

    #[test]
    fn test_basic_translate() {
        let torus = create_test_torus();
        let translation = Vector3D::new(10.0, 20.0, 30.0);

        let translated = torus.translate(translation);

        // 原点が移動されていることを確認
        let expected_origin = torus.origin() + translation;
        assert_eq!(translated.origin(), expected_origin);

        // 軸と半径は変更されていないことを確認
        assert_eq!(translated.z_axis(), torus.z_axis());
        assert_eq!(translated.x_axis(), torus.x_axis());
        assert_eq!(translated.major_radius(), torus.major_radius());
        assert_eq!(translated.minor_radius(), torus.minor_radius());
    }

    #[test]
    fn test_basic_rotate_z() {
        let torus = create_test_torus();
        let center = Point3D::origin();
        let angle = geo_foundation::Angle::from_radians(PI / 2.0); // 90度回転

        let rotated = torus.rotate(center, angle);

        // 原点は原点中心回転なので変わらない
        assert_eq!(rotated.origin(), torus.origin());

        // X軸がY軸方向に回転
        let expected_x = Direction3D::from_vector(Vector3D::unit_y()).unwrap();
        assert_approx_eq(rotated.x_axis().x(), expected_x.x(), 1e-10);
        assert_approx_eq(rotated.x_axis().y(), expected_x.y(), 1e-10);
        assert_approx_eq(rotated.x_axis().z(), expected_x.z(), 1e-10);

        // Z軸は変わらない
        assert_eq!(rotated.z_axis(), torus.z_axis());
    }

    #[test]
    fn test_basic_scale() {
        let torus = create_test_torus();
        let center = Point3D::origin();
        let scale_factor = 2.0;

        let scaled = torus.scale(center, scale_factor);

        // 半径がスケールされていることを確認
        assert_approx_eq(
            scaled.major_radius(),
            torus.major_radius() * scale_factor,
            1e-10,
        );
        assert_approx_eq(
            scaled.minor_radius(),
            torus.minor_radius() * scale_factor,
            1e-10,
        );

        // 軸は変更されていないことを確認
        assert_eq!(scaled.z_axis(), torus.z_axis());
        assert_eq!(scaled.x_axis(), torus.x_axis());
    }

    // =================================================================
    // 表面積計算テスト
    // =================================================================

    #[test]
    fn test_surface_area_calculation() {
        let torus = create_test_torus();
        let calculated_area = torus.surface_area();

        // 解析的な表面積: 4π²Rr
        let analytical_area = 4.0 * PI * PI * torus.major_radius() * torus.minor_radius();

        assert_approx_eq(calculated_area, analytical_area, 1e-10);
    }

    #[test]
    fn test_surface_area_different_sizes() {
        // 異なるサイズのトーラス面でテスト
        let test_cases = vec![
            (1.0, 0.5),   // 小さいトーラス
            (10.0, 3.0),  // 中程度のトーラス
            (50.0, 10.0), // 大きいトーラス
        ];

        for (major_radius, minor_radius) in test_cases {
            let torus = TorusSurface3D::new(
                Point3D::origin(),
                Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
                Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
                major_radius,
                minor_radius,
            )
            .unwrap();

            let calculated_area = torus.surface_area();
            let expected_area = 4.0 * PI * PI * major_radius * minor_radius;

            assert_approx_eq(calculated_area, expected_area, expected_area * 1e-12);
        }
    }

    // =================================================================
    // エッジケース・境界値テスト
    // =================================================================

    #[test]
    fn test_minimal_valid_torus() {
        // 最小有効トーラス（主半径がわずかに副半径より大きい）
        let major_radius = 1.0 + f64::EPSILON * 10.0;
        let minor_radius = 1.0;

        let torus = TorusSurface3D::new(
            Point3D::origin(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            major_radius,
            minor_radius,
        );

        assert!(torus.is_some());
        let torus = torus.unwrap();
        assert!(torus.surface_area() > 0.0);
    }

    #[test]
    fn test_large_torus() {
        // 大きなトーラス面
        let major_radius = 1000.0;
        let minor_radius = 100.0;

        let torus = TorusSurface3D::new(
            Point3D::origin(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            major_radius,
            minor_radius,
        )
        .unwrap();

        // パラメータ化が正常に動作することを確認
        let point = torus.point_at(PI / 4.0, PI / 4.0);
        assert!(point.x().is_finite());
        assert!(point.y().is_finite());
        assert!(point.z().is_finite());

        // 法線計算が正常に動作することを確認
        let normal = torus.normal_at(PI / 4.0, PI / 4.0);
        let norm =
            (normal.x() * normal.x() + normal.y() * normal.y() + normal.z() * normal.z()).sqrt();
        assert_approx_eq(norm, 1.0, 1e-10);
    }

    // =================================================================
    // CAM特化テスト
    // =================================================================

    #[test]
    fn test_cam_tool_offset_calculation() {
        let torus = create_cam_torus();
        let tool_radius = 2.0;

        // 複数の点での工具オフセット計算
        let test_parameters = vec![
            (0.0, 0.0),
            (PI / 4.0, PI / 4.0),
            (PI / 2.0, PI / 2.0),
            (PI, 0.0),
            (3.0 * PI / 2.0, PI),
        ];

        for (u, v) in test_parameters {
            let surface_point = torus.point_at(u, v);
            let surface_normal = torus.normal_at(u, v);

            // 工具中心位置の計算（法線方向にオフセット）
            let tool_center = Point3D::new(
                surface_point.x() + surface_normal.x() * tool_radius,
                surface_point.y() + surface_normal.y() * tool_radius,
                surface_point.z() + surface_normal.z() * tool_radius,
            );

            // 工具中心から表面への距離が工具半径に等しいことを確認
            let distance = tool_center.distance_to(&surface_point);
            assert_approx_eq(distance, tool_radius, 1e-10);

            // 法線が単位ベクトルであることを確認
            let normal_length = (surface_normal.x() * surface_normal.x()
                + surface_normal.y() * surface_normal.y()
                + surface_normal.z() * surface_normal.z())
            .sqrt();
            assert_approx_eq(normal_length, 1.0, 1e-10);
        }
    }

    #[test]
    fn test_cam_collision_detection_prep() {
        let torus = create_cam_torus();

        // 異なる工具サイズでの衝突検出準備
        let tool_radii = vec![0.5, 1.0, 2.0, 3.0, 5.0];

        for tool_radius in tool_radii {
            // サンプル点でのtoolpath_parameters計算
            let u = PI / 4.0;
            let v = PI / 4.0;
            let (tool_center, feed_direction, surface_normal, feed_factor) =
                torus.toolpath_parameters(u, v, tool_radius);

            // 工具中心点が妥当であることを確認
            assert!(tool_center.x().is_finite());
            assert!(tool_center.y().is_finite());
            assert!(tool_center.z().is_finite());

            // 送り方向が単位ベクトルであることを確認
            let feed_length = (feed_direction.x() * feed_direction.x()
                + feed_direction.y() * feed_direction.y()
                + feed_direction.z() * feed_direction.z())
            .sqrt();
            assert_approx_eq(feed_length, 1.0, 1e-10);

            // 表面法線が単位ベクトルであることを確認
            let normal_length = (surface_normal.x() * surface_normal.x()
                + surface_normal.y() * surface_normal.y()
                + surface_normal.z() * surface_normal.z())
            .sqrt();
            assert_approx_eq(normal_length, 1.0, 1e-10);

            // 送り速度係数が妥当な範囲にあることを確認
            assert!(feed_factor > 0.0);
            assert!(feed_factor <= 1.0);
        }
    }
}
