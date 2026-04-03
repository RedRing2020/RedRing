//! InfiniteLine3D テストモジュール

use crate::{InfiniteLine3D, Plane3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, default_orthogonality_dot_error_tolerance, OnPlaneRelation,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_plane_relation_true_when_line_lies_on_plane() {
        let plane = Plane3D::xy_plane(0.0);
        let line = InfiniteLine3D::new(Point3D::new(1.0, 2.0, 0.0), Vector3D::unit_x()).unwrap();

        assert!(OnPlaneRelation::is_on_plane(&line, &plane));
    }

    #[test]
    fn test_on_plane_relation_false_when_direction_is_not_orthogonal_to_plane_normal() {
        let plane = Plane3D::xy_plane(0.0);
        let orthogonality_tolerance = default_orthogonality_dot_error_tolerance::<f64>();
        let line = InfiniteLine3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, orthogonality_tolerance * 10.0),
        )
        .unwrap();

        assert!(!OnPlaneRelation::is_on_plane(&line, &plane));
    }

    #[test]
    fn test_on_plane_relation_true_when_point_offset_is_within_distance_tolerance() {
        let plane = Plane3D::xy_plane(0.0);
        let distance_tolerance = default_distance_tolerance::<f64>();
        let line = InfiniteLine3D::new(
            Point3D::new(0.0, 0.0, distance_tolerance * 0.5),
            Vector3D::unit_x(),
        )
        .unwrap();

        assert!(OnPlaneRelation::is_on_plane(&line, &plane));
    }

    #[test]
    fn test_on_plane_relation_false_when_point_offset_exceeds_distance_tolerance() {
        let plane = Plane3D::xy_plane(0.0);
        let distance_tolerance = default_distance_tolerance::<f64>();
        let line = InfiniteLine3D::new(
            Point3D::new(0.0, 0.0, distance_tolerance * 2.0),
            Vector3D::unit_x(),
        )
        .unwrap();

        assert!(!OnPlaneRelation::is_on_plane(&line, &plane));
    }
}
