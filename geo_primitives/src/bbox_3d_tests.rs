//! BBox3D の基本テスト
//!
//! 3D境界ボックスの基本機能をテスト

use crate::{BBox3D, Point3D};

/// 基本機能のテスト
#[test]
fn test_bbox_3d_basic() {
    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let p2 = Point3D::new(4.0, 6.0, 9.0);
    let bbox = BBox3D::new(p1, p2);

    assert_eq!(bbox.min(), p1);
    assert_eq!(bbox.max(), p2);
    assert_eq!(bbox.width(), 3.0);
    assert_eq!(bbox.height(), 4.0);
    assert_eq!(bbox.depth(), 6.0);
}

/// 境界ボックスの作成（順序が逆の場合）
#[test]
fn test_bbox_3d_creation_reversed() {
    // 反転した座標で作成してもそのまま保存される（自動修正なし）
    let bbox = BBox3D::new(Point3D::new(4.0, 6.0, 9.0), Point3D::new(1.0, 2.0, 3.0));
    // BBox3D::new() は引数をそのまま保存（自動修正しない）
    assert_eq!(bbox.min(), Point3D::new(4.0, 6.0, 9.0));
    assert_eq!(bbox.max(), Point3D::new(1.0, 2.0, 3.0));
}
/// 点からの境界ボックス作成
#[test]
fn test_bbox_3d_from_point() {
    let point = Point3D::new(2.0, 3.0, 4.0);
    let bbox = BBox3D::from_point(point);

    assert_eq!(bbox.min(), point);
    assert_eq!(bbox.max(), point);
    assert_eq!(bbox.width(), 0.0);
    assert_eq!(bbox.height(), 0.0);
    assert_eq!(bbox.depth(), 0.0);
}

/// 中心点の計算
#[test]
fn test_bbox_3d_center() {
    let bbox = BBox3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(5.0, 8.0, 9.0));
    let center = bbox.center();

    assert_eq!(center.x(), 3.0); // (1+5)/2
    assert_eq!(center.y(), 5.0); // (2+8)/2
    assert_eq!(center.z(), 6.0); // (3+9)/2
}

/// 点の包含判定
#[test]
fn test_bbox_3d_contains_point() {
    let bbox = BBox3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(5.0, 6.0, 9.0));

    // 境界ボックス内の点
    assert!(bbox.contains_point(&Point3D::new(3.0, 4.0, 6.0)));

    // 境界上の点
    assert!(bbox.contains_point(&Point3D::new(1.0, 2.0, 3.0))); // min
    assert!(bbox.contains_point(&Point3D::new(5.0, 6.0, 9.0))); // max
    assert!(bbox.contains_point(&Point3D::new(3.0, 2.0, 6.0))); // 境界上

    // 境界ボックス外の点
    assert!(!bbox.contains_point(&Point3D::new(0.0, 4.0, 6.0))); // X軸外
    assert!(!bbox.contains_point(&Point3D::new(3.0, 1.0, 6.0))); // Y軸外
    assert!(!bbox.contains_point(&Point3D::new(3.0, 4.0, 2.0))); // Z軸外
    assert!(!bbox.contains_point(&Point3D::new(6.0, 7.0, 10.0))); // 全軸外
}

/// 境界ボックスのサイズ計算
#[test]
fn test_bbox_3d_dimensions() {
    let bbox = BBox3D::new(Point3D::new(2.0, 3.0, 1.0), Point3D::new(8.0, 7.0, 5.0));

    assert_eq!(bbox.width(), 6.0); // 8-2
    assert_eq!(bbox.height(), 4.0); // 7-3
    assert_eq!(bbox.depth(), 4.0); // 5-1
}

/// 単位立方体のテスト
#[test]
fn test_bbox_3d_unit_cube() {
    let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0));

    assert_eq!(bbox.width(), 1.0);
    assert_eq!(bbox.height(), 1.0);
    assert_eq!(bbox.depth(), 1.0);
    assert_eq!(bbox.center(), Point3D::new(0.5, 0.5, 0.5));
}

/// 負の座標での境界ボックス
#[test]
fn test_bbox_3d_negative_coordinates() {
    let bbox = BBox3D::new(
        Point3D::new(-5.0, -3.0, -2.0),
        Point3D::new(-1.0, -1.0, 1.0),
    );

    assert_eq!(bbox.width(), 4.0); // -1-(-5) = 4
    assert_eq!(bbox.height(), 2.0); // -1-(-3) = 2
    assert_eq!(bbox.depth(), 3.0); // 1-(-2) = 3
    assert_eq!(bbox.center(), Point3D::new(-3.0, -2.0, -0.5));
}

/// 原点を含む境界ボックス
#[test]
fn test_bbox_3d_contains_origin() {
    let bbox = BBox3D::new(Point3D::new(-2.0, -1.0, -3.0), Point3D::new(3.0, 2.0, 1.0));

    assert!(bbox.contains_point(&Point3D::new(0.0, 0.0, 0.0)));
    assert_eq!(bbox.width(), 5.0);
    assert_eq!(bbox.height(), 3.0);
    assert_eq!(bbox.depth(), 4.0);
}

/// ゼロサイズの境界ボックス（退化ケース）
#[test]
fn test_bbox_3d_degenerate_cases() {
    // X軸のみゼロ
    let bbox_x = BBox3D::new(Point3D::new(2.0, 1.0, 0.0), Point3D::new(2.0, 3.0, 4.0));
    assert_eq!(bbox_x.width(), 0.0);
    assert_eq!(bbox_x.height(), 2.0);
    assert_eq!(bbox_x.depth(), 4.0);

    // Y軸のみゼロ
    let bbox_y = BBox3D::new(Point3D::new(1.0, 2.0, 0.0), Point3D::new(3.0, 2.0, 4.0));
    assert_eq!(bbox_y.width(), 2.0);
    assert_eq!(bbox_y.height(), 0.0);
    assert_eq!(bbox_y.depth(), 4.0);

    // Z軸のみゼロ
    let bbox_z = BBox3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 3.0));
    assert_eq!(bbox_z.width(), 3.0);
    assert_eq!(bbox_z.height(), 3.0);
    assert_eq!(bbox_z.depth(), 0.0);
}

/// 境界上の点の詳細テスト
#[test]
fn test_bbox_3d_boundary_points() {
    let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 3.0, 2.0));

    // 8つの頂点全てが含まれることを確認
    assert!(bbox.contains_point(&Point3D::new(0.0, 0.0, 0.0))); // 前面左下
    assert!(bbox.contains_point(&Point3D::new(4.0, 0.0, 0.0))); // 前面右下
    assert!(bbox.contains_point(&Point3D::new(4.0, 3.0, 0.0))); // 前面右上
    assert!(bbox.contains_point(&Point3D::new(0.0, 3.0, 0.0))); // 前面左上
    assert!(bbox.contains_point(&Point3D::new(0.0, 0.0, 2.0))); // 背面左下
    assert!(bbox.contains_point(&Point3D::new(4.0, 0.0, 2.0))); // 背面右下
    assert!(bbox.contains_point(&Point3D::new(4.0, 3.0, 2.0))); // 背面右上
    assert!(bbox.contains_point(&Point3D::new(0.0, 3.0, 2.0))); // 背面左上

    // 辺の中点
    assert!(bbox.contains_point(&Point3D::new(2.0, 0.0, 0.0))); // 前面下辺中点
    assert!(bbox.contains_point(&Point3D::new(4.0, 1.5, 0.0))); // 前面右辺中点
    assert!(bbox.contains_point(&Point3D::new(2.0, 3.0, 0.0))); // 前面上辺中点
    assert!(bbox.contains_point(&Point3D::new(0.0, 1.5, 0.0))); // 前面左辺中点

    // 面の中心
    assert!(bbox.contains_point(&Point3D::new(2.0, 1.5, 0.0))); // 前面中心
    assert!(bbox.contains_point(&Point3D::new(2.0, 1.5, 2.0))); // 背面中心
    assert!(bbox.contains_point(&Point3D::new(2.0, 0.0, 1.0))); // 下面中心
    assert!(bbox.contains_point(&Point3D::new(2.0, 3.0, 1.0))); // 上面中心
    assert!(bbox.contains_point(&Point3D::new(0.0, 1.5, 1.0))); // 左面中心
    assert!(bbox.contains_point(&Point3D::new(4.0, 1.5, 1.0))); // 右面中心
}

/// 大きな座標値での境界ボックス
#[test]
fn test_bbox_3d_large_coordinates() {
    let bbox = BBox3D::new(
        Point3D::new(1000.0, 2000.0, 3000.0),
        Point3D::new(1005.0, 2003.0, 3002.0),
    );

    assert_eq!(bbox.width(), 5.0);
    assert_eq!(bbox.height(), 3.0);
    assert_eq!(bbox.depth(), 2.0);
    assert_eq!(bbox.center(), Point3D::new(1002.5, 2001.5, 3001.0));
    assert!(bbox.contains_point(&Point3D::new(1002.0, 2001.0, 3001.0)));
}

/// 非常に小さな境界ボックス
#[test]
fn test_bbox_3d_tiny_dimensions() {
    let bbox = BBox3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(0.001, 0.002, 0.003),
    );

    assert_eq!(bbox.width(), 0.001);
    assert_eq!(bbox.height(), 0.002);
    assert_eq!(bbox.depth(), 0.003);
    assert_eq!(bbox.center(), Point3D::new(0.0005, 0.001, 0.0015));
}

/// デフォルト実装のテスト
#[test]
fn test_bbox_3d_default() {
    let bbox: BBox3D<f64> = Default::default();

    // デフォルトは単位立方体の半分サイズ（-0.5 to 0.5）
    assert_eq!(bbox.center(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(bbox.width(), 1.0);
    assert_eq!(bbox.height(), 1.0);
    assert_eq!(bbox.depth(), 1.0);
    assert!(bbox.contains_point(&Point3D::new(0.0, 0.0, 0.0))); // 原点を含む
}

// ============================================================================
// Builder テスト (bbox_3d_builder_tests.rs から統合)
// ============================================================================

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
