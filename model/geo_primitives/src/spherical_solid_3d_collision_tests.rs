#[cfg(test)]
mod tests {
    use crate::{
        InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSolid3D, Triangle3D,
        Vector3D,
    };
    use geo_foundation::{extensions::BasicCollision, Scalar};

    fn create_test_sphere<T: Scalar>() -> SphericalSolid3D<T> {
        let center = Point3D::origin();
        let axis = Vector3D::new(T::ZERO, T::ZERO, T::ONE);
        let ref_direction = Vector3D::new(T::ONE, T::ZERO, T::ZERO);
        let radius = T::ONE;
        SphericalSolid3D::new(center, axis, ref_direction, radius)
            .expect("Failed to create test sphere")
    }

    #[test]
    fn test_collision_with_point_on_surface() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(1.0, 0.0, 0.0); // 表面上
        assert!(sphere.intersects(&point, 1e-6));
    }

    #[test]
    fn test_collision_with_point_inside() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(0.5, 0.0, 0.0); // 内部
        assert!(sphere.intersects(&point, 1e-6));
    }

    #[test]
    fn test_collision_with_point_outside() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(2.0, 0.0, 0.0); // 外部
        assert!(!sphere.intersects(&point, 1e-6));
    }

    #[test]
    fn test_distance_to_point_inside() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(0.5, 0.0, 0.0);
        let distance = sphere.distance_to(&point);
        assert!(distance.abs() < 1e-6); // 内部なのでゼロ
    }

    #[test]
    fn test_distance_to_point_outside() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(2.0, 0.0, 0.0);
        let distance = sphere.distance_to(&point);
        assert!((distance - 1.0).abs() < 1e-6); // 表面までの距離
    }

    #[test]
    fn test_distance_to_point_on_surface() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(1.0, 0.0, 0.0);
        let distance = sphere.distance_to(&point);
        assert!(distance.abs() < 1e-6);
    }

    #[test]
    fn test_collision_with_line_segment() {
        let sphere = create_test_sphere::<f64>();
        let start = Point3D::new(-0.5, 0.0, 0.0);
        let end = Point3D::new(0.5, 0.0, 0.0);
        let line = LineSegment3D::new(start, end).expect("Failed to create line segment");
        assert!(sphere.intersects(&line, 1e-6));
    }

    // TODO: Ray3D との距離計算実装が完了したらテストを追加
    // #[test]
    // fn test_collision_with_ray() {
    //     let sphere = create_test_sphere::<f64>();
    //     let origin = Point3D::new(-2.0, 0.0, 0.0);
    //     let direction = Vector3D::new(1.0, 0.0, 0.0); // X軸方向
    //     let ray = Ray3D::new(origin, direction).expect("Failed to create ray");
    //     assert!(sphere.intersects(&ray, 1e-6));
    // }

    #[test]
    fn test_collision_with_infinite_line_through_center() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(-2.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let line = InfiniteLine3D::new(point, direction).expect("Failed to create line");
        assert!(sphere.intersects(&line, 1e-6));
        assert!(sphere.distance_to(&line) < 1e-6);
    }

    #[test]
    fn test_collision_with_infinite_line_tangent() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(0.0, 1.0, 0.0); // 接線
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let line = InfiniteLine3D::new(point, direction).expect("Failed to create line");
        assert!(sphere.intersects(&line, 1e-6));
    }

    #[test]
    fn test_collision_with_infinite_line_miss() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(0.0, 2.0, 0.0); // 外側
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let line = InfiniteLine3D::new(point, direction).expect("Failed to create line");
        assert!(!sphere.intersects(&line, 1e-6));
    }

    #[test]
    fn test_collision_with_triangle() {
        let sphere = create_test_sphere::<f64>();
        let va = Point3D::new(0.0, 0.0, 0.0);
        let vb = Point3D::new(1.0, 0.0, 0.0);
        let vc = Point3D::new(0.0, 1.0, 0.0);
        let triangle = Triangle3D::new(va, vb, vc).expect("Failed to create triangle");
        assert!(sphere.intersects(&triangle, 1e-6));
    }

    #[test]
    fn test_collision_with_plane_through_center() {
        let sphere = create_test_sphere::<f64>();
        let plane = Plane3D::xy_plane(0.0);
        assert!(sphere.intersects(&plane, 1e-6));
        assert!(sphere.distance_to(&plane) < 1e-6);
    }

    #[test]
    fn test_collision_with_plane_tangent() {
        let sphere = create_test_sphere::<f64>();
        let plane = Plane3D::xy_plane(1.0); // z=1 平面（接平面）
        assert!(sphere.intersects(&plane, 1e-6));
    }

    #[test]
    fn test_collision_with_plane_miss() {
        let sphere = create_test_sphere::<f64>();
        let plane = Plane3D::xy_plane(2.0); // z=2 平面（外側）
        assert!(!sphere.intersects(&plane, 1e-6));
    }

    #[test]
    fn test_collision_with_self() {
        let sphere = create_test_sphere::<f64>();
        assert!(sphere.intersects(&sphere, 1e-6));
        assert!(sphere.distance_to(&sphere) < 1e-6);
    }

    #[test]
    fn test_collision_with_other_sphere_intersecting() {
        let sphere1 = create_test_sphere::<f64>();
        let center = Point3D::new(1.5, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let sphere2 = SphericalSolid3D::new(center, axis, ref_direction, 1.0)
            .expect("Failed to create sphere2");
        assert!(sphere1.intersects(&sphere2, 1e-6));
    }

    #[test]
    fn test_collision_with_other_sphere_separate() {
        let sphere1 = create_test_sphere::<f64>();
        let center = Point3D::new(3.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let sphere2 = SphericalSolid3D::new(center, axis, ref_direction, 1.0)
            .expect("Failed to create sphere2");
        assert!(!sphere1.intersects(&sphere2, 1e-6));
    }
}
