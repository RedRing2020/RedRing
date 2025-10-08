//! 幾何ユーティリティ関数のテスト
//! 点変換、バウンディングボックス、重心計算のテスト

use crate::geometry2d::Point2D;
use crate::geometry3d::Point3D;
use crate::traits::common::geometry_utils::*;

#[test]
fn test_point2d_conversion() {
    let point = Point2D::new(1.5, 2.5);
    let (x, y) = point2d_to_f64(&point);
    assert_eq!(x, 1.5);
    assert_eq!(y, 2.5);

    let restored = point2d_from_f64(x, y);
    assert_eq!(restored.x(), 1.5);
    assert_eq!(restored.y(), 2.5);
}

#[test]
fn test_point3d_conversion() {
    let point = Point3D::new(1.5, 2.5, 3.5);
    let (x, y, z) = point3d_to_f64(&point);
    assert_eq!(x, 1.5);
    assert_eq!(y, 2.5);
    assert_eq!(z, 3.5);

    let restored = point3d_from_f64(x, y, z);
    assert_eq!(restored.x(), 1.5);
    assert_eq!(restored.y(), 2.5);
    assert_eq!(restored.z(), 3.5);
}

#[test]
fn test_bounding_box_2d() {
    let points = vec![
        Point2D::new(0.0, 0.0),
        Point2D::new(2.0, 1.0),
        Point2D::new(1.0, 3.0),
    ];

    let bbox = point2d_bounding_box(&points).unwrap();
    assert_eq!(bbox, (0.0, 0.0, 2.0, 3.0));
}

#[test]
fn test_centroid_2d() {
    let points = vec![
        Point2D::new(0.0, 0.0),
        Point2D::new(2.0, 0.0),
        Point2D::new(1.0, 2.0),
    ];

    let centroid = point2d_centroid(&points).unwrap();
    assert_eq!(centroid.x(), 1.0);
    assert_eq!(centroid.y(), 2.0 / 3.0);
}
