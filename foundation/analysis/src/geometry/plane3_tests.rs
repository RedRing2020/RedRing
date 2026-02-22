use super::Plane3;
use crate::{Point3, Vector3};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_plane_creation() {
        let point = Point3::new(1.0, 2.0, 3.0);
        let normal = Vector3::new(0.0, 0.0, 1.0);

        let plane = Plane3::from_point_and_normal(point, normal).unwrap();

        assert_eq!(plane.reference_point(), point);
        assert_eq!(plane.normal(), Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_distance_calculation() {
        let plane = Plane3::from_point_and_normal(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        let test_point = Point3::new(1.0, 2.0, 5.0);
        assert_eq!(plane.distance_to_point(test_point), 5.0);
    }

    #[test]
    fn test_projection() {
        let plane = Plane3::from_point_and_normal(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        let test_point = Point3::new(1.0, 2.0, 5.0);
        let projected = plane.project_point(test_point);

        assert_eq!(projected, Point3::new(1.0, 2.0, 0.0));
    }
}
