/// 2D/3D Bounding Box統合テスト
/// BBox2D, BBox3D両方のテストとジェネリックトレイトテスト

use crate::geometry2d::{Point2D, BBox2D};
use crate::geometry3d::{Point3D, BBox3D};
use crate::traits::bbox_trait::{BoundingBox, BoundingBoxOps, CollisionBounds};

// =============================================================================
// BBox2D Tests
// =============================================================================

#[test]
fn test_bbox2d_basic_creation() {
    let bbox = BBox2D::new((0.0, 0.0), (2.0, 3.0));
    assert_eq!(bbox.min, Point2D::new(0.0, 0.0));
    assert_eq!(bbox.max, Point2D::new(2.0, 3.0));
    assert_eq!(bbox.width(), 2.0);
    assert_eq!(bbox.height(), 3.0);
    assert_eq!(bbox.area(), 6.0);
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
    assert_eq!(bbox.width(), 4.0);
    assert_eq!(bbox.height(), 5.0);
}

#[test]
fn test_bbox2d_collision_detection() {
    let bbox1 = BBox2D::new((0.0, 0.0), (2.0, 2.0));
    let bbox2 = BBox2D::new((1.0, 1.0), (3.0, 3.0));
    let bbox3 = BBox2D::new((3.0, 3.0), (4.0, 4.0));

    // 基本的な交差テスト
    assert!(bbox1.intersects(&bbox2));
    assert!(!bbox1.intersects(&bbox3));

    // 高速重複テスト
    assert!(bbox1.fast_overlaps(&bbox2));
    assert!(!bbox1.fast_overlaps(&bbox3));

    // 分離距離
    assert!(bbox1.separation_distance(&bbox2).is_none()); // 重複
    assert_eq!(bbox1.separation_distance(&bbox3).unwrap(), 1.0);
}

#[test]
fn test_bbox2d_special_properties() {
    // 正方形
    let square = BBox2D::new((0.0, 0.0), (2.0, 2.0));
    assert!(square.is_square(1e-10));
    assert_eq!(square.aspect_ratio(), 1.0);

    // 長方形
    let rect = BBox2D::new((0.0, 0.0), (4.0, 2.0));
    assert!(!rect.is_square(1e-10));
    assert_eq!(rect.aspect_ratio(), 2.0);

    // 線分（高さ0）
    let line = BBox2D::new((0.0, 1.0), (2.0, 1.0));
    assert_eq!(line.height(), 0.0);
    assert_eq!(line.area(), 0.0);
    assert_eq!(line.aspect_ratio(), f64::INFINITY);
}

#[test]
fn test_bbox2d_to_3d_conversion() {
    let bbox2d = BBox2D::new((1.0, 2.0), (3.0, 4.0));
    let bbox3d = bbox2d.to_3d();

    assert_eq!(bbox3d.min, Point3D::new(1.0, 2.0, 0.0));
    assert_eq!(bbox3d.max, Point3D::new(3.0, 4.0, 0.0));
    assert_eq!(bbox3d.depth(), 0.0);
    assert_eq!(bbox3d.volume(), 0.0);
}

// =============================================================================
// BBox3D Tests
// =============================================================================

#[test]
fn test_bbox3d_basic_creation() {
    let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
    assert_eq!(bbox.min, Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(bbox.max, Point3D::new(2.0, 3.0, 4.0));
    assert_eq!(bbox.width(), 2.0);
    assert_eq!(bbox.height(), 3.0);
    assert_eq!(bbox.depth(), 4.0);
    assert_eq!(bbox.volume(), 24.0);
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
    assert_eq!(bbox.width(), 4.0);
    assert_eq!(bbox.height(), 5.0);
    assert_eq!(bbox.depth(), 3.0);
}

#[test]
fn test_bbox3d_from_2d_points() {
    let min_2d = Point2D::new(1.0, 2.0);
    let max_2d = Point2D::new(3.0, 4.0);
    let bbox = BBox3D::from_2d_points(min_2d, max_2d);

    assert_eq!(bbox.min, Point3D::new(1.0, 2.0, 0.0));
    assert_eq!(bbox.max, Point3D::new(3.0, 4.0, 0.0));
    assert_eq!(bbox.depth(), 0.0);
}

#[test]
fn test_bbox3d_collision_detection() {
    let bbox1 = BBox3D::new((0.0, 0.0, 0.0), (2.0, 2.0, 2.0));
    let bbox2 = BBox3D::new((1.0, 1.0, 1.0), (3.0, 3.0, 3.0));
    let bbox3 = BBox3D::new((3.0, 3.0, 3.0), (4.0, 4.0, 4.0));

    // 基本的な交差テスト
    assert!(bbox1.intersects(&bbox2));
    assert!(!bbox1.intersects(&bbox3));

    // 高速重複テスト
    assert!(bbox1.fast_overlaps(&bbox2));
    assert!(!bbox1.fast_overlaps(&bbox3));

    // 分離距離
    assert!(bbox1.separation_distance(&bbox2).is_none()); // 重複
    assert_eq!(bbox1.separation_distance(&bbox3).unwrap(), 1.0);
}

#[test]
fn test_bbox3d_advanced_properties() {
    let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));

    // 表面積
    assert_eq!(bbox.surface_area(), 52.0); // 2*(2*3 + 2*4 + 3*4) = 52

    // 対角線の長さ
    let expected_diagonal = (4.0 + 9.0 + 16.0_f64).sqrt(); // sqrt(2^2 + 3^2 + 4^2)
    assert!((bbox.diagonal_length() - expected_diagonal).abs() < 1e-10);

    // 中心点
    assert_eq!(bbox.center_tuple(), (1.0, 1.5, 2.0));
}

#[test]
fn test_bbox3d_edge_cases() {
    // 点としての境界ボックス（min == max）
    let point_bbox = BBox3D::new((1.0, 1.0, 1.0), (1.0, 1.0, 1.0));
    assert_eq!(point_bbox.width(), 0.0);
    assert_eq!(point_bbox.height(), 0.0);
    assert_eq!(point_bbox.depth(), 0.0);
    assert_eq!(point_bbox.volume(), 0.0);
    assert!(point_bbox.contains_point_tuple((1.0, 1.0, 1.0)));

    // 線としての境界ボックス（1次元のみ非ゼロ）
    let line_bbox = BBox3D::new((0.0, 1.0, 1.0), (2.0, 1.0, 1.0));
    assert_eq!(line_bbox.width(), 2.0);
    assert_eq!(line_bbox.height(), 0.0);
    assert_eq!(line_bbox.depth(), 0.0);
    assert_eq!(line_bbox.volume(), 0.0);

    // 面としての境界ボックス（1次元がゼロ）
    let plane_bbox = BBox3D::new((0.0, 0.0, 1.0), (2.0, 3.0, 1.0));
    assert_eq!(plane_bbox.width(), 2.0);
    assert_eq!(plane_bbox.height(), 3.0);
    assert_eq!(plane_bbox.depth(), 0.0);
    assert_eq!(plane_bbox.volume(), 0.0);
}

// =============================================================================
// Generic Trait Tests
// =============================================================================

#[test]
fn test_generic_bbox_trait_2d() {
    let bbox = BBox2D::new((0.0, 0.0), (2.0, 3.0));

    // BoundingBoxトレイト
    assert_eq!(bbox.min(), [0.0, 0.0]);
    assert_eq!(bbox.max(), [2.0, 3.0]);
    assert_eq!(bbox.extent(0), 2.0);
    assert_eq!(bbox.extent(1), 3.0);
    assert_eq!(bbox.center(), [1.0, 1.5]);
    assert_eq!(bbox.volume(), 6.0);

    // BoundingBoxOpsトレイト
    assert!(bbox.contains_point([1.0, 1.5]));
    assert!(!bbox.contains_point([3.0, 1.0]));
    assert!(bbox.is_valid());

    let expanded = bbox.expand(0.5);
    assert_eq!(expanded.min, Point2D::new(-0.5, -0.5));
    assert_eq!(expanded.max, Point2D::new(2.5, 3.5));
}

#[test]
fn test_generic_bbox_trait_3d() {
    let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));

    // BoundingBoxトレイト
    assert_eq!(bbox.min(), [0.0, 0.0, 0.0]);
    assert_eq!(bbox.max(), [2.0, 3.0, 4.0]);
    assert_eq!(bbox.extent(0), 2.0);
    assert_eq!(bbox.extent(1), 3.0);
    assert_eq!(bbox.extent(2), 4.0);
    assert_eq!(bbox.center(), [1.0, 1.5, 2.0]);
    assert_eq!(bbox.volume(), 24.0);

    // BoundingBoxOpsトレイト
    assert!(bbox.contains_point([1.0, 1.5, 2.0]));
    assert!(!bbox.contains_point([3.0, 1.0, 1.0]));
    assert!(bbox.is_valid());

    let expanded = bbox.expand(0.5);
    assert_eq!(expanded.min, Point3D::new(-0.5, -0.5, -0.5));
    assert_eq!(expanded.max, Point3D::new(2.5, 3.5, 4.5));
}

#[test]
fn test_collision_bounds_closest_point() {
    // 2D
    let bbox2d = BBox2D::new((0.0, 0.0), (2.0, 2.0));
    let closest_2d = bbox2d.closest_point_on_surface([5.0, 1.0]);
    assert_eq!(closest_2d, [2.0, 1.0]); // X軸でクランプ

    // 3D
    let bbox3d = BBox3D::new((0.0, 0.0, 0.0), (2.0, 2.0, 2.0));
    let closest_3d = bbox3d.closest_point_on_surface([5.0, 1.0, 1.0]);
    assert_eq!(closest_3d, [2.0, 1.0, 1.0]); // X軸でクランプ
}

#[test]
fn test_bbox_union_operations() {
    // 2D union
    let bbox2d_1 = BBox2D::new((0.0, 0.0), (2.0, 2.0));
    let bbox2d_2 = BBox2D::new((1.0, 1.0), (3.0, 3.0));
    let union_2d = bbox2d_1.union(&bbox2d_2);
    assert_eq!(union_2d.min, Point2D::new(0.0, 0.0));
    assert_eq!(union_2d.max, Point2D::new(3.0, 3.0));

    // 3D union
    let bbox3d_1 = BBox3D::new((0.0, 0.0, 0.0), (2.0, 2.0, 2.0));
    let bbox3d_2 = BBox3D::new((1.0, 1.0, 1.0), (3.0, 3.0, 3.0));
    let union_3d = bbox3d_1.union(&bbox3d_2);
    assert_eq!(union_3d.min, Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(union_3d.max, Point3D::new(3.0, 3.0, 3.0));
}

#[test]
fn test_bbox_validation() {
    // 有効なボックス
    let valid_2d = BBox2D::new((0.0, 0.0), (2.0, 3.0));
    let valid_3d = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
    assert!(valid_2d.is_valid());
    assert!(valid_3d.is_valid());

    // 無効なボックス（min > max）
    let invalid_2d = BBox2D::new((2.0, 3.0), (0.0, 0.0));
    let invalid_3d = BBox3D::new((2.0, 3.0, 4.0), (0.0, 0.0, 0.0));
    assert!(!invalid_2d.is_valid());
    assert!(!invalid_3d.is_valid());

    // 境界ケース（min == max）は有効
    let point_2d = BBox2D::new((1.0, 1.0), (1.0, 1.0));
    let point_3d = BBox3D::new((1.0, 1.0, 1.0), (1.0, 1.0, 1.0));
    assert!(point_2d.is_valid());
    assert!(point_3d.is_valid());
}
