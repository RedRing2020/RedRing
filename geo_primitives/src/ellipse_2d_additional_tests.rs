//! Ellipse2D実装の追加テスト

#[cfg(test)]
mod ellipse_2d_additional_tests {
    use crate::ellipse_2d::Ellipse2D;
    use crate::Point2D;

    #[test]
    fn test_basic_ellipse_creation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).expect("楕円の作成に失敗");

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.semi_major(), 5.0);
        assert_eq!(ellipse.semi_minor(), 3.0);
        assert_eq!(ellipse.rotation(), 0.0);
    }

    #[test]
    fn test_ellipse_area_calculation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

        let area = ellipse.area();
        let expected = std::f64::consts::PI * 4.0 * 2.0; // π * a * b
        assert!((area - expected).abs() < 1e-10);
    }

    #[test]
    fn test_ellipse_point_evaluation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

        // t=0での点（長軸の正の端）
        let point_0 = ellipse.point_at_parameter(0.0);
        assert!((point_0.x() - 4.0_f64).abs() < 1e-10);
        assert!(point_0.y().abs() < 1e-10);

        // t=π/2での点（短軸の正の端）
        let point_pi_2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!(point_pi_2.x().abs() < 1e-10);
        assert!((point_pi_2.y() - 2.0_f64).abs() < 1e-10);
    }

    #[test]
    fn test_ellipse_contains_point() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let tolerance = 1e-10;

        // 中心点
        assert!(ellipse.contains_point(&center, tolerance));

        // 内部の点
        let inside = Point2D::new(1.0, 0.5);
        assert!(ellipse.contains_point(&inside, tolerance));

        // 外部の点
        let outside = Point2D::new(5.0, 3.0);
        assert!(!ellipse.contains_point(&outside, tolerance));
    }

    #[test]
    fn test_ellipse_bounding_box() {
        let center = Point2D::new(1.0, 2.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

        let bbox = ellipse.bounding_box();

        // 境界ボックスに中心が含まれる
        assert!(bbox.contains_point(&center));
    }

    #[test]
    fn test_circle_conversion() {
        let center = Point2D::new(0.0, 0.0);

        // 円に近い楕円
        let circle_ellipse = Ellipse2D::new(center, 3.0, 3.0, 0.0).expect("楕円の作成に失敗");
        assert!(circle_ellipse.to_circle().is_some());
        assert!(circle_ellipse.is_circle(1e-10));

        // 明確な楕円
        let regular_ellipse = Ellipse2D::new(center, 5.0, 2.0, 0.0).expect("楕円の作成に失敗");
        assert!(regular_ellipse.to_circle().is_none());
        assert!(!regular_ellipse.is_circle(1e-10));
    }
}
