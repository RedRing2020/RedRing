#[cfg(test)]
mod point3d_tests {
    use crate::geometry3d::Point3D;

    #[test]
    fn test_point3d_creation() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_point3d_origin() {
        let origin = Point3D::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);
    }

    #[test]
    fn test_point3d_distance() {
        let p1 = Point3D::origin();
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        let distance = p1.distance_to(&p2);
        assert!((distance - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point3d_to_point2d() {
        let p3d = Point3D::new(1.0, 2.0, 3.0);
        let p2d = p3d.to_point2d();
        assert_eq!(p2d.x(), 1.0);
        assert_eq!(p2d.y(), 2.0);
    }

    #[test]
    fn test_point3d_arithmetic() {
        let p1 = Point3D::new(1.0, 2.0, 3.0);
        let p2 = Point3D::new(4.0, 5.0, 6.0);

        let add = p1 + p2;
        assert_eq!(add, Point3D::new(5.0, 7.0, 9.0));

        let sub = p2 - p1;
        assert_eq!(sub, Point3D::new(3.0, 3.0, 3.0));
    }
}