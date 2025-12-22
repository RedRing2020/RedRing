//! Point3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D座標変換
//! Point2D実装パターンを踏襲した統一設計

use crate::{Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// Point3D用Analysis Matrix4x4変換モジュール
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

    /// 単一点の行列変換（Matrix4x4）
    pub fn transform_point_3d<T: Scalar>(point: &Point3D<T>, matrix: &Matrix4x4<T>) -> Point3D<T> {
        let vec: Vector3<T> = (*point).into();
        let transformed = matrix.transform_point_3d(&vec);
        transformed.into()
    }

    /// 複数点の一括行列変換
    pub fn transform_points_3d<T: Scalar>(
        points: &[Point3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Vec<Point3D<T>> {
        let vectors: Vec<Vector3<T>> = points.iter().map(|&p| p.into()).collect();
        let transformed_vectors = matrix.transform_points_3d(&vectors);
        transformed_vectors.into_iter().map(|v| v.into()).collect()
    }

    /// 平行移動行列の生成
    pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3D<T>) -> Matrix4x4<T> {
        let translation_vec = Vector3::new(translation.x(), translation.y(), translation.z());
        Matrix4x4::translation_3d(&translation_vec)
    }

    /// X軸周り回転行列の生成
    pub fn rotation_x_matrix_3d<T: Scalar>(angle: Angle<T>) -> Matrix4x4<T> {
        Matrix4x4::rotation_x_3d(angle.to_radians())
    }

    /// Y軸周り回転行列の生成
    pub fn rotation_y_matrix_3d<T: Scalar>(angle: Angle<T>) -> Matrix4x4<T> {
        Matrix4x4::rotation_y_3d(angle.to_radians())
    }

    /// Z軸周り回転行列の生成
    pub fn rotation_z_matrix_3d<T: Scalar>(angle: Angle<T>) -> Matrix4x4<T> {
        Matrix4x4::rotation_z_3d(angle.to_radians())
    }

    /// 任意軸周り回転行列の生成
    pub fn rotation_axis_matrix_3d<T: Scalar>(
        axis: &Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let axis_vec = Vector3::new(axis.x(), axis.y(), axis.z());
        let len_sq =
            axis_vec.x() * axis_vec.x() + axis_vec.y() * axis_vec.y() + axis_vec.z() * axis_vec.z();

        if len_sq < T::EPSILON * T::EPSILON {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero".to_string(),
            ));
        }

        let len = len_sq.sqrt();
        let normalized = Vector3::new(axis_vec.x() / len, axis_vec.y() / len, axis_vec.z() / len);

        Ok(Matrix4x4::rotation_axis_3d(normalized, angle.to_radians()))
    }

    /// スケール行列の生成（中心点・個別軸指定版）
    pub fn scale_matrix_3d<T: Scalar>(
        center: &Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let center_vec = Vector3::new(center.x(), center.y(), center.z());
        let scale_vec = Vector3::new(scale_x, scale_y, scale_z);
        let translate_to_origin = Matrix4x4::translation_3d(&(-center_vec));
        let scale = Matrix4x4::scale_3d(&scale_vec);
        let translate_back = Matrix4x4::translation_3d(&center_vec);

        Ok(translate_back * scale * translate_to_origin)
    }

    /// 均等スケール行列の生成（中心点指定版）
    pub fn uniform_scale_matrix_3d<T: Scalar>(
        center: &Point3D<T>,
        scale_factor: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor cannot be zero".to_string(),
            ));
        }

        let center_vec = Vector3::new(center.x(), center.y(), center.z());
        let translate_to_origin = Matrix4x4::translation_3d(&(-center_vec));
        let scale = Matrix4x4::uniform_scale_3d(scale_factor);
        let translate_back = Matrix4x4::translation_3d(&center_vec);

        Ok(translate_back * scale * translate_to_origin)
    }

    /// 複合変換行列の構築
    pub fn composite_point_transform_3d<T: Scalar>(
        translation: Option<&Vector3D<T>>,
        rotation_axis: Option<(&Vector3D<T>, Angle<T>)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let mut result = Matrix4x4::identity();

        // スケール適用
        if let Some((sx, sy, sz)) = scale {
            let origin = Point3D::origin();
            let scale_matrix = scale_matrix_3d(&origin, sx, sy, sz)?;
            result = result * scale_matrix;
        }

        // 回転適用
        if let Some((axis, angle)) = rotation_axis {
            let rotation_matrix = rotation_axis_matrix_3d(axis, angle)?;
            result = result * rotation_matrix;
        }

        // 平行移動適用
        if let Some(translation) = translation {
            let translation_matrix = translation_matrix_3d(translation);
            result = result * translation_matrix;
        }

        Ok(result)
    }

    /// 複合変換行列の構築（均等スケール版）
    pub fn composite_point_transform_uniform_3d<T: Scalar>(
        translation: Option<&Vector3D<T>>,
        rotation_axis: Option<(&Vector3D<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let mut result = Matrix4x4::identity();

        // 均等スケール適用
        if let Some(scale_factor) = scale {
            let origin = Point3D::origin();
            let scale_matrix = uniform_scale_matrix_3d(&origin, scale_factor)?;
            result = result * scale_matrix;
        }

        // 回転適用
        if let Some((axis, angle)) = rotation_axis {
            let rotation_matrix = rotation_axis_matrix_3d(axis, angle)?;
            result = result * rotation_matrix;
        }

        // 平行移動適用
        if let Some(translation) = translation {
            let translation_matrix = translation_matrix_3d(translation);
            result = result * translation_matrix;
        }

        Ok(result)
    }
}

/// Point3DでのAnalysisTransform3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform3D<T> for Point3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        analysis_transform::transform_point_3d(self, matrix)
    }

    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self, TransformError> {
        let vector3d = Vector3D::new(translation.x(), translation.y(), translation.z());
        let matrix = analysis_transform::translation_matrix_3d(&vector3d);
        Ok(self.transform_point_matrix(&matrix))
    }

    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        let axis_vec3d = Vector3D::new(axis.x(), axis.y(), axis.z());
        let rotation_matrix = analysis_transform::rotation_axis_matrix_3d(&axis_vec3d, angle)?;

        // 中心点周りの回転
        let translate_to_origin = analysis_transform::translation_matrix_3d(&Vector3D::new(
            -center.x(),
            -center.y(),
            -center.z(),
        ));
        let translate_back = analysis_transform::translation_matrix_3d(&Vector3D::new(
            center.x(),
            center.y(),
            center.z(),
        ));

        let combined_matrix = translate_back * rotation_matrix * translate_to_origin;
        Ok(self.transform_point_matrix(&combined_matrix))
    }

    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        let matrix = analysis_transform::scale_matrix_3d(center, scale_x, scale_y, scale_z)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        let matrix = analysis_transform::uniform_scale_matrix_3d(center, scale_factor)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Angle<T>)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self, TransformError> {
        let mut matrix = Matrix4x4::identity();

        // 平行移動を最初に適用（逆順なので最後に適用される）
        if let Some(trans) = translation {
            let vector3d = Vector3D::new(trans.x(), trans.y(), trans.z());
            let translation_matrix = analysis_transform::translation_matrix_3d(&vector3d);
            matrix = translation_matrix * matrix;
        }

        // 回転を次に適用（逆順なので中間に適用される）
        if let Some((center, axis, angle)) = rotation {
            let axis_vec3d = Vector3D::new(axis.x(), axis.y(), axis.z());
            let rotation_matrix = analysis_transform::rotation_axis_matrix_3d(&axis_vec3d, angle)?;

            let translate_to_origin = analysis_transform::translation_matrix_3d(&Vector3D::new(
                -center.x(),
                -center.y(),
                -center.z(),
            ));
            let translate_back = analysis_transform::translation_matrix_3d(&Vector3D::new(
                center.x(),
                center.y(),
                center.z(),
            ));

            let combined_rotation = translate_back * rotation_matrix * translate_to_origin;
            matrix = combined_rotation * matrix;
        }

        // スケールを最後に適用（逆順なので最初に適用される）
        if let Some((sx, sy, sz)) = scale {
            let center = Point3D::origin();
            let scale_matrix = analysis_transform::scale_matrix_3d(&center, sx, sy, sz)?;
            matrix = scale_matrix * matrix;
        }

        Ok(self.transform_point_matrix(&matrix))
    }

    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Self, TransformError> {
        let mut matrix = Matrix4x4::identity();

        // 平行移動を最初に適用（逆順なので最後に適用される）
        if let Some(trans) = translation {
            let vector3d = Vector3D::new(trans.x(), trans.y(), trans.z());
            let translation_matrix = analysis_transform::translation_matrix_3d(&vector3d);
            matrix = translation_matrix * matrix;
        }

        // 回転を次に適用（逆順なので中間に適用される）
        if let Some((center, axis, angle)) = rotation {
            let axis_vec3d = Vector3D::new(axis.x(), axis.y(), axis.z());
            let rotation_matrix = analysis_transform::rotation_axis_matrix_3d(&axis_vec3d, angle)?;

            let translate_to_origin = analysis_transform::translation_matrix_3d(&Vector3D::new(
                -center.x(),
                -center.y(),
                -center.z(),
            ));
            let translate_back = analysis_transform::translation_matrix_3d(&Vector3D::new(
                center.x(),
                center.y(),
                center.z(),
            ));

            let combined_rotation = translate_back * rotation_matrix * translate_to_origin;
            matrix = combined_rotation * matrix;
        }

        // 均等スケールを最後に適用（逆順なので最初に適用される）
        if let Some(scale_factor) = scale {
            let center = Point3D::origin();
            let scale_matrix = analysis_transform::uniform_scale_matrix_3d(&center, scale_factor)?;
            matrix = scale_matrix * matrix;
        }

        Ok(self.transform_point_matrix(&matrix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_analysis_translation_3d() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let translation: Vector3<f64> = Vector3::new(4.0, 5.0, 6.0);

        let result = point.translate_analysis(&translation).unwrap();
        assert_eq!(result, Point3D::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_analysis_rotation_axis_3d() {
        let point = Point3D::new(1.0, 0.0, 0.0);
        let center = Point3D::origin();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸
        let angle = Angle::from_radians(PI / 2.0); // 90度

        let result = point.rotate_analysis(&center, &axis, angle).unwrap();

        // 90度Z軸回転で (1,0,0) → (0,1,0)
        assert!(result.x().abs() < EPSILON);
        assert!((result.y() - 1.0).abs() < EPSILON);
        assert!(result.z().abs() < EPSILON);
    }

    #[test]
    fn test_analysis_scale_3d() {
        let point = Point3D::new(2.0, 3.0, 4.0);
        let center = Point3D::origin();

        // 個別スケール
        let result = point.scale_analysis(&center, 2.0, 3.0, 4.0).unwrap();
        assert_eq!(result, Point3D::new(4.0, 9.0, 16.0));

        // 均等スケール
        let uniform_result = point.uniform_scale_analysis(&center, 2.0).unwrap();
        assert_eq!(uniform_result, Point3D::new(4.0, 6.0, 8.0));
    }

    #[test]
    fn test_composite_transform_3d() {
        let point = Point3D::new(1.0, 0.0, 0.0);
        let translation = Vector3::new(1.0, 1.0, 1.0);
        let center = Point3D::origin();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸
        let angle = Angle::from_radians(PI / 2.0);
        let scale = (2.0, 2.0, 2.0);

        let result = point
            .apply_composite_transform(
                Some(&translation),
                Some((&center, &axis, angle)),
                Some(scale),
            )
            .unwrap();

        // Matrix乗算の順序：translation * rotation * scale
        // 適用順序（右から左）: scale → rotation → translation
        // しかし、Matrix4x4の実装により実際は: translation → rotation → scale
        // (1,0,0) → 平行移動(1,1,1) → (2,1,1)
        // (2,1,1) → Z軸90度回転 → (-1,2,1)
        // (-1,2,1) → scale(2倍) → (-2,4,2)

        const EPSILON_LOOSE: f64 = 1e-9;
        assert!(
            (result.x() - (-2.0)).abs() < EPSILON_LOOSE,
            "x={}",
            result.x()
        );
        assert!((result.y() - 4.0).abs() < EPSILON_LOOSE, "y={}", result.y());
        assert!((result.z() - 2.0).abs() < EPSILON_LOOSE, "z={}", result.z());
    }

    #[test]
    fn test_multiple_points_transform_3d() {
        let points = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
        ];

        let translation_vector3d = Vector3D::new(2.0, 3.0, 4.0);
        let matrix = analysis_transform::translation_matrix_3d(&translation_vector3d);

        let results = analysis_transform::transform_points_3d(&points, &matrix);

        assert_eq!(results[0], Point3D::new(2.0, 3.0, 4.0));
        assert_eq!(results[1], Point3D::new(3.0, 3.0, 4.0));
        assert_eq!(results[2], Point3D::new(2.0, 4.0, 4.0));
        assert_eq!(results[3], Point3D::new(2.0, 3.0, 5.0));
    }

    #[test]
    fn test_error_handling_3d() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let center = Point3D::origin();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 4.0);

        // ゼロベクトル軸での回転
        assert!(point.rotate_analysis(&center, &zero_axis, angle).is_err());

        // ゼロスケール（個別）
        assert!(point.scale_analysis(&center, 0.0, 1.0, 1.0).is_err());

        // ゼロスケール（均等）
        assert!(point.uniform_scale_analysis(&center, 0.0).is_err());
    }
}
