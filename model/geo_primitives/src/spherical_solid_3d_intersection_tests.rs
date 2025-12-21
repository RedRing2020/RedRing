#[cfg(test)]
mod tests {
    use crate::{InfiniteLine3D, Plane3D, Point3D, Ray3D, SphericalSolid3D, Vector3D};
    use geo_foundation::{
        extensions::{BasicIntersection, MultipleIntersection, SelfIntersection},
        Scalar,
    };

    fn create_test_sphere<T: Scalar>() -> SphericalSolid3D<T> {
        let center = Point3D::origin();
        let axis = Vector3D::new(T::ZERO, T::ZERO, T::ONE);
        let ref_direction = Vector3D::new(T::ONE, T::ZERO, T::ZERO);
        let radius = T::ONE;
        SphericalSolid3D::new(center, axis, ref_direction, radius)
            .expect("Failed to create test sphere")
    }

    #[test]
    fn test_intersection_with_point_inside() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(0.5, 0.0, 0.0);
        let intersection = sphere.intersection_with(&point, 1e-6);
        assert!(intersection.is_some());
    }

    #[test]
    fn test_intersection_with_point_outside() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(2.0, 0.0, 0.0);
        let intersection = sphere.intersection_with(&point, 1e-6);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_intersection_with_plane_through_center() {
        let sphere = create_test_sphere::<f64>();
        let plane = Plane3D::xy_plane(0.0);
        let intersection = sphere.intersection_with(&plane, 1e-6);
        assert!(intersection.is_some());
    }

    #[test]
    fn test_intersection_with_plane_outside() {
        let sphere = create_test_sphere::<f64>();
        let plane = Plane3D::xy_plane(2.0);
        let intersection = sphere.intersection_with(&plane, 1e-6);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_multiple_intersections_with_line_through_center() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(-2.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let line = InfiniteLine3D::new(point, direction).expect("Failed to create line");
        let intersections = sphere.intersections_with(&line, 1e-6);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn test_multiple_intersections_with_line_tangent() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(0.0, 1.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let line = InfiniteLine3D::new(point, direction).expect("Failed to create line");
        let intersections = sphere.intersections_with(&line, 1e-6);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_multiple_intersections_with_line_miss() {
        let sphere = create_test_sphere::<f64>();
        let point = Point3D::new(0.0, 2.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let line = InfiniteLine3D::new(point, direction).expect("Failed to create line");
        let intersections = sphere.intersections_with(&line, 1e-6);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_multiple_intersections_with_ray_through_center() {
        let sphere = create_test_sphere::<f64>();
        let origin = Point3D::new(-2.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).expect("Failed to create ray");
        let intersections = sphere.intersections_with(&ray, 1e-6);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn test_multiple_intersections_with_ray_from_inside() {
        let sphere = create_test_sphere::<f64>();
        let origin = Point3D::new(0.0, 0.0, 0.0); // 中心から
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).expect("Failed to create ray");
        let intersections = sphere.intersections_with(&ray, 1e-6);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_multiple_intersections_with_ray_miss() {
        let sphere = create_test_sphere::<f64>();
        let origin = Point3D::new(0.0, 2.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).expect("Failed to create ray");
        let intersections = sphere.intersections_with(&ray, 1e-6);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_self_intersections() {
        let sphere = create_test_sphere::<f64>();
        let intersections = sphere.self_intersections(1e-6);
        assert_eq!(intersections.len(), 0); // 球ソリッド単体では自己交差しない
    }
}
