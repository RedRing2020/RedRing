use super::point3::{Coordinates3D, Point3};
use super::Vector3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3_creation() {
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_origin() {
        let origin = Point3::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);
    }

    #[test]
    fn test_vector_conversion() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = p.to_vector();
        let p2 = Point3::from_vector(v);

        assert_eq!(p, p2);
    }

    #[test]
    fn test_distance() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(3.0, 4.0, 0.0);

        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.distance_squared_to(&p2), 25.0);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(4.0, 6.0, 8.0);
        let mid = p1.midpoint(&p2);

        assert_eq!(mid, Point3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_point_vector_operations() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vector3::new(1.0, 1.0, 1.0);

        let p_plus_v = p + v;
        assert_eq!(p_plus_v, Point3::new(2.0, 3.0, 4.0));

        let p_minus_v = p - v;
        assert_eq!(p_minus_v, Point3::new(0.0, 1.0, 2.0));

        let p1 = Point3::new(3.0, 4.0, 5.0);
        let p2 = Point3::new(1.0, 2.0, 3.0);
        let diff = p1 - p2;
        assert_eq!(diff, Vector3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_coordinates_trait() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vector3::new(4.0, 5.0, 6.0);

        assert_eq!(Coordinates3D::x(&p), 1.0);
        assert_eq!(Coordinates3D::y(&v), 5.0);
    }

    #[test]
    fn test_to_2d() {
        let p3 = Point3::new(1.0, 2.0, 5.0);
        let p2 = p3.to_2d();

        assert_eq!(p2.x(), 1.0);
        assert_eq!(p2.y(), 2.0);
    }
}
