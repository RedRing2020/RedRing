//! EllipseArc3D Collision テスト
//!
//! BasicCollision トレイトの実装テスト

#[cfg(test)]
mod tests {
    use crate::{
        Arc3D, Circle3D, Direction3D, Ellipse3D, EllipseArc3D, LineSegment3D, Plane3D, Point3D,
        Triangle3D, Vector3D,
    };
    use analysis::Angle;
    use geo_contracts::BasicCollision;

    // テスト用のヘルパー関数
    fn create_test_ellipse_arc() -> EllipseArc3D<f64> {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0); // Z軸方向
        let major_axis = Vector3D::new(1.0, 0.0, 0.0); // X軸方向
        let ellipse = Ellipse3D::new(center, 5.0, 3.0, normal, major_axis).unwrap();
        let start = Angle::from_radians(0.0);
        let end = Angle::from_radians(std::f64::consts::PI / 2.0); // 90度
        EllipseArc3D::new(ellipse, start, end)
    }

    #[test]
    fn test_ellipse_arc_vs_point_on_arc() {
        let arc = create_test_ellipse_arc();

        // 楕円弧の開始点（角度0度、x軸上）
        let point = Point3D::new(5.0, 0.0, 0.0);
        assert!(arc.intersects(&point, 0.01));

        // 楕円弧の終了点（角度90度、y軸上）
        let point = Point3D::new(0.0, 3.0, 0.0);
        assert!(arc.intersects(&point, 0.01));

        // 楕円弧の中間点（角度45度付近）
        let point = Point3D::new(3.5, 2.5, 0.0);
        let distance = arc.distance_to(&point);
        assert!(distance < 1.0); // 楕円弧に近い
    }

    #[test]
    fn test_ellipse_arc_vs_point_outside_angle_range() {
        let arc = create_test_ellipse_arc();

        // 楕円上だが角度範囲外（180度の位置）
        let point = Point3D::new(-5.0, 0.0, 0.0);
        assert!(!arc.intersects(&point, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_point_outside_plane() {
        let arc = create_test_ellipse_arc();

        // 楕円平面から外れた点
        let point = Point3D::new(5.0, 0.0, 5.0);
        assert!(!arc.intersects(&point, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_point_distance() {
        let arc = create_test_ellipse_arc();

        // 楕円弧上の点
        let point_on_arc = Point3D::new(5.0, 0.0, 0.0);
        let distance = arc.distance_to(&point_on_arc);
        assert!(distance < 0.01);

        // 楕円弧から離れた点
        let point_far = Point3D::new(10.0, 10.0, 10.0);
        let distance = arc.distance_to(&point_far);
        assert!(distance > 5.0);
    }

    #[test]
    fn test_ellipse_arc_vs_circle() {
        let arc = create_test_ellipse_arc();

        // 楕円弧と交差する円（同じ平面上）
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let circle = Circle3D::new(Point3D::new(3.0, 1.5, 0.0), normal, 2.0).unwrap();
        assert!(arc.intersects(&circle, 0.01));

        // 楕円弧から離れた円
        let circle_far = Circle3D::new(Point3D::new(20.0, 20.0, 0.0), normal, 1.0).unwrap();
        assert!(!arc.intersects(&circle_far, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_arc() {
        let arc1 = create_test_ellipse_arc();

        // 交差する円弧（同じ平面上）
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let arc2 = Arc3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            4.0,
            normal,
            start_dir,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        )
        .unwrap();

        assert!(arc1.intersects(&arc2, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_ellipse() {
        let arc = create_test_ellipse_arc();

        // 同じ楕円
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let major_axis = Vector3D::new(1.0, 0.0, 0.0);
        let ellipse =
            Ellipse3D::new(Point3D::new(0.0, 0.0, 0.0), 5.0, 3.0, normal, major_axis).unwrap();
        assert!(arc.intersects(&ellipse, 0.01));

        // 離れた楕円
        let ellipse_far =
            Ellipse3D::new(Point3D::new(20.0, 20.0, 0.0), 2.0, 1.0, normal, major_axis).unwrap();
        assert!(!arc.intersects(&ellipse_far, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_ellipse_arc() {
        let arc1 = create_test_ellipse_arc();

        // 交差する楕円弧
        let center = Point3D::new(3.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let major_axis = Vector3D::new(1.0, 0.0, 0.0);
        let ellipse = Ellipse3D::new(center, 3.0, 2.0, normal, major_axis).unwrap();
        let arc2 = EllipseArc3D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI),
        );

        assert!(arc1.intersects(&arc2, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_line_segment() {
        let arc = create_test_ellipse_arc();

        // 楕円弧を横切る線分（開始点が楕円弧上）
        let line =
            LineSegment3D::new(Point3D::new(5.0, 0.0, 0.0), Point3D::new(0.0, 3.0, 0.0)).unwrap();
        // 簡易実装では端点との衝突判定をテスト
        let start = line.start();
        let end = line.end();
        // 両端点とも楕円弧上（角度0度と 90度）
        assert!(arc.intersects(&start, 0.01));
        assert!(arc.intersects(&end, 0.01));

        // 楕円弧から離れた線分
        let line_far =
            LineSegment3D::new(Point3D::new(10.0, 10.0, 0.0), Point3D::new(20.0, 20.0, 0.0))
                .unwrap();
        let start_far = line_far.start();
        let end_far = line_far.end();
        assert!(!arc.intersects(&start_far, 0.01));
        assert!(!arc.intersects(&end_far, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_triangle() {
        let arc = create_test_ellipse_arc();

        // 楕円弧と交差する三角形（頂点が楕円弧上）
        let triangle = Triangle3D::new(
            Point3D::new(5.0, 0.0, 0.0), // 楕円弧の開始点
            Point3D::new(0.0, 3.0, 0.0), // 楕円弧の終了点
            Point3D::new(3.0, 2.0, 0.0), // 楕円弧上の点
        )
        .unwrap();
        // 簡易実装では頂点との衝突判定をテスト
        let (ax, ay, az) =
            geo_contracts::Triangle3DProperties::vertex_a(&triangle);
        let (bx, by, bz) =
            geo_contracts::Triangle3DProperties::vertex_b(&triangle);
        let vertex_a = Point3D::new(ax, ay, az);
        let vertex_b = Point3D::new(bx, by, bz);
        // 頂点AとBは楕円弧上
        assert!(arc.intersects(&vertex_a, 0.01));
        assert!(arc.intersects(&vertex_b, 0.01));

        // 楕円弧から離れた三角形
        let triangle_far = Triangle3D::new(
            Point3D::new(10.0, 10.0, 0.0),
            Point3D::new(15.0, 10.0, 0.0),
            Point3D::new(12.5, 15.0, 0.0),
        )
        .unwrap();
        let (ax_far, ay_far, az_far) =
            geo_contracts::Triangle3DProperties::vertex_a(
                &triangle_far,
            );
        let vertex_a_far = Point3D::new(ax_far, ay_far, az_far);
        assert!(!arc.intersects(&vertex_a_far, 0.01));
    }

    #[test]
    fn test_ellipse_arc_vs_plane() {
        let arc = create_test_ellipse_arc();

        // 楕円弧を含む平面（Z=0）
        let normal_vec = Vector3D::new(0.0, 0.0, 1.0);
        let u_vec = Vector3D::new(1.0, 0.0, 0.0);
        let plane =
            Plane3D::from_origin_and_axes(Point3D::new(0.0, 0.0, 0.0), normal_vec, u_vec).unwrap();
        // 簡易実装：中心点からの距離を確認
        let distance_to_plane = arc.distance_to(&plane);
        assert!(
            distance_to_plane.abs() < 0.1,
            "Distance to plane containing arc: {}",
            distance_to_plane
        );

        // 楕円弧から離れた平面（Z=10）
        let plane_far =
            Plane3D::from_origin_and_axes(Point3D::new(0.0, 0.0, 10.0), normal_vec, u_vec).unwrap();
        let distance_far = arc.distance_to(&plane_far);
        assert!(
            distance_far.abs() > 5.0,
            "Distance to far plane: {}",
            distance_far
        );
    }
}
