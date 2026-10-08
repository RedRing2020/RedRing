//! Ellipse2D実装の追加テスト

#[cfg(test)]
mod tests {
    use crate::ellipse_2d::Ellipse2D;
    use crate::Point2D;
    use analysis::test_constants::TOLERANCE_F64;

    #[test]
    fn test_basic_ellipse_creation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).expect("楕円の作成に失敗");

        assert_eq!(ellipse.center_internal(), center);
        assert_eq!(ellipse.semi_major_internal(), 5.0);
        assert_eq!(ellipse.semi_minor_internal(), 3.0);
        assert_eq!(ellipse.rotation(), 0.0);
    }

    #[test]
    fn test_ellipse_area_calculation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

        let area = ellipse.area();
        let expected = std::f64::consts::PI * 4.0 * 2.0; // π * a * b
        assert!((area - expected).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_point_evaluation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

        // Ellipse core parameter は local angle domain (0..2π)
        // t=0での点（長軸の正の端）
        let point_0 = ellipse.point_at_parameter(0.0);
        assert!((point_0.x() - 4.0_f64).abs() < TOLERANCE_F64);
        assert!(point_0.y().abs() < TOLERANCE_F64);

        // t=π/2での点（短軸の正の端）
        let point_pi_2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!(point_pi_2.x().abs() < TOLERANCE_F64);
        assert!((point_pi_2.y() - 2.0_f64).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_contains_point() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let tolerance = TOLERANCE_F64;

        // 楕円上の点
        assert!(ellipse.contains_point(&Point2D::new(4.0, 0.0), tolerance));
        assert!(ellipse.contains_point(&Point2D::new(0.0, -2.0), tolerance));

        // 楕円が囲む領域の内部の点は楕円上にない
        assert!(!ellipse.contains_point(&center, tolerance));
        assert!(!ellipse.contains_point(&Point2D::new(1.0, 0.5), tolerance));

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
        assert!(circle_ellipse.is_circle(TOLERANCE_F64));

        // 明確な楕円
        let regular_ellipse = Ellipse2D::new(center, 5.0, 2.0, 0.0).expect("楕円の作成に失敗");
        assert!(regular_ellipse.to_circle().is_none());
        assert!(!regular_ellipse.is_circle(TOLERANCE_F64));
    }

    #[test]
    fn test_ellipse_distance_and_closest_point_to_curve() {
        // 長半軸 2・短半軸 1 の楕円を 30° 回転し、中心を (1, -1) に置く
        let rotation = std::f64::consts::FRAC_PI_6;
        let center = Point2D::new(1.0, -1.0);
        let ellipse = Ellipse2D::new(center, 2.0, 1.0, rotation).expect("楕円の作成に失敗");
        let to_world = |x: f64, y: f64| {
            Point2D::new(
                center.x() + x * rotation.cos() - y * rotation.sin(),
                center.y() + x * rotation.sin() + y * rotation.cos(),
            )
        };

        // 中心から楕円までの距離は短半軸の長さ
        assert!((ellipse.distance_to_point(&center) - 1.0).abs() < TOLERANCE_F64);

        // 局所座標 (1, 0.5) の内部の点。距離と最近点は曲線のパラメータ方程式の数値解で求めた値
        let inside = to_world(1.0, 0.5);
        assert!((ellipse.distance_to_point(&inside) - 0.349_605_694_569_673).abs() < TOLERANCE_F64);
        let closest = ellipse.closest_point_to(&inside);
        let expected = to_world(1.110_726_977_965_884, 0.831_607_717_078_608);
        assert!(closest.distance_to(&expected) < TOLERANCE_F64);
        assert!(
            (closest.distance_to(&inside) - ellipse.distance_to_point(&inside)).abs()
                < TOLERANCE_F64
        );
    }

    #[test]
    fn test_ellipse_classify_point() {
        use geo_contracts::{Ellipse2DContainment, PointClassification};

        // 長半軸 4・短半軸 2 の楕円を 90° 回転（長軸が Y 軸方向）
        let ellipse = Ellipse2D::new(
            Point2D::new(0.0, 0.0),
            4.0,
            2.0,
            std::f64::consts::FRAC_PI_2,
        )
        .expect("楕円の作成に失敗");
        let tolerance = 1e-3;
        let classify = |x: f64, y: f64| ellipse.classify_point(&Point2D::new(x, y), tolerance);

        assert_eq!(classify(0.0, 0.0), PointClassification::Inside);
        assert_eq!(classify(0.0, 4.0), PointClassification::OnBoundary);
        assert_eq!(classify(-2.0, 0.0), PointClassification::OnBoundary);
        assert_eq!(classify(3.0, 0.0), PointClassification::Outside);

        // 許容誤差の境目
        assert_eq!(
            classify(0.0, 4.0 - 0.5 * tolerance),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(0.0, 4.0 + 0.5 * tolerance),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(0.0, 4.0 - 2.0 * tolerance),
            PointClassification::Inside
        );
        assert_eq!(
            classify(0.0, 4.0 + 2.0 * tolerance),
            PointClassification::Outside
        );

        // classify_point が OnBoundary のときだけ contains_point が真
        for (x, y) in [
            (0.0, 0.0),
            (0.0, 4.0),
            (3.0, 0.0),
            (0.0, 4.0 + 2.0 * tolerance),
        ] {
            let point = Point2D::new(x, y);
            assert_eq!(
                ellipse.contains_point(&point, tolerance),
                ellipse.classify_point(&point, tolerance) == PointClassification::OnBoundary
            );
        }

        // trait定義は既定の距離トレランスで判定する
        assert!(Ellipse2DContainment::contains_point(&ellipse, (0.0, -4.0)));
        assert_eq!(
            Ellipse2DContainment::classify_point(&ellipse, (1.0, 1.0)),
            PointClassification::Inside
        );
    }
}
