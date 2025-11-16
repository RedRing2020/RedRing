//! Vector3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix/Vectorを直接使用した効率的な方向ベクトル変換
//! geo_nurbsのmatrix_transformパターンを基盤とする統一実装
//! 方向ベクトルとしての特性（平行移動無効化）を考慮

use crate::Vector3D;
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransformVector3D, Angle, Scalar, TransformError};

/// Vector3D用Analysis Matrix/Vector変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector3への変換
    impl<T: Scalar> From<Vector3D<T>> for Vector3<T> {
        fn from(vector: Vector3D<T>) -> Self {
            Vector3::new(vector.x(), vector.y(), vector.z())
        }
    }

    /// Analysis Vector3からの変換
    impl<T: Scalar> From<Vector3<T>> for Vector3D<T> {
        fn from(vector: Vector3<T>) -> Self {
            Vector3D::new(vector.x(), vector.y(), vector.z())
        }
    }

    /// 単一ベクトルの4x4行列変換（方向ベクトルとして、平行移動成分を無視）
    pub fn transform_vector_3d<T: Scalar>(
        vector: &Vector3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Vector3D<T> {
        let vec: Vector3<T> = (*vector).into();
        let transformed = matrix.transform_vector_3d(&vec);
        transformed.into()
    }

    /// 複数ベクトルの一括4x4行列変換
    pub fn transform_vectors_3d<T: Scalar>(
        vectors: &[Vector3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Vec<Vector3D<T>> {
        vectors
            .iter()
            .map(|v| transform_vector_3d(v, matrix))
            .collect()
    }

    /// 回転行列の生成（4x4、軸回転）
    pub fn rotation_matrix_3d<T: Scalar>(
        axis: &Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let axis_vec: Vector3<T> = Vector3::new(axis.x(), axis.y(), axis.z());

        // ゼロベクトルチェック
        if axis_vec.norm_squared().is_zero() {
            return Err(TransformError::ZeroVector(
                "Cannot rotate around zero vector".to_string(),
            ));
        }

        let normalized_axis = axis_vec.normalize().map_err(TransformError::ZeroVector)?;
        Ok(Matrix4x4::rotation_axis(
            &normalized_axis,
            angle.to_radians(),
        ))
    }

    /// スケール行列の生成（4x4）
    pub fn scale_matrix_3d<T: Scalar>(
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }
        let scale_vec = Vector3::new(scale_x, scale_y, scale_z);
        Ok(Matrix4x4::scale_3d(&scale_vec))
    }

    /// 均等スケール行列の生成（4x4）
    pub fn uniform_scale_matrix_3d<T: Scalar>(
        scale_factor: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor cannot be zero".to_string(),
            ));
        }
        Ok(Matrix4x4::uniform_scale_3d(scale_factor))
    }

    /// 複合変換行列の構築（4x4、ベクトル用）
    pub fn composite_vector_transform_3d<T: Scalar>(
        rotation: Option<(&Vector3D<T>, Angle<T>)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let mut result = Matrix4x4::identity();

        // スケール適用
        if let Some((sx, sy, sz)) = scale {
            let scale_matrix = scale_matrix_3d(sx, sy, sz)?;
            result = result * scale_matrix;
        }

        // 回転適用
        if let Some((axis, angle)) = rotation {
            let rotation_matrix = rotation_matrix_3d(axis, angle)?;
            result = result * rotation_matrix;
        }

        Ok(result)
    }

    /// 複合変換行列の構築（均等スケール版）
    pub fn composite_vector_transform_uniform_3d<T: Scalar>(
        rotation: Option<(&Vector3D<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let mut result = Matrix4x4::identity();

        // 均等スケール適用
        if let Some(scale_factor) = scale {
            let scale_matrix = uniform_scale_matrix_3d(scale_factor)?;
            result = result * scale_matrix;
        }

        // 回転適用
        if let Some((axis, angle)) = rotation {
            let rotation_matrix = rotation_matrix_3d(axis, angle)?;
            result = result * rotation_matrix;
        }

        Ok(result)
    }
}

/// Vector3DでのAnalysisTransformVector3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransformVector3D<T> for Vector3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_vector_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        analysis_transform::transform_vector_3d(self, matrix)
    }

    fn rotate_vector_analysis(
        &self,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // Vector3からVector3Dへの変換
        let axis_vector3d = Vector3D::new(axis.x(), axis.y(), axis.z());
        let matrix = analysis_transform::rotation_matrix_3d(&axis_vector3d, angle)?;
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn scale_vector_analysis(
        &self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        let matrix = analysis_transform::scale_matrix_3d(scale_x, scale_y, scale_z)?;
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn uniform_scale_vector_analysis(&self, scale_factor: T) -> Result<Self, TransformError> {
        let matrix = analysis_transform::uniform_scale_matrix_3d(scale_factor)?;
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn apply_vector_composite_transform(
        &self,
        rotation: Option<(&Vector3<T>, Angle<T>)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self, TransformError> {
        // Vector3からVector3Dへの変換（所有権の問題を回避）
        let rotation_vector3d =
            rotation.map(|(axis, angle)| (Vector3D::new(axis.x(), axis.y(), axis.z()), angle));
        let rotation_ref = rotation_vector3d.as_ref().map(|(v, a)| (v, *a));

        let matrix = analysis_transform::composite_vector_transform_3d(rotation_ref, scale)?;
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn apply_vector_composite_transform_uniform(
        &self,
        rotation: Option<(&Vector3<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Self, TransformError> {
        // Vector3からVector3Dへの変換（所有権の問題を回避）
        let rotation_vector3d =
            rotation.map(|(axis, angle)| (Vector3D::new(axis.x(), axis.y(), axis.z()), angle));
        let rotation_ref = rotation_vector3d.as_ref().map(|(v, a)| (v, *a));

        let matrix =
            analysis_transform::composite_vector_transform_uniform_3d(rotation_ref, scale)?;
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn normalize_analysis(&self) -> Result<Self, TransformError> {
        let analysis_vec: Vector3<T> = (*self).into();
        let normalized = analysis_vec
            .normalize()
            .map_err(TransformError::ZeroVector)?;
        Ok(normalized.into())
    }
}
