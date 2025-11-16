//! LineSegment2D Analysis Matrix統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D線分変換
//! 始点・終点Matrix変換による高性能変換

use crate::{LineSegment2D, Point2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// LineSegment2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector2への変換（LineSegment2D専用）
    pub fn point_to_analysis_vector<T: Scalar>(point: Point2D<T>) -> Vector2<T> {
        Vector2::new(point.x(), point.y())
    }

    /// Analysis Vector2からの変換（LineSegment2D専用）
    pub fn analysis_vector_to_point<T: Scalar>(vector: Vector2<T>) -> Point2D<T> {
        Point2D::new(vector.x(), vector.y())
    }

    /// 単一線分の行列変換（Matrix3x3）
    ///
    /// 始点・終点をMatrix変換し、新しい線分を構築
    pub fn transform_line_segment_2d<T: Scalar>(
        line_segment: &LineSegment2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<LineSegment2D<T>, TransformError> {
        // 始点の変換
        let start_vec: Vector2<T> = point_to_analysis_vector(line_segment.start_point());
        let transformed_start_vec = matrix.transform_point_2d(&start_vec);
        let new_start = analysis_vector_to_point(transformed_start_vec);

        // 終点の変換
        let end_vec: Vector2<T> = point_to_analysis_vector(line_segment.end_point());
        let transformed_end_vec = matrix.transform_point_2d(&end_vec);
        let new_end = analysis_vector_to_point(transformed_end_vec);

        // 変換後の線分を構築
        LineSegment2D::new(new_start, new_end).ok_or_else(|| {
            TransformError::InvalidGeometry(
                "Transformed line segment has coincident points".to_string(),
            )
        })
    }

    /// 複数線分の一括行列変換
    pub fn transform_line_segments_2d<T: Scalar>(
        line_segments: &[LineSegment2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Result<Vec<LineSegment2D<T>>, TransformError> {
        line_segments
            .iter()
            .map(|segment| transform_line_segment_2d(segment, matrix))
            .collect()
    }

    /// 平行移動行列の生成
    pub fn translation_matrix_2d<T: Scalar>(translation: &Vector2D<T>) -> Matrix3x3<T> {
        let translation_vec: Vector2<T> = Vector2::new(translation.x(), translation.y());
        Matrix3x3::translation_2d(&translation_vec)
    }

    /// 回転行列の生成（中心点指定版）
    pub fn rotation_matrix_2d<T: Scalar>(center: &Point2D<T>, angle: Angle<T>) -> Matrix3x3<T> {
        let center_vec = Vector2::new(center.x(), center.y());
        Matrix3x3::rotation_around_point_2d(&center_vec, angle.to_radians())
    }

    /// スケール行列の生成（中心点指定版）
    pub fn scale_matrix_2d<T: Scalar>(
        center: &Point2D<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let center_vec = Vector2::new(center.x(), center.y());
        let scale_vec = Vector2::new(scale_x, scale_y);
        let translate_to_origin = Matrix3x3::translation_2d(&(-center_vec));
        let scale = Matrix3x3::scale_2d(&scale_vec);
        let translate_back = Matrix3x3::translation_2d(&center_vec);

        Ok(translate_back * scale * translate_to_origin)
    }

    /// 均等スケール行列の生成（中心点指定版）
    pub fn uniform_scale_matrix_2d<T: Scalar>(
        center: &Point2D<T>,
        scale_factor: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        if scale_factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor cannot be zero".to_string(),
            ));
        }

        scale_matrix_2d(center, scale_factor, scale_factor)
    }

    /// 複合変換行列の生成（平行移動 + 回転 + スケール）
    pub fn composite_line_segment_transform_2d<T: Scalar>(
        translation: Option<&Vector2D<T>>,
        rotation: Option<(&Point2D<T>, Angle<T>)>,
        scale: Option<(&Point2D<T>, T, T)>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let mut result = Matrix3x3::identity();

        // スケール適用
        if let Some((center, scale_x, scale_y)) = scale {
            let scale_matrix = scale_matrix_2d(center, scale_x, scale_y)?;
            result = result * scale_matrix;
        }

        // 回転適用
        if let Some((center, angle)) = rotation {
            let rotation_matrix = rotation_matrix_2d(center, angle);
            result = result * rotation_matrix;
        }

        // 平行移動適用
        if let Some(translation) = translation {
            let translation_matrix = translation_matrix_2d(translation);
            result = result * translation_matrix;
        }

        Ok(result)
    }

    /// 均等スケール版複合変換行列
    pub fn composite_uniform_transform_2d<T: Scalar>(
        translation: Option<&Vector2D<T>>,
        rotation: Option<(&Point2D<T>, Angle<T>)>,
        scale: Option<(&Point2D<T>, T)>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let scale_3d = scale.map(|(center, factor)| (center, factor, factor));
        composite_line_segment_transform_2d(translation, rotation, scale_3d)
    }

    /// 線分の長さ保持スケール（方向のみ変更、長さ維持）
    pub fn length_preserving_transform<T: Scalar>(
        line_segment: &LineSegment2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<LineSegment2D<T>, TransformError> {
        let original_length = line_segment.length();
        let transformed = transform_line_segment_2d(line_segment, matrix)?;

        // 長さが変わった場合、元の長さに調整
        let current_length = transformed.length();
        if current_length.is_zero() {
            return Err(TransformError::InvalidGeometry(
                "Transformed line segment has zero length".to_string(),
            ));
        }

        if (current_length - original_length).abs() > T::EPSILON {
            let _scale_factor = original_length / current_length;
            let center = transformed.midpoint();
            let direction = transformed.direction();
            let half_vector = direction * (original_length / (T::ONE + T::ONE));
            let new_start = center - half_vector;
            let new_end = center + half_vector;

            LineSegment2D::new(new_start, new_end).ok_or_else(|| {
                TransformError::InvalidGeometry("Length-preserving adjustment failed".to_string())
            })
        } else {
            Ok(transformed)
        }
    }
}

// ============================================================================
// AnalysisTransform2D Trait Implementation for LineSegment2D
// ============================================================================

/// LineSegment2DでのAnalysisTransform2D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform2D<T> for LineSegment2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = LineSegment2D<T>;

    /// Matrix3x3による汎用変換
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        // Analysis Transformではエラー処理をデフォルト値で対応
        analysis_transform::transform_line_segment_2d(self, matrix)
            .unwrap_or_else(|_| LineSegment2D::default())
    }

    /// Analysis統合平行移動
    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, crate::TransformError> {
        let translation_2d = Vector2D::new(translation.x(), translation.y());
        let matrix = analysis_transform::translation_matrix_2d(&translation_2d);
        analysis_transform::transform_line_segment_2d(self, &matrix)
    }

    /// Analysis統合回転（中心点指定）
    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, crate::TransformError> {
        let center_point = center.midpoint();
        let matrix = analysis_transform::rotation_matrix_2d(&center_point, angle);
        analysis_transform::transform_line_segment_2d(self, &matrix)
    }

    /// Analysis統合スケール（中心点指定）
    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, crate::TransformError> {
        let center_point = center.midpoint();
        let matrix = analysis_transform::scale_matrix_2d(&center_point, scale_x, scale_y)?;
        analysis_transform::transform_line_segment_2d(self, &matrix)
    }

    /// Analysis統合均等スケール（中心点指定）
    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, crate::TransformError> {
        let center_point = center.midpoint();
        let matrix = analysis_transform::uniform_scale_matrix_2d(&center_point, scale_factor)?;
        analysis_transform::transform_line_segment_2d(self, &matrix)
    }
}

// ============================================================================
// Default Implementation for LineSegment2D
// ============================================================================

impl<T: Scalar> Default for LineSegment2D<T> {
    fn default() -> Self {
        // デフォルト線分: X軸方向の単位線分
        LineSegment2D::new(Point2D::origin(), Point2D::new(T::ONE, T::ZERO))
            .expect("Default LineSegment2D construction should not fail")
    }
}

// ============================================================================
// Analysis Transform Support Marker
// ============================================================================

impl<T: Scalar> geo_foundation::AnalysisTransformSupport for LineSegment2D<T> {
    const HAS_ANALYSIS_INTEGRATION: bool = true;
    const PERFORMANCE_OPTIMIZED: bool = true;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point2D;
    use analysis::linalg::matrix::Matrix3x3;
    use geo_foundation::Angle;
    use std::f64::consts::PI;

    /// 基本的な平行移動テスト
    #[test]
    fn test_analysis_translation() {
        let start = Point2D::new(1.0_f64, 2.0);
        let end = Point2D::new(4.0, 6.0);
        let segment = LineSegment2D::new(start, end).unwrap();

        let translation = Vector2::new(2.0, 3.0);
        let result = segment.translate_analysis_2d(&translation).unwrap();

        assert!((result.start_point().x() - 3.0).abs() < f64::EPSILON);
        assert!((result.start_point().y() - 5.0).abs() < f64::EPSILON);
        assert!((result.end_point().x() - 6.0).abs() < f64::EPSILON);
        assert!((result.end_point().y() - 9.0).abs() < f64::EPSILON);

        // 長さが保持されているか確認
        assert!((result.length() - segment.length()).abs() < f64::EPSILON);
    }

    /// 回転変換テスト（90度回転）
    #[test]
    fn test_analysis_rotation() {
        let segment =
            LineSegment2D::new(Point2D::new(1.0_f64, 0.0), Point2D::new(3.0, 0.0)).unwrap();

        // 原点中心で回転させるための中心線分を原点に配置
        let center_segment = LineSegment2D::new(
            Point2D::origin(),
            Point2D::new(1e-10, 0.0), // 極小線分で原点中心を表現
        )
        .unwrap();

        let angle = Angle::from_radians(PI / 2.0);
        let result = segment.rotate_analysis_2d(&center_segment, angle).unwrap();

        // 90度回転後、Y軸方向になるはず
        assert!((result.start_point().x() - 0.0).abs() < 1e-10);
        assert!((result.start_point().y() - 1.0).abs() < 1e-10);
        assert!((result.end_point().x() - 0.0).abs() < 1e-10);
        assert!((result.end_point().y() - 3.0).abs() < 1e-10);
    }

    /// スケール変換テスト
    #[test]
    fn test_analysis_scale() {
        let segment =
            LineSegment2D::new(Point2D::new(1.0_f64, 1.0), Point2D::new(3.0, 3.0)).unwrap();

        // スケール中心を原点に設定
        let center_segment = LineSegment2D::new(
            Point2D::origin(),
            Point2D::new(1e-10, 0.0), // 極小線分で原点中心を表現
        )
        .unwrap();

        let result = segment
            .scale_analysis_2d(&center_segment, 2.0, 2.0)
            .unwrap();

        // 原点中心で2倍スケール（浮動小数点誤差を考慮した許容値を使用）
        assert!((result.start_point().x() - 2.0).abs() < 1e-10);
        assert!((result.start_point().y() - 2.0).abs() < 1e-10);
        assert!((result.end_point().x() - 6.0).abs() < 1e-10);
        assert!((result.end_point().y() - 6.0).abs() < 1e-10);
    }

    /// 均等スケール変換テスト（半径のみ変更）
    #[test]
    fn test_analysis_uniform_scale() {
        let segment =
            LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let center_segment = segment;
        let result = segment
            .uniform_scale_analysis_2d(&center_segment, 1.5)
            .unwrap();

        // 中心からスケールされるため、長さが1.5倍になる
        assert!((result.length() - 3.0).abs() < f64::EPSILON);
    }

    /// Matrix変換テスト
    #[test]
    fn test_analysis_matrix_transform() {
        let segment =
            LineSegment2D::new(Point2D::new(1.0_f64, 0.0), Point2D::new(3.0, 0.0)).unwrap();

        // 2倍スケール + 90度回転の複合マトリックス
        let scale = Matrix3x3::scale_2d(&Vector2::new(2.0, 2.0));
        let rotation = Matrix3x3::rotation_2d(PI / 2.0);
        let matrix = rotation * scale;

        let result = segment.transform_point_matrix_2d(&matrix);

        // X軸上の線分が、Y軸上に2倍スケールされて配置
        assert!((result.start_point().x() - 0.0).abs() < 1e-10);
        assert!((result.start_point().y() - 2.0).abs() < 1e-10);
        assert!((result.end_point().x() - 0.0).abs() < 1e-10);
        assert!((result.end_point().y() - 6.0).abs() < 1e-10);
    }

    /// 複数線分の一括変換テスト
    #[test]
    fn test_analysis_multiple_segments() {
        let segments = vec![
            LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(1.0, 0.0)).unwrap(),
            LineSegment2D::new(Point2D::new(0.0, 1.0), Point2D::new(1.0, 1.0)).unwrap(),
            LineSegment2D::new(Point2D::new(1.0, 0.0), Point2D::new(1.0, 1.0)).unwrap(),
        ];

        let translation = Matrix3x3::translation_2d(&Vector2::new(5.0, 5.0));
        let results =
            analysis_transform::transform_line_segments_2d(&segments, &translation).unwrap();

        assert_eq!(results.len(), 3);
        for (i, result) in results.iter().enumerate() {
            // 全ての線分が(5,5)だけ平行移動されている
            let original = &segments[i];
            assert!(
                (result.start_point().x() - (original.start_point().x() + 5.0)).abs()
                    < f64::EPSILON
            );
            assert!(
                (result.start_point().y() - (original.start_point().y() + 5.0)).abs()
                    < f64::EPSILON
            );
            assert!(
                (result.end_point().x() - (original.end_point().x() + 5.0)).abs() < f64::EPSILON
            );
            assert!(
                (result.end_point().y() - (original.end_point().y() + 5.0)).abs() < f64::EPSILON
            );
        }
    }

    /// エラーハンドリングテスト（ゼロスケール）
    #[test]
    fn test_error_handling_zero_scale() {
        let segment =
            LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(1.0, 0.0)).unwrap();

        let center = segment;
        let result = segment.scale_analysis_2d(&center, 0.0, 1.0);

        assert!(result.is_err());
        if let Err(TransformError::InvalidScaleFactor(_)) = result {
            // 期待されるエラー
        } else {
            panic!("Expected InvalidScaleFactor error");
        }
    }

    /// エラーハンドリングテスト（退化線分）
    #[test]
    fn test_error_handling_degenerate_segment() {
        let matrix = Matrix3x3::scale_2d(&Vector2::new(0.0, 1.0)); // X方向に0倍スケール
        let segment =
            LineSegment2D::new(Point2D::new(1.0_f64, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let result = analysis_transform::transform_line_segment_2d(&segment, &matrix);

        assert!(result.is_err());
        if let Err(TransformError::InvalidGeometry(_)) = result {
            // 期待されるエラー
        } else {
            panic!("Expected InvalidGeometry error");
        }
    }
}
