//! BBox3D の形状 builder テスト
//!
//! 3D 形状から境界ボックスを自動生成する builder パターンのテスト

#[cfg(test)]
mod tests {
    use crate::{BBox3D, Point3D};

    // ========================================================================
    // BBox3D Builder Tests
    // ========================================================================

    #[test]
    fn test_bbox3d_from_point_collection() {
        let points = vec![
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(5.0, 1.0, 0.0),
            Point3D::new(3.0, 6.0, 4.0),
            Point3D::new(0.0, 3.0, 2.0),
        ];
        let bbox = BBox3D::from_point_collection(&points).unwrap();

        assert_eq!(bbox.min(), Point3D::new(0.0, 1.0, 0.0));
        assert_eq!(bbox.max(), Point3D::new(5.0, 6.0, 4.0));
        assert_eq!(bbox.width(), 5.0);
        assert_eq!(bbox.height(), 5.0);
        assert_eq!(bbox.depth(), 4.0);
    }

    #[test]
    fn test_bbox3d_from_empty_points() {
        let points: Vec<Point3D<f64>> = vec![];
        let bbox = BBox3D::from_point_collection(&points);
        assert!(bbox.is_none());
    }

    #[test]
    fn test_bbox3d_from_single_point() {
        let points = vec![Point3D::new(2.0, 3.0, 4.0)];
        let bbox = BBox3D::from_point_collection(&points).unwrap();

        // 単一点の境界ボックスは点自身
        assert_eq!(bbox.min(), Point3D::new(2.0, 3.0, 4.0));
        assert_eq!(bbox.max(), Point3D::new(2.0, 3.0, 4.0));
        assert_eq!(bbox.width(), 0.0);
        assert_eq!(bbox.height(), 0.0);
        assert_eq!(bbox.depth(), 0.0);
    }

    #[test]
    fn test_bbox3d_from_large_point_collection() {
        // より多くの点での境界ボックス計算テスト
        let points = vec![
            Point3D::new(-10.0, -5.0, -2.0),
            Point3D::new(10.0, 5.0, 2.0),
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(-5.0, 3.0, 1.0),
            Point3D::new(7.0, -2.0, -1.0),
            Point3D::new(-3.0, 4.0, 3.0),
        ];
        let bbox = BBox3D::from_point_collection(&points).unwrap();

        assert_eq!(bbox.min(), Point3D::new(-10.0, -5.0, -2.0));
        assert_eq!(bbox.max(), Point3D::new(10.0, 5.0, 3.0));
        assert_eq!(bbox.width(), 20.0);
        assert_eq!(bbox.height(), 10.0);
        assert_eq!(bbox.depth(), 5.0);
        assert_eq!(bbox.volume(), 1000.0); // 20 * 10 * 5
    }

    #[test]
    fn test_bbox3d_from_coplanar_points() {
        // 同一平面上の点群での境界ボックステスト
        let points = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        ];
        let bbox = BBox3D::from_point_collection(&points).unwrap();

        assert_eq!(bbox.min(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.max(), Point3D::new(1.0, 1.0, 0.0));
        assert_eq!(bbox.width(), 1.0);
        assert_eq!(bbox.height(), 1.0);
        assert_eq!(bbox.depth(), 0.0); // Z軸方向は平面なので深さ0
        assert_eq!(bbox.volume(), 0.0); // 深さが0なので体積も0
    }

    // ========================================================================
    // Integration Tests (Builder + Extensions)
    // ========================================================================

    #[test]
    fn test_bbox3d_builder_with_intersection() {
        // 3D点群から境界ボックスを作成し、交差判定を行う
        let points1 = vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 2.0, 2.0)];
        let bbox1 = BBox3D::from_point_collection(&points1).unwrap();

        let points2 = vec![Point3D::new(1.0, 1.0, 1.0), Point3D::new(3.0, 3.0, 3.0)];
        let bbox2 = BBox3D::from_point_collection(&points2).unwrap();

        // 2つの境界ボックスが交差することを確認
        assert!(bbox1.intersects(&bbox2));

        // 交差領域を取得
        let intersection = bbox1.intersection(&bbox2);
        assert!(intersection.is_some());

        let intersection_bbox = intersection.unwrap();
        assert_eq!(intersection_bbox.min(), Point3D::new(1.0, 1.0, 1.0));
        assert_eq!(intersection_bbox.max(), Point3D::new(2.0, 2.0, 2.0));
        assert_eq!(intersection_bbox.volume(), 1.0); // 1 * 1 * 1
    }

    #[test]
    fn test_bbox3d_builder_with_union() {
        // 分離した点群から境界ボックスを作成し、結合を行う
        let points1 = vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0)];
        let bbox1 = BBox3D::from_point_collection(&points1).unwrap();

        let points2 = vec![Point3D::new(3.0, 3.0, 3.0), Point3D::new(4.0, 4.0, 4.0)];
        let bbox2 = BBox3D::from_point_collection(&points2).unwrap();

        // 2つの境界ボックスの結合を取得
        let union_bbox = bbox1.union(&bbox2);

        // 結合結果が両方の領域を包含することを確認
        assert_eq!(union_bbox.min(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(union_bbox.max(), Point3D::new(4.0, 4.0, 4.0));
        assert_eq!(union_bbox.width(), 4.0);
        assert_eq!(union_bbox.height(), 4.0);
        assert_eq!(union_bbox.depth(), 4.0);
        assert_eq!(union_bbox.volume(), 64.0); // 4 * 4 * 4

        // 元の点群を全て包含することを確認
        assert!(union_bbox.contains_point(&Point3D::new(0.0, 0.0, 0.0)));
        assert!(union_bbox.contains_point(&Point3D::new(1.0, 1.0, 1.0)));
        assert!(union_bbox.contains_point(&Point3D::new(3.0, 3.0, 3.0)));
        assert!(union_bbox.contains_point(&Point3D::new(4.0, 4.0, 4.0)));
    }

    #[test]
    fn test_bbox3d_builder_non_intersecting() {
        // 交差しない境界ボックスのテスト
        let points1 = vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0)];
        let bbox1 = BBox3D::from_point_collection(&points1).unwrap();

        let points2 = vec![Point3D::new(5.0, 5.0, 5.0), Point3D::new(6.0, 6.0, 6.0)];
        let bbox2 = BBox3D::from_point_collection(&points2).unwrap();

        // 2つの境界ボックスが交差しないことを確認
        assert!(!bbox1.intersects(&bbox2));

        // 交差領域が存在しないことを確認
        let intersection = bbox1.intersection(&bbox2);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_bbox3d_builder_expansion() {
        // 境界ボックス生成後の拡張操作テスト
        let points = vec![Point3D::new(1.0, 1.0, 1.0), Point3D::new(2.0, 2.0, 2.0)];
        let bbox = BBox3D::from_point_collection(&points).unwrap();

        // 各軸方向に1.0ずつ拡張
        let expanded_bbox = bbox.expand_by_margin(1.0);

        assert_eq!(expanded_bbox.min(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(expanded_bbox.max(), Point3D::new(3.0, 3.0, 3.0));
        assert_eq!(expanded_bbox.width(), 3.0);
        assert_eq!(expanded_bbox.height(), 3.0);
        assert_eq!(expanded_bbox.depth(), 3.0);
        assert_eq!(expanded_bbox.volume(), 27.0); // 3 * 3 * 3
    }
}
