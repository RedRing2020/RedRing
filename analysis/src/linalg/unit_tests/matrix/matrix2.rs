use crate::linalg::{Mat2d, Mat2f, Matrix2x2, Vector2};

#[test]
fn test_matrix2x2_operations_f64() {
    let m1 = Matrix2x2::<f64>::new(1.0, 2.0, 3.0, 4.0);
    let m2 = Matrix2x2::<f64>::new(5.0, 6.0, 7.0, 8.0);

    let sum = m1 + m2;
    assert_eq!(sum.data[0][0], 6.0);
    assert_eq!(sum.data[1][1], 12.0);

    let det = m1.determinant();
    assert_eq!(det, -2.0); // 1*4 - 2*3 = -2
}

#[test]
fn test_matrix2x2_operations_f32() {
    let m1 = Matrix2x2::<f32>::new(1.0, 2.0, 3.0, 4.0);
    let m2 = Matrix2x2::<f32>::new(5.0, 6.0, 7.0, 8.0);

    let sum = m1 + m2;
    assert_eq!(sum.data[0][0], 6.0);
    assert_eq!(sum.data[1][1], 12.0);

    let det = m1.determinant();
    assert_eq!(det, -2.0); // 1*4 - 2*3 = -2
}

#[test]
fn test_matrix2x2_inverse() {
    let m = Matrix2x2::<f64>::new(1.0, 2.0, 3.0, 4.0);
    let inv = m.inverse().unwrap();

    let product = m * inv;
    let identity = Matrix2x2::<f64>::identity();

    // 結果が単位行列に近いことを確認
    for i in 0..2 {
        for j in 0..2 {
            assert!((product.data[i][j] - identity.data[i][j]).abs() < 1e-10);
        }
    }
}

#[test]
fn test_matrix2x2_vector_multiplication() {
    let m = Matrix2x2::<f64>::identity();
    let v = Vector2::<f64>::new(1.0, 2.0);
    let result = m.mul_vector(&v);

    assert_eq!(result.x(), 1.0);
    assert_eq!(result.y(), 2.0);
}

#[test]
fn test_matrix2x2_rotation() {
    use std::f64::consts::PI;

    let rotation = Matrix2x2::<f64>::rotation(PI / 2.0); // 90度回転
    let v = Vector2::<f64>::new(1.0, 0.0);
    let rotated = rotation.mul_vector(&v);

    // (1,0) が (0,1) になることを確認（誤差を考慮）
    assert!(rotated.x().abs() < 1e-10);
    assert!((rotated.y() - 1.0).abs() < 1e-10);
}

#[test]
fn test_matrix2x2_scale() {
    let scale = Matrix2x2::<f64>::scale(2.0, 3.0);
    let v = Vector2::<f64>::new(1.0, 1.0);
    let scaled = scale.mul_vector(&v);

    assert_eq!(scaled.x(), 2.0);
    assert_eq!(scaled.y(), 3.0);
}

#[test]
fn test_type_aliases() {
    let m_f32 = Mat2f::identity();
    let m_f64 = Mat2d::identity();

    assert_eq!(m_f32.trace(), 2.0);
    assert_eq!(m_f64.trace(), 2.0);
}
