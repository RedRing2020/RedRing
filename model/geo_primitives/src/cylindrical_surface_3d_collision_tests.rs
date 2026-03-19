//! CylindricalSurface3D - Collision & AdvancedCollision Tests
//!
//! 3次元円柱サーフェスの衝突判定・高度衝突判定のテストスイート

#[cfg(test)]
mod tests {
    use crate::{
        Circle3D, CylindricalSurface3D, Direction3D, LineSegment3D, Plane3D, Point3D, Triangle3D,
        Vector3D,
    };
    use geo_contracts::{AdvancedCollision, BasicCollision};

    const TOLERANCE: f64 = 1e-10;

    /// テスト用の標準的な円柱サーフェスを作成
    /// 中心: (0, 0, 0)、軸: Z軸正方向、半径: 1.0
    fn create_test_cylinder() -> CylindricalSurface3D<f64> {
        CylindricalSurface3D::new_z_axis(Point3D::origin(), 1.0).unwrap()
    }

    // ============================================================================
    // BasicCollision Tests - Point3D
    // ============================================================================

    #[test]
    fn test_point_on_surface() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(1.0, 0.0, 0.0);

        assert!(cylinder.intersects(&point, TOLERANCE));
        assert!(cylinder.overlaps(&point, TOLERANCE));
        assert!(cylinder.distance_to(&point) < TOLERANCE);
    }

    #[test]
    fn test_point_inside() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(0.5, 0.0, 0.0);

        assert!(!cylinder.intersects(&point, TOLERANCE));
        let distance = cylinder.distance_to(&point);
        assert!((distance - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_point_outside() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(2.0, 0.0, 0.0);

        assert!(!cylinder.intersects(&point, TOLERANCE));
        let distance = cylinder.distance_to(&point);
        assert!((distance - 1.0).abs() < TOLERANCE);
    }

    // ============================================================================
    // AdvancedCollision Tests - Point3D
    // ============================================================================

    #[test]
    fn test_closest_points_to_point() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(2.0, 0.0, 5.0);

        let (closest_on_surface, closest_on_point) = cylinder.closest_points(&point);

        // 円柱面上の点は (1.0, 0.0, 5.0) のはず
        assert!((closest_on_surface.x() - 1.0).abs() < TOLERANCE);
        assert!((closest_on_surface.y() - 0.0).abs() < TOLERANCE);
        assert!((closest_on_surface.z() - 5.0).abs() < TOLERANCE);

        // 点自身が返される
        assert_eq!(closest_on_point, point);
    }

    #[test]
    fn test_separated_by_axis_point() {
        let cylinder = create_test_cylinder();
        let far_point = Point3D::new(5.0, 0.0, 0.0);
        let near_point = Point3D::new(0.5, 0.0, 0.0);

        // X軸で分離判定
        let axis = Vector3D::new(1.0, 0.0, 0.0);

        assert!(cylinder.separated_by_axis(&far_point, axis));
        assert!(!cylinder.separated_by_axis(&near_point, axis));
    }

    #[test]
    fn test_containment_relation_point() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(0.5, 0.0, 0.0);

        let (cylinder_contains, point_contains) = cylinder.containment_relation(&point, TOLERANCE);

        // 円柱面（無限に薄い）は点を包含しない
        assert!(!cylinder_contains);
        // 点が面上にないので、point_contains = true
        assert!(point_contains);
    }

    // ============================================================================
    // BasicCollision Tests - Circle3D
    // ============================================================================

    #[test]
    fn test_circle_intersection() {
        let cylinder = create_test_cylinder();

        // 円の中心が円柱面上にある円（半径1.0）
        // 中心が (1.0, 0.0, 1.0) で円柱面上にある
        let circle = Circle3D::new(
            Point3D::new(1.0, 0.0, 1.0),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
            0.5,
        )
        .unwrap();

        assert!(cylinder.intersects(&circle, TOLERANCE));
    }

    // ============================================================================
    // BasicCollision Tests - LineSegment3D
    // ============================================================================

    #[test]
    fn test_line_segment_intersecting() {
        let cylinder = create_test_cylinder();

        // 始点が円柱面上にある線分
        let segment =
            LineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();

        assert!(cylinder.intersects(&segment, TOLERANCE));
    }

    // ============================================================================
    // AdvancedCollision Tests - LineSegment3D
    // ============================================================================

    #[test]
    fn test_separated_by_axis_segment() {
        let cylinder = create_test_cylinder();

        let far_segment =
            LineSegment3D::new(Point3D::new(5.0, 0.0, 0.0), Point3D::new(6.0, 0.0, 0.0)).unwrap();

        let axis = Vector3D::new(1.0, 0.0, 0.0);
        assert!(cylinder.separated_by_axis(&far_segment, axis));
    }

    // ============================================================================
    // BasicCollision Tests - Triangle3D
    // ============================================================================

    #[test]
    fn test_triangle_intersection() {
        let cylinder = create_test_cylinder();

        // 円柱面に接する三角形
        let triangle = Triangle3D::new(
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 1.0),
            Point3D::new(1.0, 1.0, 0.0),
        )
        .unwrap();

        assert!(cylinder.intersects(&triangle, TOLERANCE));
    }

    // ============================================================================
    // AdvancedCollision Tests - Triangle3D
    // ============================================================================

    #[test]
    fn test_separated_by_axis_triangle() {
        let cylinder = create_test_cylinder();

        // 遠くの三角形
        let far_triangle = Triangle3D::new(
            Point3D::new(5.0, 0.0, 0.0),
            Point3D::new(6.0, 0.0, 0.0),
            Point3D::new(5.5, 1.0, 0.0),
        )
        .unwrap();

        let axis = Vector3D::new(1.0, 0.0, 0.0);
        assert!(cylinder.separated_by_axis(&far_triangle, axis));
    }

    // ============================================================================
    // BasicCollision Tests - Plane3D
    // ============================================================================

    #[test]
    fn test_plane_intersection() {
        let cylinder = create_test_cylinder();

        // XY平面（円柱を貫通）
        let plane = Plane3D::from_origin_and_axes(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        assert!(cylinder.intersects(&plane, TOLERANCE));
    }

    // ============================================================================
    // BasicCollision Tests - CylindricalSurface3D
    // ============================================================================

    #[test]
    fn test_cylinder_cylinder_parallel_intersecting() {
        let cylinder1 = create_test_cylinder();

        // 平行で交差する円柱
        let cylinder2 = CylindricalSurface3D::new_z_axis(Point3D::new(1.5, 0.0, 0.0), 1.0).unwrap();

        assert!(cylinder1.intersects(&cylinder2, TOLERANCE));
    }

    #[test]
    fn test_cylinder_cylinder_parallel_separated() {
        let cylinder1 = create_test_cylinder();

        // 平行で離れた円柱
        let cylinder2 = CylindricalSurface3D::new_z_axis(Point3D::new(5.0, 0.0, 0.0), 1.0).unwrap();

        assert!(!cylinder1.intersects(&cylinder2, TOLERANCE));
    }

    // ============================================================================
    // AdvancedCollision Tests - CylindricalSurface3D
    // ============================================================================

    #[test]
    fn test_containment_relation_cylinders() {
        let outer = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 2.0).unwrap();

        let inner = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();

        let (outer_contains, inner_contains) = outer.containment_relation(&inner, TOLERANCE);

        assert!(outer_contains); // 外側の円柱が内側を包含
        assert!(!inner_contains); // 内側は外側を包含しない
    }

    #[test]
    fn test_separated_by_axis_cylinders() {
        let cylinder1 = create_test_cylinder();

        let cylinder2 = CylindricalSurface3D::new_z_axis(Point3D::new(5.0, 0.0, 0.0), 1.0).unwrap();

        let axis = Vector3D::new(1.0, 0.0, 0.0);
        assert!(cylinder1.separated_by_axis(&cylinder2, axis));
    }

    #[test]
    fn test_overlap_measure() {
        let cylinder = create_test_cylinder();
        let point = Point3D::new(1.0, 0.0, 0.0);

        // 点との重なり測定は None
        assert!(cylinder.overlap_measure(&point).is_none());
    }
}
