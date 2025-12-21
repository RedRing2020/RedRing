//! CylindricalSurface3D intersection tests
//!
//! 円柱サーフェスの交差計算テスト

#[cfg(test)]
mod tests {
    use crate::{CylindricalSurface3D, LineSegment3D, Plane3D, Point3D, Triangle3D, Vector3D};
    use geo_foundation::extensions::{
        BasicIntersection, MultipleIntersection, SelfIntersection,
    };

    const TOLERANCE: f64 = 1e-10;

    fn create_test_cylinder() -> CylindricalSurface3D<f64> {
        // Z軸方向、半径1.0の円柱
        CylindricalSurface3D::new_z_axis(Point3D::origin(), 1.0).unwrap()
    }

    #[test]
    fn test_basic_intersection_with_point() {
        let cylinder = create_test_cylinder();
        let point_on_surface = Point3D::new(1.0, 0.0, 0.0);
        let result = cylinder.intersection_with(&point_on_surface, TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_no_intersection_with_point() {
        let cylinder = create_test_cylinder();
        let point_outside = Point3D::new(2.0, 0.0, 0.0);
        let result = cylinder.intersection_with(&point_outside, TOLERANCE);
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_intersections_with_line_segment() {
        let cylinder = create_test_cylinder();
        let start = Point3D::new(1.0, 0.0, -1.0);
        let end = Point3D::new(1.0, 0.0, 1.0);
        let segment = LineSegment3D::new(start, end).unwrap();
        let results = cylinder.intersections_with(&segment, TOLERANCE);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_intersections_with_triangle() {
        let cylinder = create_test_cylinder();
        let v1 = Point3D::new(1.0, 0.0, 0.0);
        let v2 = Point3D::new(0.0, 1.0, 0.0);
        let v3 = Point3D::new(0.0, 0.0, 1.0);
        let triangle = Triangle3D::new(v1, v2, v3).unwrap();
        let results = cylinder.intersections_with(&triangle, TOLERANCE);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_intersection_with_plane() {
        let cylinder = create_test_cylinder();
        let origin = Point3D::origin();
        let z_axis = Vector3D::new(0.0, 0.0, 1.0);
        let x_axis = Vector3D::new(1.0, 0.0, 0.0);
        let plane = Plane3D::from_origin_and_axes(origin, z_axis, x_axis).unwrap();
        let result = cylinder.intersection_with(&plane, TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_self_intersections() {
        let cylinder = create_test_cylinder();
        let results = cylinder.self_intersections(TOLERANCE);
        // 円柱サーフェスは自己交差しない
        assert!(results.is_empty());
    }

    #[test]
    fn test_intersections_with_another_cylinder() {
        let cylinder1 = create_test_cylinder();
        let cylinder2 = CylindricalSurface3D::new_z_axis(Point3D::new(1.5, 0.0, 0.0), 1.0).unwrap();
        let results = cylinder1.intersections_with(&cylinder2, TOLERANCE);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_no_intersection_with_distant_cylinder() {
        let cylinder1 = create_test_cylinder();
        let cylinder2 = CylindricalSurface3D::new_z_axis(Point3D::new(3.0, 0.0, 0.0), 1.0).unwrap();
        let results = cylinder1.intersections_with(&cylinder2, TOLERANCE);
        assert!(results.is_empty());
    }
}
