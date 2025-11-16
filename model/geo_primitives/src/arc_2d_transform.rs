//! Arc2D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D円弧変換
//! Direction2D/Vector2D実装パターンを踏襲した統一設計
//! Arc2Dの円弧構造を保持した変換処理

use crate::{Arc2D, Circle2D, Point2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// Arc2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Arc2Dの3x3行列変換（円弧構造保持）
    pub fn transform_arc_2d<T: Scalar>(
        arc: &Arc2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<Arc2D<T>, TransformError> {
        // 基底円の変換
        let transformed_circle = transform_circle_2d(arc.circle(), matrix)?;

        // 角度は回転成分のみ影響を受ける
        let rotation_radians = matrix.extract_rotation_2d();
        let rotation_angle = Angle::from_radians(rotation_radians);
        let new_start_angle = arc.start_angle() + rotation_angle;
        let new_end_angle = arc.end_angle() + rotation_angle;

        Arc2D::new(transformed_circle, new_start_angle, new_end_angle).ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create transformed Arc2D".to_string())
        })
    }

    /// Circle2Dの3x3行列変換
    fn transform_circle_2d<T: Scalar>(
        circle: &Circle2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<Circle2D<T>, TransformError> {
        // 中心点の変換
        let center_vec = Vector2::new(circle.center().x(), circle.center().y());
        let transformed_center_vec = matrix.transform_point_2d(&center_vec);
        let transformed_center =
            Point2D::new(transformed_center_vec.x(), transformed_center_vec.y());

        // 半径のスケール変換
        let scale_factors = matrix.extract_scale_2d();
        let transformed_radius = circle.radius() * scale_factors.x().abs(); // 均等スケールを仮定

        if transformed_radius <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "Transformed circle radius is non-positive".to_string(),
            ));
        }

        Circle2D::new(transformed_center, transformed_radius).ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create transformed Circle2D".to_string())
        })
    }

    /// 複数円弧の一括3x3行列変換
    pub fn transform_arcs_2d<T: Scalar>(
        arcs: &[Arc2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Result<Vec<Arc2D<T>>, TransformError> {
        arcs.iter()
            .map(|arc| transform_arc_2d(arc, matrix))
            .collect()
    }

    /// 平行移動行列生成（2D用）
    pub fn translation_matrix_2d<T: Scalar>(dx: T, dy: T) -> Matrix3x3<T> {
        let translation = Vector2::new(dx, dy);
        Matrix3x3::translation_2d(&translation)
    }

    /// 回転行列生成（2D用）
    pub fn rotation_matrix_2d<T: Scalar>(angle: Angle<T>) -> Matrix3x3<T> {
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

impl<T: Scalar> AnalysisTransform2D<T> for Arc2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Arc2D<T>;

    /// Matrix3x3による直接座標変換
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        analysis_transform::transform_arc_2d(self, matrix).unwrap_or_else(|_| *self)
        // エラー時は元のArc2Dを返す
    }

    /// 平行移動変換（Analysis Vector2使用）
    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::translation_matrix_2d(translation.x(), translation.y());
        analysis_transform::transform_arc_2d(self, &matrix)
    }

    /// 回転変換（中心点指定）
    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        // 中心点への移動 -> 回転 -> 元位置への移動の合成変換
        let center_point = center.circle().center();
        let to_origin =
            analysis_transform::translation_matrix_2d(-center_point.x(), -center_point.y());
        let rotation = analysis_transform::rotation_matrix_2d(angle);
        let from_origin =
            analysis_transform::translation_matrix_2d(center_point.x(), center_point.y());

        let combined_matrix = from_origin.mul_matrix(&rotation.mul_matrix(&to_origin));
        analysis_transform::transform_arc_2d(self, &combined_matrix)
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

        let center_point = center.circle().center();
        let to_origin =
            analysis_transform::translation_matrix_2d(-center_point.x(), -center_point.y());
        let scale = analysis_transform::scale_matrix_2d(scale_x, scale_y);
        let from_origin =
            analysis_transform::translation_matrix_2d(center_point.x(), center_point.y());

        let combined_matrix = from_origin.mul_matrix(&scale.mul_matrix(&to_origin));
        analysis_transform::transform_arc_2d(self, &combined_matrix)
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
    use crate::{Circle2D, Point2D};
    use analysis::Angle;

    /// テスト用Arc2D生成
    fn create_test_arc() -> Arc2D<f64> {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let start_angle = Angle::from_degrees(0.0);
        let end_angle = Angle::from_degrees(90.0);
        Arc2D::new(circle, start_angle, end_angle).unwrap()
    }

    #[test]
    fn test_translation_analysis_transform() {
        let arc = create_test_arc();
        let translation = Vector2::new(2.0, 3.0);

        let result = arc.translate_analysis_2d(&translation).unwrap();

        assert_eq!(result.circle().center().x(), 2.0);
        assert_eq!(result.circle().center().y(), 3.0);
        assert_eq!(result.circle().radius(), 1.0);
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 90.0);
    }

    #[test]
    fn test_rotation_analysis_transform() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();
        let rotation_angle = Angle::from_degrees(45.0);

        let result = arc.rotate_analysis_2d(&center_arc, rotation_angle).unwrap();

        // 回転後の角度チェック
        assert!((result.start_angle().to_degrees() - 45.0).abs() < 1e-10);
        assert!((result.end_angle().to_degrees() - 135.0).abs() < 1e-10);
        assert_eq!(result.circle().radius(), 1.0);
    }

    #[test]
    fn test_scale_analysis_transform() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();

        let result = arc.scale_analysis_2d(&center_arc, 2.0, 2.0).unwrap();

        assert_eq!(result.circle().radius(), 2.0);
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 90.0);
    }

    #[test]
    fn test_uniform_scale_analysis_transform() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();

        let result = arc.uniform_scale_analysis_2d(&center_arc, 3.0).unwrap();

        assert_eq!(result.circle().radius(), 3.0);
    }

    #[test]
    fn test_matrix_direct_transform() {
        let arc = create_test_arc();
        let matrix = analysis_transform::translation_matrix_2d(1.0, 2.0);

        let result = arc.transform_point_matrix_2d(&matrix);

        assert_eq!(result.circle().center().x(), 1.0);
        assert_eq!(result.circle().center().y(), 2.0);
    }

    #[test]
    fn test_zero_scale_error() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();

        let result = arc.scale_analysis_2d(&center_arc, 0.0, 1.0);

        assert!(result.is_err());
    }

    #[test]
    fn test_batch_transform() {
        let arcs = vec![create_test_arc(); 3];
        let matrix = analysis_transform::translation_matrix_2d(5.0, 10.0);

        let results = analysis_transform::transform_arcs_2d(&arcs, &matrix).unwrap();

        assert_eq!(results.len(), 3);
        for result in results {
            assert_eq!(result.circle().center().x(), 5.0);
            assert_eq!(result.circle().center().y(), 10.0);
        }
    }
}
