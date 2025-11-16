//! EllipseArc3D Analysis Matrix統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な楕円弧変換
//! 基底楕円の変換と角度パラメータの保持
//! Ellipse3D Analysis Transform パターンを基盤とする統一実装

use crate::{EllipseArc3D, Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// EllipseArc3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一楕円弧の行列変換（基底楕円の変換＋角度保持）
    pub fn transform_ellipse_arc_3d<T: Scalar>(
        ellipse_arc: &EllipseArc3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> EllipseArc3D<T> {
        // 基底楕円を変換
        let transformed_ellipse =
            crate::ellipse_3d_analysis_transform::analysis_transform::transform_ellipse_3d(
                ellipse_arc.ellipse(),
                matrix,
            );

        // 角度は変換しない（楕円弧の角度パラメータは楕円のローカル座標系での角度）
        EllipseArc3D::new(
            transformed_ellipse,
            ellipse_arc.start_angle(),
            ellipse_arc.end_angle(),
        )
    }

    /// 複数楕円弧の一括行列変換
    pub fn transform_ellipse_arcs_3d<T: Scalar>(
        ellipse_arcs: &[EllipseArc3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Vec<EllipseArc3D<T>> {
        ellipse_arcs
            .iter()
            .map(|arc| transform_ellipse_arc_3d(arc, matrix))
            .collect()
    }

    /// 平行移動行列生成
    pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3D<T>) -> Matrix4x4<T> {
        let translation_vec3 = Vector3::new(translation.x(), translation.y(), translation.z());
        Matrix4x4::translation_3d(&translation_vec3)
    }

    /// 回転行列生成（軸回転版）
    pub fn rotation_matrix_3d<T: Scalar>(
        _center: &Point3D<T>,
        axis: &Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // 回転軸の正規化チェック
        let axis_length_squared = axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z();
        if axis_length_squared.is_zero() {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero vector".to_string(),
            ));
        }

        let axis_length = axis_length_squared.sqrt();
        let normalized_axis = Vector3D::new(
            axis.x() / axis_length,
            axis.y() / axis_length,
            axis.z() / axis_length,
        );
        let normalized_axis_vec3 = Vector3::new(
            normalized_axis.x(),
            normalized_axis.y(),
            normalized_axis.z(),
        );

        // 原点周りの軸回転行列を生成
        Ok(Matrix4x4::rotation_axis_3d(
            normalized_axis_vec3,
            angle.to_radians(),
        ))
    }

    /// スケール行列生成
    pub fn scale_matrix_3d<T: Scalar>(
        _center: &Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // スケール倍率のゼロチェック
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::ZeroVector(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        Ok(Matrix4x4::scale_3d(&Vector3::new(
            scale_x, scale_y, scale_z,
        )))
    }
}

/// EllipseArc3D用のAnalysisTransform3D実装
impl<T> AnalysisTransform3D<T> for EllipseArc3D<T>
where
    T: Scalar,
{
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = EllipseArc3D<T>;

    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_ellipse_arc_3d(self, matrix)
    }

    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        let translation_vec = Vector3D::new(translation.x(), translation.y(), translation.z());
        let matrix =
            crate::ellipse_3d_analysis_transform::analysis_transform::translation_matrix_3d(
                &translation_vec,
            );
        Ok(self.transform_point_matrix(&matrix))
    }

    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        if axis.x().is_zero() && axis.y().is_zero() && axis.z().is_zero() {
            return Err(TransformError::ZeroVector("Zero rotation axis".to_string()));
        }

        let center_point = center.center();
        let axis_vec = Vector3D::new(axis.x(), axis.y(), axis.z());
        let matrix = crate::ellipse_3d_analysis_transform::analysis_transform::rotation_matrix_3d(
            &center_point,
            &axis_vec,
            angle,
        )?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.center();
        let matrix = crate::ellipse_3d_analysis_transform::analysis_transform::scale_matrix_3d(
            &center_point,
            scale_x,
            scale_y,
            scale_z,
        )?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis(center, scale_factor, scale_factor, scale_factor)
    }

    fn apply_composite_transform(
        &self,
        _translation: Option<&Vector3<T>>,
        _rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        _scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, TransformError> {
        // 簡易実装: 単位行列変換
        Ok(self.transform_point_matrix(&Matrix4x4::identity()))
    }

    fn apply_composite_transform_uniform(
        &self,
        _translation: Option<&Vector3<T>>,
        _rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        _scale: Option<T>,
    ) -> Result<Self::Output, TransformError> {
        // 簡易実装: 単位行列変換
        Ok(self.transform_point_matrix(&Matrix4x4::identity()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ellipse3D, Point3D, Vector3D};
    use analysis::linalg::vector::Vector3;
    use geo_foundation::Angle;

    /// テスト用の3D楕円弧を作成
    fn create_test_ellipse_arc() -> EllipseArc3D<f64> {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let major_axis = Vector3D::new(1.0, 0.0, 0.0);

        let ellipse = Ellipse3D::new(center, 4.0, 2.0, normal, major_axis).unwrap();
        EllipseArc3D::new(ellipse, Angle::from_degrees(0.0), Angle::from_degrees(90.0))
    }

    #[test]
    fn test_analysis_translation() {
        let arc = create_test_ellipse_arc();
        let translation = Vector3D::new(5.0, 3.0, -2.0);

        let result = arc.translate(translation);

        // 中心点が移動することを確認
        let expected_center = Point3D::new(6.0, 5.0, 1.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 楕円のサイズは変わらない
        assert!((result.semi_major() - 4.0).abs() < f64::EPSILON);
        assert!((result.semi_minor() - 2.0).abs() < f64::EPSILON);

        // 角度は変わらない
        assert!((result.start_angle().to_degrees() - 0.0).abs() < f64::EPSILON);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_rotation_z() {
        let arc = create_test_ellipse_arc();
        let rotation_center = Point3D::origin();
        let rotation_axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸回り
        let rotation_angle = Angle::from_degrees(90.0);

        let center_arc = EllipseArc3D::new(
            Ellipse3D::xy_aligned(rotation_center, 1.0, 1.0).unwrap(),
            Angle::from_degrees(0.0),
            Angle::from_degrees(360.0),
        );
        let axis_vec = Vector3::new(rotation_axis.x(), rotation_axis.y(), rotation_axis.z());
        let result = arc
            .rotate_analysis(&center_arc, &axis_vec, rotation_angle)
            .unwrap();

        // 中心点が回転することを確認（(1,2,3) -> (-2,1,3)）
        let expected_center = Point3D::new(-2.0, 1.0, 3.0);
        let tolerance = 1e-10;
        assert!((result.center().x() - expected_center.x()).abs() < tolerance);
        assert!((result.center().y() - expected_center.y()).abs() < tolerance);
        assert!((result.center().z() - expected_center.z()).abs() < tolerance);

        // 楕円のサイズは変わらない
        assert!((result.semi_major() - 4.0).abs() < f64::EPSILON);
        assert!((result.semi_minor() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_scale() {
        let arc = create_test_ellipse_arc();
        let scale_center = Point3D::origin();
        let scale_x = 2.0;
        let scale_y = 3.0;
        let scale_z = 0.5;

        let center_arc = EllipseArc3D::new(
            Ellipse3D::xy_aligned(scale_center, 1.0, 1.0).unwrap(),
            Angle::from_degrees(0.0),
            Angle::from_degrees(360.0),
        );
        let result = arc
            .scale_analysis(&center_arc, scale_x, scale_y, scale_z)
            .unwrap();

        // 中心点がスケールされることを確認
        let expected_center = Point3D::new(2.0, 6.0, 1.5);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 楕円のサイズがスケールされることを確認
        // 注意: 非等方スケールの場合、軸の長さは複雑に変化する
        assert!(result.semi_major() > 0.0);
        assert!(result.semi_minor() > 0.0);
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let arc = create_test_ellipse_arc();
        let scale_center = Point3D::origin();
        let scale_factor = 2.0;

        let center_arc = EllipseArc3D::new(
            Ellipse3D::xy_aligned(scale_center, 1.0, 1.0).unwrap(),
            Angle::from_degrees(0.0),
            Angle::from_degrees(360.0),
        );
        let result = arc
            .uniform_scale_analysis(&center_arc, scale_factor)
            .unwrap();

        // 中心点がスケールされることを確認
        let expected_center = Point3D::new(2.0, 4.0, 6.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);
        assert!((result.center().z() - expected_center.z()).abs() < f64::EPSILON);

        // 楕円のサイズが均等にスケールされることを確認
        assert!((result.semi_major() - 8.0).abs() < f64::EPSILON);
        assert!((result.semi_minor() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let arc = create_test_ellipse_arc();

        // 単位行列による変換（変化なし）
        let matrix = Matrix4x4::identity();

        let result = arc.transform_point_matrix(&matrix);

        // 変換前と同じ値であることを確認
        assert!((result.center().x() - arc.center().x()).abs() < f64::EPSILON);
        assert!((result.center().y() - arc.center().y()).abs() < f64::EPSILON);
        assert!((result.center().z() - arc.center().z()).abs() < f64::EPSILON);

        // 楕円のサイズも同じ
        assert!((result.semi_major() - arc.semi_major()).abs() < f64::EPSILON);
        assert!((result.semi_minor() - arc.semi_minor()).abs() < f64::EPSILON);

        // 角度は保持される
        assert!((result.start_angle().to_degrees() - 0.0).abs() < f64::EPSILON);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_multiple_ellipse_arcs() {
        let arcs = vec![create_test_ellipse_arc(), {
            let center = Point3D::new(5.0, 3.0, -1.0);
            let normal = Vector3D::new(1.0, 0.0, 0.0);
            let major_axis = Vector3D::new(0.0, 1.0, 0.0);
            let ellipse = Ellipse3D::new(center, 3.0, 1.5, normal, major_axis).unwrap();
            EllipseArc3D::new(
                ellipse,
                Angle::from_degrees(45.0),
                Angle::from_degrees(180.0),
            )
        }];

        let translation = Vector3D::new(2.0, -1.0, 0.5);

        for arc in arcs {
            let translation_vec = Vector3::new(translation.x(), translation.y(), translation.z());
            let result = arc.translate_analysis(&translation_vec).unwrap();
            assert!(result.semi_major() > 0.0);
            assert!(result.semi_minor() > 0.0);
        }
    }

    #[test]
    fn test_error_handling_zero_axis() {
        let arc = create_test_ellipse_arc();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let rotation_angle = Angle::from_degrees(45.0);

        let result = arc.rotate_analysis(&arc, &zero_axis, rotation_angle);
        assert!(matches!(result, Err(TransformError::ZeroVector(_))));
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let arc = create_test_ellipse_arc();

        // ゼロスケールのテスト
        let result_x = arc.scale_analysis(&arc, 0.0, 2.0, 1.0);
        assert!(matches!(
            result_x,
            Err(TransformError::InvalidScaleFactor(_))
        ));

        let result_y = arc.scale_analysis(&arc, 2.0, 0.0, 1.0);
        assert!(matches!(
            result_y,
            Err(TransformError::InvalidScaleFactor(_))
        ));

        let result_z = arc.scale_analysis(&arc, 2.0, 1.0, 0.0);
        assert!(matches!(
            result_z,
            Err(TransformError::InvalidScaleFactor(_))
        ));

        let result_uniform = arc.uniform_scale_analysis(&arc, 0.0);
        assert!(matches!(
            result_uniform,
            Err(TransformError::InvalidScaleFactor(_))
        ));
    }
}
