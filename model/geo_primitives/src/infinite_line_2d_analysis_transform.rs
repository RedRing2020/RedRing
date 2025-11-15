//! InfiniteLine2D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D無限直線変換
//! Point2D/LineSegment2D Analysis Transform パターンを基盤とする統一実装
//! 無限直線の特性（点と方向による表現）を考慮したMatrix変換

use crate::{InfiniteLine2D, Point2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// InfiniteLine2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector2への変換（Point2D専用）
    pub fn point_to_analysis_vector<T: Scalar>(point: Point2D<T>) -> Vector2<T> {
        Vector2::new(point.x(), point.y())
    }

    /// Analysis Vector2からの変換（Point2D専用）
    pub fn analysis_vector_to_point<T: Scalar>(vector: Vector2<T>) -> Point2D<T> {
        Point2D::new(vector.x(), vector.y())
    }

    /// Analysis Vector2への変換（Vector2D専用）
    pub fn vector_to_analysis_vector<T: Scalar>(vector: Vector2D<T>) -> Vector2<T> {
        Vector2::new(vector.x(), vector.y())
    }

    /// Analysis Vector2からの変換（Vector2D専用）
    pub fn analysis_vector_to_vector<T: Scalar>(vector: Vector2<T>) -> Vector2D<T> {
        Vector2D::new(vector.x(), vector.y())
    }

    /// 無限直線の行列変換（Matrix3x3）
    ///
    /// 直線上の点と方向ベクトルをMatrix変換し、新しい無限直線を構築
    pub fn transform_infinite_line_2d<T: Scalar>(
        infinite_line: &InfiniteLine2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<InfiniteLine2D<T>, TransformError> {
        // 直線上の点を変換
        let point_vec = point_to_analysis_vector(infinite_line.point());
        let transformed_point_vec = matrix.transform_point_2d(&point_vec);
        let new_point = analysis_vector_to_point(transformed_point_vec);

        // 方向ベクトルを変換（平行移動成分を除去するため原点中心変換）
        let direction_vec = vector_to_analysis_vector(*infinite_line.direction());
        let transformed_direction_vec = matrix.transform_vector_2d(&direction_vec);
        let new_direction_vector = analysis_vector_to_vector(transformed_direction_vec);

        // 変換後の無限直線を構築
        InfiniteLine2D::new(new_point, new_direction_vector).ok_or_else(|| {
            TransformError::InvalidGeometry("Transformed direction vector is zero".to_string())
        })
    }

    /// 平行移動行列を生成（2D用）
    pub fn translation_matrix_2d<T: Scalar>(translation: &Vector2D<T>) -> Matrix3x3<T> {
        let translation_vec = Vector2::new(translation.x(), translation.y());
        Matrix3x3::translation_2d(&translation_vec)
    }

    /// 回転行列を生成（中心点指定）
    pub fn rotation_matrix_2d<T: Scalar>(center: &Point2D<T>, angle: Angle<T>) -> Matrix3x3<T> {
        let center_vec = point_to_analysis_vector(*center);
        Matrix3x3::rotation_around_point_2d(&center_vec, angle.to_radians())
    }

    /// スケール行列を生成（中心点指定）
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
        let center_vec = point_to_analysis_vector(*center);
        // Analysis Matrix3x3にはscale_around_point_2dがないので、手動で計算
        let translation_to_origin =
            Matrix3x3::translation_2d(&Vector2::new(-center_vec.x(), -center_vec.y()));
        let scale_vec = Vector2::new(scale_x, scale_y);
        let scale = Matrix3x3::scale_2d(&scale_vec);
        let translation_back = Matrix3x3::translation_2d(&center_vec);
        Ok(translation_back * scale * translation_to_origin)
    }

    /// 均等スケール行列を生成（中心点指定）
    pub fn uniform_scale_matrix_2d<T: Scalar>(
        center: &Point2D<T>,
        scale_factor: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        scale_matrix_2d(center, scale_factor, scale_factor)
    }
}

// ============================================================================
// AnalysisTransform2D Trait Implementation for InfiniteLine2D
// ============================================================================

/// InfiniteLine2DでのAnalysisTransform2D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform2D<T> for InfiniteLine2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = InfiniteLine2D<T>;

    /// Matrix3x3による汎用変換
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        // Analysis Transformではエラー処理をデフォルト値で対応
        analysis_transform::transform_infinite_line_2d(self, matrix)
            .unwrap_or_else(|_| InfiniteLine2D::default())
    }

    /// Analysis統合平行移動
    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError> {
        // Vector2からVector2Dへの変換
        let vector2d = Vector2D::new(translation.x(), translation.y());
        let matrix = analysis_transform::translation_matrix_2d(&vector2d);
        analysis_transform::transform_infinite_line_2d(self, &matrix)
    }

    /// Analysis統合回転（中心点指定）
    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.point();
        let matrix = analysis_transform::rotation_matrix_2d(&center_point, angle);
        analysis_transform::transform_infinite_line_2d(self, &matrix)
    }

    /// Analysis統合スケール（中心点指定）
    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.point();
        let matrix = analysis_transform::scale_matrix_2d(&center_point, scale_x, scale_y)?;
        analysis_transform::transform_infinite_line_2d(self, &matrix)
    }

    /// Analysis統合均等スケール（中心点指定）
    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.point();
        let matrix = analysis_transform::uniform_scale_matrix_2d(&center_point, scale_factor)?;
        analysis_transform::transform_infinite_line_2d(self, &matrix)
    }
}

// ============================================================================
// Default Implementation for InfiniteLine2D
// ============================================================================

impl<T: Scalar> Default for InfiniteLine2D<T> {
    fn default() -> Self {
        // デフォルト無限直線: X軸方向の原点を通る直線
        InfiniteLine2D::new(Point2D::origin(), Vector2D::new(T::ONE, T::ZERO))
            .expect("Default InfiniteLine2D construction should not fail")
    }
}

// ============================================================================
// Analysis Transform Support Marker
// ============================================================================

impl<T: Scalar> geo_foundation::AnalysisTransformSupport for InfiniteLine2D<T> {
    const HAS_ANALYSIS_INTEGRATION: bool = true;
    const PERFORMANCE_OPTIMIZED: bool = true;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_infinite_line_2d() {
        let line = InfiniteLine2D::<f64>::default();
        assert_eq!(line.point(), Point2D::origin());
        assert_eq!(line.direction().x(), 1.0);
        assert_eq!(line.direction().y(), 0.0);
    }

    #[test]
    fn test_analysis_transform_translate() {
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let translation = Vector2::new(1.0, 2.0);

        let result = line.translate_analysis_2d(&translation).unwrap();

        assert_eq!(result.point().x(), 1.0);
        assert_eq!(result.point().y(), 2.0);
        // 方向ベクトルは変わらない
        assert_eq!(result.direction().x(), 1.0);
        assert_eq!(result.direction().y(), 0.0);
    }

    #[test]
    fn test_analysis_transform_rotate() {
        let line = InfiniteLine2D::new(Point2D::new(1.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let center_line = InfiniteLine2D::new(Point2D::origin(), Vector2D::new(1.0, 0.0)).unwrap();
        let angle = Angle::from_degrees(90.0);

        let result = line.rotate_analysis_2d(&center_line, angle).unwrap();

        // 90度回転後、点 (1,0) は (0,1) になる
        const TOLERANCE: f64 = 1e-10;
        assert!((result.point().x() - 0.0).abs() < TOLERANCE);
        assert!((result.point().y() - 1.0).abs() < TOLERANCE);
        // 方向ベクトル (1,0) は (0,1) になる
        assert!((result.direction().x() - 0.0).abs() < TOLERANCE);
        assert!((result.direction().y() - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_analysis_transform_scale() {
        let line = InfiniteLine2D::new(Point2D::new(2.0, 1.0), Vector2D::new(1.0, 0.0)).unwrap();
        let center_line = InfiniteLine2D::new(Point2D::origin(), Vector2D::new(1.0, 0.0)).unwrap();

        let result = line.scale_analysis_2d(&center_line, 2.0, 3.0).unwrap();

        // 点 (2,1) が (4,3) になる
        assert_eq!(result.point().x(), 4.0);
        assert_eq!(result.point().y(), 3.0);
        // 方向ベクトル (1,0) が (2,0) になる
        assert_eq!(result.direction().x(), 1.0); // 正規化されるため
        assert_eq!(result.direction().y(), 0.0);
    }

    #[test]
    fn test_analysis_transform_uniform_scale() {
        let line = InfiniteLine2D::new(Point2D::new(1.0, 1.0), Vector2D::new(1.0, 0.0)).unwrap();
        let center_line = InfiniteLine2D::new(Point2D::origin(), Vector2D::new(1.0, 0.0)).unwrap();

        let result = line.uniform_scale_analysis_2d(&center_line, 2.0).unwrap();

        // 点 (1,1) が (2,2) になる
        assert_eq!(result.point().x(), 2.0);
        assert_eq!(result.point().y(), 2.0);
        // 方向ベクトルは変わらない（均等スケール）
        assert_eq!(result.direction().x(), 1.0);
        assert_eq!(result.direction().y(), 0.0);
    }

    #[test]
    fn test_transform_zero_scale_error() {
        let line = InfiniteLine2D::new(Point2D::new(1.0, 1.0), Vector2D::new(1.0, 0.0)).unwrap();
        let center_line = InfiniteLine2D::new(Point2D::origin(), Vector2D::new(1.0, 0.0)).unwrap();

        let result = line.scale_analysis_2d(&center_line, 0.0, 1.0);
        assert!(result.is_err());

        let result = line.uniform_scale_analysis_2d(&center_line, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_point_matrix_2d() {
        let line = InfiniteLine2D::new(Point2D::new(1.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let translation_vec = Vector2::new(2.0, 3.0);
        let matrix = Matrix3x3::translation_2d(&translation_vec);

        let result = line.transform_point_matrix_2d(&matrix);

        assert_eq!(result.point().x(), 3.0);
        assert_eq!(result.point().y(), 3.0);
        assert_eq!(result.direction().x(), 1.0);
        assert_eq!(result.direction().y(), 0.0);
    }
}
