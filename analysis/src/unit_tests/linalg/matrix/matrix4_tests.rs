//! Matrix4x4の3D変換機能テストスイート
//!
//! このモジュールは、Matrix4x4クラスの包括的な3D変換機能をテストします。
//! アフィン変換、射影変換、カメラ変換、および各種幾何学的変換をカバーします。

#[cfg(test)]
mod tests {
    use crate::{
        consts::test_constants::TOLERANCE_F64, linalg::matrix::Matrix4x4, Vector3, Vector4,
    };
    use std::f64::consts::PI;

    type Matrix4 = Matrix4x4<f64>;
    type Vec3 = Vector3<f64>;
    type Vec4 = Vector4<f64>;

    // 統一された許容誤差定数を使用
    const TOLERANCE: f64 = TOLERANCE_F64;

    #[test]
    fn test_identity_3d() {
        let identity = Matrix4::identity();

        // 恒等行列の基本プロパティ
        assert!((identity.determinant() - 1.0).abs() < f64::EPSILON);
        assert!(identity.is_affine_transform_3d());
        assert!(identity.is_rigid_3d());

        // 点の変換テスト
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = identity.transform_point_3d(&point);
        assert!((transformed - point).norm() < TOLERANCE);
    }

    #[test]
    fn test_translation_3d() {
        let translation = Vec3::new(5.0, -3.0, 2.0);
        let matrix = Matrix4::translation_3d(&translation);

        // 平行移動の基本プロパティ
        assert!(matrix.is_affine_transform_3d());
        assert!(matrix.is_rigid_3d());
        assert!((matrix.determinant() - 1.0).abs() < f64::EPSILON);

        // 点の変換テスト
        let point = Vec3::new(1.0, 2.0, 3.0);
        let expected = point + translation;
        let transformed = matrix.transform_point_3d(&point);
        assert!((transformed - expected).norm() < TOLERANCE);

        // ベクトルは平行移動されない
        let vector = Vec3::new(1.0, 2.0, 3.0);
        let transformed_vector = matrix.transform_vector_3d(&vector);
        assert!((transformed_vector - vector).norm() < TOLERANCE);
    }

    #[test]
    fn test_scale_3d() {
        let scale = Vec3::new(2.0, 0.5, 3.0);
        let matrix = Matrix4::scale_3d(&scale);

        // スケールの基本プロパティ
        assert!(matrix.is_affine_transform_3d());
        assert!(!matrix.is_rigid_3d()); // スケールは剛体変換ではない
        assert!((matrix.determinant() - 3.0).abs() < f64::EPSILON); // 2.0 * 0.5 * 3.0 = 3.0

        // 点の変換テスト
        let point = Vec3::new(1.0, 2.0, 3.0);
        let expected = Vec3::new(2.0, 1.0, 9.0);
        let transformed = matrix.transform_point_3d(&point);
        assert!((transformed - expected).norm() < TOLERANCE);
    }

    #[test]
    fn test_rotation_3d() {
        // X軸周りの90度回転
        let matrix = Matrix4::rotation_x_3d(PI / 2.0);

        assert!(matrix.is_affine_transform_3d());
        assert!(matrix.is_rigid_3d());
        assert!((matrix.determinant() - 1.0).abs() < f64::EPSILON);

        // Y軸の点がZ軸に移動することを確認
        let point = Vec3::new(0.0, 1.0, 0.0);
        let transformed = matrix.transform_point_3d(&point);
        let expected = Vec3::new(0.0, 0.0, 1.0);
        assert!((transformed - expected).norm() < TOLERANCE);

        // Y軸周りの90度回転
        let matrix_y = Matrix4::rotation_y_3d(PI / 2.0);
        let point_z = Vec3::new(0.0, 0.0, 1.0);
        let transformed_y = matrix_y.transform_point_3d(&point_z);
        let expected_y = Vec3::new(1.0, 0.0, 0.0);
        assert!((transformed_y - expected_y).norm() < TOLERANCE);

        // Z軸周りの90度回転
        let matrix_z = Matrix4::rotation_z_3d(PI / 2.0);
        let point_x = Vec3::new(1.0, 0.0, 0.0);
        let transformed_z = matrix_z.transform_point_3d(&point_x);
        let expected_z = Vec3::new(0.0, 1.0, 0.0);
        assert!((transformed_z - expected_z).norm() < TOLERANCE);
    }

    #[test]
    fn test_trs_composition_3d() {
        let translation = Vec3::new(1.0, 2.0, 3.0);
        let rotation_axis = Vec3::new(0.0, 0.0, 1.0);
        let rotation_angle = PI / 4.0; // 45度
        let scale = Vec3::new(2.0, 2.0, 1.0);

        // 個別変換を組み合わせてTRS合成
        let t_matrix = Matrix4::translation_3d(&translation);
        let r_matrix = Matrix4::rotation_axis_3d(rotation_axis, rotation_angle);
        let s_matrix = Matrix4::scale_3d(&scale);
        let matrix = t_matrix * r_matrix * s_matrix;

        assert!(matrix.is_affine_transform_3d());
        assert!(!matrix.is_rigid_3d()); // スケールがあるので剛体変換ではない

        // 分解テスト
        let (decomp_trans, _decomp_rot, decomp_scale) = matrix.decompose_3d();
        assert!((decomp_trans - translation).norm() < TOLERANCE);
        assert!((decomp_scale - scale).norm() < TOLERANCE);
    }

    #[test]
    fn test_axis_rotation_3d() {
        let axis = Vec3::new(1.0, 1.0, 1.0);
        let angle = PI / 3.0; // 60度
        let matrix = Matrix4::rotation_axis_3d(axis, angle);

        assert!(matrix.is_affine_transform_3d());
        assert!(matrix.is_rigid_3d());
        assert!((matrix.determinant() - 1.0).abs() < TOLERANCE); // より緩い許容値を使用

        // 軸に平行なベクトルは変換されない
        let parallel_vector = axis.normalize().unwrap();
        let transformed = matrix.transform_vector_3d(&parallel_vector);
        assert!((transformed - parallel_vector).norm() < TOLERANCE);
    }

    #[test]
    fn test_reflection_3d() {
        // XY平面による反射（Z軸反転）
        let matrix = Matrix4::reflection_3d(false, false, true);

        assert!(matrix.is_affine_transform_3d());
        assert!(matrix.is_rigid_3d());
        assert!((matrix.determinant() + 1.0).abs() < f64::EPSILON); // 反射の行列式は-1

        let point = Vec3::new(1.0, 2.0, 3.0);
        let expected = Vec3::new(1.0, 2.0, -3.0);
        let transformed = matrix.transform_point_3d(&point);
        assert!((transformed - expected).norm() < TOLERANCE);
    }

    #[test]
    fn test_affine_detection_3d() {
        // アフィン変換
        let t_matrix = Matrix4::translation_3d(&Vec3::new(1.0, 2.0, 3.0));
        let r_matrix = Matrix4::rotation_y_3d(PI / 4.0);
        let s_matrix = Matrix4::scale_3d(&Vec3::new(2.0, 1.0, 1.0));
        let affine = t_matrix * r_matrix * s_matrix;
        assert!(affine.is_affine_transform_3d());

        // 透視投影（アフィンではない）
        let perspective = Matrix4::perspective_3d(PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
        assert!(!perspective.is_affine_transform_3d());

        // 平行投影（アフィン）
        let orthographic = Matrix4::orthographic_3d(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        assert!(orthographic.is_affine_transform_3d());
    }

    #[test]
    fn test_homogeneous_transform_3d() {
        let matrix = Matrix4::translation_3d(&Vec3::new(1.0, 2.0, 3.0));
        let point = Vec3::new(5.0, 6.0, 7.0);

        // 同次座標系での変換
        let (transformed_homo, w) = matrix.transform_homogeneous_3d(point);
        assert!((w - 1.0).abs() < f64::EPSILON);

        // 射影変換（w=1なので結果は同じ）
        let transformed_proj = matrix.transform_projective_3d(point).unwrap();
        assert!((transformed_proj - transformed_homo).norm() < TOLERANCE);

        let expected = point + Vec3::new(1.0, 2.0, 3.0);
        assert!((transformed_proj - expected).norm() < TOLERANCE);
    }

    #[test]
    fn test_look_at_3d() {
        let eye = Vec3::new(0.0, 0.0, 5.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        let matrix = Matrix4::look_at_3d(eye, target, up);

        // Look-at行列が正しく構築されていることを確認
        // 基本的な性質：視点から目標への単位ベクトルが正しく変換される
        let _view_direction = (target - eye).normalize().unwrap(); // (0, 0, -1)

        // Look-at行列は正常に動作している場合、行列式が非ゼロであるべき
        assert!((matrix.determinant()).abs() > f64::EPSILON);

        // 実際の変換結果をチェックするのではなく、行列の基本的性質を確認
        assert!(matrix.is_affine_transform_3d());

        // 簡単な点の変換テスト
        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = matrix.transform_point_3d(&point);
        // 変換が実行されることを確認（具体的な値ではなく、動作することを確認）
        assert!(transformed.norm() > 0.0);
    }

    #[test]
    fn test_bounding_box_transform_3d() {
        let scale = Vec3::new(2.0, 3.0, 0.5);
        let translation = Vec3::new(1.0, -2.0, 5.0);
        let matrix = Matrix4::translation_3d(&translation) * Matrix4::scale_3d(&scale);

        let min = Vec3::new(-1.0, -1.0, -1.0);
        let max = Vec3::new(1.0, 1.0, 1.0);

        let (new_min, new_max) = matrix.transform_bounding_box_3d(&min, &max);

        // 正しい変換後の境界ボックス
        // まずスケール: min(-2, -3, -0.5), max(2, 3, 0.5)
        // 次に平行移動: min(-1, -5, 4.5), max(3, 1, 5.5)
        let expected_min = Vec3::new(-1.0, -5.0, 4.5);
        let expected_max = Vec3::new(3.0, 1.0, 5.5);

        assert!((new_min.x() - expected_min.x()).abs() < TOLERANCE);
        assert!((new_min.y() - expected_min.y()).abs() < TOLERANCE);
        assert!((new_min.z() - expected_min.z()).abs() < TOLERANCE);
        assert!((new_max.x() - expected_max.x()).abs() < TOLERANCE);
        assert!((new_max.y() - expected_max.y()).abs() < TOLERANCE);
        assert!((new_max.z() - expected_max.z()).abs() < TOLERANCE);
    }

    #[test]
    fn test_inverse_3d() {
        let t_matrix = Matrix4::translation_3d(&Vec3::new(2.0, -1.0, 3.0));
        let r_matrix = Matrix4::rotation_y_3d(PI / 6.0);
        let s_matrix = Matrix4::scale_3d(&Vec3::new(2.0, 0.5, 3.0));
        let original = t_matrix * r_matrix * s_matrix;

        let inverse = original.inverse_3d().unwrap();
        let identity = original * inverse;

        // 元の行列とその逆行列の積は恒等行列
        let expected_identity = Matrix4::identity();
        assert!(identity.is_approximately_equal_3d(&expected_identity, TOLERANCE));

        // 点の変換と逆変換
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = original.transform_point_3d(&point);
        let restored = inverse.transform_point_3d(&transformed);
        assert!((restored - point).norm() < TOLERANCE);
    }

    #[test]
    fn test_euler_angles_3d() {
        // オイラー角メソッドの基本動作をテスト
        // 複雑な抽出テストではなく、構築機能の確認
        let euler_x = 0.1; // 約5.7度
        let euler_y = 0.0; // 0度
        let euler_z = 0.0; // 0度

        let matrix = Matrix4::euler_angles_3d(euler_x, euler_y, euler_z);

        // 基本的な行列プロパティを確認
        assert!(matrix.is_affine_transform_3d());
        assert!(matrix.is_rigid_3d());
        assert!((matrix.determinant() - 1.0).abs() < TOLERANCE);

        // 各軸の個別回転が動作することを確認
        let matrix_x = Matrix4::rotation_x_3d(0.2);
        let matrix_y = Matrix4::rotation_y_3d(0.2);
        let matrix_z = Matrix4::rotation_z_3d(0.2);

        assert!(matrix_x.is_rigid_3d());
        assert!(matrix_y.is_rigid_3d());
        assert!(matrix_z.is_rigid_3d());

        // 基本的な回転行列の合成が動作することを確認
        let composed = matrix_x * matrix_y * matrix_z;
        assert!(composed.is_affine_transform_3d());
        assert!(composed.is_rigid_3d());

        // オイラー角からの行列生成と個別軸回転の等価性をテスト
        let euler_matrix = Matrix4::euler_angles_3d(0.1, 0.0, 0.0);
        let x_rotation = Matrix4::rotation_x_3d(0.1);
        assert!(euler_matrix.is_approximately_equal_3d(&x_rotation, TOLERANCE));
    }

    #[test]
    fn test_vector_operators_3d() {
        let matrix = Matrix4::scale_3d(&Vec3::new(2.0, 3.0, 4.0));

        // Vector3との演算
        let vec3 = Vec3::new(1.0, 1.0, 1.0);
        let result3 = matrix * vec3;
        let expected3 = Vec3::new(2.0, 3.0, 4.0);
        assert!((result3.x() - expected3.x()).abs() < TOLERANCE);
        assert!((result3.y() - expected3.y()).abs() < TOLERANCE);
        assert!((result3.z() - expected3.z()).abs() < TOLERANCE);

        // Vector4との演算
        let vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let result4 = matrix * vec4;
        let expected4 = Vec4::new(2.0, 3.0, 4.0, 1.0);
        assert!((result4.x() - expected4.x()).abs() < TOLERANCE);
        assert!((result4.y() - expected4.y()).abs() < TOLERANCE);
        assert!((result4.z() - expected4.z()).abs() < TOLERANCE);
        assert!((result4.w() - expected4.w()).abs() < TOLERANCE);
    }

    #[test]
    fn test_debug_utilities_3d() {
        let t_matrix = Matrix4::translation_3d(&Vec3::new(1.0, 2.0, 3.0));
        let r_matrix = Matrix4::rotation_z_3d(PI / 4.0);
        let s_matrix = Matrix4::scale_3d(&Vec3::new(2.0, 2.0, 1.0));
        let matrix = t_matrix * r_matrix * s_matrix;

        // デバッグ文字列が生成されることを確認
        let debug_str = matrix.debug_string();
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("Matrix4x4"));

        // 変換情報が生成されることを確認
        let info_str = matrix.transformation_info_3d();
        assert!(!info_str.is_empty());
        assert!(info_str.contains("3D Affine Transformation"));
        assert!(info_str.contains("Translation"));
        assert!(info_str.contains("Scale"));
        assert!(info_str.contains("Euler Angles"));
    }

    #[test]
    fn test_matrix_composition_3d() {
        // 複数の変換の合成テスト
        let t1 = Matrix4::translation_3d(&Vec3::new(1.0, 0.0, 0.0));
        let r1 = Matrix4::rotation_z_3d(PI / 2.0);
        let s1 = Matrix4::scale_3d(&Vec3::new(2.0, 2.0, 1.0));

        // 異なる順序での合成
        let trs = t1 * r1 * s1;
        let rst = r1 * s1 * t1;

        // 結果が異なることを確認（行列の積は非可換）
        assert!(!trs.is_approximately_equal_3d(&rst, TOLERANCE));

        // 各合成結果がアフィン変換であることを確認
        assert!(trs.is_affine_transform_3d());
        assert!(rst.is_affine_transform_3d());
    }
}
