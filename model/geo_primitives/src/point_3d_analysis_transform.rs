//! Point3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix/Vectorを直接使用した効率的な座標変換
//! geo_nurbsのmatrix_transformパターンを基盤とする統一実装

use crate::{Point3D, Vector3D};
use analysis::linalg::{
    matrix::Matrix4x4,
    vector::Vector3,
};
use geo_foundation::{Scalar, Angle, TransformError};

/// Point3D用Analysis Matrix/Vector変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector3への変換
    impl<T: Scalar> From<Point3D<T>> for Vector3<T> {
        fn from(point: Point3D<T>) -> Self {
            Vector3::new(point.x(), point.y(), point.z())
        }
    }

    /// Analysis Vector3からの変換
    impl<T: Scalar> From<Vector3<T>> for Point3D<T> {
        fn from(vector: Vector3<T>) -> Self {
            Point3D::new(vector.x(), vector.y(), vector.z())
        }
    }

    /// 単一点の行列変換
    pub fn transform_point_3d<T: Scalar>(
        point: &Point3D<T>, 
        matrix: &Matrix4x4<T>
    ) -> Point3D<T> {
        let vec: Vector3<T> = (*point).into();
        let transformed = matrix.transform_point_3d(&vec);
        transformed.into()
    }

    /// 複数点の一括行列変換
    pub fn transform_points_3d<T: Scalar>(
        points: &[Point3D<T>], 
        matrix: &Matrix4x4<T>
    ) -> Vec<Point3D<T>> {
        let vectors: Vec<Vector3<T>> = points.iter().map(|&p| p.into()).collect();
        let transformed_vectors = matrix.transform_points_3d(&vectors);
        transformed_vectors.into_iter().map(|v| v.into()).collect()
    }

    /// 平行移動行列の生成
    pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3D<T>) -> Matrix4x4<T> {
        let translation_vec: Vector3<T> = Vector3::new(
            translation.x(), 
            translation.y(), 
            translation.z()
        );
        Matrix4x4::translation_3d(&translation_vec)
    }

    /// 回転行列の生成（軸と角度指定）
    pub fn rotation_matrix_3d<T: Scalar>(
        axis: &Vector3D<T>, 
        angle: Angle<T>
    ) -> Result<Matrix4x4<T>, TransformError> {
        let axis_vec: Vector3<T> = Vector3::new(axis.x(), axis.y(), axis.z());
        
        // ゼロベクトルチェック
        if axis_vec.norm_squared().is_zero() {
            return Err(TransformError::ZeroVector("Cannot rotate around zero vector".to_string()));
        }

        let normalized_axis = axis_vec.normalize()
            .map_err(|e| TransformError::ZeroVector(e))?;
        Ok(Matrix4x4::rotation_axis(&normalized_axis, angle.to_radians()))
    }

    /// スケール行列の生成
    pub fn scale_matrix_3d<T: Scalar>(scale_factor: T) -> Result<Matrix4x4<T>, TransformError> {
        if scale_factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor("Scale factor cannot be zero".to_string()));
        }
        Ok(Matrix4x4::uniform_scale_3d(scale_factor))
    }

    /// 複合変換行列の構築
    pub fn composite_transform_3d<T: Scalar>(
        translation: Option<&Vector3D<T>>,
        rotation: Option<(&Vector3D<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let mut result = Matrix4x4::identity();

        // スケール適用
        if let Some(scale_factor) = scale {
            let scale_matrix = scale_matrix_3d(scale_factor)?;
            result = result * scale_matrix;
        }

        // 回転適用
        if let Some((axis, angle)) = rotation {
            let rotation_matrix = rotation_matrix_3d(axis, angle)?;
            result = result * rotation_matrix;
        }

        // 平行移動適用
        if let Some(translation) = translation {
            let translation_matrix = translation_matrix_3d(translation);
            result = translation_matrix * result;
        }

        Ok(result)
    }
}

/// Point3D用の効率的なAnalysis変換トレイト
pub trait AnalysisTransform3D<T: Scalar> {
    /// Matrix4x4を使った直接変換
    fn transform_matrix(&self, matrix: &Matrix4x4<T>) -> Self;

    /// 高効率平行移動
    fn translate_analysis(&self, translation: &Vector3D<T>) -> Self;

    /// 高効率回転（軸回転）
    fn rotate_analysis(&self, axis: &Vector3D<T>, angle: Angle<T>) -> Result<Self, TransformError>
    where 
        Self: Sized;

    /// 高効率スケール
    fn scale_analysis(&self, scale_factor: T) -> Result<Self, TransformError>
    where 
        Self: Sized;

    /// 複合変換の一括実行
    fn apply_composite_transform(&self,
        translation: Option<&Vector3D<T>>,
        rotation: Option<(&Vector3D<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Self, TransformError>
    where 
        Self: Sized;
}

impl<T: Scalar> AnalysisTransform3D<T> for Point3D<T> {
    fn transform_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        analysis_transform::transform_point_3d(self, matrix)
    }

    fn translate_analysis(&self, translation: &Vector3D<T>) -> Self {
        let matrix = analysis_transform::translation_matrix_3d(translation);
        self.transform_matrix(&matrix)
    }

    fn rotate_analysis(&self, axis: &Vector3D<T>, angle: Angle<T>) -> Result<Self, TransformError> {
        let matrix = analysis_transform::rotation_matrix_3d(axis, angle)?;
        Ok(self.transform_matrix(&matrix))
    }

    fn scale_analysis(&self, scale_factor: T) -> Result<Self, TransformError> {
        let matrix = analysis_transform::scale_matrix_3d(scale_factor)?;
        Ok(self.transform_matrix(&matrix))
    }

    fn apply_composite_transform(&self,
        translation: Option<&Vector3D<T>>,
        rotation: Option<(&Vector3D<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Self, TransformError> {
        let matrix = analysis_transform::composite_transform_3d(translation, rotation, scale)?;
        Ok(self.transform_matrix(&matrix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analysis_translation() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let translation = Vector3D::new(1.0, 1.0, 1.0);
        
        let result = point.translate_analysis(&translation);
        assert_eq!(result, Point3D::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_analysis_rotation() {
        let point = Point3D::new(1.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸
        let angle = Angle::from_radians(std::f64::consts::PI / 2.0); // 90度
        
        let result = point.rotate_analysis(&axis, angle).unwrap();
        
        // 90度Z軸回転で (1,0,0) → (0,1,0)
        assert!(result.x().abs() < f64::EPSILON);
        assert!((result.y() - 1.0).abs() < f64::EPSILON);
        assert!((result.z() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_scale() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let scale_factor = 2.0;
        
        let result = point.scale_analysis(scale_factor).unwrap();
        assert_eq!(result, Point3D::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_composite_transform() {
        let point = Point3D::new(1.0, 0.0, 0.0);
        let translation = Vector3D::new(1.0, 1.0, 1.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_radians(std::f64::consts::PI / 2.0);
        let scale = 2.0;
        
        let result = point.apply_composite_transform(
            Some(&translation),
            Some((&axis, angle)),
            Some(scale)
        ).unwrap();
        
        println!("Result: ({}, {}, {})", result.x(), result.y(), result.z());
        
        // 変換順序を確認: スケール → 回転 → 平行移動
        // 手動計算:
        // 1. スケール: (1,0,0) → (2,0,0)  
        // 2. 回転(Z軸90度): (2,0,0) → (0,2,0)
        // 3. 平行移動: (0,2,0) → (1,3,1)
        const EPSILON: f64 = 1e-10;
        assert!((result.x() - 1.0).abs() < EPSILON, "X coordinate: expected 1.0, got {}", result.x());
        assert!((result.y() - 3.0).abs() < EPSILON, "Y coordinate: expected 3.0, got {}", result.y());
        assert!((result.z() - 1.0).abs() < EPSILON, "Z coordinate: expected 1.0, got {}", result.z());
    }

    #[test]
    fn test_multiple_points_transform() {
        let points = vec![
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
        ];
        
        let translation = Vector3D::new(1.0, 1.0, 1.0);
        let matrix = analysis_transform::translation_matrix_3d(&translation);
        
        let results = analysis_transform::transform_points_3d(&points, &matrix);
        
        assert_eq!(results[0], Point3D::new(2.0, 1.0, 1.0));
        assert_eq!(results[1], Point3D::new(1.0, 2.0, 1.0));
        assert_eq!(results[2], Point3D::new(1.0, 1.0, 2.0));
    }

    #[test]
    fn test_error_handling() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        
        // ゼロベクトル軸での回転
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(1.0);
        assert!(point.rotate_analysis(&zero_axis, angle).is_err());
        
        // ゼロスケール
        assert!(point.scale_analysis(0.0).is_err());
    }
}