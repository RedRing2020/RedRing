//! BBox2D の形状 builder テスト
//!
//! 2D 形状から境界ボックスを自動生成する builder パターンのテスト

#[cfg(test)]
mod tests {
    use crate::{BBox2D, Circle2D, Point2D};

    // ========================================================================
    // BBox2D Builder Tests
    // ========================================================================

    #[test]
    fn test_bbox2d_from_circle() {
        let circle = Circle2D::new(Point2D::new(5.0, 3.0), 2.0).unwrap();
        let bbox = BBox2D::from_circle(&circle);
        
        assert_eq!(bbox.min(), Point2D::new(3.0, 1.0));
        assert_eq!(bbox.max(), Point2D::new(7.0, 5.0));
        assert_eq!(bbox.width(), 4.0);
        assert_eq!(bbox.height(), 4.0);
    }

    #[test]
    fn test_bbox2d_from_circle_unit_radius() {
        let circle = Circle2D::new(Point2D::new(1.0, 2.0), 1.0).unwrap();
        let bbox = BBox2D::from_circle(&circle);
        
        // 半径1の円の境界ボックス
        assert_eq!(bbox.min(), Point2D::new(0.0, 1.0));
        assert_eq!(bbox.max(), Point2D::new(2.0, 3.0));
        assert_eq!(bbox.width(), 2.0);
        assert_eq!(bbox.height(), 2.0);
    }

    #[test]
    fn test_bbox2d_from_point_collection() {
        let points = vec![
            Point2D::new(1.0, 2.0),
            Point2D::new(5.0, 1.0),
            Point2D::new(3.0, 6.0),
            Point2D::new(0.0, 3.0),
        ];
        let bbox = BBox2D::from_point_collection(&points).unwrap();
        
        assert_eq!(bbox.min(), Point2D::new(0.0, 1.0));
        assert_eq!(bbox.max(), Point2D::new(5.0, 6.0));
        assert_eq!(bbox.width(), 5.0);
        assert_eq!(bbox.height(), 5.0);
    }

    #[test]
    fn test_bbox2d_from_empty_points() {
        let points: Vec<Point2D<f64>> = vec![];
        let bbox = BBox2D::from_point_collection(&points);
        assert!(bbox.is_none());
    }

    #[test]
    fn test_bbox2d_from_single_point() {
        let points = vec![Point2D::new(2.0, 3.0)];
        let bbox = BBox2D::from_point_collection(&points).unwrap();
        
        // 単一点の境界ボックスは点自身
        assert_eq!(bbox.min(), Point2D::new(2.0, 3.0));
        assert_eq!(bbox.max(), Point2D::new(2.0, 3.0));
        assert_eq!(bbox.width(), 0.0);
        assert_eq!(bbox.height(), 0.0);
    }

    // ========================================================================
    // Integration Tests (Builder + Extensions)
    // ========================================================================

    #[test]
    fn test_bbox2d_builder_with_union() {
        // 2つの円から境界ボックスを作成し、結合を取得
        let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let bbox1 = BBox2D::from_circle(&circle1);

        let circle2 = Circle2D::new(Point2D::new(3.0, 3.0), 1.0).unwrap();
        let bbox2 = BBox2D::from_circle(&circle2);
        
        // 2つの境界ボックスの結合を取得
        let union_bbox = bbox1.union(&bbox2);
        
        // 結合結果が両方の円を包含することを確認
        assert!(union_bbox.contains_point(&Point2D::new(0.0, 0.0))); // 円1の中心
        assert!(union_bbox.contains_point(&Point2D::new(3.0, 3.0))); // 円2の中心
        assert!(union_bbox.contains_point(&Point2D::new(-1.0, -1.0))); // 円1の境界
        assert!(union_bbox.contains_point(&Point2D::new(4.0, 4.0))); // 円2の境界
    }

    #[test]
    fn test_bbox2d_builder_with_intersection() {
        // 2つの点群から境界ボックスを作成し、交差判定を行う
        let points1 = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 2.0),
        ];
        let bbox1 = BBox2D::from_point_collection(&points1).unwrap();
        
        let points2 = vec![
            Point2D::new(1.0, 1.0),
            Point2D::new(3.0, 3.0),
        ];
        let bbox2 = BBox2D::from_point_collection(&points2).unwrap();
        
        // 2つの境界ボックスが交差することを確認
        assert!(bbox1.intersects(&bbox2));
        
        // 交差領域を取得
        let intersection = bbox1.intersection(&bbox2);
        assert!(intersection.is_some());
        
        let intersection_bbox = intersection.unwrap();
        assert_eq!(intersection_bbox.min(), Point2D::new(1.0, 1.0));
        assert_eq!(intersection_bbox.max(), Point2D::new(2.0, 2.0));
    }

    #[test]
    fn test_bbox2d_builder_chaining() {
        // 複数の形状から段階的に境界ボックスを構築
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let initial_bbox = BBox2D::from_circle(&circle);
        
        let additional_points = vec![
            Point2D::new(3.0, 0.0),
            Point2D::new(0.0, 3.0),
        ];
        let points_bbox = BBox2D::from_point_collection(&additional_points).unwrap();
        
        // 2つの境界ボックスを結合して全体の境界ボックスを取得
        let combined_bbox = initial_bbox.union(&points_bbox);
        
        // 結合結果が全体を包含することを確認
        assert_eq!(combined_bbox.min(), Point2D::new(-1.0, -1.0));
        assert_eq!(combined_bbox.max(), Point2D::new(3.0, 3.0));
        assert_eq!(combined_bbox.width(), 4.0);
        assert_eq!(combined_bbox.height(), 4.0);
    }
}