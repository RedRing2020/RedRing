//! EllipseArc2D Analysis Matrix統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な楕円弧変換
//! 基底楕円の変換と角度パラメータの保持
//! Ellipse2D Analysis Transform パターンを基盤とする統一実装

use crate::{EllipseArc2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// EllipseArc2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一楕円弧の行列変換（基底楕円の変換＋角度保持）
    pub fn transform_ellipse_arc_2d<T: Scalar>(
        ellipse_arc: &EllipseArc2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> EllipseArc2D<T> {
        // 基底楕円を変換
        let transformed_ellipse =
            crate::ellipse_2d_analysis_transform::analysis_transform::transform_ellipse_2d(
                ellipse_arc.ellipse(),
                matrix,
            );

        // 角度は変換しない（楕円弧の角度パラメータは楕円のローカル座標系での角度）
        EllipseArc2D::new(
            transformed_ellipse,
            ellipse_arc.start_angle(),
            ellipse_arc.end_angle(),
        )
    }

    /// 複数楕円弧の一括行列変換
    pub fn transform_ellipse_arcs_2d<T: Scalar>(
        ellipse_arcs: &[EllipseArc2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Vec<EllipseArc2D<T>> {
        ellipse_arcs
            .iter()
            .map(|arc| transform_ellipse_arc_2d(arc, matrix))
            .collect()
    }
}

/// EllipseArc2D用のAnalysisTransform2D実装
impl<T> AnalysisTransform2D<T> for EllipseArc2D<T>
where
    T: Scalar,
{
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = EllipseArc2D<T>;

    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        analysis_transform::transform_ellipse_arc_2d(self, matrix)
    }

    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError> {
        let translation_vec = Vector2D::new(translation.x(), translation.y());
        let matrix =
            crate::ellipse_2d_analysis_transform::analysis_transform::translation_matrix_2d(
                &translation_vec,
            );
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.center();
        let matrix = crate::ellipse_2d_analysis_transform::analysis_transform::rotation_matrix_2d(
            &center_point,
            angle,
        );
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.center();
        let matrix = crate::ellipse_2d_analysis_transform::analysis_transform::scale_matrix_2d(
            &center_point,
            scale_x,
            scale_y,
        )?;
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis_2d(center, scale_factor, scale_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ellipse2D, Point2D, Vector2D};
    use analysis::linalg::vector::Vector2;
    use geo_foundation::Angle;

    /// テスト用の楕円弧を作成
    fn create_test_ellipse_arc() -> EllipseArc2D<f64> {
        let ellipse = Ellipse2D::axis_aligned(
            Point2D::new(1.0, 2.0), // center
            3.0,                    // semi_major
            2.0,                    // semi_minor
        )
        .unwrap();
        EllipseArc2D::new(ellipse, Angle::from_degrees(0.0), Angle::from_degrees(90.0))
    }

    #[test]
    fn test_analysis_translation() {
        let arc = create_test_ellipse_arc();
        let translation = Vector2D::new(5.0, 3.0);

        let result = arc.translate(translation);

        // 中心点が移動することを確認
        let expected_center = Point2D::new(6.0, 5.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);

        // 楕円のサイズは変わらない
        assert!((result.semi_major() - 3.0).abs() < f64::EPSILON);
        assert!((result.semi_minor() - 2.0).abs() < f64::EPSILON);

        // 角度は変わらない
        assert!((result.start_angle().to_degrees() - 0.0).abs() < f64::EPSILON);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_rotation() {
        let arc = create_test_ellipse_arc();
        let rotation_center = Point2D::origin();
        let rotation_angle = Angle::from_degrees(90.0);

        let center_arc = EllipseArc2D::new(
            Ellipse2D::axis_aligned(rotation_center, 1.0, 1.0).unwrap(),
            Angle::from_degrees(0.0),
            Angle::from_degrees(360.0),
        );
        let result = arc.rotate_analysis_2d(&center_arc, rotation_angle).unwrap();

        // 中心点が回転することを確認（(1,2) -> (-2,1)）
        let expected_center = Point2D::new(-2.0, 1.0);
        let tolerance = 1e-10;
        assert!((result.center().x() - expected_center.x()).abs() < tolerance);
        assert!((result.center().y() - expected_center.y()).abs() < tolerance);

        // 楕円のサイズは変わらない
        assert!((result.semi_major() - 3.0).abs() < f64::EPSILON);
        assert!((result.semi_minor() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_scale() {
        let arc = create_test_ellipse_arc();
        let scale_center = Point2D::origin();
        let scale_x = 2.0;
        let scale_y = 3.0;

        let center_arc = EllipseArc2D::new(
            Ellipse2D::axis_aligned(scale_center, 1.0, 1.0).unwrap(),
            Angle::from_degrees(0.0),
            Angle::from_degrees(360.0),
        );
        let result = arc
            .scale_analysis_2d(&center_arc, scale_x, scale_y)
            .unwrap();

        // 中心点がスケールされることを確認
        let expected_center = Point2D::new(2.0, 6.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);

        // 楕円のサイズがスケールされることを確認
        assert!((result.semi_major() - 6.0).abs() < f64::EPSILON); // 3.0 * 2.0
        assert!((result.semi_minor() - 6.0).abs() < f64::EPSILON); // 2.0 * 3.0
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let arc = create_test_ellipse_arc();
        let scale_center = Point2D::origin();
        let scale_factor = 2.0;

        let center_arc = EllipseArc2D::new(
            Ellipse2D::axis_aligned(scale_center, 1.0, 1.0).unwrap(),
            Angle::from_degrees(0.0),
            Angle::from_degrees(360.0),
        );
        let result = arc
            .uniform_scale_analysis_2d(&center_arc, scale_factor)
            .unwrap();

        // 中心点がスケールされることを確認
        let expected_center = Point2D::new(2.0, 4.0);
        assert!((result.center().x() - expected_center.x()).abs() < f64::EPSILON);
        assert!((result.center().y() - expected_center.y()).abs() < f64::EPSILON);

        // 楕円のサイズが均等にスケールされることを確認
        assert!((result.semi_major() - 6.0).abs() < f64::EPSILON);
        assert!((result.semi_minor() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let arc = create_test_ellipse_arc();

        // 45度回転行列
        let angle = std::f64::consts::PI / 4.0; // 45度
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let matrix = Matrix3x3::new(cos_a, -sin_a, 0.0, sin_a, cos_a, 0.0, 0.0, 0.0, 1.0);

        let result = arc.transform_point_matrix_2d(&matrix);

        // 変換後も楕円弧として有効
        assert!(result.semi_major() > 0.0);
        assert!(result.semi_minor() > 0.0);

        // 角度は保持される
        assert!((result.start_angle().to_degrees() - 0.0).abs() < f64::EPSILON);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_multiple_ellipse_arcs() {
        let arcs = vec![
            create_test_ellipse_arc(),
            EllipseArc2D::new(
                Ellipse2D::axis_aligned(Point2D::new(5.0, 3.0), 4.0, 1.0).unwrap(),
                Angle::from_degrees(45.0),
                Angle::from_degrees(180.0),
            ),
        ];

        let translation = Vector2D::new(2.0, -1.0);

        for arc in arcs {
            let translation_vec = Vector2::new(translation.x(), translation.y());
            let result = arc.translate_analysis_2d(&translation_vec).unwrap();
            assert!(result.semi_major() > 0.0);
            assert!(result.semi_minor() > 0.0);
        }
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let arc = create_test_ellipse_arc();
        let scale_center = Point2D::origin();

        // ゼロスケールのテスト
        let center_arc = EllipseArc2D::new(
            Ellipse2D::axis_aligned(scale_center, 1.0, 1.0).unwrap(),
            Angle::from_degrees(0.0),
            Angle::from_degrees(360.0),
        );
        let result_x = arc.scale_analysis_2d(&center_arc, 0.0, 2.0);
        assert!(matches!(result_x, Err(TransformError::ZeroVector(_))));

        let result_y = arc.scale_analysis_2d(&arc, 2.0, 0.0);
        assert!(matches!(result_y, Err(TransformError::ZeroVector(_))));

        let result_uniform = arc.uniform_scale_analysis_2d(&arc, 0.0);
        assert!(matches!(result_uniform, Err(TransformError::ZeroVector(_))));
    }
}
