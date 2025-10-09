//! Ellipse tests - 完全分離型テスト構造
//!
//! ellipse.rs から分離されたテストコード。
//! test_utils.rs を活用した統一されたテスト環境。

#[cfg(test)]
mod tests {
    // テストユーティリティを使用（統一された型とヘルパー）
    use crate::unit_tests::test_utils::helpers::*;
    use crate::unit_tests::test_utils::*;

    // マクロのインポート
    use crate::{assert_approx_eq, assert_point_approx_eq};

    // 必要な型のみを明示的にインポート
    use crate::geometry2d::Ellipse;
    use geo_foundation::Angle;
    use std::f64::consts::PI;

    #[test]
    fn test_ellipse_creation() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();

        assert_point_approx_eq!(&ellipse.center(), &center);
        assert_approx_eq!(ellipse.major_radius(), 3.0);
        assert_approx_eq!(ellipse.minor_radius(), 2.0);
        assert_approx_eq!(ellipse.rotation().to_radians(), 0.0);
    }

    #[test]
    fn test_ellipse_creation_with_rotation() {
        let center = TestPoint2D::new(1.0, 1.0);
        let rotation = Angle::from_radians(PI / 4.0); // 45度
        let ellipse = Ellipse::new(center, 4.0, 2.0, rotation).unwrap();

        assert_point_approx_eq!(&ellipse.center(), &center);
        assert_approx_eq!(ellipse.major_radius(), 4.0);
        assert_approx_eq!(ellipse.minor_radius(), 2.0);
        assert_approx_eq!(ellipse.rotation().to_radians(), rotation.to_radians());
    }

    #[test]
    fn test_ellipse_invalid_parameters() {
        let center = TestPoint2D::new(0.0, 0.0);

        // 負の半径
        assert!(Ellipse::axis_aligned(center, -1.0, 2.0).is_err());
        assert!(Ellipse::axis_aligned(center, 2.0, -1.0).is_err());

        // 短軸が長軸より長い
        assert!(Ellipse::axis_aligned(center, 2.0, 3.0).is_err());
    }

    #[test]
    fn test_ellipse_area() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();

        let expected_area = PI * 3.0 * 2.0;
        assert_approx_eq!(ellipse.area(), expected_area);
    }

    #[test]
    fn test_ellipse_eccentricity() {
        let center = TestPoint2D::new(0.0, 0.0);

        // 円の場合
        let circle = Ellipse::axis_aligned(center, 2.0, 2.0).unwrap();
        assert_approx_eq!(circle.eccentricity(), 0.0);

        // 楕円の場合
        let ellipse = Ellipse::axis_aligned(center, 5.0, 3.0).unwrap();
        let expected_eccentricity = (1.0f64 - (3.0 * 3.0) / (5.0 * 5.0)).sqrt();
        assert_approx_eq!(ellipse.eccentricity(), expected_eccentricity);
    }

    #[test]
    fn test_ellipse_contains_point() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();

        // 中心点
        assert!(ellipse.contains_point(&center));

        // 楕円内部の点
        assert!(ellipse.contains_point(&TestPoint2D::new(1.0, 1.0)));

        // 楕円外部の点
        assert!(!ellipse.contains_point(&TestPoint2D::new(4.0, 0.0)));
        assert!(!ellipse.contains_point(&TestPoint2D::new(0.0, 3.0)));
    }

    #[test]
    fn test_ellipse_on_boundary() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();

        // 長軸の端点
        assert!(ellipse.on_boundary(&TestPoint2D::new(3.0, 0.0)));
        assert!(ellipse.on_boundary(&TestPoint2D::new(-3.0, 0.0)));

        // 短軸の端点
        assert!(ellipse.on_boundary(&TestPoint2D::new(0.0, 2.0)));
        assert!(ellipse.on_boundary(&TestPoint2D::new(0.0, -2.0)));
    }

    #[test]
    fn test_ellipse_bounding_box() {
        let center = TestPoint2D::new(1.0, 1.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();

        let bbox = ellipse.bounding_box();
        assert_point_approx_eq!(&bbox.min, &TestPoint2D::new(-2.0, -1.0));
        assert_point_approx_eq!(&bbox.max, &TestPoint2D::new(4.0, 3.0));
    }

    #[test]
    fn test_ellipse_scale() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();
        let scaled = ellipse.scale(2.0);

        assert_approx_eq!(scaled.major_radius(), 6.0);
        assert_approx_eq!(scaled.minor_radius(), 4.0);
        assert_point_approx_eq!(&scaled.center(), &center);
    }

    #[test]
    fn test_ellipse_translate() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();
        let vector = TestVector2D::new(2.0, 3.0);
        let translated = ellipse.translate(&vector);

        assert_point_approx_eq!(&translated.center(), &TestPoint2D::new(2.0, 3.0));
        assert_approx_eq!(translated.major_radius(), 3.0);
        assert_approx_eq!(translated.minor_radius(), 2.0);
    }

    #[test]
    fn test_ellipse_from_circle() {
        let center = TestPoint2D::new(1.0, 2.0);
        let circle = TestCircle::new(center, 5.0);
        let ellipse = Ellipse::from_circle(&circle);

        assert_point_approx_eq!(&ellipse.center(), &center);
        assert_approx_eq!(ellipse.major_radius(), 5.0);
        assert_approx_eq!(ellipse.minor_radius(), 5.0);
        assert!(ellipse.is_circle());
    }

    #[test]
    fn test_ellipse_foci() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 5.0, 3.0).unwrap();
        let (f1, f2) = ellipse.foci();

        let focal_distance = ellipse.focal_distance();
        assert_point_approx_eq!(&f1, &TestPoint2D::new(focal_distance, 0.0));
        assert_point_approx_eq!(&f2, &TestPoint2D::new(-focal_distance, 0.0));
    }

    #[test]
    fn test_ellipse_is_circle() {
        let center = TestPoint2D::new(0.0, 0.0);

        // 円
        let circle = Ellipse::axis_aligned(center, 2.0, 2.0).unwrap();
        assert!(circle.is_circle());

        // 楕円
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();
        assert!(!ellipse.is_circle());
    }

    #[test]
    fn test_ellipse_point_at_angle() {
        let center = TestPoint2D::new(0.0, 0.0);
        let ellipse = Ellipse::axis_aligned(center, 3.0, 2.0).unwrap();

        // 0度の点（長軸上）
        let point_0 = ellipse.point_at_angle(0.0);
        assert!(approx_eq_f64(point_0.x(), 3.0));
        assert!(approx_eq_f64(point_0.y(), 0.0));

        // 90度の点（短軸上）
        let point_90 = ellipse.point_at_angle(PI / 2.0);
        assert!(approx_eq_f64(point_90.x(), 0.0));
        assert!(approx_eq_f64(point_90.y(), 2.0));
    }

    #[test]
    fn test_ellipse_integration_with_circles() {
        // 楕円と円の統合テスト（test_utils のおかげで簡単）
        let ellipse_center = TestPoint2D::new(1.0, 1.0);
        let ellipse = Ellipse::axis_aligned(ellipse_center, 4.0, 3.0).unwrap();

        // 楕円から円を生成
        let circle_from_ellipse = TestCircle::new(ellipse.center(), ellipse.minor_radius());

        // 円が楕円内に含まれることを確認
        assert!(ellipse.contains_point(&circle_from_ellipse.center()));
        assert_approx_eq!(circle_from_ellipse.radius(), 3.0);
    }

    #[test]
    fn test_ellipse_with_test_helpers() {
        // test_utils のヘルパー関数を活用
        let points = test_points_2d();
        let center = points[0]; // 原点

        let ellipse = Ellipse::axis_aligned(center, 2.0, 1.5).unwrap();

        // 基本プロパティの確認
        assert!(ellipse.area() > 0.0);
        assert!(ellipse.major_radius() >= ellipse.minor_radius());
        assert!(ellipse.eccentricity() >= 0.0 && ellipse.eccentricity() < 1.0);
    }
}
