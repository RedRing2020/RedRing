//! CylindricalSolid3D - Collision Tests
//!
//! 3次元円柱ソリッドの衝突判定テスト

#[cfg(test)]
mod tests {
    use crate::{
        Circle3D, CylindricalSolid3D, Direction3D, Plane3D, Point3D, Triangle3D, Vector3D,
    };
    use geo_foundation::extensions::BasicCollision;

    const TOLERANCE: f64 = 1e-10;

    fn create_test_cylinder() -> CylindricalSolid3D<f64> {
        // 原点を中心とする半径1.0、高さ2.0の円柱（Z軸方向）
        CylindricalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0), // axis
            Vector3D::new(1.0, 0.0, 0.0), // ref_direction
            1.0,                          // radius
            2.0,                          // height
        )
        .unwrap()
    }

    #[test]
    fn test_point_internal() {
        let cylinder = create_test_cylinder();
        let internal_point = Point3D::new(0.5, 0.0, 1.0);

        assert!(cylinder.intersects(&internal_point, TOLERANCE));
        assert!(cylinder.overlaps(&internal_point, TOLERANCE));
        assert_eq!(cylinder.distance_to(&internal_point), 0.0);
    }

    #[test]
    fn test_point_on_surface() {
        let cylinder = create_test_cylinder();
        let surface_point = Point3D::new(1.0, 0.0, 1.0);

        assert!(cylinder.intersects(&surface_point, TOLERANCE));
        assert!(cylinder.overlaps(&surface_point, TOLERANCE));
        assert!(cylinder.distance_to(&surface_point) < TOLERANCE);
    }

    #[test]
    fn test_point_outside() {
        let cylinder = create_test_cylinder();
        let outside_point = Point3D::new(2.0, 0.0, 1.0);

        assert!(!cylinder.intersects(&outside_point, TOLERANCE));
        assert!(!cylinder.overlaps(&outside_point, TOLERANCE));
        assert!(cylinder.distance_to(&outside_point) > 0.9);
    }

    #[test]
    fn test_circle_intersection() {
        let cylinder = create_test_cylinder();
        let circle = Circle3D::new(
            Point3D::new(0.0, 0.0, 1.0),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
            0.5,
        )
        .unwrap();

        assert!(cylinder.intersects(&circle, TOLERANCE));
        assert!(cylinder.overlaps(&circle, TOLERANCE));
    }

    // TODO: 改善が必要 - 線分の両端だけでなく全体と円柱の交差を検出する必要あり
    // #[test]
    // fn test_line_segment_through() {
    //     let cylinder = create_test_cylinder();
    //     let line = LineSegment3D::new(
    //         Point3D::new(0.0, 0.0, -1.0),
    //         Point3D::new(0.0, 0.0, 3.0),
    //     ).unwrap();
    //
    //     assert!(cylinder.intersects(&line, TOLERANCE));
    // }

    #[test]
    fn test_triangle_intersection() {
        let cylinder = create_test_cylinder();
        let triangle = Triangle3D::new(
            Point3D::new(-0.5, -0.5, 1.0),
            Point3D::new(0.5, -0.5, 1.0),
            Point3D::new(0.0, 0.5, 1.0),
        )
        .unwrap();

        assert!(cylinder.intersects(&triangle, TOLERANCE));
    }

    #[test]
    fn test_plane_intersection() {
        let cylinder = create_test_cylinder();
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 1.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        assert!(cylinder.intersects(&plane, TOLERANCE));
    }

    // TODO: 改善が必要 - Ray の原点だけでなく方向を考慮した交差判定が必要
    // #[test]
    // fn test_ray_intersection() {
    //     let cylinder = create_test_cylinder();
    //     let ray = Ray3D::new(
    //         Point3D::new(0.0, 0.0, -1.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     ).unwrap();
    //
    //     assert!(cylinder.intersects(&ray, TOLERANCE));
    // }

    #[test]
    fn test_distance_to_point_above() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(0.0, 0.0, 3.0); // 高さ範囲外（上）

        let distance = cylinder.distance_to(&point);
        assert!((distance - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_distance_to_point_below() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(0.0, 0.0, -1.0); // 高さ範囲外（下）

        let distance = cylinder.distance_to(&point);
        assert!((distance - 1.0).abs() < TOLERANCE);
    }
}
