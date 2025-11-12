//! Vector3D Analysis Matrix/Vector統合変換のテストスイート
//!
//! Matrix4x4を使用した効率的な方向ベクトル変換のテスト
//! Point3Dテストパターンを踏襲したテスト設計

#[cfg(test)]
mod tests {
    use super::super::Vector3D;
    use geo_foundation::{AnalysisTransformVector3D, Angle, Scalar, TransformError};
    use std::f64::consts::PI;

    /// 基本的なMatrix4x4変換テスト（方向ベクトルとして平行移動成分は無視される）
    #[test]
    fn test_vector_matrix_4x4_transform() {
        let vector = Vector3D::new(1.0, 2.0, 3.0);

        // Identity行列での変換（変化なし）
        use analysis::linalg::matrix::Matrix4x4;
        let identity = Matrix4x4::identity();
        let transformed = vector.transform_vector_matrix(&identity);

        assert!((transformed.x() - 1.0).abs() < f64::EPSILON);
        assert!((transformed.y() - 2.0).abs() < f64::EPSILON);
        assert!((transformed.z() - 3.0).abs() < f64::EPSILON);

        // スケール行列での変換
        use analysis::linalg::vector::Vector3;
        let scale_vec = Vector3::new(2.0, 3.0, 4.0);
        let scale_matrix = Matrix4x4::scale_3d(&scale_vec);
        let scaled = vector.transform_vector_matrix(&scale_matrix);

        assert!((scaled.x() - 2.0).abs() < f64::EPSILON); // 1.0 * 2.0
        assert!((scaled.y() - 6.0).abs() < f64::EPSILON); // 2.0 * 3.0
        assert!((scaled.z() - 12.0).abs() < f64::EPSILON); // 3.0 * 4.0
    }

    /// 軸回転変換テスト（Analysis Matrix4x4使用）
    #[test]
    fn test_vector_rotation_analysis() {
        let vector = Vector3D::new(1.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸周り
        let angle = Angle::from_degrees(90.0); // 90度回転

        let rotated = vector.rotate_vector_analysis(&axis, angle).unwrap();

        // X軸ベクトルをZ軸周りに90度回転すると、Y軸ベクトルになる
        assert!(rotated.x().abs() < 1e-10); // 0.0に近い
        assert!((rotated.y() - 1.0).abs() < 1e-10); // 1.0に近い
        assert!(rotated.z().abs() < 1e-10); // 0.0に近い
    }

    /// スケール変換テスト（Analysis Matrix4x4使用）
    #[test]
    fn test_vector_scale_analysis() {
        let vector = Vector3D::new(1.0, 2.0, 3.0);

        // 個別スケール
        let scaled = vector.scale_vector_analysis(2.0, 3.0, 4.0).unwrap();
        assert!((scaled.x() - 2.0).abs() < f64::EPSILON);
        assert!((scaled.y() - 6.0).abs() < f64::EPSILON);
        assert!((scaled.z() - 12.0).abs() < f64::EPSILON);

        // 均等スケール
        let uniform_scaled = vector.uniform_scale_vector_analysis(2.5).unwrap();
        assert!((uniform_scaled.x() - 2.5).abs() < f64::EPSILON);
        assert!((uniform_scaled.y() - 5.0).abs() < f64::EPSILON);
        assert!((uniform_scaled.z() - 7.5).abs() < f64::EPSILON);
    }

    /// 複合変換テスト（回転+スケール）
    #[test]
    fn test_vector_composite_transform() {
        let vector = Vector3D::new(1.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_radians(PI / 2.0); // 90度

        // 回転+スケールの複合変換
        let transformed = vector
            .apply_vector_composite_transform(Some((&axis, angle)), Some((2.0, 3.0, 4.0)))
            .unwrap();

        // Matrix4x4複合変換: 実際の適用順序は回転→スケール
        // 1. 90度Z軸回転: (1,0,0) -> (0,1,0) （X軸→Y軸）
        // 2. スケール適用: (0,1,0) -> (0,3,0) （Y成分が3倍）
        assert!(transformed.x().abs() < 1e-10);
        assert!((transformed.y() - 3.0).abs() < 1e-10);
        assert!(transformed.z().abs() < 1e-10);
    }

    /// 均等スケール複合変換テスト
    #[test]
    fn test_vector_composite_uniform_transform() {
        let vector = Vector3D::new(1.0, 1.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(45.0); // 45度回転

        let transformed = vector
            .apply_vector_composite_transform_uniform(Some((&axis, angle)), Some(2.0))
            .unwrap();

        // (1,1,0) -> 均等スケール -> (2,2,0) -> 45度回転
        let expected_x = 2.0 * (45.0_f64.to_radians().cos() - 45.0_f64.to_radians().sin());
        let expected_y = 2.0 * (45.0_f64.to_radians().sin() + 45.0_f64.to_radians().cos());

        assert!((transformed.x() - expected_x).abs() < 1e-10);
        assert!((transformed.y() - expected_y).abs() < 1e-10);
        assert!(transformed.z().abs() < 1e-10);
    }

    /// Analysis Vector正規化テスト
    #[test]
    fn test_vector_normalize_analysis() {
        let vector = Vector3D::new(3.0, 4.0, 0.0);
        let normalized = vector.normalize_analysis().unwrap();

        // 長さが1になることを確認
        let length =
            (normalized.x().powi(2) + normalized.y().powi(2) + normalized.z().powi(2)).sqrt();
        assert!((length - 1.0).abs() < f64::EPSILON);

        // 方向が保たれることを確認
        assert!((normalized.x() - 0.6).abs() < f64::EPSILON); // 3/5
        assert!((normalized.y() - 0.8).abs() < f64::EPSILON); // 4/5
        assert!(normalized.z().abs() < f64::EPSILON);
    }

    /// エラーケーステスト - ゼロベクトル回転軸
    #[test]
    fn test_error_zero_axis_rotation() {
        let vector = Vector3D::new(1.0, 0.0, 0.0);
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = vector.rotate_vector_analysis(&zero_axis, angle);
        assert!(matches!(result, Err(TransformError::ZeroVector(_))));
    }

    /// エラーケーステスト - ゼロスケールファクター
    #[test]
    fn test_error_zero_scale() {
        let vector = Vector3D::new(1.0, 1.0, 1.0);

        // 個別スケールでゼロ
        let result = vector.scale_vector_analysis(0.0, 1.0, 1.0);
        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));

        // 均等スケールでゼロ
        let result = vector.uniform_scale_vector_analysis(0.0);
        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    /// エラーケーステスト - ゼロベクトル正規化
    #[test]
    fn test_error_zero_vector_normalize() {
        let zero_vector = Vector3D::new(0.0, 0.0, 0.0);
        let result = zero_vector.normalize_analysis();
        assert!(matches!(result, Err(TransformError::ZeroVector(_))));
    }
}
