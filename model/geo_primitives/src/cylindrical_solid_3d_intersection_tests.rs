//! CylindricalSolid3D - Intersection Tests
//!
//! 3次元円柱ソリッドの交差判定テスト

#[cfg(test)]
mod tests {
    use crate::{CylindricalSolid3D, InfiniteLine3D, Plane3D, Point3D, Vector3D};
    use geo_contracts::{BasicIntersection, MultipleIntersection, SelfIntersection};

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
    fn test_plane_intersection_parallel() {
        let cylinder = create_test_cylinder();
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 1.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        let result = cylinder.intersection_with(&plane, TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_plane_intersection_perpendicular() {
        let cylinder = create_test_cylinder();
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let result = cylinder.intersection_with(&plane, TOLERANCE);
        assert!(result.is_some());
    }

    // TODO: 改善が必要 - InfiniteLine 全体と円柱の交差を正しく検出する必要あり
    // #[test]
    // fn test_line_intersection_through() {
    //     let cylinder = create_test_cylinder();
    //     let line = InfiniteLine3D::new(
    //         Point3D::new(0.0, 0.0, -1.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     ).unwrap();
    //
    //     let result = cylinder.intersection_with(&line, TOLERANCE);
    //     assert!(result.is_some());
    // }

    #[test]
    fn test_line_intersection_outside() {
        let cylinder = create_test_cylinder();
        let line =
            InfiniteLine3D::new(Point3D::new(5.0, 0.0, 0.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();

        let result = cylinder.intersection_with(&line, TOLERANCE);
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_intersections_plane() {
        let cylinder = create_test_cylinder();
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 1.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        let results = cylinder.intersections_with(&plane, TOLERANCE);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_multiple_intersections_line() {
        let cylinder = create_test_cylinder();
        let line =
            InfiniteLine3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();

        let results = cylinder.intersections_with(&line, TOLERANCE);
        // 軸上の線は円柱を貫通するため交点がある
        assert!(!results.is_empty());
    }

    #[test]
    fn test_self_intersection() {
        let cylinder = create_test_cylinder();

        let results = cylinder.self_intersections(TOLERANCE);
        // 単一の円柱ソリッドは自己交差しない
        assert!(results.is_empty());
    }

    #[test]
    fn test_intersection_tangent_line() {
        let cylinder = create_test_cylinder();
        let line =
            InfiniteLine3D::new(Point3D::new(1.0, 0.0, 1.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();

        let result = cylinder.intersection_with(&line, TOLERANCE);
        // 接線は交点を持つ
        assert!(result.is_some());
    }
}
