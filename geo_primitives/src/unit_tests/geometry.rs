/// 幾何構造体のテスト
/// 基本的な幾何プリミティブのテスト

use crate::geometry3d::{Point3D, Vector3D};

#[test]
fn test_point_operations() {
    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let p2 = Point3D::new(4.0, 5.0, 6.0);

    assert_eq!(p1.x(), 1.0);
    assert_eq!(p1.y(), 2.0);
    assert_eq!(p1.z(), 3.0);

    let distance = p1.distance_to(&p2);
    let expected = ((4.0f64-1.0)*(4.0-1.0) + (5.0-2.0)*(5.0-2.0) + (6.0-3.0)*(6.0-3.0)).sqrt();
    assert!((distance - expected).abs() < 1e-10);
}

#[test]
fn test_vector_operations() {
    let v1 = Vector3D::new(1.0, 0.0, 0.0);
    let v2 = Vector3D::new(0.0, 1.0, 0.0);

    let cross = v1.cross(&v2);
    assert!((cross.x() - 0.0).abs() < 1e-10);
    assert!((cross.y() - 0.0).abs() < 1e-10);
    assert!((cross.z() - 1.0).abs() < 1e-10);

    let dot = v1.dot(&v2);
    assert!(dot.abs() < 1e-10);
}

#[test]
fn test_point_vector_integration() {
    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let v1 = Vector3D::new(1.0, 1.0, 1.0);

    let p2 = p1 + v1;
    assert_eq!(p2.y(), 3.0);
    assert_eq!(p2.z(), 4.0);
}
