//! BBox2D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D境界ボックス変換
//! Arc2D実装パターンを踏襲した統一設計
//! BBox2Dの矩形構造を保持した変換処理

use crate::{BBox2D, Point2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Scalar, TransformError};

/// BBox2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// BBox2Dの3x3行列変換（境界ボックス構造保持）
    pub fn transform_bbox_2d<T: Scalar>(
        bbox: &BBox2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<BBox2D<T>, TransformError> {
        // 境界ボックスの4つの角点を変換
        let min_point = bbox.min();
        let max_point = bbox.max();

        // 4つの角点を定義
        let corners = [
            min_point,                                  // 左下
            Point2D::new(max_point.x(), min_point.y()), // 右下
            max_point,                                  // 右上
            Point2D::new(min_point.x(), max_point.y()), // 左上
        ];

        // 全ての角点を変換
        let mut transformed_points = Vec::with_capacity(4);
        for corner in &corners {
            let corner_vec = Vector2::new(corner.x(), corner.y());
            let transformed_vec = matrix.transform_point_2d(&corner_vec);
            transformed_points.push(Point2D::new(transformed_vec.x(), transformed_vec.y()));
        }

        // 変換後の点群から新しい境界ボックスを生成
        BBox2D::from_points(&transformed_points).ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create transformed BBox2D".to_string())
        })
    }

    /// 複数境界ボックスの一括3x3行列変換
    pub fn transform_bboxes_2d<T: Scalar>(
        bboxes: &[BBox2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Result<Vec<BBox2D<T>>, TransformError> {
        bboxes
            .iter()
            .map(|bbox| transform_bbox_2d(bbox, matrix))
            .collect()
    }

    /// 平行移動行列生成（2D用）
    pub fn translation_matrix_2d<T: Scalar>(dx: T, dy: T) -> Matrix3x3<T> {
        let translation = Vector2::new(dx, dy);
        Matrix3x3::translation_2d(&translation)
    }

    /// 回転行列生成（2D用）
    pub fn rotation_matrix_2d<T: Scalar>(angle: geo_foundation::Angle<T>) -> Matrix3x3<T> {
        Matrix3x3::rotation_2d(angle.to_radians())
    }

    /// スケール行列生成（2D用）
    pub fn scale_matrix_2d<T: Scalar>(sx: T, sy: T) -> Matrix3x3<T> {
        let scale = Vector2::new(sx, sy);
        Matrix3x3::scale_2d(&scale)
    }
}

// ============================================================================
// AnalysisTransform2D Implementation
// ============================================================================

impl<T: Scalar> AnalysisTransform2D<T> for BBox2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = geo_foundation::Angle<T>;
    type Output = BBox2D<T>;

    /// Matrix3x3による直接座標変換
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        analysis_transform::transform_bbox_2d(self, matrix).unwrap_or_else(|_| *self)
        // エラー時は元のBBox2Dを返す
    }

    /// 平行移動変換（Analysis Vector2使用）
    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::translation_matrix_2d(translation.x(), translation.y());
        analysis_transform::transform_bbox_2d(self, &matrix)
    }

    /// 回転変換（中心点指定）
    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        // 中心点への移動 -> 回転 -> 元位置への移動の合成変換
        let center_point = center.center();
        let to_origin =
            analysis_transform::translation_matrix_2d(-center_point.x(), -center_point.y());
        let rotation = analysis_transform::rotation_matrix_2d(angle);
        let from_origin =
            analysis_transform::translation_matrix_2d(center_point.x(), center_point.y());

        let combined_matrix = from_origin.mul_matrix(&rotation.mul_matrix(&to_origin));
        analysis_transform::transform_bbox_2d(self, &combined_matrix)
    }

    /// スケール変換
    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let center_point = center.center();
        let to_origin =
            analysis_transform::translation_matrix_2d(-center_point.x(), -center_point.y());
        let scale = analysis_transform::scale_matrix_2d(scale_x, scale_y);
        let from_origin =
            analysis_transform::translation_matrix_2d(center_point.x(), center_point.y());

        let combined_matrix = from_origin.mul_matrix(&scale.mul_matrix(&to_origin));
        analysis_transform::transform_bbox_2d(self, &combined_matrix)
    }

    /// 均等スケール変換
    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis_2d(center, scale_factor, scale_factor)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point2D;
    use geo_foundation::Angle;

    /// テスト用BBox2D生成
    fn create_test_bbox() -> BBox2D<f64> {
        BBox2D::new(Point2D::new(-1.0, -1.0), Point2D::new(1.0, 1.0))
    }

    #[test]
    fn test_translation_analysis_transform() {
        let bbox = create_test_bbox();
        let translation = Vector2::new(3.0, 2.0);

        let result = bbox.translate_analysis_2d(&translation).unwrap();

        assert_eq!(result.min().x(), 2.0);
        assert_eq!(result.min().y(), 1.0);
        assert_eq!(result.max().x(), 4.0);
        assert_eq!(result.max().y(), 3.0);
        assert_eq!(result.width(), 2.0);
        assert_eq!(result.height(), 2.0);
    }

    #[test]
    fn test_rotation_analysis_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();
        let rotation_angle = Angle::from_degrees(90.0);

        let result = bbox
            .rotate_analysis_2d(&center_bbox, rotation_angle)
            .unwrap();

        // 90度回転後は同じサイズの境界ボックスになる（対称な正方形のため）
        assert!((result.width() - 2.0).abs() < 1e-10);
        assert!((result.height() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_analysis_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.scale_analysis_2d(&center_bbox, 2.0, 3.0).unwrap();

        assert_eq!(result.width(), 4.0); // 2.0 * 2.0
        assert_eq!(result.height(), 6.0); // 2.0 * 3.0
    }

    #[test]
    fn test_uniform_scale_analysis_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.uniform_scale_analysis_2d(&center_bbox, 2.5).unwrap();

        assert_eq!(result.width(), 5.0); // 2.0 * 2.5
        assert_eq!(result.height(), 5.0); // 2.0 * 2.5
    }

    #[test]
    fn test_matrix_direct_transform() {
        let bbox = create_test_bbox();
        let matrix = analysis_transform::translation_matrix_2d(5.0, -3.0);

        let result = bbox.transform_point_matrix_2d(&matrix);

        assert_eq!(result.min().x(), 4.0); // -1.0 + 5.0
        assert_eq!(result.min().y(), -4.0); // -1.0 + (-3.0)
        assert_eq!(result.max().x(), 6.0); // 1.0 + 5.0
        assert_eq!(result.max().y(), -2.0); // 1.0 + (-3.0)
    }

    #[test]
    fn test_zero_scale_error() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.scale_analysis_2d(&center_bbox, 0.0, 1.0);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => {}
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_batch_transform() {
        let bboxes = vec![create_test_bbox(); 3];
        let matrix = analysis_transform::translation_matrix_2d(10.0, -5.0);

        let results = analysis_transform::transform_bboxes_2d(&bboxes, &matrix).unwrap();

        assert_eq!(results.len(), 3);
        for result in results {
            assert_eq!(result.min().x(), 9.0); // -1.0 + 10.0
            assert_eq!(result.min().y(), -6.0); // -1.0 + (-5.0)
            assert_eq!(result.max().x(), 11.0); // 1.0 + 10.0
            assert_eq!(result.max().y(), -4.0); // 1.0 + (-5.0)
        }
    }

    #[test]
    fn test_corner_transform_consistency() {
        let bbox = create_test_bbox();
        let rotation_angle = Angle::from_degrees(45.0);
        let rotation_matrix = analysis_transform::rotation_matrix_2d(rotation_angle);

        let result = analysis_transform::transform_bbox_2d(&bbox, &rotation_matrix).unwrap();

        // 45度回転後の境界ボックスは元より大きくなる（対角線方向に拡大）
        assert!(result.width() > bbox.width());
        assert!(result.height() > bbox.height());

        // 中心は原点のまま（原点中心回転のため）
        let center = result.center();
        assert!((center.x()).abs() < 1e-10);
        assert!((center.y()).abs() < 1e-10);
    }

    #[test]
    fn test_negative_scale_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.scale_analysis_2d(&center_bbox, -1.0, 1.0).unwrap();

        // 負のスケールで反転
        assert_eq!(result.width(), 2.0);
        assert_eq!(result.height(), 2.0);
        assert!((result.center().x()).abs() < 1e-10);
        assert!((result.center().y()).abs() < 1e-10);
    }
}
