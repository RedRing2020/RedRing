//! EllipsoidalSolid3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D楕円体ソリッド変換
//! Point3D/Vector3D Analysis Transform パターンを基盤とする統一実装
//! 3D楕円体ソリッドの特性（中心点・軸・参照方向・3つの半径）を考慮したMatrix変換

use crate::{EllipsoidalSolid3D, Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// EllipsoidalSolid3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 楕円体ソリッドの行列変換（Matrix4x4）
    ///
    /// 楕円体の中心点、軸方向、参照方向をMatrix変換し、新しい楕円体ソリッドを構築
    /// スケール変換は各半径に適用される
    pub fn transform_ellipsoidal_solid_3d<T: Scalar>(
        ellipsoidal_solid: &EllipsoidalSolid3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<EllipsoidalSolid3D<T>, TransformError> {
        // 中心点を変換
        let center_vec: Vector3<T> = ellipsoidal_solid.center_internal().into();
        let transformed_center_vec = matrix.transform_point_3d(&center_vec);
        let new_center: Point3D<T> = transformed_center_vec.into();

        // 軸方向を変換
        let axis_vec: Vector3<T> = ellipsoidal_solid.axis_internal().as_vector().into();
        let transformed_axis_vec = matrix.transform_vector_3d(&axis_vec);
        let new_axis_vector: Vector3D<T> = transformed_axis_vec.into();

        // 参照方向を変換
        let ref_dir_vec: Vector3<T> = ellipsoidal_solid
            .ref_direction_internal()
            .as_vector()
            .into();
        let transformed_ref_dir_vec = matrix.transform_vector_3d(&ref_dir_vec);
        let new_ref_direction_vector: Vector3D<T> = transformed_ref_dir_vec.into();

        // Y軸方向を計算して変換（スケール倍率計算用）
        let y_axis_vec: Vector3<T> = ellipsoidal_solid.y_axis_internal().as_vector().into();
        let transformed_y_axis_vec = matrix.transform_vector_3d(&y_axis_vec);
        let new_y_axis_vector: Vector3D<T> = transformed_y_axis_vec.into();

        // 各軸方向のスケール倍率を計算
        let original_ref_length = ellipsoidal_solid
            .ref_direction_internal()
            .as_vector()
            .length();
        let original_y_length = ellipsoidal_solid.y_axis_internal().as_vector().length();
        let original_axis_length = ellipsoidal_solid.axis_internal().as_vector().length();

        let transformed_ref_length = new_ref_direction_vector.length();
        let transformed_y_length = new_y_axis_vector.length();
        let transformed_axis_length = new_axis_vector.length();

        if transformed_ref_length.is_zero()
            || transformed_y_length.is_zero()
            || transformed_axis_length.is_zero()
        {
            return Err(TransformError::InvalidGeometry(
                "Transformed axis vectors are zero".to_string(),
            ));
        }

        let scale_x = transformed_ref_length / original_ref_length;
        let scale_y = transformed_y_length / original_y_length;
        let scale_z = transformed_axis_length / original_axis_length;

        // 新しい半径を計算（各軸のスケール変換を考慮）
        let new_a_radius = ellipsoidal_solid.a_radius_internal() * scale_x;
        let new_b_radius = ellipsoidal_solid.b_radius_internal() * scale_y;
        let new_c_radius = ellipsoidal_solid.c_radius_internal() * scale_z;

        // 変換後の楕円体ソリッドを構築
        EllipsoidalSolid3D::new(
            new_center,
            new_axis_vector,
            new_ref_direction_vector,
            new_a_radius,
            new_b_radius,
            new_c_radius,
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry(
                "Failed to create transformed EllipsoidalSolid3D".to_string(),
            )
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

        let center_vec: Vector3<T> = (*center).into();
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

        let center_vec: Vector3<T> = (*center).into();
        let scale_matrix = Matrix4x4::scale(scale_x, scale_y, scale_z);
        let translation_to_origin =
            Matrix4x4::translation(-center_vec.x(), -center_vec.y(), -center_vec.z());
        let translation_back =
            Matrix4x4::translation(center_vec.x(), center_vec.y(), center_vec.z());
        Ok(translation_back * scale_matrix * translation_to_origin)
    }
}

impl<T: Scalar> AnalysisTransform3D<T> for EllipsoidalSolid3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = EllipsoidalSolid3D<T>;

    /// Matrix4x4による直接変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_ellipsoidal_solid_3d(self, matrix)
            .expect("EllipsoidalSolid transformation should be valid")
    }

    /// 平行移動
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        // 高速化: 中心点のみ平行移動、他の属性は不変
        let new_center = Point3D::new(
            self.center_internal().x() + translation.x(),
            self.center_internal().y() + translation.y(),
            self.center_internal().z() + translation.z(),
        );

        EllipsoidalSolid3D::new(
            new_center,
            self.axis_internal().as_vector(),
            self.ref_direction_internal().as_vector(),
            self.a_radius_internal(),
            self.b_radius_internal(),
            self.c_radius_internal(),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("Translation failed".to_string()))
    }

    /// 軸回転（中心点指定）
    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::rotation_matrix(&center.center_internal(), axis, angle)?;
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
        let matrix = analysis_transform::scale_matrix(&center.center_internal(), scale_x, scale_y, scale_z)?;
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
                &scale_center.center_internal(),
                scale_factors.0,
                scale_factors.1,
                scale_factors.2,
            )?;
            matrix = scale_mat * matrix;
        }

        if let Some((center, axis, angle)) = rotation {
            let rot_mat = analysis_transform::rotation_matrix(&center.center_internal(), axis, angle)?;
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

    #[test]
    fn test_translate() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();
        let translation = Vector3::new(1.0, 2.0, 3.0);

        let translated = ellipsoid.translate_analysis(&translation).unwrap();

        assert_eq!(
            translated.center_internal(),
            Point3D::new(1.0, 2.0, 3.0)
        );
        assert_eq!(translated.a_radius_internal(), 2.0);
        assert_eq!(translated.b_radius_internal(), 3.0);
        assert_eq!(translated.c_radius_internal(), 4.0);
    }

    #[test]
    fn test_rotate_z_axis() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let rotated = ellipsoid.rotate_analysis(&ellipsoid, &axis, angle).unwrap();

        // 中心は変わらない
        assert!((rotated.center_internal().x() - 0.0).abs() < 1e-10);
        assert!((rotated.center_internal().y() - 0.0).abs() < 1e-10);
        assert!((rotated.center_internal().z() - 0.0).abs() < 1e-10);

        // Z軸周りの回転なのでc_radiusは変わらない
        assert!((rotated.c_radius_internal() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        let scaled = ellipsoid.scale_analysis(&ellipsoid, 2.0, 2.0, 2.0).unwrap();

        // 各半径が2倍になる
        assert!((scaled.a_radius_internal() - 4.0).abs() < 1e-10);
        assert!((scaled.b_radius_internal() - 6.0).abs() < 1e-10);
        assert!((scaled.c_radius_internal() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_non_uniform_scale() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // 非均等スケール
        let scaled = ellipsoid.scale_analysis(&ellipsoid, 2.0, 1.5, 0.5).unwrap();

        assert!((scaled.a_radius_internal() - 4.0).abs() < 1e-10);
        assert!((scaled.b_radius_internal() - 4.5).abs() < 1e-10);
        assert!((scaled.c_radius_internal() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_transform() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // 平行移動行列
        let matrix = Matrix4x4::translation(5.0, 6.0, 7.0);
        let transformed = ellipsoid.transform_point_matrix(&matrix);

        assert_eq!(
            transformed.center_internal(),
            Point3D::new(5.0, 6.0, 7.0)
        );
    }

    #[test]
    fn test_invalid_scale() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // ゼロスケール
        let result = ellipsoid.scale_analysis(&ellipsoid, 0.0, 1.0, 1.0);
        assert!(result.is_err());
    }
}
