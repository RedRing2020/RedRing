//! Direction2D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D方向ベクトル変換
//! Vector2D実装パターンを踏襲した統一設計
//! Direction2Dの正規化保証を维持した変換処理

use crate::{Direction2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// Direction2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector2への変換
    pub fn direction_to_analysis_vector<T: Scalar>(direction: &Direction2D<T>) -> Vector2<T> {
        Vector2::new(direction.x(), direction.y())
    }

    /// Analysis Vector2からの変換（正規化保証）
    pub fn analysis_vector_to_direction<T: Scalar>(
        vector: Vector2<T>,
    ) -> Result<Direction2D<T>, TransformError> {
        let vector2d = Vector2D::new(vector.x(), vector.y());
        Direction2D::from_vector(vector2d).ok_or_else(|| {
            TransformError::ZeroVector("Transformed direction vector is zero".to_string())
        })
    }

    /// 単一方向ベクトルの3x3行列変換（方向ベクトルとして、平行移動成分を無視）
    pub fn transform_direction_2d<T: Scalar>(
        direction: &Direction2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<Direction2D<T>, TransformError> {
        let vec = direction_to_analysis_vector(direction);
        let transformed = matrix.transform_vector_2d(&vec);
        analysis_vector_to_direction(transformed)
    }

    /// 複数方向ベクトルの一括3x3行列変換
    pub fn transform_directions_2d<T: Scalar>(
        directions: &[Direction2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Result<Vec<Direction2D<T>>, TransformError> {
        directions
            .iter()
            .map(|d| transform_direction_2d(d, matrix))
            .collect()
    }

    /// 回転行列の生成（3x3、原点中心）
    pub fn rotation_matrix_2d<T: Scalar>(angle: Angle<T>) -> Matrix3x3<T> {
        Matrix3x3::rotation_2d(angle.to_radians())
    }

    /// スケール行列の生成（3x3、原点中心）
    pub fn scale_matrix_2d<T: Scalar>(
        scale_x: T,
        scale_y: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero for direction vectors".to_string(),
            ));
        }

        let scale_vec = Vector2::new(scale_x, scale_y);
        Ok(Matrix3x3::scale_2d(&scale_vec))
    }
}

/// Direction2DでのAnalysisTransform2D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform2D<T> for Direction2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix_2d(&self, matrix: &Matrix3x3<T>) -> Self {
        // Direction2Dは点ではないので、ベクトル変換を使用
        analysis_transform::transform_direction_2d(self, matrix).unwrap_or_else(|_| *self)
        // エラー時は元の方向を维持
    }

    fn translate_analysis_2d(&self, _translation: &Vector2<T>) -> Result<Self, TransformError> {
        // 方向ベクトルは平行移動の影響を受けない
        Ok(*self)
    }

    fn rotate_analysis_2d(&self, _center: &Self, angle: Angle<T>) -> Result<Self, TransformError> {
        // 方向ベクトルは中心点に依存しない
        let matrix = analysis_transform::rotation_matrix_2d(angle);
        analysis_transform::transform_direction_2d(self, &matrix)
    }

    fn scale_analysis_2d(
        &self,
        _center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        // 方向ベクトルは中心点に依存しない
        let matrix = analysis_transform::scale_matrix_2d(scale_x, scale_y)?;
        analysis_transform::transform_direction_2d(self, &matrix)
    }

    fn uniform_scale_analysis_2d(
        &self,
        _center: &Self,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        // 均等スケール（方向ベクトルは中心点に依存しない）
        let matrix = analysis_transform::scale_matrix_2d(scale_factor, scale_factor)?;
        analysis_transform::transform_direction_2d(self, &matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector2D;
    use std::f64::consts::PI;

    #[test]
    fn test_direction_analysis_vector_conversion() {
        let direction = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();
        let analysis_vec = analysis_transform::direction_to_analysis_vector(&direction);

        assert_eq!(analysis_vec.x(), 1.0);
        assert_eq!(analysis_vec.y(), 0.0);

        let back_direction =
            analysis_transform::analysis_vector_to_direction(analysis_vec).unwrap();
        assert_eq!(direction, back_direction);
    }

    #[test]
    fn test_direction_rotation() {
        let direction = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();
        let angle = Angle::from_radians(PI / 2.0);
        let center = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap(); // 単位ベクトル使用

        let rotated = direction.rotate_analysis_2d(&center, angle).unwrap();

        // 90度回転で (1,0) -> (0,1)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction_scale() {
        let direction = Direction2D::from_vector(Vector2D::new(3.0, 4.0)).unwrap();
        let center = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();

        let scaled = direction.scale_analysis_2d(&center, 2.0, 0.5).unwrap();

        // スケール後も正規化されている
        let norm = (scaled.x() * scaled.x() + scaled.y() * scaled.y()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction_translation_ignored() {
        let direction = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();
        let translation = Vector2::new(10.0, 20.0);

        let translated = direction.translate_analysis_2d(&translation).unwrap();

        // 平行移動は方向ベクトルに影響しない
        assert_eq!(direction, translated);
    }

    #[test]
    fn test_zero_scale_error() {
        let direction = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();
        let center = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();

        let result = direction.scale_analysis_2d(&center, 0.0, 1.0);
        assert!(result.is_err());

        let result = direction.scale_analysis_2d(&center, 1.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_uniform_scale() {
        let direction = Direction2D::from_vector(Vector2D::new(3.0, 4.0)).unwrap();
        let center = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();

        let scaled = direction.uniform_scale_analysis_2d(&center, 2.0).unwrap();

        // 均等スケール後も正規化されている
        let norm = (scaled.x() * scaled.x() + scaled.y() * scaled.y()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);

        // 方向は変わらない（均等スケール）
        let original_dir = direction.x() / direction.y();
        let scaled_dir = scaled.x() / scaled.y();
        assert!((original_dir - scaled_dir).abs() < 1e-10);
    }

    #[test]
    fn test_multiple_directions_transform() {
        let directions = vec![
            Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap(),
            Direction2D::from_vector(Vector2D::new(0.0, 1.0)).unwrap(),
            Direction2D::from_vector(Vector2D::new(-1.0, 0.0)).unwrap(),
        ];

        let matrix = analysis_transform::rotation_matrix_2d(Angle::from_radians(PI / 2.0));
        let transformed =
            analysis_transform::transform_directions_2d(&directions, &matrix).unwrap();

        assert_eq!(transformed.len(), 3);

        // 90度回転の確認
        assert!((transformed[0].x() - 0.0).abs() < 1e-10);
        assert!((transformed[0].y() - 1.0).abs() < 1e-10);

        assert!((transformed[1].x() - (-1.0)).abs() < 1e-10);
        assert!((transformed[1].y() - 0.0).abs() < 1e-10);

        assert!((transformed[2].x() - 0.0).abs() < 1e-10);
        assert!((transformed[2].y() - (-1.0)).abs() < 1e-10);
    }
}
