//! SphericalSurface3D collision tests

#[cfg(test)]
mod tests {
    use crate::{InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSurface3D, Vector3D};
    use geo_foundation::extensions::BasicCollision;

    /// テスト用の標準的な球面サーフェスを作成
    /// - 中心: (0, 0, 0)
    /// - 半径: 1.0
    fn create_test_sphere() -> SphericalSurface3D<f64> {
        SphericalSurface3D::new_at_origin(1.0).unwrap()
    }

    #[test]
    fn test_point_on_surface() {
        let sphere = create_test_sphere();
        let point = Point3D::new(1.0, 0.0, 0.0);
        assert!(sphere.intersects(&point, 1e-10));
    }

    #[test]
    fn test_point_outside() {
        let sphere = create_test_sphere();
        let point = Point3D::new(2.0, 0.0, 0.0);
        assert!(!sphere.intersects(&point, 1e-10));
    }

    #[test]
    fn test_point_inside() {
        let sphere = create_test_sphere();
        let point = Point3D::new(0.5, 0.0, 0.0);
        assert!(!sphere.intersects(&point, 1e-10));
    }

    // TODO: LineSegment3D との距離計算実装が完了したらテストを追加
    // #[test]
    // fn test_line_segment_intersection() {
    //     let sphere = create_test_sphere();
    //     let line = LineSegment3D::new(
    //         Point3D::new(-2.0, 0.0, 0.0),
    //         Point3D::new(2.0, 0.0, 0.0),
    //     )
    //     .unwrap();
    //     assert!(sphere.intersects(&line, 1e-10));
    // }

    #[test]
    fn test_line_segment_no_intersection() {
        let sphere = create_test_sphere();
        let line = LineSegment3D::new(
            Point3D::new(3.0, 0.0, 0.0),
            Point3D::new(4.0, 0.0, 0.0),
        )
        .unwrap();
        assert!(!sphere.intersects(&line, 1e-10));
    }

    // TODO: Ray3D との距離計算実装が完了したらテストを追加
    // #[test]
    // fn test_ray_intersection() {
    //     let sphere = create_test_sphere();
    //     let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    //     assert!(sphere.intersects(&ray, 1e-10));
    // }

    // TODO: InfiniteLine3D との距離計算実装が完了したらテストを追加
    // #[test]
    // fn test_infinite_line_intersection() {
    //     let sphere = create_test_sphere();
    //     let line = InfiniteLine3D::new(
    //         Point3D::new(-2.0, 0.0, 0.0),
    //         Vector3D::new(1.0, 0.0, 0.0),
    //     )
    //     .unwrap();
    //     assert!(sphere.intersects(&line, 1e-10));
    // }

    // TODO: Plane3D との衝突判定実装が完了したらテストを追加
    // #[test]
    // fn test_plane_intersection() {
    //     let sphere = create_test_sphere();
    //     let plane = Plane3D::from_point_and_normal(
    //         Point3D::new(0.0, 0.0, 0.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     )
    //     .unwrap();
    //     assert!(sphere.intersects(&plane, 1e-10));
    // }

    // #[test]
    // fn test_plane_tangent() {
    //     let sphere = create_test_sphere();
    //     let plane = Plane3D::from_point_and_normal(
    //         Point3D::new(0.0, 0.0, 1.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     )
    //     .unwrap();
    //     assert!(sphere.intersects(&plane, 1e-6));
    // }

    // #[test]
    // fn test_plane_no_intersection() {
    //     let sphere = create_test_sphere();
    //     let plane = Plane3D::from_point_and_normal(
    //         Point3D::new(0.0, 0.0, 5.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     )
    //     .unwrap();
    //     assert!(!sphere.intersects(&plane, 1e-10));
    // }

    #[test]
    fn test_sphere_self_intersection() {
        let sphere1 = create_test_sphere();
        let sphere2 = create_test_sphere();
        // 同一の球面
        assert!(sphere1.intersects(&sphere2, 1e-10));
    }

    #[test]
    fn test_sphere_no_intersection() {
        let sphere1 = create_test_sphere();
        let sphere2 = SphericalSurface3D::new_standard(Point3D::new(5.0, 0.0, 0.0), 1.0).unwrap();
        // 離れた球面
        assert!(!sphere1.intersects(&sphere2, 1e-10));
    }

    #[test]
    fn test_distance_to_point() {
        let sphere = create_test_sphere();
        let point = Point3D::new(2.0, 0.0, 0.0);
        let distance = sphere.distance_to(&point);
        // 球面までの距離 = 2.0 - 1.0 = 1.0
        assert!((distance - 1.0).abs() < 1e-10);
    }
}
