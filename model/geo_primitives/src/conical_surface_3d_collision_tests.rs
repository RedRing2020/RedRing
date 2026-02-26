//! ConicalSurface3D - Collision Tests
//!
//! 3次元円錐サーフェスの衝突判定テスト

#[cfg(test)]
mod tests {
    use crate::{ConicalSurface3D, LineSegment3D, Plane3D, Point3D, Ray3D, Vector3D};
    use geo_foundation::extensions::BasicCollision;

    const TOLERANCE: f64 = 1e-10;

    fn create_test_cone() -> ConicalSurface3D<f64> {
        // 原点中心、Z軸方向、半径1.0、半頂角45度の円錐サーフェス
        ConicalSurface3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            std::f64::consts::PI / 4.0, // 45度
        )
        .unwrap()
    }

    #[test]
    fn test_point_on_surface() {
        let cone = create_test_cone();
        // 基準点での半径1.0の位置
        let surface_point = Point3D::new(1.0, 0.0, 0.0);

        assert!(cone.intersects(&surface_point, TOLERANCE));
        assert!(cone.overlaps(&surface_point, TOLERANCE));
        assert!(cone.distance_to(&surface_point) < TOLERANCE);
    }

    #[test]
    fn test_point_outside() {
        let cone = create_test_cone();
        let outside_point = Point3D::new(5.0, 0.0, 0.0);

        assert!(!cone.intersects(&outside_point, TOLERANCE));
        assert!(!cone.overlaps(&outside_point, TOLERANCE));
        assert!(cone.distance_to(&outside_point) > 3.0);
    }

    // TODO: 円錐サーフェスと円の距離計算の改善が必要
    // #[test]
    // fn test_circle_intersection() {
    //     let cone = create_test_cone();
    //     let circle = Circle3D::new(
    //         Point3D::new(0.0, 0.0, 0.0),
    //         Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
    //         0.5
    //     ).unwrap();
    //
    //     assert!(cone.intersects(&circle, TOLERANCE));
    //     assert!(cone.overlaps(&circle, TOLERANCE));
    // }

    #[test]
    fn test_line_segment_through() {
        let cone = create_test_cone();
        let line =
            LineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 1.0)).unwrap();

        assert!(cone.intersects(&line, TOLERANCE));
    }

    // TODO: 三角形の各頂点が円錐サーフェス上にあるテストデータが必要
    // #[test]
    // fn test_triangle_intersection() {
    //     let cone = create_test_cone();
    //     let triangle = Triangle3D::new(
    //         Point3D::new(0.5, 0.0, 0.0),
    //         Point3D::new(1.5, 0.0, 0.0),
    //         Point3D::new(1.0, 0.5, 0.0),
    //     ).unwrap();
    //
    //     assert!(cone.intersects(&triangle, TOLERANCE));
    // }

    #[test]
    fn test_plane_intersection() {
        let cone = create_test_cone();
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        assert!(cone.intersects(&plane, TOLERANCE));
    }

    #[test]
    fn test_ray_intersection() {
        let cone = create_test_cone();
        let ray = Ray3D::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        assert!(cone.intersects(&ray, TOLERANCE));
    }

    #[test]
    fn test_distance_to_point_above_cone() {
        let cone = create_test_cone();
        // 円錐の上方（Z=1の位置では半径が2.0になる）
        let point = Point3D::new(0.0, 0.0, 1.0);

        let distance = cone.distance_to(&point);
        // 軸上の点なので期待半径は2.0、実際の半径方向距離は0.0
        assert!((distance - 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_distance_to_point_on_extended_surface() {
        let cone = create_test_cone();
        // Z=1での円錐サーフェス上（半径2.0）
        let point = Point3D::new(2.0, 0.0, 1.0);

        let distance = cone.distance_to(&point);
        assert!(distance < TOLERANCE);
    }

    // TODO: 同一円錐の交差判定ロジックの改善が必要
    // #[test]
    // fn test_conical_surface_self_intersection() {
    //     let cone1 = create_test_cone();
    //     let cone2 = create_test_cone();
    //
    //     assert!(cone1.intersects(&cone2, TOLERANCE));
    // }
}
