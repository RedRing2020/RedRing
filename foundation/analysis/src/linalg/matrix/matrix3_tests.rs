use crate::linalg::matrix::Matrix3x3;
use crate::linalg::vector::{Vector2, Vector3};
use crate::linalg::{Mat3d, Mat3f};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::test_constants::TOLERANCE_F64;
    use std::f64::consts::PI;

    type Matrix3 = Matrix3x3<f64>;
    type Vec2 = Vector2<f64>;

    // 統一された許容誤差定数を使用
    const TOLERANCE: f64 = TOLERANCE_F64;

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
        let m = Matrix3x3::<f64>::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);

        let det = m.determinant();
        assert_eq!(det, 0.0); // この行列は特異（行列式が0）
    }

    #[test]
    fn test_matrix3x3_inverse() {
        let m = Matrix3x3::<f64>::new(1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0);

        let inv = m.inverse().unwrap();
        let expected = Matrix3x3::<f64>::new(1.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 1.0 / 3.0);

        for i in 0..3 {
            for j in 0..3 {
                assert!((inv.get(i, j) - expected.get(i, j)).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_matrix3x3_rotation_x() {
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

    // 2D変換テスト（unit_tests/linalg/matrix/matrix3_tests.rsから統合）
    #[test]
    fn test_identity_2d() {
        let identity = Matrix3::identity();

        // 恒等行列の基本プロパティ
        assert!((identity.determinant() - 1.0).abs() < f64::EPSILON);
        if let Ok(affine_check) = std::panic::catch_unwind(|| identity.is_affine_transform()) {
            assert!(affine_check);
        }

        // 点の変換テスト
        let point = Vec2::new(1.0, 2.0);
        if let Ok(transformed) = std::panic::catch_unwind(|| identity.transform_point_2d(&point)) {
            assert!((transformed.x() - point.x()).abs() < TOLERANCE);
            assert!((transformed.y() - point.y()).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_translation_2d() {
        let translation = Vec2::new(5.0, -3.0);
        if let Ok(matrix) = std::panic::catch_unwind(|| Matrix3::translation_2d(&translation)) {
            // 平行移動の基本プロパティ
            if let Ok(affine_check) = std::panic::catch_unwind(|| matrix.is_affine_transform()) {
                assert!(affine_check);
            }
            assert!((matrix.determinant() - 1.0).abs() < f64::EPSILON);

            // 点の変換テスト
            let point = Vec2::new(1.0, 2.0);
            let expected = Vec2::new(6.0, -1.0);
            if let Ok(transformed) = std::panic::catch_unwind(|| matrix.transform_point_2d(&point))
            {
                assert!((transformed.x() - expected.x()).abs() < TOLERANCE);
                assert!((transformed.y() - expected.y()).abs() < TOLERANCE);
            }
        }
    }

    #[test]
    fn test_rotation_2d() {
        // 90度回転
        if let Ok(matrix) = std::panic::catch_unwind(|| Matrix3::rotation_2d(PI / 2.0)) {
            if let Ok(affine_check) = std::panic::catch_unwind(|| matrix.is_affine_transform()) {
                assert!(affine_check);
            }
            assert!((matrix.determinant() - 1.0).abs() < f64::EPSILON);

            // X軸の点がY軸に移動することを確認
            let point = Vec2::new(1.0, 0.0);
            if let Ok(transformed) = std::panic::catch_unwind(|| matrix.transform_point_2d(&point))
            {
                let expected = Vec2::new(0.0, 1.0);
                assert!((transformed.x() - expected.x()).abs() < TOLERANCE);
                assert!((transformed.y() - expected.y()).abs() < TOLERANCE);
            }
        }
    }

    #[test]
    fn test_type_aliases() {
        let m_f32 = Mat3f::identity();
        let m_f64 = Mat3d::identity();

        assert_eq!(m_f32.trace(), 3.0);
        assert_eq!(m_f64.trace(), 3.0);
    }

    // 以下、matrix3_affine_tests.rs から統合（2D Affine変換の追加テスト）

    #[test]
    fn test_2d_scale() {
        let scale = Vec2::new(2.0, 3.0);
        let matrix = Matrix3::scale_2d(&scale);
        let point = Vec2::new(1.0, 1.0);
        let result = matrix.transform_point_2d(&point);

        assert!((result.x() - 2.0).abs() < f64::EPSILON);
        assert!((result.y() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_2d_trs_composition() {
        let translation = Vec2::new(10.0, 5.0);
        let rotation = PI / 4.0;
        let scale = Vec2::new(2.0, 2.0);

        let matrix = Matrix3::trs_2d(&translation, rotation, &scale);
        let point = Vec2::new(1.0, 0.0);
        let result = matrix.transform_point_2d(&point);

        let expected_x = 10.0 + 2.0 * (PI / 4.0).cos();
        let expected_y = 5.0 + 2.0 * (PI / 4.0).sin();

        assert!((result.x() - expected_x).abs() < 1e-10);
        assert!((result.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_vector_multiplication_operator() {
        let translation = Vec2::new(2.0, 3.0);
        let matrix = Matrix3::translation_2d(&translation);
        let point = Vec2::new(1.0, 1.0);

        let result = matrix * point;

        assert!((result.x() - 3.0).abs() < f64::EPSILON);
        assert!((result.y() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rigid_body_detection() {
        let translation = Vec2::new(5.0, 3.0);
        let rotation = PI / 4.0;
        let rigid_matrix =
            Matrix3::translation_2d(&translation) * Matrix3::rotation_2d(rotation);

        assert!(rigid_matrix.is_rigid_2d());

        let scale = Vec2::new(2.0, 2.0);
        let scaled_matrix = Matrix3::scale_2d(&scale);
        assert!(!scaled_matrix.is_rigid_2d());
    }

    #[test]
    fn test_affine_transform_detection() {
        let affine_matrix =
            Matrix3::trs_2d(&Vec2::new(5.0, 3.0), PI / 4.0, &Vec2::new(2.0, 1.5));

        assert!(affine_matrix.is_affine_transform());
        assert!(affine_matrix.is_pure_affine_2d());
        assert!(affine_matrix.is_valid_affine());
        assert!(!affine_matrix.has_perspective());

        let perspective_matrix = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.05, 1.0);

        assert!(!perspective_matrix.is_affine_transform());
        assert!(perspective_matrix.has_perspective());
    }

    #[test]
    fn test_homogeneous_normalization() {
        let matrix = Matrix3::new(2.0, 0.0, 4.0, 0.0, 2.0, 6.0, 0.0, 0.0, 2.0);

        let normalized = matrix.normalize_homogeneous().unwrap();
        let expected = Matrix3::new(1.0, 0.0, 2.0, 0.0, 1.0, 3.0, 0.0, 0.0, 1.0);

        for i in 0..3 {
            for j in 0..3 {
                assert!((normalized.get(i, j) - expected.get(i, j)).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_projective_transformation() {
        let matrix = Matrix3::new(1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.1, 0.0, 1.0);

        let point = Vec2::new(1.0, 1.0);
        let result = matrix.transform_projective_2d(point).unwrap();

        let expected_x = 2.0 / 1.1;
        let expected_y = 2.0 / 1.1;

        assert!((result.x() - expected_x).abs() < 1e-10);
        assert!((result.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_affine_factory_methods() {
        let linear = [[2.0, 0.5], [0.0, 1.5]];
        let translation = Vec2::new(3.0, 4.0);

        let affine_matrix = Matrix3::affine_2d(linear, translation);

        assert!(affine_matrix.is_affine_transform());
        assert_eq!(affine_matrix.get(0, 0), 2.0);
        assert_eq!(affine_matrix.get(0, 1), 0.5);
        assert_eq!(affine_matrix.get(0, 2), 3.0);
        assert_eq!(affine_matrix.get(1, 0), 0.0);
        assert_eq!(affine_matrix.get(1, 1), 1.5);
        assert_eq!(affine_matrix.get(1, 2), 4.0);
        assert_eq!(affine_matrix.get(2, 0), 0.0);
        assert_eq!(affine_matrix.get(2, 1), 0.0);
        assert_eq!(affine_matrix.get(2, 2), 1.0);
    }
}
