/// 共通ユーティリティと分類システムのテスト
/// classification, geometry_utilsのテスト

use crate::traits::common::{PrimitiveKind, DimensionClass, GeometryPrimitive};
use crate::traits::common::geometry_utils::*;
use crate::geometry2d::Point2D;
use crate::geometry3d::Point3D;

#[test]
fn test_primitive_kind_dimension() {
    assert_eq!(PrimitiveKind::Point.dimension(), DimensionClass::Zero);
    assert_eq!(PrimitiveKind::LineSegment.dimension(), DimensionClass::One);
    assert_eq!(PrimitiveKind::Circle.dimension(), DimensionClass::Two);
    assert_eq!(PrimitiveKind::Sphere.dimension(), DimensionClass::Three);
}

#[test]
fn test_primitive_kind_properties() {
    assert!(PrimitiveKind::BezierCurve.is_parametric());
    assert!(PrimitiveKind::Circle.is_analytical());
    assert!(PrimitiveKind::TriangleMesh.is_mesh());

    assert!(PrimitiveKind::LineSegment.is_curve());
    assert!(PrimitiveKind::Circle.is_surface());
    assert!(PrimitiveKind::Sphere.is_solid());
}

#[test]
fn test_point2d_utilities() {
    let p1 = Point2D::new(1.0, 2.0);
    let p2 = Point2D::new(3.0, 4.0);

    // Conversion test
    let (x, y) = point2d_to_f64(&p1);
    assert_eq!(x, 1.0);
    assert_eq!(y, 2.0);

    let p3 = point2d_from_f64(x, y);
    assert_eq!(p3.x(), 1.0);
    assert_eq!(p3.y(), 2.0);
}

#[test]
fn test_point3d_utilities() {
    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let p2 = Point3D::new(4.0, 5.0, 6.0);

    // Conversion test
    let (x, y, z) = point3d_to_f64(&p1);
    assert_eq!(x, 1.0);
    assert_eq!(y, 2.0);
    assert_eq!(z, 3.0);

    let p3 = point3d_from_f64(x, y, z);
    assert_eq!(p3.x(), 1.0);
    assert_eq!(p3.y(), 2.0);
    assert_eq!(p3.z(), 3.0);
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
        Point2D::new(3.0, 0.0),
        Point2D::new(0.0, 3.0),
    ];

    let centroid = point2d_centroid(&points).unwrap();
    assert_eq!(centroid.x(), 1.0);
    assert_eq!(centroid.y(), 1.0);
}

#[test]
fn test_bounding_box_3d() {
    let points = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(2.0, 1.0, 3.0),
        Point3D::new(1.0, 3.0, 2.0),
    ];

    let bbox = point3d_bounding_box(&points).unwrap();
    assert_eq!(bbox, (0.0, 0.0, 0.0, 2.0, 3.0, 3.0));
}

#[test]
fn test_centroid_3d() {
    let points = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(3.0, 0.0, 0.0),
        Point3D::new(0.0, 3.0, 0.0),
        Point3D::new(0.0, 0.0, 3.0),
    ];

    let centroid = point3d_centroid(&points).unwrap();
    assert_eq!(centroid.x(), 0.75);
    assert_eq!(centroid.y(), 0.75);
    assert_eq!(centroid.z(), 0.75);
}
