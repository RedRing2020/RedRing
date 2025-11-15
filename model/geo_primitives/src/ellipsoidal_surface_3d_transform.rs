//! EllipsoidalSurface3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D楕円体サーフェス変換
//! Point3D/Vector3D Analysis Transform パターンを基盤とする統一実装
//! 3D楕円体サーフェスの特性（中心点・軸・参照方向・3軸半径）を考慮したMatrix変換

use crate::{EllipsoidalSurface3D, Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// EllipsoidalSurface3D用Analysis Matrix4x4変換モジュール
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

    /// 楕円体サーフェスの行列変換（Matrix4x4）
    ///
    /// 楕円体の中心点、軸方向、参照方向をMatrix変換し、新しい楕円体サーフェスを構築
    pub fn transform_ellipsoidal_surface_3d<T: Scalar>(
        ellipsoidal_surface: &EllipsoidalSurface3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<EllipsoidalSurface3D<T>, TransformError> {
        // 中心点を変換
        let center_vec = point_to_analysis_vector(ellipsoidal_surface.center());
        let transformed_center_vec = matrix.transform_point_3d(&center_vec);
        let new_center = analysis_vector_to_point(transformed_center_vec);

        // 軸方向を変換
        let axis_vec = vector_to_analysis_vector(ellipsoidal_surface.axis().as_vector());
        let transformed_axis_vec = matrix.transform_vector_3d(&axis_vec);
        let new_axis_vector = analysis_vector_to_vector(transformed_axis_vec);

        // 参照方向を変換
        let ref_dir_vec =
            vector_to_analysis_vector(ellipsoidal_surface.ref_direction().as_vector());
        let transformed_ref_dir_vec = matrix.transform_vector_3d(&ref_dir_vec);
        let new_ref_direction_vector = analysis_vector_to_vector(transformed_ref_dir_vec);

        // Y軸方向を変換（派生軸）
        let y_axis_vec =
            vector_to_analysis_vector(ellipsoidal_surface.derived_y_axis().as_vector());
        let transformed_y_axis_vec = matrix.transform_vector_3d(&y_axis_vec);
        let new_y_axis_vector = analysis_vector_to_vector(transformed_y_axis_vec);

        // スケール倍率を各軸方向から計算
        let original_x_length = ellipsoidal_surface.ref_direction().as_vector().length();
        let transformed_x_length = new_ref_direction_vector.length();
        let original_y_length = ellipsoidal_surface.derived_y_axis().as_vector().length();
        let transformed_y_length = new_y_axis_vector.length();
        let original_z_length = ellipsoidal_surface.axis().as_vector().length();
        let transformed_z_length = new_axis_vector.length();

        if transformed_x_length.is_zero()
            || transformed_y_length.is_zero()
            || transformed_z_length.is_zero()
        {
            return Err(TransformError::InvalidGeometry(
                "One or more transformed axis vectors are zero".to_string(),
            ));
        }

        let x_scale_factor = transformed_x_length / original_x_length;
        let y_scale_factor = transformed_y_length / original_y_length;
        let z_scale_factor = transformed_z_length / original_z_length;

        // 新しい半径を計算（各軸独立でスケール変換）
        let new_a_radius = ellipsoidal_surface.a_radius() * x_scale_factor;
        let new_b_radius = ellipsoidal_surface.b_radius() * y_scale_factor;
        let new_c_radius = ellipsoidal_surface.c_radius() * z_scale_factor;

        // 変換後の楕円体サーフェスを構築
        EllipsoidalSurface3D::new(
            new_center,
            new_axis_vector,
            new_ref_direction_vector,
            new_a_radius,
            new_b_radius,
            new_c_radius,
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry(
                "Failed to create transformed EllipsoidalSurface3D".to_string(),
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

/// EllipsoidalSurface3D用AnalysisTransform3Dトレイト実装
impl<T: Scalar> AnalysisTransform3D<T> for EllipsoidalSurface3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = EllipsoidalSurface3D<T>;

    /// Matrix4x4による直接変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_ellipsoidal_surface_3d(self, matrix)
            .expect("EllipsoidalSurface transformation should be valid")
    }

    /// 平行移動
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        // 高速化: 中心点のみ平行移動、他の属性は不変
        let new_center = Point3D::new(
            self.center().x() + translation.x(),
            self.center().y() + translation.y(),
            self.center().z() + translation.z(),
        );

        EllipsoidalSurface3D::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.a_radius(),
            self.b_radius(),
            self.c_radius(),
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
        let matrix = analysis_transform::rotation_matrix(&center.center(), axis, angle)?;
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
        let matrix = analysis_transform::scale_matrix(&center.center(), scale_x, scale_y, scale_z)?;
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
                &scale_center.center(),
                scale_factors.0,
                scale_factors.1,
                scale_factors.2,
            )?;
            matrix = scale_mat * matrix;
        }

        if let Some((center, axis, angle)) = rotation {
            let rot_mat = analysis_transform::rotation_matrix(&center.center(), axis, angle)?;
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
    use crate::Point3D;
    use analysis::linalg::vector::Vector3;
    use geo_foundation::Angle;

    fn create_test_ellipsoidal_surface() -> EllipsoidalSurface3D<f64> {
        EllipsoidalSurface3D::new_at_origin(
            3.0, // a_radius (X軸)
            2.0, // b_radius (Y軸)
            1.0, // c_radius (Z軸)
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let surface = create_test_ellipsoidal_surface();
        let translation = Vector3::new(5.0, 3.0, 1.0);

        let result = surface.translate_analysis(&translation).unwrap();

        // 中心点が移動することを確認
        let expected_center = Point3D::new(5.0, 3.0, 1.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 軸と参照方向は変わらない
        assert!((result.axis().x() - surface.axis().x()).abs() < f64::EPSILON);
        assert!((result.axis().y() - surface.axis().y()).abs() < f64::EPSILON);
        assert!((result.axis().z() - surface.axis().z()).abs() < f64::EPSILON);

        // 各軸の半径は変わらない
        assert!((result.a_radius() - surface.a_radius()).abs() < f64::EPSILON);
        assert!((result.b_radius() - surface.b_radius()).abs() < f64::EPSILON);
        assert!((result.c_radius() - surface.c_radius()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_rotation() {
        let surface = create_test_ellipsoidal_surface();
        let center_surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();
        let axis = Vector3::new(1.0, 0.0, 0.0); // x軸回転
        let angle = Angle::from_degrees(90.0);

        let result = surface
            .rotate_analysis(&center_surface, &axis, angle)
            .unwrap();

        // 90度X軸回転後の中心点確認（原点なので変わらない）
        let expected_center = Point3D::new(0.0, 0.0, 0.0);
        assert!((result.center().x() - expected_center.x()).abs() < 1e-10);
        assert!((result.center().y() - expected_center.y()).abs() < 1e-10);
        assert!((result.center().z() - expected_center.z()).abs() < 1e-10);

        // 軸方向が回転される（Z軸(0,0,1)をX軸周りに90度回転すると(0,-1,0)になる）
        assert!((result.axis().x() - 0.0).abs() < 1e-10);
        assert!((result.axis().y() - (-1.0)).abs() < 1e-10);
        assert!((result.axis().z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let surface = create_test_ellipsoidal_surface();
        let center_surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();
        let scale_x = 2.0;
        let scale_y = 3.0;
        let scale_z = 4.0;

        let result = surface
            .scale_analysis(&center_surface, scale_x, scale_y, scale_z)
            .unwrap();

        // 中心点がスケールされることを確認（原点なので変わらない）
        let expected_center = Point3D::new(0.0, 0.0, 0.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 各軸の半径が独立してスケールされることを確認
        assert!((result.a_radius() - surface.a_radius() * scale_x).abs() < f64::EPSILON);
        assert!((result.b_radius() - surface.b_radius() * scale_y).abs() < f64::EPSILON);
        assert!((result.c_radius() - surface.c_radius() * scale_z).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let surface = create_test_ellipsoidal_surface();
        let center_surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();
        let scale_factor = 2.0;

        let result = surface
            .uniform_scale_analysis(&center_surface, scale_factor)
            .unwrap();

        // 中心点がスケールされることを確認（原点なので変わらない）
        let expected_center = Point3D::new(0.0, 0.0, 0.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 全ての半径が均等にスケールされることを確認
        assert!((result.a_radius() - surface.a_radius() * scale_factor).abs() < f64::EPSILON);
        assert!((result.b_radius() - surface.b_radius() * scale_factor).abs() < f64::EPSILON);
        assert!((result.c_radius() - surface.c_radius() * scale_factor).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_composite_transform() {
        let surface = create_test_ellipsoidal_surface();
        let translation = Vector3::new(1.0, 1.0, 1.0);
        let rotation_center_surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();
        let rotation_axis = Vector3::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_degrees(0.0); // 回転なし
        let scale_factors = (2.0, 2.0, 2.0);

        let result = surface
            .apply_composite_transform(
                Some(&translation),
                Some((&rotation_center_surface, &rotation_axis, rotation_angle)),
                Some(scale_factors),
            )
            .unwrap();

        // 複合変換の結果を確認
        // Scale(2,2,2) -> Rotate(0) -> Translate(1,1,1)
        let expected_center = Point3D::new(1.0, 1.0, 1.0); // (0*2+1, 0*2+1, 0*2+1)
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_scale_error() {
        let surface = create_test_ellipsoidal_surface();
        let center_surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();

        let result = surface.scale_analysis(&center_surface, 0.0, 1.0, 1.0);
        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    #[test]
    fn test_zero_rotation_axis_error() {
        let surface = create_test_ellipsoidal_surface();
        let center_surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = surface.rotate_analysis(&center_surface, &zero_axis, angle);
        assert!(matches!(result, Err(TransformError::InvalidRotation(_))));
    }

    #[test]
    fn test_transform_point_matrix() {
        let surface = create_test_ellipsoidal_surface();
        let matrix = Matrix4x4::translation(2.0, 3.0, 4.0);

        let result = surface.transform_point_matrix(&matrix);

        // 平行移動による中心点の変化を確認
        let expected_center = Point3D::new(2.0, 3.0, 4.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 軸と参照方向は変わらない（平行移動のため）
        assert!((result.axis().x() - surface.axis().x()).abs() < f64::EPSILON);
        assert!((result.axis().y() - surface.axis().y()).abs() < f64::EPSILON);
        assert!((result.axis().z() - surface.axis().z()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ellipsoidal_surface_properties() {
        let surface = create_test_ellipsoidal_surface();
        let center_surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();
        let scale_factor = 2.0;

        let result = surface
            .uniform_scale_analysis(&center_surface, scale_factor)
            .unwrap();

        // 楕円体特性の確認: パラメトリックサーフェス上の点
        let u = std::f64::consts::PI / 4.0; // 45度
        let v = std::f64::consts::PI / 6.0; // 30度

        let original_point = surface.point_at_uv(u, v);
        let scaled_point = result.point_at_uv(u, v);

        // スケールされた点が期待される位置にあることを確認
        assert!((scaled_point.x() - original_point.x() * scale_factor).abs() < f64::EPSILON * 10.0);
        assert!((scaled_point.y() - original_point.y() * scale_factor).abs() < f64::EPSILON * 10.0);
        assert!((scaled_point.z() - original_point.z() * scale_factor).abs() < f64::EPSILON * 10.0);
    }
}
