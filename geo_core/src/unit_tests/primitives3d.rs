/// 3Dプリミティブのユニットテスト - removed, use geo_primitives tests instead
#[cfg(not(any()))] // disabled - legacy 3D primitives removed
mod legacy_tests {
    use crate::primitives3d::{Point3D, LineSegment3D, Plane, Sphere};
    use crate::scalar::Scalar;

    #[test]
    fn test_point_distance_3d() {
        let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
        let p2 = Point3D::from_f64(1.0, 1.0, 1.0);
        let distance = p1.distance_to(&p2);
        assert!((distance.value() - f64::sqrt(3.0)).abs() < 1e-10);
    }

    #[test]
    fn test_plane_from_three_points() {
        let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
        let p2 = Point3D::from_f64(1.0, 0.0, 0.0);
        let p3 = Point3D::from_f64(0.0, 1.0, 0.0);

        let plane = Plane::from_three_points(&p1, &p2, &p3).unwrap();

        // Z軸方向が法線になるはず
        assert!((plane.normal().z().value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sphere_volume() {
        let center = Point3D::origin();
        let radius = Scalar::new(1.0);
        let sphere = Sphere::new(center, radius);

        let expected_volume = 4.0 * std::f64::consts::PI / 3.0;
        assert!((sphere.volume().value() - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_line_segment_distance_to_point() {
        let start = Point3D::from_f64(0.0, 0.0, 0.0);
        let end = Point3D::from_f64(1.0, 0.0, 0.0);
        let line = LineSegment3D::new(start, end);

        let point = Point3D::from_f64(0.5, 1.0, 0.0);
        let distance = line.distance_to_point(&point);
        assert!((distance.value() - 1.0).abs() < 1e-10);
    }
}
