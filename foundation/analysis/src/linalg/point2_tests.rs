use super::point2::{Coordinates2D, Point2};
use super::Vector2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2_creation() {
        let p = Point2::new(1.0, 2.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
    }

    #[test]
    fn test_origin() {
        let origin = Point2::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
    }

    #[test]
    fn test_vector_conversion() {
        let p = Point2::new(1.0, 2.0);
        let v = p.to_vector();
        let p2 = Point2::from_vector(v);

        assert_eq!(p, p2);
    }

    #[test]
    fn test_distance() {
        let p1 = Point2::new(0.0, 0.0);
        let p2 = Point2::new(3.0, 4.0);

        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.distance_squared_to(&p2), 25.0);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point2::new(0.0, 0.0);
        let p2 = Point2::new(4.0, 6.0);
        let mid = p1.midpoint(&p2);

        assert_eq!(mid, Point2::new(2.0, 3.0));
    }

    #[test]
    fn test_point_vector_operations() {
        let p = Point2::new(1.0, 2.0);
        let v = Vector2::new(1.0, 1.0);

        let p_plus_v = p + v;
        assert_eq!(p_plus_v, Point2::new(2.0, 3.0));

        let p_minus_v = p - v;
        assert_eq!(p_minus_v, Point2::new(0.0, 1.0));

        let p1 = Point2::new(3.0, 4.0);
        let p2 = Point2::new(1.0, 2.0);
        let diff = p1 - p2;
        assert_eq!(diff, Vector2::new(2.0, 2.0));
    }

    #[test]
    fn test_to_3d() {
        let p2 = Point2::new(1.0, 2.0);
        let p3_default = p2.to_3d();
        let p3_with_z = p2.to_3d_with_z(5.0);

        assert_eq!(p3_default.x(), 1.0);
        assert_eq!(p3_default.y(), 2.0);
        assert_eq!(p3_default.z(), 0.0);

        assert_eq!(p3_with_z.x(), 1.0);
        assert_eq!(p3_with_z.y(), 2.0);
        assert_eq!(p3_with_z.z(), 5.0);
    }

    #[test]
    fn test_coordinates_trait() {
        let p = Point2::new(1.0, 2.0);
        let v = Vector2::new(4.0, 5.0);

        assert_eq!(Coordinates2D::x(&p), 1.0);
        assert_eq!(Coordinates2D::y(&v), 5.0);
    }
}
