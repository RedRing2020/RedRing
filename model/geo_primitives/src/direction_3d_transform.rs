//! Direction3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D方向ベクトル変換
//! Vector3D実装パターンを踏襲した統一設計
//! Direction3Dの正規化保証を维持した変換処理

use crate::{Direction3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// Direction3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector3への変換
    pub fn direction_to_analysis_vector<T: Scalar>(direction: &Direction3D<T>) -> Vector3<T> {
        Vector3::new(direction.x(), direction.y(), direction.z())
    }

    /// Analysis Vector3からの変換（正規化保証）
    pub fn analysis_vector_to_direction<T: Scalar>(
        vector: Vector3<T>,
    ) -> Result<Direction3D<T>, TransformError> {
        let vector3d = Vector3D::new(vector.x(), vector.y(), vector.z());
        Direction3D::from_vector(vector3d).ok_or_else(|| {
            TransformError::ZeroVector("Transformed direction vector is zero".to_string())
        })
    }

    /// 単一方向ベクトルの4x4行列変換（方向ベクトルとして、平行移動成分を無視）
    pub fn transform_direction_3d<T: Scalar>(
        direction: &Direction3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<Direction3D<T>, TransformError> {
        let vec = direction_to_analysis_vector(direction);
        let transformed = matrix.transform_vector_3d(&vec);
        analysis_vector_to_direction(transformed)
    }

    /// 複数方向ベクトルの一括4x4行列変換
    pub fn transform_directions_3d<T: Scalar>(
        directions: &[Direction3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Result<Vec<Direction3D<T>>, TransformError> {
        directions
            .iter()
            .map(|d| transform_direction_3d(d, matrix))
            .collect()
    }

    /// 回転行列の生成（4x4、軸回転）
    pub fn axis_rotation_matrix_3d<T: Scalar>(
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let norm = axis.norm();
        if norm.is_zero() {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero".to_string(),
            ));
        }
        let normalized_axis = axis.normalize().map_err(|_| {
            TransformError::ZeroVector("Failed to normalize rotation axis".to_string())
        })?;
        Ok(Matrix4x4::rotation_axis_3d(
            normalized_axis,
            angle.to_radians(),
        ))
    }

    /// スケール行列の生成（4x4、原点中心）
    pub fn scale_matrix_3d<T: Scalar>(
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero for direction vectors".to_string(),
            ));
        }

        let scale_vec = Vector3::new(scale_x, scale_y, scale_z);
        Ok(Matrix4x4::scale_3d(&scale_vec))
    }
}

/// Direction3DでのAnalysisTransform3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform3D<T> for Direction3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        // Direction3Dは点ではないので、ベクトル変換を使用
        analysis_transform::transform_direction_3d(self, matrix).unwrap_or_else(|_| *self)
        // エラー時は元の方向を维持
    }

    fn translate_analysis(&self, _translation: &Vector3<T>) -> Result<Self, TransformError> {
        // 方向ベクトルは平行移動の影響を受けない
        Ok(*self)
    }

    fn rotate_analysis(
        &self,
        _center: &Self,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 方向ベクトルは中心点に依存しない
        let matrix = analysis_transform::axis_rotation_matrix_3d(axis, angle)?;
        analysis_transform::transform_direction_3d(self, &matrix)
    }

    fn scale_analysis(
        &self,
        _center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        // 方向ベクトルは中心点に依存しない
        let matrix = analysis_transform::scale_matrix_3d(scale_x, scale_y, scale_z)?;
        analysis_transform::transform_direction_3d(self, &matrix)
    }

    fn uniform_scale_analysis(
        &self,
        _center: &Self,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        // 均等スケール（方向ベクトルは中心点に依存しない）
        let matrix = analysis_transform::scale_matrix_3d(scale_factor, scale_factor, scale_factor)?;
        analysis_transform::transform_direction_3d(self, &matrix)
    }

    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self, TransformError> {
        // 平行移動は無視（方向ベクトルの特性）
        let _ = translation;

        let mut composite = Matrix4x4::identity();

        // スケール変換を適用
        if let Some((sx, sy, sz)) = scale {
            let scale_matrix = analysis_transform::scale_matrix_3d(sx, sy, sz)?;
            composite = scale_matrix * composite;
        }

        // 回転変換を適用
        if let Some((_, axis, angle)) = rotation {
            let rotation_matrix = analysis_transform::axis_rotation_matrix_3d(axis, angle)?;
            composite = rotation_matrix * composite;
        }

        analysis_transform::transform_direction_3d(self, &composite)
    }

    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self, TransformError> {
        // 平行移動は無視（方向ベクトルの特性）
        let scale_tuple = scale.map(|s| (s, s, s));
        self.apply_composite_transform(translation, rotation, scale_tuple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector3D;
    use std::f64::consts::PI;

    #[test]
    fn test_direction_analysis_vector_conversion() {
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let analysis_vec = analysis_transform::direction_to_analysis_vector(&direction);

        assert_eq!(analysis_vec.x(), 1.0);
        assert_eq!(analysis_vec.y(), 0.0);
        assert_eq!(analysis_vec.z(), 0.0);

        let back_direction =
            analysis_transform::analysis_vector_to_direction(analysis_vec).unwrap();
        assert_eq!(direction, back_direction);
    }

    #[test]
    fn test_direction_axis_rotation() {
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_radians(PI / 2.0);
        let center = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let rotated = direction.rotate_analysis(&center, &z_axis, angle).unwrap();

        // Z軸回りに90度回転で (1,0,0) -> (0,1,0)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction_scale() {
        let direction = Direction3D::from_vector(Vector3D::new(3.0, 4.0, 0.0)).unwrap();
        let center = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let scaled = direction.scale_analysis(&center, 2.0, 0.5, 1.0).unwrap();

        // スケール後も正規化されている
        let norm =
            (scaled.x() * scaled.x() + scaled.y() * scaled.y() + scaled.z() * scaled.z()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction_translation_ignored() {
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let translation = Vector3::new(10.0, 20.0, 30.0);

        let translated = direction.translate_analysis(&translation).unwrap();

        // 平行移動は方向ベクトルに影響しない
        assert_eq!(direction, translated);
    }

    #[test]
    fn test_zero_scale_error() {
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = direction.scale_analysis(&center, 0.0, 1.0, 1.0);
        assert!(result.is_err());

        let result = direction.scale_analysis(&center, 1.0, 0.0, 1.0);
        assert!(result.is_err());

        let result = direction.scale_analysis(&center, 1.0, 1.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_uniform_scale() {
        let direction = Direction3D::from_vector(Vector3D::new(3.0, 4.0, 5.0)).unwrap();
        let center = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let scaled = direction.uniform_scale_analysis(&center, 2.0).unwrap();

        // 均等スケール後も正規化されている
        let norm =
            (scaled.x() * scaled.x() + scaled.y() * scaled.y() + scaled.z() * scaled.z()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);

        // 方向は変わらない（均等スケール）
        let original_ratio_xy = direction.x() / direction.y();
        let scaled_ratio_xy = scaled.x() / scaled.y();
        assert!((original_ratio_xy - scaled_ratio_xy).abs() < 1e-10);

        let original_ratio_xz = direction.x() / direction.z();
        let scaled_ratio_xz = scaled.x() / scaled.z();
        assert!((original_ratio_xz - scaled_ratio_xz).abs() < 1e-10);
    }

    #[test]
    fn test_zero_rotation_axis_error() {
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0);

        let matrix_result = analysis_transform::axis_rotation_matrix_3d(&zero_axis, angle);
        assert!(matrix_result.is_err());

        if let Err(TransformError::ZeroVector(_)) = matrix_result {
            // 期待通り
        } else {
            panic!("Expected ZeroVector error");
        }
    }

    #[test]
    fn test_multiple_directions_transform() {
        let directions = vec![
            Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap(),
            Direction3D::from_vector(Vector3D::new(0.0, 1.0, 0.0)).unwrap(),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
        ];

        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let matrix =
            analysis_transform::axis_rotation_matrix_3d(&z_axis, Angle::from_radians(PI / 2.0))
                .unwrap();
        let transformed =
            analysis_transform::transform_directions_3d(&directions, &matrix).unwrap();

        assert_eq!(transformed.len(), 3);

        // Z軸回りに90度回転の確認
        assert!((transformed[0].x() - 0.0).abs() < 1e-10);
        assert!((transformed[0].y() - 1.0).abs() < 1e-10);
        assert!((transformed[0].z() - 0.0).abs() < 1e-10);

        assert!((transformed[1].x() - (-1.0)).abs() < 1e-10);
        assert!((transformed[1].y() - 0.0).abs() < 1e-10);
        assert!((transformed[1].z() - 0.0).abs() < 1e-10);

        // Z軸方向は変わらない
        assert!((transformed[2].x() - 0.0).abs() < 1e-10);
        assert!((transformed[2].y() - 0.0).abs() < 1e-10);
        assert!((transformed[2].z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_composite_transform() {
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_radians(PI / 4.0); // 45度
        let scale_factors = (1.0, 1.0, 1.0); // 等倍
        let translation = Vector3::new(10.0, 20.0, 30.0);

        let result = direction
            .apply_composite_transform(
                Some(&translation),
                Some((&center, &z_axis, rotation_angle)),
                Some(scale_factors),
            )
            .unwrap();

        // 45度回転で (1,0,0) -> (√2/2, √2/2, 0)
        let expected_x = (PI / 4.0).cos();
        let expected_y = (PI / 4.0).sin();

        assert!((result.x() - expected_x).abs() < 1e-10);
        assert!((result.y() - expected_y).abs() < 1e-10);
        assert!((result.z() - 0.0).abs() < 1e-10);

        // 正規化確認
        let norm =
            (result.x() * result.x() + result.y() * result.y() + result.z() * result.z()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }
}
