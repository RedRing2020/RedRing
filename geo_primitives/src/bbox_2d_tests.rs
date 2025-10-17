//! BBox2D のテスト

use crate::{BBox2D, Point2D};

/// 基本機能のテスト
#[test]
fn test_bbox_2d_basic() {
    let p1 = Point2D::new(1.0, 2.0);
    let p2 = Point2D::new(4.0, 6.0);
    let bbox = BBox2D::new(p1, p2);

    assert_eq!(bbox.min(), p1);
    assert_eq!(bbox.max(), p2);
    assert_eq!(bbox.width(), 3.0);
    assert_eq!(bbox.height(), 4.0);
    assert_eq!(bbox.area(), 12.0);
}

/// 点からの境界ボックス生成テスト
#[test]
fn test_from_point() {
    let point = Point2D::new(5.0, 3.0);
    let bbox = BBox2D::from_point(point);

    assert_eq!(bbox.min(), point);
    assert_eq!(bbox.max(), point);
    assert_eq!(bbox.width(), 0.0);
    assert_eq!(bbox.height(), 0.0);
    assert_eq!(bbox.area(), 0.0);
}

/// 複数点からの境界ボックス生成テスト
#[test]
fn test_from_points() {
    let points = vec![
        Point2D::new(1.0, 1.0),
        Point2D::new(5.0, 3.0),
        Point2D::new(2.0, 7.0),
        Point2D::new(6.0, 2.0),
    ];

    let bbox = BBox2D::from_points(&points).unwrap();

    assert_eq!(bbox.min(), Point2D::new(1.0, 1.0));
    assert_eq!(bbox.max(), Point2D::new(6.0, 7.0));
    assert_eq!(bbox.width(), 5.0);
    assert_eq!(bbox.height(), 6.0);
}

/// 空配列からの境界ボックス生成テスト
#[test]
fn test_from_empty_points() {
    let points: Vec<Point2D<f64>> = vec![];
    let bbox = BBox2D::from_points(&points);
    assert!(bbox.is_none());
}

/// 中心点計算テスト
#[test]
fn test_center() {
    let bbox = BBox2D::new(Point2D::new(2.0, 4.0), Point2D::new(8.0, 10.0));
    let center = bbox.center();

    assert_eq!(center, Point2D::new(5.0, 7.0));
}

/// 点の包含判定テスト
#[test]
fn test_contains_point() {
    let bbox = BBox2D::new(Point2D::new(1.0, 2.0), Point2D::new(5.0, 6.0));

    // 内部の点
    assert!(bbox.contains_point(&Point2D::new(3.0, 4.0)));

    // 境界上の点
    assert!(bbox.contains_point(&Point2D::new(1.0, 4.0)));
    assert!(bbox.contains_point(&Point2D::new(5.0, 4.0)));
    assert!(bbox.contains_point(&Point2D::new(3.0, 2.0)));
    assert!(bbox.contains_point(&Point2D::new(3.0, 6.0)));

    // 角の点
    assert!(bbox.contains_point(&Point2D::new(1.0, 2.0)));
    assert!(bbox.contains_point(&Point2D::new(5.0, 6.0)));

    // 外部の点
    assert!(!bbox.contains_point(&Point2D::new(0.0, 4.0)));
    assert!(!bbox.contains_point(&Point2D::new(6.0, 4.0)));
    assert!(!bbox.contains_point(&Point2D::new(3.0, 1.0)));
    assert!(!bbox.contains_point(&Point2D::new(3.0, 7.0)));
}

/// 境界ボックスの包含判定テスト
#[test]
fn test_contains_bbox() {
    let bbox1 = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(10.0, 10.0));
    let bbox2 = BBox2D::new(Point2D::new(3.0, 3.0), Point2D::new(7.0, 7.0));
    let bbox3 = BBox2D::new(Point2D::new(5.0, 5.0), Point2D::new(15.0, 15.0));

    assert!(bbox1.contains_bbox(&bbox2)); // bbox1 は bbox2 を完全に含む
    assert!(!bbox1.contains_bbox(&bbox3)); // bbox1 は bbox3 を完全には含まない
    assert!(!bbox2.contains_bbox(&bbox1)); // bbox2 は bbox1 を含まない
}

/// 交差判定テスト
#[test]
fn test_intersects() {
    let bbox1 = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(5.0, 5.0));
    let bbox2 = BBox2D::new(Point2D::new(3.0, 3.0), Point2D::new(7.0, 7.0));
    let bbox3 = BBox2D::new(Point2D::new(6.0, 6.0), Point2D::new(8.0, 8.0));
    let bbox4 = BBox2D::new(Point2D::new(10.0, 10.0), Point2D::new(12.0, 12.0));

    assert!(bbox1.intersects(&bbox2)); // 重複あり
    assert!(!bbox1.intersects(&bbox3)); // 隣接（重複なし）
    assert!(!bbox1.intersects(&bbox4)); // 完全に離れている
}

/// 交差領域計算テスト
#[test]
fn test_intersection() {
    let bbox1 = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(5.0, 5.0));
    let bbox2 = BBox2D::new(Point2D::new(3.0, 3.0), Point2D::new(7.0, 7.0));
    let bbox3 = BBox2D::new(Point2D::new(10.0, 10.0), Point2D::new(12.0, 12.0));

    // 交差あり
    let intersection = bbox1.intersection(&bbox2).unwrap();
    assert_eq!(intersection.min(), Point2D::new(3.0, 3.0));
    assert_eq!(intersection.max(), Point2D::new(5.0, 5.0));

    // 交差なし
    assert!(bbox1.intersection(&bbox3).is_none());
}

/// 結合領域計算テスト
#[test]
fn test_union() {
    let bbox1 = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));
    let bbox2 = BBox2D::new(Point2D::new(2.0, 2.0), Point2D::new(5.0, 4.0));

    let union = bbox1.union(&bbox2);
    assert_eq!(union.min(), Point2D::new(1.0, 1.0));
    assert_eq!(union.max(), Point2D::new(5.0, 4.0));
}

/// 拡張テスト
#[test]
fn test_expand() {
    let bbox = BBox2D::new(Point2D::new(2.0, 3.0), Point2D::new(6.0, 7.0));
    let expanded = bbox.expand(1.0);

    assert_eq!(expanded.min(), Point2D::new(1.0, 2.0));
    assert_eq!(expanded.max(), Point2D::new(7.0, 8.0));
}

/// 退化判定テスト
#[test]
fn test_is_degenerate() {
    let normal_bbox = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));
    let line_bbox = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(1.0, 3.0)); // 幅0
    let point_bbox = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(1.0, 1.0)); // 点

    assert!(!normal_bbox.is_degenerate(f64::EPSILON));
    assert!(line_bbox.is_degenerate(f64::EPSILON));
    assert!(point_bbox.is_degenerate(f64::EPSILON));
}

/// 3D変換テスト
#[test]
fn test_to_3d() {
    let bbox_2d = BBox2D::new(Point2D::new(1.0, 2.0), Point2D::new(4.0, 6.0));
    let bbox_3d = bbox_2d.to_3d();

    assert_eq!(bbox_3d.min().x(), 1.0);
    assert_eq!(bbox_3d.min().y(), 2.0);
    assert_eq!(bbox_3d.min().z(), 0.0);
    assert_eq!(bbox_3d.max().x(), 4.0);
    assert_eq!(bbox_3d.max().y(), 6.0);
    assert_eq!(bbox_3d.max().z(), 0.0);
}

/// Z範囲指定での3D変換テスト
#[test]
fn test_to_3d_with_z() {
    let bbox_2d = BBox2D::new(Point2D::new(1.0, 2.0), Point2D::new(4.0, 6.0));
    let bbox_3d = bbox_2d.to_3d_with_z(-1.0, 3.0);

    assert_eq!(bbox_3d.min().x(), 1.0);
    assert_eq!(bbox_3d.min().y(), 2.0);
    assert_eq!(bbox_3d.min().z(), -1.0);
    assert_eq!(bbox_3d.max().x(), 4.0);
    assert_eq!(bbox_3d.max().y(), 6.0);
    assert_eq!(bbox_3d.max().z(), 3.0);
}

/// f32での基本機能テスト
#[test]
fn test_bbox_2d_f32() {
    let bbox: BBox2D<f32> = BBox2D::new(Point2D::new(1.0f32, 2.0f32), Point2D::new(4.0f32, 6.0f32));

    assert_eq!(bbox.width(), 3.0f32);
    assert_eq!(bbox.height(), 4.0f32);
    assert_eq!(bbox.area(), 12.0f32);
}
