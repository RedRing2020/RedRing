#[cfg(test)]
mod bbox3d_tests {
    use crate::geometry3d::{BBox3D, Point3D};
    use crate::traits::bbox_trait::{BoundingBox, BoundingBoxOps, CollisionBounds};

    #[test]
    fn test_bbox3d_creation() {
        let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
        assert_eq!(bbox.min, Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.max, Point3D::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_bbox3d_dimensions() {
        let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
        assert_eq!(bbox.width(), 2.0);
        assert_eq!(bbox.height(), 3.0);
        assert_eq!(bbox.depth(), 4.0);
        assert_eq!(bbox.volume(), 24.0);
        assert_eq!(bbox.surface_area(), 52.0); // 2*(2*3 + 2*4 + 3*4) = 2*26 = 52
    }

    #[test]
    fn test_bbox3d_from_points() {
        let points = vec![
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(-1.0, 5.0, 1.0),
            Point3D::new(3.0, 0.0, 4.0),
        ];

        let bbox = BBox3D::from_point_array(&points).unwrap();
        assert_eq!(bbox.min, Point3D::new(-1.0, 0.0, 1.0));
        assert_eq!(bbox.max, Point3D::new(3.0, 5.0, 4.0));
    }

    #[test]
    fn test_collision_bounds_interface() {
        let bbox1 = BBox3D::new((0.0, 0.0, 0.0), (2.0, 2.0, 2.0));
        let bbox2 = BBox3D::new((1.0, 1.0, 1.0), (3.0, 3.0, 3.0));
        let bbox3 = BBox3D::new((3.0, 3.0, 3.0), (4.0, 4.0, 4.0));

        // 高速重複テスト
        assert!(bbox1.fast_overlaps(&bbox2));
        assert!(!bbox1.fast_overlaps(&bbox3));

        // 分離距離
        assert!(bbox1.separation_distance(&bbox2).is_none()); // 重複している
        let sep_dist = bbox1.separation_distance(&bbox3).unwrap();
        assert_eq!(sep_dist, (3.0_f64 - 2.0).sqrt().abs()); // 距離計算
    }

    #[test]
    fn test_generic_trait_implementation() {
        let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));

        // BoundingBoxトレイト
        assert_eq!(bbox.min(), [0.0, 0.0, 0.0]);
        assert_eq!(bbox.max(), [2.0, 3.0, 4.0]);
        assert_eq!(bbox.extent(0), 2.0);
        assert_eq!(bbox.extent(1), 3.0);
        assert_eq!(bbox.extent(2), 4.0);
        assert_eq!(bbox.center(), [1.0, 1.5, 2.0]);

        // BoundingBoxOpsトレイト
        assert!(bbox.contains_point([1.0, 1.5, 2.0]));
        assert!(!bbox.contains_point([3.0, 1.0, 2.0]));
        assert!(bbox.is_valid());

        let expanded = bbox.expand(0.5);
        assert_eq!(expanded.min, Point3D::new(-0.5, -0.5, -0.5));
        assert_eq!(expanded.max, Point3D::new(2.5, 3.5, 4.5));
    }

    #[test]
    fn test_special_cases() {
        // 立方体テスト
        let cube = BBox3D::new((0.0, 0.0, 0.0), (2.0, 2.0, 2.0));
        assert!(cube.is_cube(1e-10));

        // 平面（深度0）
        let plane = BBox3D::new((0.0, 0.0, 1.0), (2.0, 2.0, 1.0));
        assert_eq!(plane.depth(), 0.0);
        assert_eq!(plane.volume(), 0.0);

        // 点（サイズ0）
        let point = BBox3D::new((1.0, 1.0, 1.0), (1.0, 1.0, 1.0));
        assert_eq!(point.width(), 0.0);
        assert_eq!(point.height(), 0.0);
        assert_eq!(point.depth(), 0.0);
        assert_eq!(point.volume(), 0.0);
    }
}
