//! Triangle3D のテスト

use super::*;
use crate::{Point3D, Triangle3D, Vector3D};
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata, TolerantEq};

#[test]
fn test_triangle_creation() {
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(0.0, 1.0, 0.0);

    let triangle = Triangle3D::new(a, b, c);
    assert!(triangle.is_some());

    let triangle = triangle.unwrap();
    assert_eq!(triangle.vertex_a, a);
    assert_eq!(triangle.vertex_b, b);
    assert_eq!(triangle.vertex_c, c);
}

#[test]
fn test_degenerate_triangle() {
    // 一直線上の3点で退化三角形
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(2.0, 0.0, 0.0);

    let triangle = Triangle3D::new(a, b, c);
    assert!(triangle.is_none());
}

#[test]
fn test_triangle_area() {
    // 直角三角形（底辺1、高さ1）
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(0.0, 1.0, 0.0);

    let triangle = Triangle3D::new(a, b, c).unwrap();
    let area = triangle.area();

    // 面積は 0.5 のはず
    assert!((area - 0.5).abs() < 1e-10);
}

#[test]
fn test_triangle_normal() {
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(0.0, 1.0, 0.0);

    let triangle = Triangle3D::new(a, b, c).unwrap();
    let normal = triangle.normal().unwrap();

    // XY平面の三角形なので法線はZ方向
    assert!((normal.z() - 1.0).abs() < 1e-10);
    assert!(normal.x().abs() < 1e-10);
    assert!(normal.y().abs() < 1e-10);
}

#[test]
fn test_point_containment() {
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(2.0, 0.0, 0.0);
    let c = Point3D::new(0.0, 2.0, 0.0);

    let triangle = Triangle3D::new(a, b, c).unwrap();

    // 重心は内部にある
    let centroid = Point3D::new(2.0 / 3.0, 2.0 / 3.0, 0.0);
    assert!(triangle.contains_point_on_plane(centroid));

    // 頂点も含まれる
    assert!(triangle.contains_point_on_plane(a));
    assert!(triangle.contains_point_on_plane(b));
    assert!(triangle.contains_point_on_plane(c));

    // 外部の点は含まれない
    let outside = Point3D::new(3.0, 3.0, 0.0);
    assert!(!triangle.contains_point_on_plane(outside));
}

#[test]
fn test_triangle_edges() {
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(0.0, 1.0, 0.0);

    let triangle = Triangle3D::new(a, b, c).unwrap();

    let ab = triangle.edge_ab();
    let bc = triangle.edge_bc();
    let ca = triangle.edge_ca();

    assert_eq!(ab, Vector3D::new(1.0, 0.0, 0.0));
    assert_eq!(bc, Vector3D::new(-1.0, 1.0, 0.0));
    assert_eq!(ca, Vector3D::new(0.0, -1.0, 0.0));
}

#[test]
fn test_triangle_validity() {
    // 有効な三角形
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(0.0, 1.0, 0.0);

    let triangle = Triangle3D::new(a, b, c).unwrap();
    assert!(triangle.is_valid());

    // 極小三角形（面積がほぼ0）
    let a_small = Point3D::new(0.0, 0.0, 0.0);
    let b_small = Point3D::new(1e-12, 0.0, 0.0);
    let c_small = Point3D::new(0.0, 1e-12, 0.0);

    let small_triangle = Triangle3D::new(a_small, b_small, c_small);
    if let Some(triangle) = small_triangle {
        assert!(!triangle.is_valid());
    }
}

#[test]
fn test_triangle_display() {
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(0.0, 1.0, 0.0);

    let triangle = Triangle3D::new(a, b, c).unwrap();
    let display_str = format!("{}", triangle);

    assert!(display_str.contains("Triangle3D"));
    assert!(display_str.contains("A:"));
    assert!(display_str.contains("B:"));
    assert!(display_str.contains("C:"));
}

#[test]
fn test_tolerant_eq() {
    let triangle1 = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let triangle2 = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let triangle3 = Triangle3D::new(
        Point3D::new(1.0, 1.0, 1.0),
        Point3D::new(2.0, 1.0, 1.0),
        Point3D::new(1.0, 2.0, 1.0),
    )
    .unwrap();

    assert!(triangle1.tolerant_eq(&triangle2, 0.01));
    assert!(!triangle1.tolerant_eq(&triangle3, 0.01));
}

#[test]
fn test_metadata_and_bounds_capabilities() {
    let triangle = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();

    assert_eq!(triangle.primitive_kind(), PrimitiveKind::Triangle);
    assert_eq!(triangle.area(), 0.5);

    let aabb = triangle.aabb().expect("Triangle should have an AABB");
    assert_eq!(aabb.min().x(), 0.0);
    assert_eq!(aabb.max().x(), 1.0);
    assert_eq!(aabb.min().y(), 0.0);
    assert_eq!(aabb.max().y(), 1.0);
}
