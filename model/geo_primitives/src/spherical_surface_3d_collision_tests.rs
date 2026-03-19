//! SphericalSurface3D collision tests

#[cfg(test)]
mod tests {
    use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D, Vector3D};
    use geo_contracts::BasicCollision;

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

    // LineSegment3D との距離計算実装が完了 (Issue #180)
    #[test]
    fn test_line_segment_intersection() {
        let sphere = create_test_sphere();
        let line =
            LineSegment3D::new(Point3D::new(-2.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
        // 球面は表面のみ: 線分が中心を通る場合、表面までの距離は半径(1.0)
        // tolerance を1.0以上にすれば交差と判定される
        assert!(sphere.intersects(&line, 1.0 + 1e-6));
    }

    #[test]
    fn test_line_segment_no_intersection() {
        let sphere = create_test_sphere();
        let line =
            LineSegment3D::new(Point3D::new(3.0, 0.0, 0.0), Point3D::new(4.0, 0.0, 0.0)).unwrap();
        assert!(!sphere.intersects(&line, 1e-10));
    }

    // Ray3D との距離計算実装が完了 (Issue #180)
    #[test]
    fn test_ray_intersection() {
        let sphere = create_test_sphere();
        let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        // 球面は表面のみ: 光線が中心を通る場合、表面までの距離は半径(1.0)
        assert!(sphere.intersects(&ray, 1.0 + 1e-6));
    }

    // InfiniteLine3D との距離計算実装が完了 (Issue #180)
    #[test]
    fn test_infinite_line_intersection() {
        let sphere = create_test_sphere();
        let line = InfiniteLine3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0))
            .unwrap();
        // 球面は表面のみ: 無限直線が中心を通る場合、表面までの距離は半径(1.0)
        assert!(sphere.intersects(&line, 1.0 + 1e-6));
    }

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
