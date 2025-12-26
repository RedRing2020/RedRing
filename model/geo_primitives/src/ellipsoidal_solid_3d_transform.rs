//! EllipsoidalSolid3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D楕円体ソリッド変換
//! Point3D/Vector3D Analysis Transform パターンを基盤とする統一実装
//! 3D楕円体ソリッドの特性（中心点・軸・参照方向・3つの半径）を考慮したMatrix変換
//!
//! **作成日: 2025年12月27日**
//! **最終更新: 2025年12月27日**

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
    /// 楕円体ソリッドを平行移動
    fn translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {
        let translation_vec: Vector3<T> = translation.into();
        let matrix = analysis_transform::translation_matrix(&translation_vec);
        analysis_transform::transform_ellipsoidal_solid_3d(self, &matrix)
    }

    /// 楕円体ソリッドを回転
    fn rotate(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        let axis_vec: Vector3<T> = axis.into();
        let matrix = analysis_transform::rotation_matrix(&center, &axis_vec, angle)?;
        analysis_transform::transform_ellipsoidal_solid_3d(self, &matrix)
    }

    /// 楕円体ソリッドをスケール変換
    fn scale(
        &self,
        center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        let matrix = analysis_transform::scale_matrix(&center, scale_x, scale_y, scale_z)?;
        analysis_transform::transform_ellipsoidal_solid_3d(self, &matrix)
    }

    /// 楕円体ソリッドをMatrix4x4で変換
    fn transform(&self, matrix: &Matrix4x4<T>) -> Result<Self, TransformError> {
        analysis_transform::transform_ellipsoidal_solid_3d(self, matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let translated = ellipsoid.translate(translation).unwrap();

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
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let rotated = ellipsoid.rotate(center, axis, angle).unwrap();

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
        let center = Point3D::new(0.0, 0.0, 0.0);

        let scaled = ellipsoid.scale(center, 2.0, 2.0, 2.0).unwrap();

        // 各半径が2倍になる
        assert!((scaled.a_radius_internal() - 4.0).abs() < 1e-10);
        assert!((scaled.b_radius_internal() - 6.0).abs() < 1e-10);
        assert!((scaled.c_radius_internal() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_non_uniform_scale() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();
        let center = Point3D::new(0.0, 0.0, 0.0);

        // 非均等スケール
        let scaled = ellipsoid.scale(center, 2.0, 1.5, 0.5).unwrap();

        assert!((scaled.a_radius_internal() - 4.0).abs() < 1e-10);
        assert!((scaled.b_radius_internal() - 4.5).abs() < 1e-10);
        assert!((scaled.c_radius_internal() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_transform() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // 平行移動行列
        let matrix = Matrix4x4::translation(5.0, 6.0, 7.0);
        let transformed = ellipsoid.transform(&matrix).unwrap();

        assert_eq!(
            transformed.center_internal(),
            Point3D::new(5.0, 6.0, 7.0)
        );
    }

    #[test]
    fn test_invalid_scale() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();
        let center = Point3D::new(0.0, 0.0, 0.0);

        // ゼロスケール
        let result = ellipsoid.scale(center, 0.0, 1.0, 1.0);
        assert!(result.is_err());
    }
}
