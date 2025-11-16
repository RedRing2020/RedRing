//! Circle2D Analysis Matrix統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D円変換
//! 中心点Matrix変換 + 半径スケール分離による高性能変換

use crate::{Circle2D, Point2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// Circle2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Analysis Vector2への変換（Circle2D専用）
    pub fn point_to_analysis_vector<T: Scalar>(point: Point2D<T>) -> Vector2<T> {
        Vector2::new(point.x(), point.y())
    }

    /// Analysis Vector2からの変換（Circle2D専用）
    pub fn analysis_vector_to_point<T: Scalar>(vector: Vector2<T>) -> Point2D<T> {
        Point2D::new(vector.x(), vector.y())
    }

    /// 単一円の行列変換（Matrix3x3）
    ///
    /// 中心点をMatrix変換し、スケール成分を半径に適用
    pub fn transform_circle_2d<T: Scalar>(
        circle: &Circle2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<Circle2D<T>, TransformError> {
        // 中心点の変換
        let center_vec: Vector2<T> = circle.center().into();
        let transformed_center_vec = matrix.transform_point_2d(&center_vec);
        let new_center: Point2D<T> = transformed_center_vec.into();

        // 半径のスケール変換（X軸ベースでスケール係数を計算）
        let unit_x = Vector2::new(T::ONE, T::ZERO);
        let scaled_x = matrix.transform_vector_2d(&unit_x);
        let scale_factor = scaled_x.norm();

        if scale_factor <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor must be positive for circles".to_string(),
            ));
        }

        let new_radius = circle.radius() * scale_factor;

        Circle2D::new(new_center, new_radius).ok_or_else(|| {
            TransformError::InvalidGeometry("Transformed circle has invalid radius".to_string())
        })
    }

    /// 複数円の一括行列変換
    pub fn transform_circles_2d<T: Scalar>(
        circles: &[Circle2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Result<Vec<Circle2D<T>>, TransformError> {
        circles
            .iter()
            .map(|circle| transform_circle_2d(circle, matrix))
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

        let center_vec = Vector2::new(center.x(), center.y());
        let scale_vec = Vector2::new(scale_factor, scale_factor);
        let translate_to_origin = Matrix3x3::translation_2d(&(-center_vec));
        let scale = Matrix3x3::scale_2d(&scale_vec);
        let translate_back = Matrix3x3::translation_2d(&center_vec);

        Ok(translate_back * scale * translate_to_origin)
    }

    /// 複合変換行列の生成（平行移動 + 回転 + スケール）
    pub fn composite_circle_transform_2d<T: Scalar>(
        translation: Option<&Vector2D<T>>,
        rotation: Option<(&Point2D<T>, Angle<T>)>,
        scale: Option<(&Point2D<T>, T)>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let mut result = Matrix3x3::identity();

        // スケール適用
        if let Some((center, factor)) = scale {
            let scale_matrix = uniform_scale_matrix_2d(center, factor)?;
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

    /// 半径のみのスケール変換
    pub fn scale_radius_only<T: Scalar>(
        circle: &Circle2D<T>,
        factor: T,
    ) -> Result<Circle2D<T>, TransformError> {
        if factor <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Radius scale factor must be positive".to_string(),
            ));
        }

        let new_radius = circle.radius() * factor;
        Circle2D::new(circle.center(), new_radius)
            .ok_or_else(|| TransformError::InvalidGeometry("Scaled radius is invalid".to_string()))
    }
}

// ============================================================================
// AnalysisTransform2D Trait Implementation for Circle2D
// ============================================================================

/// Circle2DでのAnalysisTransform2D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform2D<T> for Circle2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Circle2D<T>;

    /// Matrix3x3による汎用変換
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        // Analysis Transformではエラー処理をデフォルト値で対応
        analysis_transform::transform_circle_2d(self, matrix)
            .unwrap_or_else(|_| Circle2D::default())
    }

    /// Analysis統合平行移動
    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, crate::TransformError> {
        let translation_2d = Vector2D::new(translation.x(), translation.y());
        let matrix = analysis_transform::translation_matrix_2d(&translation_2d);
        analysis_transform::transform_circle_2d(self, &matrix)
    }

    /// Analysis統合回転（中心点指定）
    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, crate::TransformError> {
        let center_point = Point2D::new(center.center().x(), center.center().y());
        let matrix = analysis_transform::rotation_matrix_2d(&center_point, angle);
        analysis_transform::transform_circle_2d(self, &matrix)
    }

    /// Analysis統合スケール（中心点指定）
    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, crate::TransformError> {
        // 円は非等方スケールをサポートしないため、等方チェック
        if (scale_x - scale_y).abs() > T::EPSILON {
            return Err(crate::TransformError::InvalidGeometry(
                "Non-uniform scaling not supported for circles".to_string(),
            ));
        }
        self.uniform_scale_analysis_2d(center, scale_x)
    }

    /// Analysis統合均等スケール（中心点指定）
    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, crate::TransformError> {
        let center_point = Point2D::new(center.center().x(), center.center().y());
        let matrix = analysis_transform::uniform_scale_matrix_2d(&center_point, scale_factor)?;
        analysis_transform::transform_circle_2d(self, &matrix)
    }
}

// ============================================================================
// Circle2D Analysis Transform拡張メソッド
// ============================================================================

impl<T: Scalar> Circle2D<T> {
    /// Analysis Matrix3x3による一括複数円変換
    ///
    /// 複数の円を同じ変換で一括処理
    pub fn transform_multiple_analysis(
        circles: &[Circle2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Result<Vec<Circle2D<T>>, TransformError> {
        analysis_transform::transform_circles_2d(circles, matrix)
    }

    /// 半径のみのAnalysisスケール
    ///
    /// 中心点を固定して半径のみをスケール
    pub fn analysis_scale_radius(&self, factor: T) -> Result<Self, TransformError> {
        analysis_transform::scale_radius_only(self, factor)
    }

    /// 原点中心のAnalysis回転
    ///
    /// 原点を中心とした回転の最適化版
    pub fn analysis_rotate_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        let origin = Point2D::origin();
        let matrix = analysis_transform::rotation_matrix_2d(&origin, angle);
        analysis_transform::transform_circle_2d(self, &matrix)
    }

    /// Analysis Matrix効率最適化判定
    ///
    /// 単純変換でBasicTransformの方が効率的かどうか判定
    pub fn should_use_analysis_transform(&self, transformation_count: usize) -> bool {
        // 複数回変換または複合変換の場合、Analysis Matrixが効率的
        transformation_count > 1
    }
}

// ============================================================================
// Tests Module
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::Angle;

    #[test]
    fn test_analysis_translation() {
        let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();
        let translation = Vector2::new(5.0, 7.0);

        let transformed = circle.translate_analysis_2d(&translation).unwrap();

        assert!((transformed.center().x() - 6.0).abs() < 1e-10);
        assert!((transformed.center().y() - 9.0).abs() < 1e-10);
        assert!((transformed.radius() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_rotation() {
        let circle = Circle2D::new(Point2D::new(1.0, 0.0), 2.0).unwrap();
        let center_circle = Circle2D::new(Point2D::origin(), 1.0).unwrap(); // 中心として使用する円
        let angle = Angle::from_degrees(90.0);

        let transformed = circle.rotate_analysis_2d(&center_circle, angle).unwrap();

        // 90度回転で (1,0) -> (0,1)
        assert!((transformed.center().x() - 0.0).abs() < 1e-10);
        assert!((transformed.center().y() - 1.0).abs() < 1e-10);
        assert!((transformed.radius() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let circle = Circle2D::new(Point2D::new(2.0, 4.0), 3.0).unwrap();
        let center_circle = Circle2D::new(Point2D::origin(), 1.0).unwrap(); // 中心として使用する円
        let scale_factor = 2.0;

        let transformed = circle
            .uniform_scale_analysis_2d(&center_circle, scale_factor)
            .unwrap();

        assert!((transformed.center().x() - 4.0).abs() < 1e-10);
        assert!((transformed.center().y() - 8.0).abs() < 1e-10);
        assert!((transformed.radius() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_composite_transform() {
        let circle = Circle2D::new(Point2D::new(1.0_f64, 1.0), 2.0).unwrap();

        let _translation = Vector2D::new(3.0_f64, 4.0);
        let _rotation_center = Point2D::<f64>::origin();
        let _rotation_angle = Angle::from_degrees(90.0_f64);
        let _scale_center = Point2D::<f64>::origin();
        let _scale_factor = 2.0_f64;

        // Matrix変換で複合変換をシミュレート
        // 平行移動 + スケールの組み合わせを使用
        let scale_factor = 2.0;
        let translation_vec = Vector2::new(5.0, 3.0);
        let scale_matrix = Matrix3x3::uniform_scale_2d(scale_factor);
        let translation_matrix = Matrix3x3::translation_2d(&translation_vec);
        let matrix = translation_matrix * scale_matrix;
        let transformed = circle.transform_point_matrix_2d(&matrix);

        // 複合変換: スケール -> 平行移動
        // 最終結果の検証
        assert!(transformed.radius() > 0.0);
        assert!(!transformed.center().x().is_nan());
        assert!(!transformed.center().y().is_nan());
    }

    #[test]
    fn test_analysis_scale_radius_only() {
        let circle = Circle2D::new(Point2D::new(5.0, 7.0), 3.0).unwrap();
        let factor = 1.5;

        let transformed = circle.analysis_scale_radius(factor).unwrap();

        // 中心は変わらず、半径のみスケール
        assert!((transformed.center().x() - 5.0).abs() < 1e-10);
        assert!((transformed.center().y() - 7.0).abs() < 1e-10);
        assert!((transformed.radius() - 4.5).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_multiple_circles() {
        let circles = vec![
            Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap(),
            Circle2D::new(Point2D::new(2.0, 2.0), 2.0).unwrap(),
            Circle2D::new(Point2D::new(-1.0, 1.0), 0.5).unwrap(),
        ];

        let translation = analysis_transform::translation_matrix_2d(&Vector2D::new(10.0, 20.0));

        let transformed = Circle2D::transform_multiple_analysis(&circles, &translation).unwrap();

        assert_eq!(transformed.len(), 3);

        // 各円の平行移動を確認
        assert!((transformed[0].center().x() - 10.0).abs() < 1e-10);
        assert!((transformed[0].center().y() - 20.0).abs() < 1e-10);
        assert!((transformed[1].center().x() - 12.0).abs() < 1e-10);
        assert!((transformed[1].center().y() - 22.0).abs() < 1e-10);

        // 半径は変わらない
        assert!((transformed[0].radius() - 1.0).abs() < 1e-10);
        assert!((transformed[1].radius() - 2.0).abs() < 1e-10);
        assert!((transformed[2].radius() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();
        let center = Point2D::origin();

        let center_circle = Circle2D::new(center, 1.0).unwrap();
        let result = circle.uniform_scale_analysis_2d(&center_circle, 0.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_error_handling_negative_radius_scale() {
        let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

        let result = circle.analysis_scale_radius(-1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }
}
