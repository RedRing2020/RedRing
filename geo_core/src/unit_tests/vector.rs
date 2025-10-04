/// ベクトル演算のユニットテスト

use crate::vector::{Vector2D, Vector3D, Direction3D, Vector};
use crate::tolerance::ToleranceContext;

// Vector2D テスト
#[test]
fn test_vector2d_creation() {
    let v2 = Vector2D::from_f64(1.0, 2.0);

    assert_eq!(v2.x().value(), 1.0);
    assert_eq!(v2.y().value(), 2.0);
}

#[test]
fn test_vector2d_cross_product() {
    let v1 = Vector2D::from_f64(1.0, 0.0);
    let v2 = Vector2D::from_f64(0.0, 1.0);

    let cross = v1.cross_2d(&v2);
    assert_eq!(cross.value(), 1.0);
}

#[test]
fn test_vector2d_dot_product() {
    let v1 = Vector2D::from_f64(1.0, 2.0);
    let v2 = Vector2D::from_f64(3.0, 4.0);

    let dot = v1.dot(&v2);
    assert_eq!(dot.value(), 11.0); // 1*3 + 2*4 = 11
}

#[test]
fn test_vector2d_normalization() {
    let context = ToleranceContext::standard();
    let v = Vector2D::from_f64(3.0, 4.0);

    let normalized = v.normalize(&context).unwrap();
    assert!((normalized.norm().value() - 1.0).abs() < context.linear);
    assert_eq!(normalized.x().value(), 0.6);
    assert_eq!(normalized.y().value(), 0.8);
}

#[test]
fn test_vector2d_perpendicular() {
    let v = Vector2D::from_f64(1.0, 0.0);
    let perp = v.perpendicular();

    assert_eq!(perp.x().value(), 0.0);
    assert_eq!(perp.y().value(), 1.0);
}

#[test]
fn test_vector2d_rotation() {
    use std::f64::consts::PI;
    let v = Vector2D::from_f64(1.0, 0.0);
    let rotated = v.rotate(crate::scalar::Scalar::new(PI / 2.0));

    assert!((rotated.x().value()).abs() < 1e-10); // ほぼ0
    assert!((rotated.y().value() - 1.0).abs() < 1e-10); // ほぼ1
}

#[test]
fn test_vector2d_parallel_perpendicular() {
    let context = ToleranceContext::standard();
    let v1 = Vector2D::from_f64(1.0, 0.0);
    let v2 = Vector2D::from_f64(2.0, 0.0);
    let v3 = Vector2D::from_f64(0.0, 1.0);

    assert!(v1.is_parallel_to(&v2, &context));
    assert!(v1.is_perpendicular_to(&v3, &context));
    assert!(!v1.is_parallel_to(&v3, &context));
}

// Vector3D テスト
#[test]
fn test_vector3d_creation() {
    let v3 = Vector3D::from_f64(1.0, 2.0, 3.0);

    assert_eq!(v3.x().value(), 1.0);
    assert_eq!(v3.y().value(), 2.0);
    assert_eq!(v3.z().value(), 3.0);
}

#[test]
fn test_vector3d_cross_product() {
    let v1 = Vector3D::from_f64(1.0, 0.0, 0.0);
    let v2 = Vector3D::from_f64(0.0, 1.0, 0.0);

    let cross = v1.cross(&v2);
    assert_eq!(cross.x().value(), 0.0);
    assert_eq!(cross.y().value(), 0.0);
    assert_eq!(cross.z().value(), 1.0);
}

#[test]
fn test_vector3d_dot_product() {
    let v1 = Vector3D::from_f64(1.0, 0.0, 0.0);
    let v2 = Vector3D::from_f64(0.0, 1.0, 0.0);

    let dot = v1.dot(&v2);
    assert_eq!(dot.value(), 0.0);
}

#[test]
fn test_vector3d_normalization() {
    let context = ToleranceContext::standard();
    let v = Vector3D::from_f64(3.0, 4.0, 0.0);

    let normalized = v.normalize(&context).unwrap();
    assert!((normalized.norm().value() - 1.0).abs() < context.linear);
    assert_eq!(normalized.x().value(), 0.6);
    assert_eq!(normalized.y().value(), 0.8);
}

#[test]
fn test_vector3d_scalar_triple_product() {
    let a = Vector3D::from_f64(1.0, 0.0, 0.0);
    let b = Vector3D::from_f64(0.0, 1.0, 0.0);
    let c = Vector3D::from_f64(0.0, 0.0, 1.0);

    let result = a.scalar_triple_product(&b, &c);
    assert_eq!(result.value(), 1.0);
}

#[test]
fn test_vector3d_vector_triple_product() {
    let a = Vector3D::from_f64(1.0, 0.0, 0.0);
    let b = Vector3D::from_f64(0.0, 1.0, 0.0);
    let c = Vector3D::from_f64(0.0, 0.0, 1.0);

    let result = a.vector_triple_product(&b, &c);
    // a×(b×c) = b(a·c) - c(a·b) = 0 (a·c = a·b = 0)
    assert_eq!(result.x().value(), 0.0);
    assert_eq!(result.y().value(), 0.0);
    assert_eq!(result.z().value(), 0.0);
}

// Direction3D テスト
#[test]
fn test_direction3d_creation() {
    let context = ToleranceContext::standard();
    let dir = Direction3D::new(1.0, 0.0, 0.0, &context).unwrap();

    assert!(dir.as_vector().is_unit(&context));
    assert_eq!(dir.x().value(), 1.0);
    assert_eq!(dir.y().value(), 0.0);
    assert_eq!(dir.z().value(), 0.0);
}

#[test]
fn test_direction3d_from_vector() {
    let context = ToleranceContext::standard();
    let v = Vector3D::from_f64(2.0, 0.0, 0.0);
    let dir = Direction3D::from_vector(v, &context).unwrap();

    assert!(dir.as_vector().is_unit(&context));
    assert_eq!(dir.x().value(), 1.0);
    assert_eq!(dir.y().value(), 0.0);
    assert_eq!(dir.z().value(), 0.0);
}

#[test]
fn test_direction3d_cross_product() {
    let context = ToleranceContext::standard();
    let dir1 = Direction3D::new(1.0, 0.0, 0.0, &context).unwrap();
    let dir2 = Direction3D::new(0.0, 1.0, 0.0, &context).unwrap();

    let cross = dir1.cross(&dir2, &context).unwrap();
    assert_eq!(cross.x().value(), 0.0);
    assert_eq!(cross.y().value(), 0.0);
    assert_eq!(cross.z().value(), 1.0);
}

#[test]
fn test_direction3d_orthonormal_basis() {
    let context = ToleranceContext::standard();
    let dir = Direction3D::new(1.0, 0.0, 0.0, &context).unwrap();

    let (u, v) = dir.orthonormal_basis(&context);

    // uとvは互いに直交し、dirとも直交している
    assert!(dir.as_vector().is_perpendicular_to(&u, &context));
    assert!(dir.as_vector().is_perpendicular_to(&v, &context));
    assert!(u.is_perpendicular_to(&v, &context));

    // uとvは単位ベクトル
    assert!(u.is_unit(&context));
    assert!(v.is_unit(&context));
}

#[test]
fn test_vector3d_parallel_perpendicular() {
    let context = ToleranceContext::standard();
    let v1 = Vector3D::from_f64(1.0, 0.0, 0.0);
    let v2 = Vector3D::from_f64(2.0, 0.0, 0.0);
    let v3 = Vector3D::from_f64(0.0, 1.0, 0.0);

    assert!(v1.is_parallel_to(&v2, &context));
    assert!(v1.is_perpendicular_to(&v3, &context));
    assert!(!v1.is_parallel_to(&v3, &context));
}
