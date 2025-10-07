#[cfg(test)]
use crate::geometry2d::{BBox2D, Point2D};
use crate::geometry3d::Point3D;
    use crate::traits::bbox_trait::{BoundingBox, BoundingBoxOps, CollisionBounds};

    #[test]
    fn test_bbox2d_creation() {
        let bbox = BBox2D::new((0.0, 0.0), (2.0, 3.0));
        assert_eq!(bbox.min, Point2D::new(0.0, 0.0));
        assert_eq!(bbox.max, Point2D::new(2.0, 3.0));
    }

    #[test]
    fn test_bbox2d_dimensions() {
        let bbox = BBox2D::new((0.0, 0.0), (2.0, 3.0));
        assert_eq!(bbox.width(), 2.0);
        assert_eq!(bbox.height(), 3.0);
        assert_eq!(bbox.area(), 6.0);
        assert_eq!(bbox.perimeter(), 10.0);
        assert_eq!(bbox.aspect_ratio(), 2.0 / 3.0);
    }

    #[test]
    fn test_bbox2d_from_points() {
        let points = vec![
            Point2D::new(1.0, 2.0),
            Point2D::new(-1.0, 5.0),
            Point2D::new(3.0, 0.0),
        ];

        let bbox = BBox2D::from_point_array(&points).unwrap();
        assert_eq!(bbox.min, Point2D::new(-1.0, 0.0));
        assert_eq!(bbox.max, Point2D::new(3.0, 5.0));
    }

    #[test]
    fn test_collision_bounds_interface() {
        let bbox1 = BBox2D::new((0.0, 0.0), (2.0, 2.0));
        let bbox2 = BBox2D::new((1.0, 1.0), (3.0, 3.0));
        let bbox3 = BBox2D::new((3.0, 3.0), (4.0, 4.0));

        // 高速重複テスト
        assert!(bbox1.fast_overlaps(&bbox2));
        assert!(!bbox1.fast_overlaps(&bbox3));

        // 分離距離
        assert!(bbox1.separation_distance(&bbox2).is_none()); // 重複している
        let sep_dist = bbox1.separation_distance(&bbox3).unwrap();
        assert_eq!(sep_dist, 1.0); // (3.0 - 2.0) = 1.0

        // 最近点
        let closest = bbox1.closest_point_on_surface([5.0, 1.0]);
        assert_eq!(closest, [2.0, 1.0]); // X軸でクランプ
    }

    #[test]
    fn test_generic_trait_implementation() {
        let bbox = BBox2D::new((0.0, 0.0), (2.0, 3.0));

        // BoundingBoxトレイト
        assert_eq!(bbox.min(), [0.0, 0.0]);
        assert_eq!(bbox.max(), [2.0, 3.0]);
        assert_eq!(bbox.extent(0), 2.0);
        assert_eq!(bbox.extent(1), 3.0);
        assert_eq!(bbox.center(), [1.0, 1.5]);

        // BoundingBoxOpsトレイト
        assert!(bbox.contains_point([1.0, 1.5]));
        assert!(!bbox.contains_point([3.0, 1.0]));
        assert!(bbox.is_valid());

        let expanded = bbox.expand(0.5);
        assert_eq!(expanded.min, Point2D::new(-0.5, -0.5));
        assert_eq!(expanded.max, Point2D::new(2.5, 3.5));
    }

    #[test]
    fn test_special_cases() {
        // 正方形テスト
        let square = BBox2D::new((0.0, 0.0), (2.0, 2.0));
        assert!(square.is_square(1e-10));
        assert_eq!(square.aspect_ratio(), 1.0);

        // 線分（高さ0）
        let line = BBox2D::new((0.0, 1.0), (2.0, 1.0));
        assert_eq!(line.height(), 0.0);
        assert_eq!(line.area(), 0.0);
        assert_eq!(line.aspect_ratio(), f64::INFINITY);

        // 点（サイズ0）
        let point = BBox2D::new((1.0, 1.0), (1.0, 1.0));
        assert_eq!(point.width(), 0.0);
        assert_eq!(point.height(), 0.0);
        assert_eq!(point.area(), 0.0);
    }

    #[test]
    fn test_3d_conversion() {
        let bbox2d = BBox2D::new((1.0, 2.0), (3.0, 4.0));
        let bbox3d = bbox2d.to_3d();

        assert_eq!(bbox3d.min, Point3D::new(1.0, 2.0, 0.0));
        assert_eq!(bbox3d.max, Point3D::new(3.0, 4.0, 0.0));
        assert_eq!(bbox3d.depth(), 0.0);
    }
