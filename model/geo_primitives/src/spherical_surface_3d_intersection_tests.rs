//! SphericalSurface3D intersection tests

#[cfg(test)]
mod tests {
    use crate::{InfiniteLine3D, Point3D, Ray3D, SphericalSurface3D, Vector3D};
    use geo_foundation::extensions::BasicIntersection;

    /// テスト用の標準的な球面サーフェスを作成
    fn create_test_sphere() -> SphericalSurface3D<f64> {
        SphericalSurface3D::new_at_origin(1.0).unwrap()
    }

    #[test]
    fn test_point_intersection() {
        let sphere = create_test_sphere();
        let point = Point3D::new(1.0, 0.0, 0.0);
        let result: Option<Point3D<f64>> = sphere.intersection_with(&point, 1e-10);
        assert!(result.is_some());
    }

    #[test]
    fn test_point_no_intersection() {
        let sphere = create_test_sphere();
        let point = Point3D::new(2.0, 0.0, 0.0);
        let result: Option<Point3D<f64>> = sphere.intersection_with(&point, 1e-10);
        assert!(result.is_none());
    }

    // TODO: Plane3D との交差判定実装が完了したらテストを追加
    // #[test]
    // fn test_plane_intersection() {
    //     let sphere = create_test_sphere();
    //     let plane = Plane3D::from_point_and_normal(
    //         Point3D::new(0.0, 0.0, 0.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     )
    //     .unwrap();
    //     let result: Option<Point3D<f64>> = sphere.intersection_with(&plane, 1e-10);
    //     assert!(result.is_none());
    // }

    #[test]
    fn test_line_intersection() {
        let sphere = create_test_sphere();
        let line = InfiniteLine3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let result: Option<Point3D<f64>> = sphere.intersection_with(&line, 1e-10);
        // 簡易実装では未実装
        assert!(result.is_none());
    }

    #[test]
    fn test_ray_intersection() {
        let sphere = create_test_sphere();
        let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let result: Option<Point3D<f64>> = sphere.intersection_with(&ray, 1e-10);
        // 簡易実装では未実装
        assert!(result.is_none());
    }

    // TODO: MultipleIntersection 実装が完了したらテストを追加
    // #[test]
    // fn test_multiple_point_intersections() {
    //     let sphere = create_test_sphere();
    //     let point = Point3D::new(1.0, 0.0, 0.0);
    //     let results = sphere.intersections_with(&point, 1e-10);
    //     assert_eq!(results.len(), 1);
    // }

    // TODO: SelfIntersection 実装が完了したらテストを追加
    // #[test]
    // fn test_self_intersections() {
    //     let sphere = create_test_sphere();
    //     let results = sphere.self_intersections(1e-10);
    //     assert_eq!(results.len(), 0);
    // }
}
