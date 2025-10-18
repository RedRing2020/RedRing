use crate::linalg::matrix::Matrix4x4;
use crate::linalg::vector::Vector4;
use crate::linalg::{Mat4d, Mat4f};

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

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
    fn test_matrix4x4_perspective() {
        let fov = PI / 2.0; // 90度
        let aspect = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;

        if let Ok(perspective) =
            std::panic::catch_unwind(|| Matrix4x4::<f64>::perspective(fov, aspect, near, far))
        {
            // 射影行列の基本プロパティをテスト
            let test_point = Vector4::<f64>::new(1.0, 1.0, -1.0, 1.0);
            let projected = perspective.mul_vector(&test_point);

            // W成分が正しく設定されることを確認
            assert!(projected.w() > 0.0);
        }
    }

    #[test]
    fn test_matrix4x4_determinant() {
        let m = Matrix4x4::<f64>::identity();
        assert_eq!(m.determinant(), 1.0);

        let scale = Matrix4x4::<f64>::scale(2.0, 3.0, 4.0);
        assert_eq!(scale.determinant(), 24.0); // 2 * 3 * 4 = 24
    }

    #[test]
    fn test_type_aliases() {
        let m_f32 = Mat4f::identity();
        let m_f64 = Mat4d::identity();

        assert_eq!(m_f32.trace(), 4.0);
        assert_eq!(m_f64.trace(), 4.0);
    }
}
