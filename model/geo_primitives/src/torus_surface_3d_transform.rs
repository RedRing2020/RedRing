//! TorusSurface3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3Dトーラス面変換
//! Point3D/Vector3D Analysis Transform パターンを基盤とする統一実装
//! 3Dトーラス面の特性（原点・軸・主半径・副半径・パラメトリック面）を考慮したMatrix変換

use crate::{Point3D, TorusSurface3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// TorusSurface3D用Analysis Matrix4x4変換モジュール
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

    /// トーラス面の行列変換（Matrix4x4）
    ///
    /// トーラスの原点、軸方向をMatrix変換し、半径をスケール変換して新しいトーラス面を構築
    pub fn transform_torus_surface_3d<T: Scalar>(
        torus_surface: &TorusSurface3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<TorusSurface3D<T>, TransformError> {
        // 原点を変換
        let origin_vec = point_to_analysis_vector(torus_surface.origin());
        let transformed_origin_vec = matrix.transform_point_3d(&origin_vec);
        let new_origin = analysis_vector_to_point(transformed_origin_vec);

        // Z軸方向を変換
        let z_axis_vec = vector_to_analysis_vector(torus_surface.z_axis().as_vector());
        let transformed_z_axis_vec = matrix.transform_vector_3d(&z_axis_vec);
        let new_z_axis_vector = analysis_vector_to_vector(transformed_z_axis_vec);

        // X軸方向を変換
        let x_axis_vec = vector_to_analysis_vector(torus_surface.x_axis().as_vector());
        let transformed_x_axis_vec = matrix.transform_vector_3d(&x_axis_vec);
        let new_x_axis_vector = analysis_vector_to_vector(transformed_x_axis_vec);

        // スケール倍率を計算（半径の変換に使用）
        let original_z_length = torus_surface.z_axis().as_vector().length();
        let transformed_z_length = new_z_axis_vector.length();
        let original_x_length = torus_surface.x_axis().as_vector().length();
        let transformed_x_length = new_x_axis_vector.length();

        if transformed_z_length.is_zero() || transformed_x_length.is_zero() {
            return Err(TransformError::InvalidGeometry(
                "Transformed axis vectors are zero".to_string(),
            ));
        }

        // トーラス面の場合、Z軸とX軸の平均スケールを使用
        let z_scale = transformed_z_length / original_z_length;
        let x_scale = transformed_x_length / original_x_length;
        let average_scale = (z_scale + x_scale) / T::from_f64(2.0);

        // 新しい半径を計算（スケール変換を考慮）
        let new_major_radius = torus_surface.major_radius() * average_scale;
        let new_minor_radius = torus_surface.minor_radius() * average_scale;

        // 正規化された軸を生成
        let normalized_z_axis =
            crate::Direction3D::from_vector(new_z_axis_vector).ok_or_else(|| {
                TransformError::InvalidGeometry("Failed to normalize Z axis".to_string())
            })?;
        let normalized_x_axis =
            crate::Direction3D::from_vector(new_x_axis_vector).ok_or_else(|| {
                TransformError::InvalidGeometry("Failed to normalize X axis".to_string())
            })?;

        // 変換後のトーラス面を構築
        TorusSurface3D::new(
            new_origin,
            normalized_z_axis,
            normalized_x_axis,
            new_major_radius,
            new_minor_radius,
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry(
                "Failed to create transformed TorusSurface3D".to_string(),
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

/// TorusSurface3D用AnalysisTransform3Dトレイト実装
impl<T: Scalar> AnalysisTransform3D<T> for TorusSurface3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = TorusSurface3D<T>;

    /// Matrix4x4による直接変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_torus_surface_3d(self, matrix)
            .expect("TorusSurface transformation should be valid")
    }

    /// 平行移動
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        // 高速化: 原点のみ平行移動、他の属性は不変
        let new_origin = Point3D::new(
            self.origin().x() + translation.x(),
            self.origin().y() + translation.y(),
            self.origin().z() + translation.z(),
        );

        TorusSurface3D::new(
            new_origin,
            self.z_axis(),
            self.x_axis(),
            self.major_radius(),
            self.minor_radius(),
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
    use crate::Point3D;
    use analysis::linalg::vector::Vector3;
    use geo_foundation::Angle;

    fn create_test_torus_surface() -> TorusSurface3D<f64> {
        TorusSurface3D::standard(
            3.0, // major_radius
            1.0, // minor_radius
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let surface = create_test_torus_surface();
        let translation = Vector3::new(5.0, 3.0, 1.0);

        let result = surface.translate_analysis(&translation).unwrap();

        // 原点が移動することを確認
        let expected_origin = Point3D::new(5.0, 3.0, 1.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);

        // 軸は変わらない
        assert!((result.z_axis().x() - surface.z_axis().x()).abs() < f64::EPSILON);
        assert!((result.z_axis().y() - surface.z_axis().y()).abs() < f64::EPSILON);
        assert!((result.z_axis().z() - surface.z_axis().z()).abs() < f64::EPSILON);

        // 半径は変わらない
        assert!((result.major_radius() - surface.major_radius()).abs() < f64::EPSILON);
        assert!((result.minor_radius() - surface.minor_radius()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_rotation() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
        let axis = Vector3::new(1.0, 0.0, 0.0); // x軸回転
        let angle = Angle::from_degrees(90.0);

        let result = surface
            .rotate_analysis(&center_surface, &axis, angle)
            .unwrap();

        // 90度X軸回転後の原点確認（原点なので変わらない）
        let expected_origin = Point3D::new(0.0, 0.0, 0.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < 1e-10);
        assert!((result.origin().y() - expected_origin.y()).abs() < 1e-10);
        assert!((result.origin().z() - expected_origin.z()).abs() < 1e-10);

        // Z軸方向が回転される（Z軸(0,0,1)をX軸周りに90度回転すると(0,-1,0)になる）
        assert!((result.z_axis().x() - 0.0).abs() < 1e-10);
        assert!((result.z_axis().y() - (-1.0)).abs() < 1e-10);
        assert!((result.z_axis().z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
        let scale_x = 2.0;
        let scale_y = 2.0;
        let scale_z = 2.0;

        let result = surface
            .scale_analysis(&center_surface, scale_x, scale_y, scale_z)
            .unwrap();

        // 原点がスケールされることを確認（原点なので変わらない）
        let expected_origin = Point3D::new(0.0, 0.0, 0.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);

        // 半径がスケールされることを確認
        let expected_scale = 2.0; // 均等スケール
        assert!(
            (result.major_radius() - surface.major_radius() * expected_scale).abs() < f64::EPSILON
        );
        assert!(
            (result.minor_radius() - surface.minor_radius() * expected_scale).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
        let scale_factor = 2.5;

        let result = surface
            .uniform_scale_analysis(&center_surface, scale_factor)
            .unwrap();

        // 原点がスケールされることを確認（原点なので変わらない）
        let expected_origin = Point3D::new(0.0, 0.0, 0.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);

        // 半径が均等にスケールされることを確認
        assert!(
            (result.major_radius() - surface.major_radius() * scale_factor).abs() < f64::EPSILON
        );
        assert!(
            (result.minor_radius() - surface.minor_radius() * scale_factor).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_analysis_composite_transform() {
        let surface = create_test_torus_surface();
        let translation = Vector3::new(1.0, 1.0, 1.0);
        let rotation_center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
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
        let expected_origin = Point3D::new(1.0, 1.0, 1.0); // (0*2+1, 0*2+1, 0*2+1)
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_scale_error() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();

        let result = surface.scale_analysis(&center_surface, 0.0, 1.0, 1.0);
        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    #[test]
    fn test_zero_rotation_axis_error() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = surface.rotate_analysis(&center_surface, &zero_axis, angle);
        assert!(matches!(result, Err(TransformError::InvalidRotation(_))));
    }

    #[test]
    fn test_transform_point_matrix() {
        let surface = create_test_torus_surface();
        let matrix = Matrix4x4::translation(2.0, 3.0, 4.0);

        let result = surface.transform_point_matrix(&matrix);

        // 平行移動による原点の変化を確認
        let expected_origin = Point3D::new(2.0, 3.0, 4.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
        assert!((result.origin().z() - expected_origin.z()).abs() < f64::EPSILON);

        // 軸は変わらない（平行移動のため）
        assert!((result.z_axis().x() - surface.z_axis().x()).abs() < f64::EPSILON);
        assert!((result.z_axis().y() - surface.z_axis().y()).abs() < f64::EPSILON);
        assert!((result.z_axis().z() - surface.z_axis().z()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_surface_area_preservation_uniform_scale() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
        let scale_factor = 2.0;

        let result = surface
            .uniform_scale_analysis(&center_surface, scale_factor)
            .unwrap();

        // 表面積は scale_factor^2 倍になる
        let expected_area_ratio = scale_factor * scale_factor;
        let actual_area_ratio = result.surface_area() / surface.surface_area();
        assert!((actual_area_ratio - expected_area_ratio).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn test_parametric_surface_preservation() {
        let surface = create_test_torus_surface();
        let matrix = Matrix4x4::scale(1.5, 1.5, 1.5);

        let result = surface.transform_point_matrix(&matrix);

        // パラメトリック面上の点が正しくスケールされることを確認
        let u = std::f64::consts::PI / 4.0;
        let v = std::f64::consts::PI / 3.0;

        let original_point = surface.point_at(u, v);
        let transformed_point = result.point_at(u, v);

        // スケールされた点との関係を確認
        let scale_factor = 1.5;
        assert!((transformed_point.x() - original_point.x() * scale_factor).abs() < 1e-10);
        assert!((transformed_point.y() - original_point.y() * scale_factor).abs() < 1e-10);
        assert!((transformed_point.z() - original_point.z() * scale_factor).abs() < 1e-10);
    }

    #[test]
    fn test_normal_vector_preservation() {
        let surface = create_test_torus_surface();
        let matrix = Matrix4x4::scale(2.0, 2.0, 2.0);

        let result = surface.transform_point_matrix(&matrix);

        // パラメトリック面での法線ベクトルが適切に変換されることを確認
        let u = std::f64::consts::PI / 6.0;
        let v = std::f64::consts::PI / 4.0;

        let original_normal = surface.normal_at(u, v);
        let transformed_normal = result.normal_at(u, v);

        // 法線ベクトルは正規化されているため、方向は保持される（スケールされない）
        // 均等スケールの場合、法線方向は変化しない
        assert!((transformed_normal.x() - original_normal.x()).abs() < 1e-10);
        assert!((transformed_normal.y() - original_normal.y()).abs() < 1e-10);
        assert!((transformed_normal.z() - original_normal.z()).abs() < 1e-10);
    }

    #[test]
    fn test_torus_surface_validity() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
        let scale_factor = 0.5; // 縮小変換

        let result = surface
            .uniform_scale_analysis(&center_surface, scale_factor)
            .unwrap();

        // 変換後もトーラス面が有効であることを確認
        assert!(result.is_valid());

        // 両方の半径が正の値であること
        assert!(result.major_radius() > 0.0);
        assert!(result.minor_radius() > 0.0);
    }

    #[test]
    fn test_cam_surface_properties() {
        let surface = create_test_torus_surface();
        let center_surface = TorusSurface3D::standard(2.0, 0.5).unwrap();
        let scale_factor = 1.5;

        let result = surface
            .uniform_scale_analysis(&center_surface, scale_factor)
            .unwrap();

        // CAM工具オフセット計算に重要な軸の直交性が保持されることを確認
        let z_vec = result.z_axis().as_vector();
        let x_vec = result.x_axis().as_vector();
        let dot_product = z_vec.dot(&x_vec);

        assert!(dot_product.abs() < f64::EPSILON * 10.0);

        // 軸が単位ベクトルであることを確認
        assert!((z_vec.length() - 1.0).abs() < f64::EPSILON * 10.0);
        assert!((x_vec.length() - 1.0).abs() < f64::EPSILON * 10.0);
    }
}
