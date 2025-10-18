//! Matrix3x3の2D変換機能テストスイート
//!
//! このモジュールは、Matrix3x3クラスの包括的な2D変換機能をテストします。
//! アフィン変換、2D描画変換、および各種幾何学的変換をカバーします。

#[cfg(test)]
mod tests {
    use crate::{
        consts::test_constants::TOLERANCE_F64, linalg::matrix::Matrix3x3, Vector2, Vector3,
    };
    use std::f64::consts::PI;

    type Matrix3 = Matrix3x3<f64>;
    type Vec2 = Vector2<f64>;
    type Vec3 = Vector3<f64>;

    // 統一された許容誤差定数を使用
    const TOLERANCE: f64 = TOLERANCE_F64;

    #[test]
    fn test_identity_2d() {
        let identity = Matrix3::identity();

        // 恒等行列の基本プロパティ
        assert!((identity.determinant() - 1.0).abs() < f64::EPSILON);
        assert!(identity.is_affine_transform());

        // 点の変換テスト
        let point = Vec2::new(1.0, 2.0);
        let transformed = identity.transform_point_2d(&point);
        assert!((transformed.x() - point.x()).abs() < TOLERANCE);
        assert!((transformed.y() - point.y()).abs() < TOLERANCE);
    }

    #[test]
    fn test_translation_2d() {
        let translation = Vec2::new(5.0, -3.0);
        let matrix = Matrix3::translation_2d(&translation);

        // 平行移動の基本プロパティ
        assert!(matrix.is_affine_transform());
        assert!((matrix.determinant() - 1.0).abs() < f64::EPSILON);

        // 点の変換テスト
        let point = Vec2::new(1.0, 2.0);
        let expected = Vec2::new(6.0, -1.0);
        let transformed = matrix.transform_point_2d(&point);
        assert!((transformed.x() - expected.x()).abs() < TOLERANCE);
        assert!((transformed.y() - expected.y()).abs() < TOLERANCE);
    }

    #[test]
    fn test_scale_2d() {
        let scale = Vec2::new(2.0, 0.5);
        let matrix = Matrix3::scale_2d(&scale);

        // スケールの基本プロパティ
        assert!(matrix.is_affine_transform());
        assert!((matrix.determinant() - 1.0).abs() < f64::EPSILON); // 2.0 * 0.5 = 1.0

        // 点の変換テスト
        let point = Vec2::new(1.0, 2.0);
        let expected = Vec2::new(2.0, 1.0);
        let transformed = matrix.transform_point_2d(&point);
        assert!((transformed.x() - expected.x()).abs() < TOLERANCE);
        assert!((transformed.y() - expected.y()).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotation_2d() {
        // 90度回転
        let matrix = Matrix3::rotation_2d(PI / 2.0);

        assert!(matrix.is_affine_transform());
        assert!((matrix.determinant() - 1.0).abs() < f64::EPSILON);

        // X軸の点がY軸に移動することを確認
        let point = Vec2::new(1.0, 0.0);
        let transformed = matrix.transform_point_2d(&point);
        let expected = Vec2::new(0.0, 1.0);
        assert!((transformed.x() - expected.x()).abs() < TOLERANCE);
        assert!((transformed.y() - expected.y()).abs() < TOLERANCE);
    }

    #[test]
    fn test_trs_composition_2d() {
        let translation = Vec2::new(1.0, 2.0);
        let rotation_angle = PI / 4.0; // 45度
        let scale = Vec2::new(2.0, 2.0);

        // TRS合成
        let matrix = Matrix3::trs_2d(&translation, rotation_angle, &scale);

        assert!(matrix.is_affine_transform());

        // 分解テスト
        let (decomp_trans, decomp_angle, decomp_scale) = matrix.decompose_2d();
        assert!((decomp_trans.x() - translation.x()).abs() < TOLERANCE);
        assert!((decomp_trans.y() - translation.y()).abs() < TOLERANCE);
        assert!((decomp_angle - rotation_angle).abs() < TOLERANCE);
        assert!((decomp_scale.x() - scale.x()).abs() < TOLERANCE);
        assert!((decomp_scale.y() - scale.y()).abs() < TOLERANCE);
    }

    #[test]
    fn test_reflection_2d() {
        // X軸による反射（Y軸反転）を手動で作成
        let matrix = Matrix3::new(1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0);

        assert!(matrix.is_affine_transform());
        assert!((matrix.determinant() + 1.0).abs() < f64::EPSILON); // 反射の行列式は-1

        let point = Vec2::new(1.0, 2.0);
        let expected = Vec2::new(1.0, -2.0);
        let transformed = matrix.transform_point_2d(&point);
        assert!((transformed.x() - expected.x()).abs() < TOLERANCE);
        assert!((transformed.y() - expected.y()).abs() < TOLERANCE);
    }

    #[test]
    fn test_vector_operators_2d() {
        let matrix = Matrix3::scale_2d(&Vec2::new(2.0, 3.0));

        // Vector2との演算
        let vec2 = Vec2::new(1.0, 1.0);
        let result2 = matrix * vec2;
        let expected2 = Vec2::new(2.0, 3.0);
        assert!((result2.x() - expected2.x()).abs() < TOLERANCE);
        assert!((result2.y() - expected2.y()).abs() < TOLERANCE);

        // Vector3との演算
        let vec3 = Vec3::new(1.0, 1.0, 1.0);
        let result3 = matrix * vec3;
        let expected3 = Vec3::new(2.0, 3.0, 1.0);
        assert!((result3.x() - expected3.x()).abs() < TOLERANCE);
        assert!((result3.y() - expected3.y()).abs() < TOLERANCE);
        assert!((result3.z() - expected3.z()).abs() < TOLERANCE);
    }

    #[test]
    fn test_inverse_2d() {
        let original = Matrix3::trs_2d(&Vec2::new(2.0, -1.0), PI / 6.0, &Vec2::new(2.0, 0.5));

        let inverse = original.inverse_2d().unwrap();
        let identity_result = original * inverse;

        // 恒等行列に近いことを確認（成分比較）
        let expected_identity = Matrix3::identity();
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (identity_result.data[i][j] - expected_identity.data[i][j]).abs() < TOLERANCE
                );
            }
        }

        // 点の変換と逆変換
        let point = Vec2::new(1.0, 2.0);
        let transformed = original.transform_point_2d(&point);
        let restored = inverse.transform_point_2d(&transformed);
        assert!((restored.x() - point.x()).abs() < TOLERANCE);
        assert!((restored.y() - point.y()).abs() < TOLERANCE);
    }

    #[test]
    fn test_affine_detection_2d() {
        // アフィン変換
        let affine = Matrix3::trs_2d(&Vec2::new(1.0, 2.0), PI / 4.0, &Vec2::new(2.0, 1.0));
        assert!(affine.is_affine_transform());

        // 射影変換（非アフィン）のシミュレーション
        let mut non_affine = Matrix3::identity();
        non_affine.data[2][0] = 0.1; // 底行左を非ゼロにすると射影変換
        assert!(!non_affine.is_affine_transform());
    }

    #[test]
    fn test_debug_utilities_2d() {
        let matrix = Matrix3::trs_2d(&Vec2::new(1.0, 2.0), PI / 4.0, &Vec2::new(2.0, 2.0));

        // 行列が正常に作成されていることを確認
        assert!(matrix.is_affine_transform());

        // 分解テストで内容を確認
        let (decomp_trans, decomp_angle, decomp_scale) = matrix.decompose_2d();
        assert!((decomp_trans.x() - 1.0).abs() < TOLERANCE);
        assert!((decomp_trans.y() - 2.0).abs() < TOLERANCE);
        assert!((decomp_angle - PI / 4.0).abs() < TOLERANCE);
        assert!((decomp_scale.x() - 2.0).abs() < TOLERANCE);
        assert!((decomp_scale.y() - 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_matrix_composition_2d() {
        // 複数の変換の合成テスト
        let t1 = Matrix3::translation_2d(&Vec2::new(1.0, 0.0));
        let r1 = Matrix3::rotation_2d(PI / 2.0);
        let s1 = Matrix3::scale_2d(&Vec2::new(2.0, 2.0));

        // 異なる順序での合成
        let trs = t1 * r1 * s1;
        let rst = r1 * s1 * t1;

        // 結果が異なることを確認（行列の積は非可換）
        let mut matrices_are_different = false;
        for i in 0..3 {
            for j in 0..3 {
                if (trs.data[i][j] - rst.data[i][j]).abs() > TOLERANCE {
                    matrices_are_different = true;
                    break;
                }
            }
            if matrices_are_different {
                break;
            }
        }
        assert!(matrices_are_different);

        // 各合成結果がアフィン変換であることを確認
        assert!(trs.is_affine_transform());
        assert!(rst.is_affine_transform());
    }
}
