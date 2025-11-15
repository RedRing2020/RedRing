//! LineSegment3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix/Vectorを直接使用した効率的な線分変換
//! 2端点の一括Matrix4x4変換による最適化実装
//! Point3D_analysis_transformパターンを基盤とする統一実装

use crate::{LineSegment3D, Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// LineSegment3D用Analysis Matrix/Vector変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一線分の行列変換（両端点を一括変換）
    pub fn transform_line_segment_3d<T: Scalar>(
        segment: &LineSegment3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> LineSegment3D<T> {
        let start_vec = Vector3::new(
            segment.start().x(),
            segment.start().y(),
            segment.start().z(),
        );
        let end_vec = Vector3::new(segment.end().x(), segment.end().y(), segment.end().z());

        // Matrix4x4による一括変換
        let transformed_start = matrix.transform_point_3d(&start_vec);
        let transformed_end = matrix.transform_point_3d(&end_vec);

        let new_start = Point3D::new(
            transformed_start.x(),
            transformed_start.y(),
            transformed_start.z(),
        );
        let new_end = Point3D::new(
            transformed_end.x(),
            transformed_end.y(),
            transformed_end.z(),
        );

        LineSegment3D::new(new_start, new_end).unwrap()
    }

    /// 複数線分の一括行列変換
    pub fn transform_line_segments_3d<T: Scalar>(
        segments: &[LineSegment3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Vec<LineSegment3D<T>> {
        segments
            .iter()
            .map(|segment| transform_line_segment_3d(segment, matrix))
            .collect()
    }

    /// 平行移動行列生成
    pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3<T>) -> Matrix4x4<T> {
        Matrix4x4::translation_3d(translation)
    }

    /// 軸回転行列生成
    pub fn rotation_matrix_3d<T: Scalar>(
        _center: &Point3D<T>,
        axis: &Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // 軸ベクトルのゼロチェック
        let axis_length_squared = axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z();
        if axis_length_squared <= T::EPSILON * T::EPSILON {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero".to_string(),
            ));
        }

        let axis_vec3 = Vector3::new(axis.x(), axis.y(), axis.z());

        Ok(Matrix4x4::rotation_axis_3d(axis_vec3, angle.to_radians()))
    }

    /// スケール行列生成
    pub fn scale_matrix_3d<T: Scalar>(
        center: &Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // スケール倍率のゼロチェック
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let center_vec3 = Vector3::new(center.x(), center.y(), center.z());
        let scale_vec = Vector3::new(scale_x, scale_y, scale_z);

        // 中心点周りのスケール: T * S * T^-1
        let translate_to_origin = Matrix4x4::translation_3d(&(-center_vec3));
        let scale = Matrix4x4::scale_3d(&scale_vec);
        let translate_back = Matrix4x4::translation_3d(&center_vec3);

        Ok(translate_back * scale * translate_to_origin)
    }

    /// 均等スケール行列生成
    pub fn uniform_scale_matrix_3d<T: Scalar>(
        center: &Point3D<T>,
        scale_factor: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        scale_matrix_3d(center, scale_factor, scale_factor, scale_factor)
    }

    /// 複合変換行列生成（平行移動+回転+スケール）
    pub fn composite_matrix_3d<T: Scalar>(
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Point3D<T>, &Vector3D<T>, Angle<T>)>,
        scale: Option<(T, T, T)>,
        center: &Point3D<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let mut matrix = Matrix4x4::identity();

        // スケール変換（最初に適用）
        if let Some((sx, sy, sz)) = scale {
            let scale_mat = scale_matrix_3d(center, sx, sy, sz)?;
            matrix = matrix * scale_mat;
        }

        // 回転変換
        if let Some((rot_center, axis, angle)) = rotation {
            let rotation_mat = rotation_matrix_3d(rot_center, axis, angle)?;
            matrix = matrix * rotation_mat;
        }

        // 平行移動変換（最後に適用）
        if let Some(translation_vec) = translation {
            let translation_mat = translation_matrix_3d(translation_vec);
            matrix = matrix * translation_mat;
        }

        Ok(matrix)
    }
}

/// LineSegment3DでのAnalysisTransform3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform3D<T> for LineSegment3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        analysis_transform::transform_line_segment_3d(self, matrix)
    }

    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self, TransformError> {
        let matrix = analysis_transform::translation_matrix_3d(translation);
        Ok(self.transform_point_matrix(&matrix))
    }

    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // LineSegment3D を Point3D として中心点を使用（midpoint）
        let center_point = center.midpoint();
        let axis_vector3d = Vector3D::new(axis.x(), axis.y(), axis.z());
        let matrix = analysis_transform::rotation_matrix_3d(&center_point, &axis_vector3d, angle)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        let center_point = center.midpoint();
        let matrix = analysis_transform::scale_matrix_3d(&center_point, scale_x, scale_y, scale_z)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        let center_point = center.midpoint();
        let matrix = analysis_transform::uniform_scale_matrix_3d(&center_point, scale_factor)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self, TransformError> {
        let center = self.midpoint();

        // rotation の型変換
        let rotation_converted = rotation.map(|(rot_center, axis, angle)| {
            let center_point = rot_center.midpoint();
            let axis_vector3d = Vector3D::new(axis.x(), axis.y(), axis.z());
            (center_point, axis_vector3d, angle)
        });

        let rotation_ref = rotation_converted
            .as_ref()
            .map(|(center, axis, angle)| (center, axis, *angle));

        let matrix =
            analysis_transform::composite_matrix_3d(translation, rotation_ref, scale, &center)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self, TransformError> {
        let scale_tuple = scale.map(|s| (s, s, s));
        self.apply_composite_transform(translation, rotation, scale_tuple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_segment() -> LineSegment3D<f64> {
        LineSegment3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0)).unwrap()
    }

    fn create_center_segment() -> LineSegment3D<f64> {
        // 極小線分で原点周辺の中心を表現
        LineSegment3D::new(
            Point3D::new(-1e-10, -1e-10, -1e-10),
            Point3D::new(1e-10, 1e-10, 1e-10),
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let segment = create_test_segment();
        let translation = Vector3::new(2.0, 3.0, 4.0);

        let result = segment.translate_analysis(&translation).unwrap();

        assert!((result.start().x() - 3.0).abs() < 1e-10); // 1.0 + 2.0
        assert!((result.start().y() - 5.0).abs() < 1e-10); // 2.0 + 3.0
        assert!((result.start().z() - 7.0).abs() < 1e-10); // 3.0 + 4.0
        assert!((result.end().x() - 6.0).abs() < 1e-10); // 4.0 + 2.0
        assert!((result.end().y() - 8.0).abs() < 1e-10); // 5.0 + 3.0
        assert!((result.end().z() - 10.0).abs() < 1e-10); // 6.0 + 4.0
    }

    #[test]
    fn test_analysis_rotation() {
        let segment = create_test_segment();
        let center = create_center_segment();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸周り
        let angle = Angle::from_degrees(90.0);

        let result = segment.rotate_analysis(&center, &axis, angle).unwrap();

        // Z軸周り90度回転: (x, y, z) -> (-y, x, z)
        assert!((result.start().x() - (-2.0)).abs() < 1e-10);
        assert!((result.start().y() - 1.0).abs() < 1e-10);
        assert!((result.start().z() - 3.0).abs() < 1e-10);
        assert!((result.end().x() - (-5.0)).abs() < 1e-10);
        assert!((result.end().y() - 4.0).abs() < 1e-10);
        assert!((result.end().z() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let segment = create_test_segment();
        let center = create_center_segment();

        let result = segment.scale_analysis(&center, 2.0, 2.0, 2.0).unwrap();

        assert!((result.start().x() - 2.0).abs() < 1e-10); // 1.0 * 2.0
        assert!((result.start().y() - 4.0).abs() < 1e-10); // 2.0 * 2.0
        assert!((result.start().z() - 6.0).abs() < 1e-10); // 3.0 * 2.0
        assert!((result.end().x() - 8.0).abs() < 1e-10); // 4.0 * 2.0
        assert!((result.end().y() - 10.0).abs() < 1e-10); // 5.0 * 2.0
        assert!((result.end().z() - 12.0).abs() < 1e-10); // 6.0 * 2.0
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let segment = create_test_segment();
        let center = create_center_segment();

        let result = segment.uniform_scale_analysis(&center, 3.0).unwrap();

        assert!((result.start().x() - 3.0).abs() < 1e-10); // 1.0 * 3.0
        assert!((result.start().y() - 6.0).abs() < 1e-10); // 2.0 * 3.0
        assert!((result.start().z() - 9.0).abs() < 1e-10); // 3.0 * 3.0
        assert!((result.end().x() - 12.0).abs() < 1e-10); // 4.0 * 3.0
        assert!((result.end().y() - 15.0).abs() < 1e-10); // 5.0 * 3.0
        assert!((result.end().z() - 18.0).abs() < 1e-10); // 6.0 * 3.0
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let segment = create_test_segment();

        // 平行移動行列（1, 1, 1移動）
        let translation_vec = Vector3::new(1.0, 1.0, 1.0);
        let matrix = Matrix4x4::translation_3d(&translation_vec);
        let result = segment.transform_point_matrix(&matrix);

        assert!((result.start().x() - 2.0).abs() < 1e-10); // 1.0 + 1.0
        assert!((result.start().y() - 3.0).abs() < 1e-10); // 2.0 + 1.0
        assert!((result.start().z() - 4.0).abs() < 1e-10); // 3.0 + 1.0
        assert!((result.end().x() - 5.0).abs() < 1e-10); // 4.0 + 1.0
        assert!((result.end().y() - 6.0).abs() < 1e-10); // 5.0 + 1.0
        assert!((result.end().z() - 7.0).abs() < 1e-10); // 6.0 + 1.0
    }

    #[test]
    fn test_analysis_multiple_segments() {
        let segments = vec![
            create_test_segment(),
            LineSegment3D::new(Point3D::new(7.0, 8.0, 9.0), Point3D::new(10.0, 11.0, 12.0))
                .unwrap(),
        ];

        let translation_vec = Vector3::new(1.0, 1.0, 1.0);
        let matrix = Matrix4x4::translation_3d(&translation_vec);
        let results = analysis_transform::transform_line_segments_3d(&segments, &matrix);

        assert_eq!(results.len(), 2);

        // 最初の線分
        assert!((results[0].start().x() - 2.0).abs() < 1e-10);
        assert!((results[0].start().y() - 3.0).abs() < 1e-10);
        assert!((results[0].start().z() - 4.0).abs() < 1e-10);

        // 2番目の線分
        assert!((results[1].start().x() - 8.0).abs() < 1e-10);
        assert!((results[1].start().y() - 9.0).abs() < 1e-10);
        assert!((results[1].start().z() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let segment = create_test_segment();
        let center = create_center_segment();

        let result = segment.scale_analysis(&center, 0.0, 1.0, 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_error_handling_zero_axis() {
        let segment = create_test_segment();
        let center = create_center_segment();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = segment.rotate_analysis(&center, &zero_axis, angle);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }
}
