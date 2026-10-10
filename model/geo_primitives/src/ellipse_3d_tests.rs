//! Ellipse3D の基本テスト
//!
//! 基本機能のみテスト：作成、アクセサ、基本プロパティ

use crate::{Circle3D, Direction3D, Ellipse3D, Point3D, Vector3D};
use geo_contracts::{Ellipse3DProjection, Ellipse3DProperties};

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants::TOLERANCE_F32;

    #[test]
    fn test_basic_creation() {
        // XY平面上の基本的な楕円作成
        let center = Point3D::new(1.0, 2.0, 3.0);
        let ellipse = Ellipse3D::xy_aligned(center, 5.0, 3.0).unwrap();

        assert_eq!(ellipse.center_3d(), (center.x(), center.y(), center.z()));
        assert_eq!(ellipse.semi_major_axis(), 5.0);
        assert_eq!(ellipse.semi_minor_axis(), 3.0);
        assert_eq!(
            ellipse.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );
        assert_eq!(
            ellipse.major_axis_direction(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap()
        );

        // 不正な楕円（短軸が長軸より大きい）
        let invalid = Ellipse3D::xy_aligned(center, 3.0, 5.0);
        assert!(invalid.is_none());

        // 負の半軸
        let invalid2 = Ellipse3D::xy_aligned(center, -1.0, 2.0);
        assert!(invalid2.is_none());
    }

    #[test]
    fn test_from_circle() {
        let circle = Circle3D::new_xy_plane(Point3D::new(1.0, 2.0, 3.0), 4.0).unwrap();
        let ellipse = Ellipse3D::from_circle(&circle).unwrap();

        let c = circle.center_internal();
        assert_eq!(ellipse.center_3d(), (c.x(), c.y(), c.z()));
        assert_eq!(ellipse.semi_major_axis(), circle.radius_internal());
        assert_eq!(ellipse.semi_minor_axis(), circle.radius_internal());
        assert_eq!(ellipse.normal(), circle.normal());
        assert!(ellipse.is_circle());
    }

    #[test]
    fn test_basic_properties() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 5.0, 3.0).unwrap();

        // 離心率
        let eccentricity = ellipse.eccentricity();
        let expected_e = (1.0f64 - (3.0f64 * 3.0f64) / (5.0f64 * 5.0f64)).sqrt();
        assert!((eccentricity - expected_e).abs() < 1e-10);

        // 面積
        let area = ellipse.area();
        let expected_area = std::f64::consts::PI * 5.0 * 3.0;
        assert!((area - expected_area).abs() < 1e-10);

        // 円判定
        assert!(!ellipse.is_circle());

        // 退化判定
        assert!(!ellipse.is_degenerate());
    }

    #[test]
    fn test_circle_detection() {
        let circle_ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 3.0, 3.0).unwrap();
        assert!(circle_ellipse.is_circle());
        assert_eq!(circle_ellipse.eccentricity(), 0.0);

        // 円への変換
        let circle = circle_ellipse.to_circle().unwrap();
        assert_eq!(circle.radius_internal(), 3.0);
    }

    #[test]
    fn test_degenerate_ellipse() {
        let tiny_ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 1e-12, 1e-12).unwrap();
        assert!(tiny_ellipse.is_degenerate());
    }

    #[test]
    fn test_axis_directions() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 4.0, 2.0).unwrap();

        // 長軸方向
        let major_dir = ellipse.major_axis_direction();
        assert_eq!(
            major_dir,
            Direction3D::from_vector(Vector3D::unit_x()).unwrap()
        );

        // 短軸方向
        let minor_dir = ellipse.minor_axis_direction();
        assert_eq!(
            minor_dir,
            Direction3D::from_vector(Vector3D::unit_y()).unwrap()
        );

        // 基本的な直交性確認
        let dot1: f64 = major_dir.dot(&minor_dir);
        let dot2: f64 = major_dir.dot(&ellipse.normal());
        let dot3: f64 = minor_dir.dot(&ellipse.normal());
        assert!(dot1.abs() < 1e-10f64);
        assert!(dot2.abs() < 1e-10f64);
        assert!(dot3.abs() < 1e-10f64);
    }

    #[test]
    fn test_simple_parametric() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 4.0, 2.0).unwrap();

        // 基本的なパラメータでの点
        let point_0 = ellipse.point_at_parameter(0.0);
        assert!((point_0.x() - 4.0f64).abs() < 1e-10);
        assert!((point_0.y() - 0.0f64).abs() < 1e-10);
        assert!((point_0.z() - 0.0f64).abs() < 1e-10);

        let point_pi2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!((point_pi2.x() - 0.0f64).abs() < 1e-10);
        assert!((point_pi2.y() - 2.0f64).abs() < 1e-10);
        assert!((point_pi2.z() - 0.0f64).abs() < 1e-10);
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let ellipse =
            Ellipse3D::xy_aligned(Point3D::new(0.0f32, 0.0f32, 0.0f32), 3.0f32, 2.0f32).unwrap();

        assert_eq!(ellipse.semi_major_axis(), 3.0f32);
        assert_eq!(ellipse.semi_minor_axis(), 2.0f32);

        let area = ellipse.area();
        let expected_area = std::f32::consts::PI * 3.0f32 * 2.0f32;
        assert!((area - expected_area).abs() < TOLERANCE_F32);
    }

    #[test]
    fn test_closest_point_projection() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 4.0, 2.0).unwrap();
        let closest =
            <Ellipse3D<f64> as Ellipse3DProjection<f64>>::closest_point(&ellipse, (10.0, 0.0, 3.0));

        assert!((closest.0 - 4.0).abs() < 1e-4);
        assert!(closest.1.abs() < 1e-4);
        assert!(closest.2.abs() < 1e-4);
    }

    #[test]
    fn test_distance_and_closest_point_to_curve() {
        use analysis::test_constants::TOLERANCE_F64;
        use geo_contracts::Ellipse3DDistance;

        // XY 平面上、長半軸 2（X 軸方向）・短半軸 1 の楕円
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            2.0,
            1.0,
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        // 中心から楕円までの距離は短半軸の長さ
        let center_distance = Ellipse3DDistance::distance_to_point(&ellipse, (0.0, 0.0, 0.0));
        assert!((center_distance - 1.0).abs() < TOLERANCE_F64);

        // 平面内部の点。距離と最近点は曲線のパラメータ方程式の数値解で求めた値
        let inside_distance = Ellipse3DDistance::distance_to_point(&ellipse, (1.0, 0.5, 0.0));
        assert!((inside_distance - 0.349_605_694_569_673).abs() < TOLERANCE_F64);

        // 平面外の点は、平面へ投影した点の最近点を最近点とする
        let closest = ellipse.closest_point(&Point3D::new(1.0, 0.5, 3.0));
        assert!(
            closest.distance_to(&Point3D::new(
                1.110_726_977_965_884,
                0.831_607_717_078_608,
                0.0
            )) < TOLERANCE_F64
        );
        let off_plane_distance = Ellipse3DDistance::distance_to_point(&ellipse, (1.0, 0.5, 3.0));
        let expected = (0.349_605_694_569_673_f64.powi(2) + 9.0).sqrt();
        assert!((off_plane_distance - expected).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_contains_and_classify_point() {
        use geo_contracts::{Ellipse3DContainment, PointClassification};

        // XY 平面上、長半軸 2（X 軸方向）・短半軸 1 の楕円
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            2.0,
            1.0,
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let tolerance = 1e-3;

        // contains_point は楕円上にあるかを判定する（同名で領域を判定するメソッドはない）
        assert!(ellipse.contains_point(&Point3D::new(2.0, 0.0, 0.0), tolerance));
        assert!(!ellipse.contains_point(&Point3D::new(0.0, 0.0, 0.0), tolerance));

        let classify =
            |x: f64, y: f64, z: f64| ellipse.classify_point(&Point3D::new(x, y, z), tolerance);
        assert_eq!(classify(0.0, 0.0, 0.0), PointClassification::Inside);
        assert_eq!(classify(0.0, 1.0, 0.0), PointClassification::OnBoundary);
        assert_eq!(classify(0.0, 1.5, 0.0), PointClassification::Outside);

        // 平面外の点
        assert_eq!(
            classify(2.0, 0.0, 0.5 * tolerance),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(0.0, 0.0, 0.5 * tolerance),
            PointClassification::Inside
        );
        assert_eq!(
            classify(0.0, 0.0, 2.0 * tolerance),
            PointClassification::Outside
        );

        // trait定義は既定の距離トレランスで判定する
        assert!(Ellipse3DContainment::contains_point(
            &ellipse,
            (-2.0, 0.0, 0.0)
        ));
        assert_eq!(
            Ellipse3DContainment::classify_point(&ellipse, (0.5, 0.0, 0.0)),
            PointClassification::Inside
        );
    }

    #[test]
    fn test_parameter_for_point_returns_parameter_angle_of_closest_point() {
        use analysis::test_constants::TOLERANCE_F64;

        // XY 平面上、長半軸 2（X 軸方向）・短半軸 1 の楕円
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            2.0,
            1.0,
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        // 楕円上の点は、その点を与えるパラメータ角（中心から見た方向の角度ではない）
        let t = std::f64::consts::FRAC_PI_4;
        let on_ellipse = ellipse.point_at_parameter(t);
        assert!((ellipse.parameter_for_point(&on_ellipse) - t).abs() < TOLERANCE_F64);

        // 楕円の内部・平面外の点は、最近点のパラメータ角
        for point in [Point3D::new(1.0, 0.5, 0.0), Point3D::new(-1.5, -0.2, 2.0)] {
            let parameter = ellipse.parameter_for_point(&point);
            let closest = ellipse.closest_point(&point);
            assert!(ellipse.point_at_parameter(parameter).distance_to(&closest) < TOLERANCE_F64);
            assert!((0.0..std::f64::consts::TAU).contains(&parameter));
        }
    }
}
