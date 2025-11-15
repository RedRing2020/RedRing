//! Vector2D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D方向ベクトル変換
//! Point3D/Vector3D実装パターンを踏襲した統一設計
//! 2D方向ベクトルとしての特性（平行移動無効化）を考慮

use crate::Vector2D;
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransformVector2D, Angle, Scalar, TransformError};

/// Vector2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector2への変換
    impl<T: Scalar> From<Vector2D<T>> for Vector2<T> {
        fn from(vector: Vector2D<T>) -> Self {
            Vector2::new(vector.x(), vector.y())
        }
    }

    /// Analysis Vector2からの変換
    impl<T: Scalar> From<Vector2<T>> for Vector2D<T> {
        fn from(vector: Vector2<T>) -> Self {
            Vector2D::new(vector.x(), vector.y())
        }
    }

    /// 単一ベクトルの3x3行列変換（方向ベクトルとして、平行移動成分を無視）
    pub fn transform_vector_2d<T: Scalar>(
        vector: &Vector2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Vector2D<T> {
        let vec: Vector2<T> = (*vector).into();
        let transformed = matrix.transform_vector_2d(&vec);
        transformed.into()
    }

    /// 複数ベクトルの一括3x3行列変換
    pub fn transform_vectors_2d<T: Scalar>(
        vectors: &[Vector2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Vec<Vector2D<T>> {
        vectors
            .iter()
            .map(|v| transform_vector_2d(v, matrix))
            .collect()
    }

    /// 回転行列の生成（3x3、原点中心）
    pub fn rotation_matrix_2d<T: Scalar>(angle: Angle<T>) -> Matrix3x3<T> {
        Matrix3x3::rotation_2d(angle.to_radians())
    }

    /// スケール行列の生成（3x3、個別軸指定版）
    pub fn scale_matrix_2d<T: Scalar>(
        scale_x: T,
        scale_y: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let scale_vec = Vector2::new(scale_x, scale_y);
        Ok(Matrix3x3::scale_2d(&scale_vec))
    }

    /// 均等スケール行列の生成（3x3）
    pub fn uniform_scale_matrix_2d<T: Scalar>(
        scale_factor: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        if scale_factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Uniform scale factor cannot be zero".to_string(),
            ));
        }

        Ok(Matrix3x3::uniform_scale_2d(scale_factor))
    }

    /// 複合変換行列の生成（回転+スケール、2D Vector用）
    pub fn composite_vector_transform_2d<T: Scalar>(
        rotation: Option<Angle<T>>,
        scale: Option<(T, T)>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let mut composite = Matrix3x3::identity();

        // スケール変換（先に適用）
        if let Some((scale_x, scale_y)) = scale {
            let scale_matrix = scale_matrix_2d(scale_x, scale_y)?;
            composite = scale_matrix * composite;
        }

        // 回転変換（後に適用）
        if let Some(angle) = rotation {
            let rotation_matrix = rotation_matrix_2d(angle);
            composite = rotation_matrix * composite;
        }

        Ok(composite)
    }

    /// 複合変換行列の生成（回転+均等スケール、2D Vector用）
    pub fn composite_vector_transform_uniform_2d<T: Scalar>(
        rotation: Option<Angle<T>>,
        scale: Option<T>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let mut composite = Matrix3x3::identity();

        // 均等スケール変換（先に適用）
        if let Some(scale_factor) = scale {
            let scale_matrix = uniform_scale_matrix_2d(scale_factor)?;
            composite = scale_matrix * composite;
        }

        // 回転変換（後に適用）
        if let Some(angle) = rotation {
            let rotation_matrix = rotation_matrix_2d(angle);
            composite = rotation_matrix * composite;
        }

        Ok(composite)
    }
}

/// Vector2DでのAnalysisTransformVector2D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransformVector2D<T> for Vector2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_vector_matrix_2d(&self, matrix: &Matrix3x3<T>) -> Self {
        analysis_transform::transform_vector_2d(self, matrix)
    }

    fn rotate_vector_analysis_2d(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        let matrix = analysis_transform::rotation_matrix_2d(angle);
        Ok(self.transform_vector_matrix_2d(&matrix))
    }

    fn scale_vector_analysis_2d(&self, scale_x: T, scale_y: T) -> Result<Self, TransformError> {
        let matrix = analysis_transform::scale_matrix_2d(scale_x, scale_y)?;
        Ok(self.transform_vector_matrix_2d(&matrix))
    }

    fn uniform_scale_vector_analysis_2d(&self, scale_factor: T) -> Result<Self, TransformError> {
        let matrix = analysis_transform::uniform_scale_matrix_2d(scale_factor)?;
        Ok(self.transform_vector_matrix_2d(&matrix))
    }

    fn normalize_analysis_2d(&self) -> Result<Self, TransformError> {
        let analysis_vec: Vector2<T> = (*self).into();
        let normalized = analysis_vec
            .normalize()
            .map_err(TransformError::ZeroVector)?;
        Ok(normalized.into())
    }

    fn to_analysis_vector_2d(&self) -> Vector2<T> {
        (*self).into()
    }

    fn from_analysis_vector_2d(analysis_vector: &Vector2<T>) -> Self {
        Vector2D::new(analysis_vector.x(), analysis_vector.y())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    type TestScalar = f64;
    type TestVector2D = Vector2D<TestScalar>;
    type TestAngle = Angle<TestScalar>;

    #[test]
    fn test_vector_matrix_transform_2d() {
        let vector = TestVector2D::new(1.0, 0.0);
        let rotation_90 = Matrix3x3::rotation_2d(PI / 2.0);
        let result = vector.transform_vector_matrix_2d(&rotation_90);

        assert_relative_eq!(result.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(result.y(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rotate_vector_analysis_2d() {
        let vector = TestVector2D::new(1.0, 0.0);
        let angle = TestAngle::from_degrees(90.0);
        let result = vector.rotate_vector_analysis_2d(angle).unwrap();

        assert_relative_eq!(result.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(result.y(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_scale_vector_analysis_2d() {
        let vector = TestVector2D::new(2.0, 3.0);
        let result = vector.scale_vector_analysis_2d(2.0, 3.0).unwrap();

        assert_relative_eq!(result.x(), 4.0);
        assert_relative_eq!(result.y(), 9.0);
    }

    #[test]
    fn test_uniform_scale_vector_analysis_2d() {
        let vector = TestVector2D::new(2.0, 3.0);
        let result = vector.uniform_scale_vector_analysis_2d(2.0).unwrap();

        assert_relative_eq!(result.x(), 4.0);
        assert_relative_eq!(result.y(), 6.0);
    }

    #[test]
    fn test_normalize_analysis_2d() {
        let vector = TestVector2D::new(3.0, 4.0); // 長さ5のベクトル
        let result = vector.normalize_analysis_2d().unwrap();

        assert_relative_eq!(result.x(), 0.6);
        assert_relative_eq!(result.y(), 0.8);

        // 正規化後の長さが1であることを確認
        let length = (result.x() * result.x() + result.y() * result.y()).sqrt();
        assert_relative_eq!(length, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_zero_vector_normalization() {
        let zero_vector = TestVector2D::new(0.0, 0.0);
        let result = zero_vector.normalize_analysis_2d();

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::ZeroVector(_) => (),
            _ => panic!("Expected ZeroVector error"),
        }
    }

    #[test]
    fn test_analysis_vector_conversion() {
        let vector2d = TestVector2D::new(3.0, 4.0);

        // 2D変換
        let analysis_vec = vector2d.to_analysis_vector_2d();
        assert_eq!(analysis_vec.x(), 3.0);
        assert_eq!(analysis_vec.y(), 4.0);

        // 逆変換
        let back_vector = TestVector2D::from_analysis_vector_2d(&analysis_vec);
        assert_eq!(back_vector.x(), 3.0);
        assert_eq!(back_vector.y(), 4.0);
    }

    #[test]
    fn test_invalid_scale_factors() {
        let vector = TestVector2D::new(1.0, 1.0);

        // ゼロスケール（個別軸）
        assert!(vector.scale_vector_analysis_2d(0.0, 1.0).is_err());
        assert!(vector.scale_vector_analysis_2d(1.0, 0.0).is_err());

        // ゼロスケール（均等）
        assert!(vector.uniform_scale_vector_analysis_2d(0.0).is_err());
    }

    #[test]
    fn test_composite_rotation_and_scale() {
        let vector = TestVector2D::new(1.0, 0.0);

        // 90度回転 + 2倍スケール
        let rotated = vector
            .rotate_vector_analysis_2d(TestAngle::from_degrees(90.0))
            .unwrap();
        let scaled = rotated.uniform_scale_vector_analysis_2d(2.0).unwrap();

        assert_relative_eq!(scaled.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(scaled.y(), 2.0, epsilon = 1e-10);
    }
}
