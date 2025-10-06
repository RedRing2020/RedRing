use crate::linalg::{Matrix3x3, Vector3, Mat3f, Mat3d};

#[test]
fn test_matrix3x3_identity() {
    let m = Matrix3x3::<f64>::identity();
    let v = Vector3::<f64>::new(1.0, 2.0, 3.0);
    let result = m.mul_vector(&v);
    
    assert_eq!(result.x(), 1.0);
    assert_eq!(result.y(), 2.0);
    assert_eq!(result.z(), 3.0);
}

#[test]
fn test_matrix3x3_determinant() {
    let m = Matrix3x3::<f64>::new(
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0
    );
    
    let det = m.determinant();
    assert_eq!(det, 0.0); // この行列は特異（行列式が0）
}

#[test]
fn test_matrix3x3_inverse() {
    let m = Matrix3x3::<f64>::new(
        1.0, 0.0, 0.0,
        0.0, 2.0, 0.0,
        0.0, 0.0, 3.0
    );
    
    let inv = m.inverse().unwrap();
    let expected = Matrix3x3::<f64>::new(
        1.0, 0.0, 0.0,
        0.0, 0.5, 0.0,
        0.0, 0.0, 1.0/3.0
    );
    
    for i in 0..3 {
        for j in 0..3 {
            assert!((inv.data[i][j] - expected.data[i][j]).abs() < 1e-10);
        }
    }
}

#[test]
fn test_matrix3x3_rotation_x() {
    use std::f64::consts::PI;
    
    let rotation = Matrix3x3::<f64>::rotation_x(PI / 2.0); // 90度回転
    let v = Vector3::<f64>::new(0.0, 1.0, 0.0);
    let rotated = rotation.mul_vector(&v);
    
    // Y軸が Z軸になることを確認
    assert!(rotated.x().abs() < 1e-10);
    assert!(rotated.y().abs() < 1e-10);
    assert!((rotated.z() - 1.0).abs() < 1e-10);
}

#[test]
fn test_matrix3x3_rotation_y() {
    use std::f64::consts::PI;
    
    let rotation = Matrix3x3::<f64>::rotation_y(PI / 2.0); // 90度回転
    let v = Vector3::<f64>::new(1.0, 0.0, 0.0);
    let rotated = rotation.mul_vector(&v);
    
    // X軸が -Z軸になることを確認
    assert!(rotated.x().abs() < 1e-10);
    assert!(rotated.y().abs() < 1e-10);
    assert!((rotated.z() + 1.0).abs() < 1e-10);
}

#[test]
fn test_matrix3x3_rotation_z() {
    use std::f64::consts::PI;
    
    let rotation = Matrix3x3::<f64>::rotation_z(PI / 2.0); // 90度回転
    let v = Vector3::<f64>::new(1.0, 0.0, 0.0);
    let rotated = rotation.mul_vector(&v);
    
    // X軸が Y軸になることを確認
    assert!(rotated.x().abs() < 1e-10);
    assert!((rotated.y() - 1.0).abs() < 1e-10);
    assert!(rotated.z().abs() < 1e-10);
}

#[test]
fn test_matrix3x3_scale() {
    let scale = Matrix3x3::<f64>::scale(2.0, 3.0, 4.0);
    let v = Vector3::<f64>::new(1.0, 1.0, 1.0);
    let scaled = scale.mul_vector(&v);
    
    assert_eq!(scaled.x(), 2.0);
    assert_eq!(scaled.y(), 3.0);
    assert_eq!(scaled.z(), 4.0);
}

#[test]
fn test_matrix3x3_translation_homogeneous() {
    let translation = Matrix3x3::<f64>::translation(2.0, 3.0);
    let v = Vector3::<f64>::new(1.0, 1.0, 1.0); // 同次座標 (1, 1, 1)
    let translated = translation.mul_vector(&v);
    
    assert_eq!(translated.x(), 3.0); // 1 + 2*1 = 3
    assert_eq!(translated.y(), 4.0); // 1 + 3*1 = 4
    assert_eq!(translated.z(), 1.0); // w成分は不変
}

#[test]
fn test_type_aliases() {
    let m_f32 = Mat3f::identity();
    let m_f64 = Mat3d::identity();
    
    assert_eq!(m_f32.trace(), 3.0);
    assert_eq!(m_f64.trace(), 3.0);
}