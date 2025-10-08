#[cfg(test)]
use crate::geometry3d::Vector3D;
#[test]
fn test_vector3d_creation() {
    let v = Vector3D::new(1.0, 2.0, 3.0);
    assert_eq!(v.x(), 1.0);
    assert_eq!(v.y(), 2.0);
    assert_eq!(v.z(), 3.0);
}

#[test]
fn test_vector3d_constants() {
    let zero = Vector3D::zero();
    assert_eq!(zero.x(), 0.0);
    assert_eq!(zero.y(), 0.0);
    assert_eq!(zero.z(), 0.0);

    let unit_x = Vector3D::unit_x();
    assert_eq!(unit_x.x(), 1.0);
    assert_eq!(unit_x.y(), 0.0);
    assert_eq!(unit_x.z(), 0.0);
}

#[test]
fn test_vector3d_length() {
    let v = Vector3D::new(3.0, 4.0, 0.0);
    assert!((v.length() - 5.0).abs() < 1e-10);
    assert!((v.length_squared() - 25.0).abs() < 1e-10);
}

#[test]
fn test_vector3d_normalize() {
    let v = Vector3D::new(3.0, 4.0, 0.0);
    let normalized = v.normalize().unwrap();
    assert!((normalized.length() - 1.0).abs() < 1e-10);

    let zero = Vector3D::zero();
    assert!(zero.normalize().is_none());
}

#[test]
fn test_vector3d_dot_product() {
    let v1 = Vector3D::new(1.0, 2.0, 3.0);
    let v2 = Vector3D::new(4.0, 5.0, 6.0);
    let dot = v1.dot(&v2);
    assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32
}

#[test]
fn test_vector3d_cross_product() {
    let v1 = Vector3D::unit_x();
    let v2 = Vector3D::unit_y();
    let cross = v1.cross(&v2);
    let unit_z = Vector3D::unit_z();
    assert!((cross.x() - unit_z.x()).abs() < 1e-10);
    assert!((cross.y() - unit_z.y()).abs() < 1e-10);
    assert!((cross.z() - unit_z.z()).abs() < 1e-10);
}

#[test]
fn test_vector3d_arithmetic() {
    let v1 = Vector3D::new(1.0, 2.0, 3.0);
    let v2 = Vector3D::new(4.0, 5.0, 6.0);

    let add = v1 + v2;
    assert_eq!(add, Vector3D::new(5.0, 7.0, 9.0));

    let sub = v2 - v1;
    assert_eq!(sub, Vector3D::new(3.0, 3.0, 3.0));

    let mul = v1 * 2.0;
    assert_eq!(mul, Vector3D::new(2.0, 4.0, 6.0));

    let neg = -v1;
    assert_eq!(neg, Vector3D::new(-1.0, -2.0, -3.0));
}

#[test]
fn test_vector3d_add_scaled() {
    let v1 = Vector3D::new(1.0, 2.0, 3.0);
    let v2 = Vector3D::new(4.0, 5.0, 6.0);
    let result = v1.add_scaled(&v2, 2.0);
    assert_eq!(result, Vector3D::new(9.0, 12.0, 15.0)); // (1,2,3) + 2*(4,5,6) = (9,12,15)
}

#[test]
fn test_vector3d_scalar_triple_product() {
    let a = Vector3D::unit_x();
    let b = Vector3D::unit_y();
    let c = Vector3D::unit_z();
    let result = a.scalar_triple_product(&b, &c);
    assert!((result - 1.0).abs() < 1e-10); // x · (y × z) = 1
}

#[test]
fn test_vector3d_vector_triple_product() {
    let a = Vector3D::new(1.0, 0.0, 0.0);
    let b = Vector3D::new(0.0, 1.0, 0.0);
    let c = Vector3D::new(0.0, 0.0, 1.0);
    let result = a.vector_triple_product(&b, &c);
    // a × (b × c) = b(a·c) - c(a·b) = b*0 - c*0 = (0,0,0)
    assert_eq!(result, Vector3D::zero());
}

// geometry.rsから移動したテスト
#[test]
fn test_vector_operations_extended() {
    let v1 = Vector3D::new(1.0, 0.0, 0.0);
    let v2 = Vector3D::new(0.0, 1.0, 0.0);

    let cross = v1.cross(&v2);
    assert!((cross.x() - 0.0).abs() < 1e-10);
    assert!((cross.y() - 0.0).abs() < 1e-10);
    assert!((cross.z() - 1.0).abs() < 1e-10);

    let dot = v1.dot(&v2);
    assert!(dot.abs() < 1e-10);
}

// vector_traits.rsから移動したトレイトテスト
#[test]
fn test_vector_trait_with_vector3d() {
    use crate::traits::Vector;

    let v1 = Vector3D::new(1.0, 2.0, 3.0);
    let v2 = Vector3D::unit_x();

    // Vector トレイトのテスト
    assert!((v1.length() - (14.0_f64).sqrt()).abs() < 1e-10);
    assert_eq!(v1.dot(&v2), 1.0);
    assert!(v2.is_unit(1e-10));

    // 成分アクセス
    assert_eq!(v1[0], 1.0);
    assert_eq!(v1[1], 2.0);
    assert_eq!(v1[2], 3.0);

    // Vector3D トレイトのテスト
    let cross = v1.cross(&v2);
    assert_eq!(cross.x(), 0.0);
    assert_eq!(cross.y(), 3.0);
    assert_eq!(cross.z(), -2.0);

    // 平行性テスト
    let parallel = Vector3D::new(2.0, 4.0, 6.0);
    assert!(v1.is_parallel_to(&parallel, 1e-10));

    // 垂直性テスト（v1 = (1,2,3)に垂直なベクトルを作成）
    let perpendicular = Vector3D::new(2.0, -1.0, 0.0); // 1*2 + 2*(-1) + 3*0 = 0
    assert!(v1.is_perpendicular_to(&perpendicular, 1e-10));
}
