//! Triangle2D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な三角形変換
//! 3頂点の一括Matrix3x3変換による最適化実装
//! Point2D/LineSegment2D Analysis Transform パターンを基盤とする統一実装

use crate::{Point2D, Triangle2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// Triangle2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一三角形の行列変換（3頂点を一括変換）
    pub fn transform_triangle_2d<T: Scalar>(
        triangle: &Triangle2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Triangle2D<T> {
        let va_vec = Vector2::new(triangle.vertex_a().x(), triangle.vertex_a().y());
        let vb_vec = Vector2::new(triangle.vertex_b().x(), triangle.vertex_b().y());
        let vc_vec = Vector2::new(triangle.vertex_c().x(), triangle.vertex_c().y());

        // Matrix3x3による一括変換
        let transformed_va = matrix.transform_point_2d(&va_vec);
        let transformed_vb = matrix.transform_point_2d(&vb_vec);
        let transformed_vc = matrix.transform_point_2d(&vc_vec);

        let new_va = Point2D::new(transformed_va.x(), transformed_va.y());
        let new_vb = Point2D::new(transformed_vb.x(), transformed_vb.y());
        let new_vc = Point2D::new(transformed_vc.x(), transformed_vc.y());

        Triangle2D::new(new_va, new_vb, new_vc).unwrap()
    }

    /// 複数三角形の一括行列変換
    pub fn transform_triangles_2d<T: Scalar>(
        triangles: &[Triangle2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Vec<Triangle2D<T>> {
        triangles
            .iter()
            .map(|triangle| transform_triangle_2d(triangle, matrix))
            .collect()
    }

    /// 平行移動行列生成
    pub fn translation_matrix_2d<T: Scalar>(translation: &Vector2D<T>) -> Matrix3x3<T> {
        let translation_vec2 = Vector2::new(translation.x(), translation.y());
        Matrix3x3::translation_2d(&translation_vec2)
    }

    /// 回転行列生成（中心点指定版）
    pub fn rotation_matrix_2d<T: Scalar>(center: &Point2D<T>, angle: Angle<T>) -> Matrix3x3<T> {
        let center_vec2 = Vector2::new(center.x(), center.y());
        Matrix3x3::rotation_around_point_2d(&center_vec2, angle.to_radians())
    }

    /// スケール行列生成
    pub fn scale_matrix_2d<T: Scalar>(
        _center: &Point2D<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        // スケール倍率のゼロチェック
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let scale_vec2 = Vector2::new(scale_x, scale_y);
        Ok(Matrix3x3::scale_2d(&scale_vec2))
    }

    /// 均等スケール行列生成
    pub fn uniform_scale_matrix_2d<T: Scalar>(
        center: &Point2D<T>,
        scale_factor: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        scale_matrix_2d(center, scale_factor, scale_factor)
    }

    /// 複合変換行列生成（平行移動+回転+スケール）
    pub fn composite_triangle_transform_2d<T: Scalar>(
        translation: Option<&Vector2D<T>>,
        rotation: Option<(&Point2D<T>, Angle<T>)>,
        scale: Option<(T, T)>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let mut matrix = Matrix3x3::identity();

        // スケール変換（最初に適用）
        if let Some((sx, sy)) = scale {
            let origin = Point2D::origin();
            let scale_mat = scale_matrix_2d(&origin, sx, sy)?;
            matrix = matrix * scale_mat;
        }

        // 回転変換
        if let Some((rot_center, angle)) = rotation {
            let rotation_mat = rotation_matrix_2d(rot_center, angle);
            matrix = matrix * rotation_mat;
        }

        // 平行移動変換（最後に適用）
        if let Some(translation_vec) = translation {
            let translation_mat = translation_matrix_2d(translation_vec);
            matrix = matrix * translation_mat;
        }

        Ok(matrix)
    }
}

/// Triangle2DでのAnalysisTransform2D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform2D<T> for Triangle2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix_2d(&self, matrix: &Matrix3x3<T>) -> Self {
        analysis_transform::transform_triangle_2d(self, matrix)
    }

    fn translate_analysis_2d(&self, translation: &Vector2<T>) -> Result<Self, TransformError> {
        // Vector2からVector2Dへの変換
        let vector2d = Vector2D::new(translation.x(), translation.y());
        let matrix = analysis_transform::translation_matrix_2d(&vector2d);
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn rotate_analysis_2d(&self, center: &Self, angle: Angle<T>) -> Result<Self, TransformError> {
        // Triangle2D を Point2D として中心点を使用（重心）
        let center_point = center.centroid();
        let matrix = analysis_transform::rotation_matrix_2d(&center_point, angle);
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        let center_point = center.centroid();
        let matrix = analysis_transform::scale_matrix_2d(&center_point, scale_x, scale_y)?;
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        let center_point = center.centroid();
        let matrix = analysis_transform::uniform_scale_matrix_2d(&center_point, scale_factor)?;
        Ok(self.transform_point_matrix_2d(&matrix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_triangle() -> Triangle2D<f64> {
        Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.5, 1.0),
        )
        .unwrap()
    }

    fn create_center_triangle() -> Triangle2D<f64> {
        // 原点周辺の小さな正三角形
        Triangle2D::new(
            Point2D::new(-0.1, -0.1),
            Point2D::new(0.1, -0.1),
            Point2D::new(0.0, 0.1),
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let triangle = create_test_triangle();
        let translation = Vector2::new(2.0, 3.0);

        let result = triangle.translate_analysis_2d(&translation).unwrap();

        assert!((result.vertex_a().x() - 2.0).abs() < 1e-10); // 0.0 + 2.0
        assert!((result.vertex_a().y() - 3.0).abs() < 1e-10); // 0.0 + 3.0
        assert!((result.vertex_b().x() - 3.0).abs() < 1e-10); // 1.0 + 2.0
        assert!((result.vertex_b().y() - 3.0).abs() < 1e-10); // 0.0 + 3.0
        assert!((result.vertex_c().x() - 2.5).abs() < 1e-10); // 0.5 + 2.0
        assert!((result.vertex_c().y() - 4.0).abs() < 1e-10); // 1.0 + 3.0
    }

    #[test]
    fn test_analysis_rotation() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();
        let angle = Angle::from_degrees(90.0);

        let result = triangle.rotate_analysis_2d(&center, angle).unwrap();

        // 回転中心が原点でない場合は、実際の変換を確認
        // 元の三角形と回転後の三角形が回転変換されていることを確認
        let original_area = triangle.area();
        let transformed_area = result.area();

        // 面積は変化しない
        assert!((original_area - transformed_area).abs() < 1e-10);

        // 重心が回転されていることを確認
        let original_centroid = triangle.centroid();
        let result_centroid = result.centroid();

        // 重心の距離は保存される（回転だけなので）
        let rotation_center = center.centroid();
        let original_distance = ((original_centroid.x() - rotation_center.x()).powi(2)
            + (original_centroid.y() - rotation_center.y()).powi(2))
        .sqrt();
        let result_distance = ((result_centroid.x() - rotation_center.x()).powi(2)
            + (result_centroid.y() - rotation_center.y()).powi(2))
        .sqrt();

        assert!((original_distance - result_distance).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();

        let result = triangle.scale_analysis_2d(&center, 2.0, 2.0).unwrap();

        assert!((result.vertex_a().x() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_a().y() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_b().x() - 2.0).abs() < 1e-10); // 1.0 * 2.0
        assert!((result.vertex_b().y() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_c().x() - 1.0).abs() < 1e-10); // 0.5 * 2.0
        assert!((result.vertex_c().y() - 2.0).abs() < 1e-10); // 1.0 * 2.0
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();

        let result = triangle.uniform_scale_analysis_2d(&center, 3.0).unwrap();

        assert!((result.vertex_a().x() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_a().y() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_b().x() - 3.0).abs() < 1e-10); // 1.0 * 3.0
        assert!((result.vertex_b().y() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_c().x() - 1.5).abs() < 1e-10); // 0.5 * 3.0
        assert!((result.vertex_c().y() - 3.0).abs() < 1e-10); // 1.0 * 3.0
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let triangle = create_test_triangle();

        // 平行移動行列（1, 1移動）
        let translation_vec = Vector2::new(1.0, 1.0);
        let matrix = Matrix3x3::translation_2d(&translation_vec);
        let result = triangle.transform_point_matrix_2d(&matrix);

        assert!((result.vertex_a().x() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_a().y() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_b().x() - 2.0).abs() < 1e-10); // 1.0 + 1.0
        assert!((result.vertex_b().y() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_c().x() - 1.5).abs() < 1e-10); // 0.5 + 1.0
        assert!((result.vertex_c().y() - 2.0).abs() < 1e-10); // 1.0 + 1.0
    }

    #[test]
    fn test_analysis_multiple_triangles() {
        let triangles = vec![
            create_test_triangle(),
            Triangle2D::new(
                Point2D::new(2.0, 2.0),
                Point2D::new(3.0, 2.0),
                Point2D::new(2.5, 3.0),
            )
            .unwrap(),
        ];

        let translation_vec = Vector2::new(1.0, 1.0);
        let matrix = Matrix3x3::translation_2d(&translation_vec);
        let results = analysis_transform::transform_triangles_2d(&triangles, &matrix);

        assert_eq!(results.len(), 2);

        // 最初の三角形
        assert!((results[0].vertex_a().x() - 1.0).abs() < 1e-10);
        assert!((results[0].vertex_a().y() - 1.0).abs() < 1e-10);

        // 2番目の三角形
        assert!((results[1].vertex_a().x() - 3.0).abs() < 1e-10);
        assert!((results[1].vertex_a().y() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();

        let result = triangle.scale_analysis_2d(&center, 0.0, 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }
}
