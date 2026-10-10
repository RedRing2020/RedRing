//! 円弧・楕円弧の角度範囲の判定のテスト

#[cfg(test)]
mod tests {
    use crate::{
        Arc2D, Arc3D, Circle2D, Direction3D, Ellipse2D, Ellipse3D, EllipseArc2D, EllipseArc3D,
        Point2D, Point3D, Vector3D,
    };
    use geo_contracts::{Angle, Arc2DTrimRange, Arc3DTrimRange, EllipseArc3DTrimRange};

    fn deg(value: f64) -> Angle<f64> {
        Angle::from_degrees(value)
    }

    fn arc2d(start: f64, end: f64) -> Arc2D<f64> {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        Arc2D::new(circle, deg(start), deg(end)).unwrap()
    }

    fn arc3d(start: f64, end: f64) -> Arc3D<f64> {
        // XY 平面上、開始方向 +X・法線 +Z の円弧
        Arc3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            deg(start),
            deg(end),
        )
        .unwrap()
    }

    #[test]
    fn arc_contains_angle_normalizes_angle() {
        let arc = arc2d(0.0, 90.0);

        // 周期の違う同じ角度も範囲に含む
        assert!(arc.contains_angle(deg(45.0)));
        assert!(arc.contains_angle(deg(405.0)));
        assert!(arc.contains_angle(deg(-315.0)));
        assert!(!arc.contains_angle(deg(180.0)));

        // 0° を跨ぐ範囲
        let crossing = arc2d(300.0, 60.0);
        assert!(crossing.contains_angle(deg(0.0)));
        assert!(crossing.contains_angle(deg(330.0)));
        assert!(!crossing.contains_angle(deg(180.0)));
    }

    #[test]
    fn full_circle_arc_contains_every_angle() {
        let arc = arc2d(0.0, 360.0);
        assert!(arc.contains_angle(deg(180.0)));
        assert!(Arc2DTrimRange::contains_angle(&arc, std::f64::consts::PI));
        assert!(arc.contains_point_angle(&Point2D::new(-2.0, 0.0)));

        let arc = arc3d(0.0, 360.0);
        assert!(Arc3DTrimRange::contains_angle(&arc, std::f64::consts::PI));
    }

    #[test]
    fn arc2d_contains_point_angle_uses_direction_from_center() {
        let arc = arc2d(0.0, 90.0);
        assert!(arc.contains_point_angle(&Point2D::new(1.0, 1.0)));
        assert!(arc.contains_point_angle(&Point2D::new(5.0, 5.0)));
        assert!(!arc.contains_point_angle(&Point2D::new(-1.0, 1.0)));
    }

    #[test]
    fn arc3d_contains_point_angle_matches_point_at_angle() {
        let arc = arc3d(0.0, 90.0);

        // point_at_angle と同じ向き（開始方向から法線まわりに反時計回り）で角度を測る
        let inside = arc.point_at_angle(45.0_f64.to_radians());
        let outside = arc.point_at_angle(315.0_f64.to_radians());
        assert!(arc.contains_point_angle(&inside));
        assert!(!arc.contains_point_angle(&outside));

        // 円弧平面外の点は平面へ投影した点で判定する
        assert!(arc.contains_point_angle(&Point3D::new(1.0, 1.0, 5.0)));
        assert!(!arc.contains_point_angle(&Point3D::new(1.0, -1.0, 5.0)));
    }

    #[test]
    fn ellipse_arc_contains_point_angle_uses_parameter_angle() {
        // 長半軸 2・短半軸 1 の楕円の、パラメータ角 30°〜60° の楕円弧
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 2.0, 1.0, 0.0).unwrap();
        let arc = EllipseArc2D::new(ellipse, deg(30.0), deg(60.0));

        // パラメータ角 35° の点は、中心から見た方向の角度（約 19.3°）では範囲外だが、パラメータ角では範囲内
        let point = ellipse.point_at_parameter(35.0_f64.to_radians());
        assert!(arc.contains_point_angle(&point));
        assert!(arc.contains_point(&point, 1e-9));

        // パラメータ角 20° の点は範囲外
        let point = ellipse.point_at_parameter(20.0_f64.to_radians());
        assert!(!arc.contains_point_angle(&point));

        // 中心の点は、最近点（短軸の端点、パラメータ角 90°）の角度で判定する
        assert!(!arc.contains_point_angle(&Point2D::new(0.0, 0.0)));
    }

    #[test]
    fn ellipse_arc3d_contains_angle_and_point_angle() {
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
            1.0,
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let arc = EllipseArc3D::new(ellipse, deg(30.0), deg(60.0));

        // 周期の違う同じ角度と、範囲の端の許容誤差
        assert!(EllipseArc3DTrimRange::contains_angle(
            &arc,
            (360.0_f64 + 45.0).to_radians()
        ));
        assert!(arc.contains_angle(deg(60.0)));
        assert!(!arc.contains_angle(deg(90.0)));

        let point = ellipse.point_at_parameter(35.0_f64.to_radians());
        assert!(arc.contains_point_angle(&point));
        assert!(arc.contains_point_angle(&Point3D::new(point.x(), point.y(), 3.0)));
    }

    #[test]
    fn arc_distance_uses_endpoints_outside_angle_range() {
        use geo_contracts::{Arc2DContainment, Arc2DDistance, Arc3DContainment, Arc3DDistance};

        // 中心 (0, 0)・半径 2・0°〜90° の円弧
        let arc = arc2d(0.0, 90.0);
        // 角度範囲内は母円までの距離
        assert!(
            (Arc2DDistance::distance_to_point(&arc, (3.0, 3.0)) - (18.0_f64.sqrt() - 2.0)).abs()
                < 1e-12
        );
        // 角度範囲外は近い端点 (0, 2) までの距離
        assert!(
            (Arc2DDistance::distance_to_point(&arc, (-3.0, 0.0)) - 13.0_f64.sqrt()).abs() < 1e-12
        );
        assert!(!Arc2DContainment::contains_point(&arc, (-2.0, 0.0)));
        assert!(Arc2DContainment::contains_point(&arc, (0.0, 2.0)));

        // XY 平面上、開始方向 +X・法線 +Z・半径 2・0°〜90° の円弧
        let arc = arc3d(0.0, 90.0);
        // 角度範囲内の平面外の点は、平面外の距離を含めた円弧までの距離
        assert!((Arc3DDistance::distance_to_point(&arc, (2.0, 0.0, 1.5)) - 1.5).abs() < 1e-12);
        let expected = (1.0_f64 + 1.0).sqrt(); // 半径方向 1・平面外 1
        assert!((Arc3DDistance::distance_to_point(&arc, (3.0, 0.0, 1.0)) - expected).abs() < 1e-12);
        // 角度範囲外は近い端点 (2, 0, 0) までの距離
        assert!((Arc3DDistance::distance_to_point(&arc, (2.0, -1.0, 0.0)) - 1.0).abs() < 1e-12);
        assert!(!Arc3DContainment::contains_point(&arc, (2.0, 0.0, 0.5)));
        assert!(Arc3DContainment::contains_point(&arc, (0.0, 2.0, 0.0)));
    }
}
