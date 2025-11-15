//! Ray3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D半無限直線変換
//! Point3D/InfiniteLine3D Analysis Transform パターンを基盤とする統一実装
//! 3D半無限直線の特性（起点と方向による表現）を考慮したMatrix変換

use crate::{Point3D, Ray3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// Ray3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector3への変換（Point3D専用）
    pub fn point_to_analysis_vector<T: Scalar>(point: Point3D<T>) -> Vector3<T> {
        Vector3::new(point.x(), point.y(), point.z())
    }

    /// Analysis Vector3からの変換（Point3D専用）
    pub fn analysis_vector_to_point<T: Scalar>(vector: Vector3<T>) -> Point3D<T> {
        Point3D::new(vector.x(), vector.y(), vector.z())
    }

    /// Analysis Vector3への変換（Vector3D専用）
    pub fn vector_to_analysis_vector<T: Scalar>(vector: Vector3D<T>) -> Vector3<T> {
        Vector3::new(vector.x(), vector.y(), vector.z())
    }

    /// Analysis Vector3からの変換（Vector3D専用）
    pub fn analysis_vector_to_vector<T: Scalar>(vector: Vector3<T>) -> Vector3D<T> {
        Vector3D::new(vector.x(), vector.y(), vector.z())
    }

    /// 半無限直線の行列変換（Matrix4x4）
    ///
    /// 起点と方向ベクトルをMatrix変換し、新しい半無限直線を構築
    pub fn transform_ray_3d<T: Scalar>(
        ray: &Ray3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<Ray3D<T>, TransformError> {
        // 起点を変換
        let origin_vec = point_to_analysis_vector(ray.origin());
        let transformed_origin_vec = matrix.transform_point_3d(&origin_vec);
        let new_origin = analysis_vector_to_point(transformed_origin_vec);

        // 方向ベクトルを変換（平行移動成分を除去するため方向ベクトル専用変換）
        let direction_vec = vector_to_analysis_vector(Vector3D::new(
            ray.direction().x(),
            ray.direction().y(),
            ray.direction().z(),
        ));
        let transformed_direction_vec = matrix.transform_vector_3d(&direction_vec);
        let new_direction_vector = analysis_vector_to_vector(transformed_direction_vec);

        // 変換後の半無限直線を構築
        Ray3D::new(new_origin, new_direction_vector).ok_or_else(|| {
            TransformError::InvalidGeometry("Transformed direction vector is zero".to_string())
        })
    }

    /// 平行移動行列を生成（3D用）
    pub fn translation_matrix<T: Scalar>(translation: &Vector3<T>) -> Matrix4x4<T> {
        Matrix4x4::translation(translation.x(), translation.y(), translation.z())
    }

    /// 軸回転行列を生成（中心点指定）
    pub fn rotation_matrix<T: Scalar>(
        center: &Point3D<T>,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // 軸ベクトルが正規化されているか確認
        let axis_length = (axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z()).sqrt();
        if axis_length.is_zero() {
            return Err(TransformError::InvalidRotation(
                "Rotation axis cannot be zero vector".to_string(),
            ));
        }

        let normalized_axis = Vector3::new(
            axis.x() / axis_length,
            axis.y() / axis_length,
            axis.z() / axis_length,
        );

        let center_vec = point_to_analysis_vector(*center);
        let rotation_matrix = Matrix4x4::rotation_axis_3d(normalized_axis, angle.to_radians());
        let translation_to_origin =
            Matrix4x4::translation(-center_vec.x(), -center_vec.y(), -center_vec.z());
        let translation_back =
            Matrix4x4::translation(center_vec.x(), center_vec.y(), center_vec.z());
        Ok(translation_back * rotation_matrix * translation_to_origin)
    }

    /// スケール行列を生成（中心点指定）
    pub fn scale_matrix<T: Scalar>(
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

        let center_vec = point_to_analysis_vector(*center);
        let scale_matrix = Matrix4x4::scale(scale_x, scale_y, scale_z);
        let translation_to_origin =
            Matrix4x4::translation(-center_vec.x(), -center_vec.y(), -center_vec.z());
        let translation_back =
            Matrix4x4::translation(center_vec.x(), center_vec.y(), center_vec.z());
        Ok(translation_back * scale_matrix * translation_to_origin)
    }

    /// 複合変換パラメータ構造体
    pub struct CompositeTransform3D<T: Scalar> {
        pub translation: Vector3<T>,
        pub rotation_center: Point3D<T>,
        pub rotation_axis: Vector3<T>,
        pub rotation_angle: Angle<T>,
        pub scale_center: Point3D<T>,
        pub scale_x: T,
        pub scale_y: T,
        pub scale_z: T,
    }

    /// 複合変換行列を生成（最も効率的な順序：Scale→Rotate→Translate）
    pub fn composite_transform_matrix<T: Scalar>(
        params: &CompositeTransform3D<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let scale_matrix = scale_matrix(
            &params.scale_center,
            params.scale_x,
            params.scale_y,
            params.scale_z,
        )?;
        let rotation_matrix = rotation_matrix(
            &params.rotation_center,
            &params.rotation_axis,
            params.rotation_angle,
        )?;
        let translation_matrix = translation_matrix(&params.translation);

        Ok(translation_matrix * rotation_matrix * scale_matrix)
    }
}

/// Ray3D用AnalysisTransform3Dトレイト実装
impl<T: Scalar> AnalysisTransform3D<T> for Ray3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Ray3D<T>;

    /// Matrix4x4による直接変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_ray_3d(self, matrix)
            .expect("Ray transformation should be valid")
    }

    /// 平行移動
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        // 高速化: 起点のみ平行移動、方向ベクトルは不変
        let new_origin = Point3D::new(
            self.origin().x() + translation.x(),
            self.origin().y() + translation.y(),
            self.origin().z() + translation.z(),
        );
        let direction_vec = Vector3D::new(
            self.direction().x(),
            self.direction().y(),
            self.direction().z(),
        );
        Ray3D::new(new_origin, direction_vec).ok_or_else(|| {
            TransformError::InvalidGeometry("Direction vector became zero".to_string())
        })
    }

    /// 軸回転（中心点指定）
    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::rotation_matrix(&center.origin(), axis, angle)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    /// スケール変換（中心点指定）
    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::scale_matrix(&center.origin(), scale_x, scale_y, scale_z)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    /// 均等スケール変換
    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis(center, scale_factor, scale_factor, scale_factor)
    }

    /// 複合変換（最適化済み）
    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, TransformError> {
        let mut matrix = Matrix4x4::identity();

        if let Some(scale_factors) = scale {
            let scale_center = rotation.as_ref().map_or(self, |(center, _, _)| center);
            let scale_mat = analysis_transform::scale_matrix(
                &scale_center.origin(),
                scale_factors.0,
                scale_factors.1,
                scale_factors.2,
            )?;
            matrix = scale_mat * matrix;
        }

        if let Some((center, axis, angle)) = rotation {
            let rot_mat = analysis_transform::rotation_matrix(&center.origin(), axis, angle)?;
            matrix = rot_mat * matrix;
        }

        if let Some(trans) = translation {
            let trans_mat = analysis_transform::translation_matrix(trans);
            matrix = trans_mat * matrix;
        }

        Ok(self.transform_point_matrix(&matrix))
    }

    /// 複合変換（均等スケール版）
    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, TransformError> {
        let scale_tuple = scale.map(|s| (s, s, s));
        self.apply_composite_transform(translation, rotation, scale_tuple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};
    use analysis::linalg::vector::Vector3;
    use geo_foundation::Angle;

    fn create_test_ray() -> Ray3D<f64> {
        Ray3D::new(
            Point3D::new(1.0, 2.0, 3.0),  // origin
            Vector3D::new(1.0, 0.0, 0.0), // direction (unit x)
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let ray = create_test_ray();
        let translation = Vector3D::new(5.0, 3.0, 1.0);

        let translation_vec = Vector3::new(translation.x(), translation.y(), translation.z());
        let result = ray.translate_analysis(&translation_vec).unwrap();

        // 起点が移動することを確認
        let expected_origin = Point3D::new(6.0, 5.0, 4.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);

        // 方向ベクトルは変わらない
        assert!((result.direction().x() - 1.0).abs() < f64::EPSILON);
        assert!((result.direction().y() - 0.0).abs() < f64::EPSILON);
        assert!((result.direction().z() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_rotation() {
        let ray = create_test_ray();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0); // z軸回転
        let angle = Angle::from_degrees(90.0);

        let center_ray = Ray3D::new(center, Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let axis_vec = Vector3::new(axis.x(), axis.y(), axis.z());
        let result = ray.rotate_analysis(&center_ray, &axis_vec, angle).unwrap();

        // 90度Z軸回転後の起点確認
        let expected_origin = Point3D::new(-2.0, 1.0, 3.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < 1e-10);
        assert!((result.origin().y() - expected_origin.y()).abs() < 1e-10);
        assert!((result.origin().z() - expected_origin.z()).abs() < 1e-10);

        // 90度Z軸回転後の方向ベクトル確認
        assert!((result.direction().x() - 0.0).abs() < f64::EPSILON);
        assert!((result.direction().y() - 1.0).abs() < f64::EPSILON);
        assert!((result.direction().z() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_scale() {
        let ray = create_test_ray();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let scale_x = 2.0;
        let scale_y = 3.0;
        let scale_z = 4.0;

        let center_ray = Ray3D::new(center, Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let result = ray
            .scale_analysis(&center_ray, scale_x, scale_y, scale_z)
            .unwrap();

        // スケール変換後の起点確認
        let expected_origin = Point3D::new(2.0, 6.0, 12.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);

        // 方向ベクトルもスケール変換される（正規化される）
        // Direction3Dは常に正規化されている（norm = 1.0）
        assert!((result.direction().x() - 1.0).abs() < f64::EPSILON); // x方向は保持
        assert!((result.direction().y() - 0.0).abs() < f64::EPSILON); // y方向は0のまま
        assert!((result.direction().z() - 0.0).abs() < f64::EPSILON); // z方向は0のまま
    }

    #[test]
    fn test_analysis_composite_transform() {
        let ray = create_test_ray();
        let translation = Vector3D::new(1.0, 1.0, 1.0);
        let rotation_center = Point3D::new(0.0, 0.0, 0.0);
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_degrees(0.0); // 回転なし

        let scale_x = 2.0;
        let scale_y = 2.0;
        let scale_z = 2.0;

        let translation_vec = Vector3::new(translation.x(), translation.y(), translation.z());
        let rotation_center_ray = Ray3D::new(rotation_center, rotation_axis).unwrap();
        let axis_vec = Vector3::new(rotation_axis.x(), rotation_axis.y(), rotation_axis.z());
        let result = ray
            .apply_composite_transform(
                Some(&translation_vec),
                Some((&rotation_center_ray, &axis_vec, rotation_angle)),
                Some((scale_x, scale_y, scale_z)),
            )
            .unwrap();

        // 複合変換の結果を確認
        // Scale(2,2,2) -> Rotate(0) -> Translate(1,1,1)
        let expected_origin = Point3D::new(3.0, 5.0, 7.0); // (1*2+1, 2*2+1, 3*2+1)
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_scale_error() {
        let ray = create_test_ray();
        let center = Point3D::new(0.0, 0.0, 0.0);

        let center_ray = Ray3D::new(center, Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let result = ray.scale_analysis(&center_ray, 0.0, 1.0, 1.0);
        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    #[test]
    fn test_zero_rotation_axis_error() {
        let ray = create_test_ray();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let center_ray = Ray3D::new(center, Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let zero_axis_vec = Vector3::new(zero_axis.x(), zero_axis.y(), zero_axis.z());
        let result = ray.rotate_analysis(&center_ray, &zero_axis_vec, angle);
        assert!(matches!(result, Err(TransformError::InvalidRotation(_))));
    }
}
