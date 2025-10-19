use crate::linalg::{Mat4d, Mat4f, Matrix4x4, Vector3, Vector4};

#[test]
fn test_matrix4x4_identity() {
    let m = Matrix4x4::<f64>::identity();
    let v = Vector4::<f64>::new(1.0, 2.0, 3.0, 1.0);
    let result = m.mul_vector(&v);

    assert_eq!(result.x(), 1.0);
    assert_eq!(result.y(), 2.0);
    assert_eq!(result.z(), 3.0);
    assert_eq!(result.w(), 1.0);
}

#[test]
fn test_matrix4x4_translation() {
    let translation = Matrix4x4::<f64>::translation(2.0, 3.0, 4.0);
    let v = Vector4::<f64>::new(1.0, 1.0, 1.0, 1.0);
    let translated = translation.mul_vector(&v);

    assert_eq!(translated.x(), 3.0); // 1 + 2
    assert_eq!(translated.y(), 4.0); // 1 + 3
    assert_eq!(translated.z(), 5.0); // 1 + 4
    assert_eq!(translated.w(), 1.0); // w成分は不変
}

#[test]
fn test_matrix4x4_scale() {
    let scale = Matrix4x4::<f64>::scale(2.0, 3.0, 4.0);
    let v = Vector4::<f64>::new(1.0, 1.0, 1.0, 1.0);
    let scaled = scale.mul_vector(&v);

    assert_eq!(scaled.x(), 2.0);
    assert_eq!(scaled.y(), 3.0);
    assert_eq!(scaled.z(), 4.0);
    assert_eq!(scaled.w(), 1.0); // w成分は不変
}

#[test]
fn test_matrix4x4_rotation_x() {
    use std::f64::consts::PI;

    let rotation = Matrix4x4::<f64>::rotation_x(PI / 2.0); // 90度回転
    let v = Vector4::<f64>::new(0.0, 1.0, 0.0, 1.0);
    let rotated = rotation.mul_vector(&v);

    // Y軸が Z軸になることを確認
    assert!(rotated.x().abs() < 1e-10);
    assert!(rotated.y().abs() < 1e-10);
    assert!((rotated.z() - 1.0).abs() < 1e-10);
    assert_eq!(rotated.w(), 1.0);
}

#[test]
fn test_matrix4x4_rotation_y() {
    use std::f64::consts::PI;

    let rotation = Matrix4x4::<f64>::rotation_y(PI / 2.0); // 90度回転
    let v = Vector4::<f64>::new(1.0, 0.0, 0.0, 1.0);
    let rotated = rotation.mul_vector(&v);

    // X軸が -Z軸になることを確認
    assert!(rotated.x().abs() < 1e-10);
    assert!(rotated.y().abs() < 1e-10);
    assert!((rotated.z() + 1.0).abs() < 1e-10);
    assert_eq!(rotated.w(), 1.0);
}

#[test]
fn test_matrix4x4_rotation_z() {
    use std::f64::consts::PI;

    let rotation = Matrix4x4::<f64>::rotation_z(PI / 2.0); // 90度回転
    let v = Vector4::<f64>::new(1.0, 0.0, 0.0, 1.0);
    let rotated = rotation.mul_vector(&v);

    // X軸が Y軸になることを確認
    assert!(rotated.x().abs() < 1e-10);
    assert!((rotated.y() - 1.0).abs() < 1e-10);
    assert!(rotated.z().abs() < 1e-10);
    assert_eq!(rotated.w(), 1.0);
}

#[test]
fn test_matrix4x4_rotation_axis() {
    use std::f64::consts::PI;

    let axis = Vector3::<f64>::new(0.0, 0.0, 1.0); // Z軸
    let rotation = Matrix4x4::<f64>::rotation_axis(&axis, PI / 2.0);
    let v = Vector4::<f64>::new(1.0, 0.0, 0.0, 1.0);
    let rotated = rotation.mul_vector(&v);

    // Z軸周りの90度回転：X軸がY軸になることを確認
    assert!(rotated.x().abs() < 1e-10);
    assert!((rotated.y() - 1.0).abs() < 1e-10);
    assert!(rotated.z().abs() < 1e-10);
    assert_eq!(rotated.w(), 1.0);
}

#[test]
fn test_matrix4x4_perspective() {
    use std::f64::consts::PI;

    let fovy = PI / 4.0; // 45度
    let aspect = 16.0 / 9.0;
    let near = 0.1;
    let far = 100.0;

    let proj = Matrix4x4::<f64>::perspective(fovy, aspect, near, far);
    // 透視投影行列の基本的な性質を確認
    assert!(proj.data[3][2] == -1.0); // w = -z
}

#[test]
fn test_matrix4x4_orthographic() {
    let left = -2.0;
    let right = 2.0;
    let bottom = -1.5;
    let top = 1.5;
    let near = 0.1;
    let far = 100.0;

    let proj = Matrix4x4::<f64>::orthographic(left, right, bottom, top, near, far);
    // 正射影行列の基本的な性質を確認
    assert!(proj.data[3][3] == 1.0); // w成分は変化しない
}

#[test]
fn test_matrix4x4_look_at() {
    let eye = Vector3::<f64>::new(0.0, 0.0, 5.0);
    let target = Vector3::<f64>::new(0.0, 0.0, 0.0);
    let up = Vector3::<f64>::new(0.0, 1.0, 0.0);

    let view = Matrix4x4::<f64>::look_at(&eye, &target, &up);
    assert!(view.is_ok());

    // カメラ変換行列が正常に作成されることを確認
    let view_matrix = view.unwrap();
    let origin = Vector4::<f64>::new(0.0, 0.0, 0.0, 1.0);
    let transformed = view_matrix.mul_vector(&origin);

    // 原点がカメラ空間で負のZ方向に変換されることを確認
    assert!(transformed.z() < 0.0);
}

#[test]
fn test_matrix4x4_determinant() {
    let m = Matrix4x4::<f64>::identity();
    assert_eq!(m.determinant(), 1.0);

    let scale = Matrix4x4::<f64>::scale(2.0, 3.0, 4.0);
    assert_eq!(scale.determinant(), 24.0); // 2 * 3 * 4 = 24
}

#[test]
fn test_matrix4x4_composition() {
    // 変換の合成：平行移動 × 回転 × スケール
    let scale = Matrix4x4::<f64>::scale(2.0, 2.0, 2.0);
    let rotation = Matrix4x4::<f64>::rotation_z(std::f64::consts::PI / 2.0);
    let translation = Matrix4x4::<f64>::translation(1.0, 0.0, 0.0);

    let combined = translation * rotation * scale;
    let v = Vector4::<f64>::new(1.0, 0.0, 0.0, 1.0);
    let result = combined.mul_vector(&v);

    // (1,0,0) → スケール(2,0,0) → 回転(0,2,0) → 平行移動(1,2,0)
    assert!((result.x() - 1.0).abs() < 1e-10);
    assert!((result.y() - 2.0).abs() < 1e-10);
    assert!(result.z().abs() < 1e-10);
    assert_eq!(result.w(), 1.0);
}

#[test]
fn test_type_aliases() {
    let m_f32 = Mat4f::identity();
    let m_f64 = Mat4d::identity();

    assert_eq!(m_f32.trace(), 4.0);
    assert_eq!(m_f64.trace(), 4.0);
}
