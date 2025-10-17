//! EllipseArc2D実装のテスト

#[cfg(test)]
mod ellipse_arc_2d_tests {
    use crate::ellipse_2d::Ellipse2D;
    use crate::ellipse_arc_2d::EllipseArc2D;
    use crate::Point2D;
    use geo_foundation::Angle;

    #[test]
    fn test_ellipse_arc_creation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI),
        );

        assert_eq!(arc.ellipse().center(), center);
        assert_eq!(arc.start_angle().to_radians(), 0.0);
        assert_eq!(arc.end_angle().to_radians(), std::f64::consts::PI);
    }

    #[test]
    fn test_arc_start_end_points() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let start_point = arc.start_point();
        let end_point = arc.end_point();

        // 開始点（t=0, 長軸の正の端）
        assert!((start_point.x() - 4.0).abs() < 1e-10);
        assert!(start_point.y().abs() < 1e-10);

        // 終了点（t=π/2, 短軸の正の端）
        assert!(end_point.x().abs() < 1e-10);
        assert!((end_point.y() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_arc_point_at_parameter() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI),
        );

        // パラメータ0.0での点（開始点）
        let point_start = arc.point_at_parameter(0.0);
        let expected_start = arc.start_point();
        assert!((point_start.x() - expected_start.x()).abs() < 1e-10);
        assert!((point_start.y() - expected_start.y()).abs() < 1e-10);

        // パラメータ1.0での点（終了点）
        let point_end = arc.point_at_parameter(1.0);
        let expected_end = arc.end_point();
        assert!((point_end.x() - expected_end.x()).abs() < 1e-10);
        assert!((point_end.y() - expected_end.y()).abs() < 1e-10);
    }

    #[test]
    fn test_arc_length_calculation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

        // 半楕円
        let half_arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI),
        );
        let half_length = half_arc.arc_length();

        // 全楕円
        let full_arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(2.0 * std::f64::consts::PI),
        );
        let full_length = full_arc.arc_length();

        // 半楕円の長さは全楕円の約半分
        assert!((full_length - 2.0 * half_length).abs() < 0.1);
        assert!(half_length > 0.0);
        assert!(full_length > 0.0);
    }

    #[test]
    fn test_arc_subdivision() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let points = arc.subdivide(4);
        assert_eq!(points.len(), 5); // 4セグメント = 5点

        // 最初と最後の点が開始点と終了点に一致
        let first_point = &points[0];
        let last_point = &points[points.len() - 1];
        let start_point = arc.start_point();
        let end_point = arc.end_point();

        assert!((first_point.x() - start_point.x()).abs() < 1e-10);
        assert!((first_point.y() - start_point.y()).abs() < 1e-10);
        assert!((last_point.x() - end_point.x()).abs() < 1e-10);
        assert!((last_point.y() - end_point.y()).abs() < 1e-10);
    }

    #[test]
    fn test_arc_angle_span() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let span = arc.angle_span();
        assert!((span.to_radians() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
}
