use crate::linalg::vector::*;

// 動的サイズベクトルのテスト
#[test]
fn test_vector_creation_f64() {
    let v = Vector::<f64>::new(vec![1.0, 2.0, 3.0]);
    assert_eq!(v.len(), 3);
    assert_eq!(v.get(0), 1.0);

    let zeros = Vector::<f64>::zeros(5);
    assert_eq!(zeros.len(), 5);
    assert_eq!(zeros.get(0), 0.0);
}

#[test]
fn test_vector_operations_f64() {
    let v1 = Vector::<f64>::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::<f64>::new(vec![4.0, 5.0, 6.0]);

    let dot = v1.dot(&v2).unwrap();
    assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32

    let norm = v1.norm();
    assert!((norm - (14.0_f64).sqrt()).abs() < 1e-10);
}

#[test]
fn test_vector_operations_f32() {
    let v1 = Vector::<f32>::new(vec![1.0, 2.0, 3.0]);
    let v2 = Vector::<f32>::new(vec![4.0, 5.0, 6.0]);

    let dot = v1.dot(&v2).unwrap();
    assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32

    let norm = v1.norm();
    assert!((norm - (14.0_f32).sqrt()).abs() < 1e-6);
}

#[test]
fn test_fixed_size_vectors_f64() {
    let v1 = Vector3::<f64>::new(1.0, 2.0, 3.0);
    let v2 = Vector3::<f64>::new(4.0, 5.0, 6.0);

    let cross = v1.cross(&v2);
    assert_eq!(cross.x(), -3.0);
    assert_eq!(cross.y(), 6.0);
    assert_eq!(cross.z(), -3.0);
}

#[test]
fn test_fixed_size_vectors_f32() {
    let v1 = Vector3::<f32>::new(1.0, 2.0, 3.0);
    let v2 = Vector3::<f32>::new(4.0, 5.0, 6.0);

    let cross = v1.cross(&v2);
    assert_eq!(cross.x(), -3.0);
    assert_eq!(cross.y(), 6.0);
    assert_eq!(cross.z(), -3.0);
}

#[test]
fn test_vector4_homogeneous_coords() {
    let v3 = Vector3::<f64>::new(1.0, 2.0, 3.0);
    let v4 = Vector4::from_euclidean(v3);
    
    assert_eq!(v4.x(), 1.0);
    assert_eq!(v4.y(), 2.0);
    assert_eq!(v4.z(), 3.0);
    assert_eq!(v4.w(), 1.0);
    
    let v3_back = v4.to_euclidean().unwrap();
    assert_eq!(v3_back.x(), 1.0);
    assert_eq!(v3_back.y(), 2.0);
    assert_eq!(v3_back.z(), 3.0);
}

#[test]
fn test_type_aliases() {
    // 型エイリアスのテスト
    use crate::linalg::{Vec3f, Vec3d};
    
    let v_f32 = Vec3f::new(1.0, 2.0, 3.0);
    let v_f64 = Vec3d::new(1.0, 2.0, 3.0);
    
    assert_eq!(v_f32.norm(), v_f64.norm() as f32);
}