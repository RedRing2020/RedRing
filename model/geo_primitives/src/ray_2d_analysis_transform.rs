//! Ray2D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な2D半無限直線変換
//! Point2D/InfiniteLine2D Analysis Transform パターンを基盤とする統一実装
//! 半無限直線の特性（起点と方向による表現）を考慮したMatrix変換

use crate::{Point2D, Ray2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// Ray2D用Analysis Matrix3x3変換モジュール
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

    /// 半無限直線の行列変換（Matrix3x3）
    ///
    /// 起点と方向ベクトルをMatrix変換し、新しい半無限直線を構築
    pub fn transform_ray_2d<T: Scalar>(
        ray: &Ray2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<Ray2D<T>, TransformError> {
        // 起点を変換
        let origin_vec = point_to_analysis_vector(ray.origin());
        let transformed_origin_vec = matrix.transform_point_2d(&origin_vec);
        let new_origin = analysis_vector_to_point(transformed_origin_vec);

        // 方向ベクトルを変換（平行移動成分を除去するため原点中心変換）
        let direction_vec =
            vector_to_analysis_vector(Vector2D::new(ray.direction().x(), ray.direction().y()));
        let transformed_direction_vec = matrix.transform_vector_2d(&direction_vec);
        let new_direction_vector = analysis_vector_to_vector(transformed_direction_vec);

        // 変換後の半無限直線を構築
        Ray2D::new(new_origin, new_direction_vector).ok_or_else(|| {
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
        let scale_vec = Vector2::new(scale_x, scale_y);
        let scale_matrix = Matrix3x3::scale_2d(&scale_vec);
        let translation_to_origin =
            Matrix3x3::translation_2d(&Vector2::new(-center_vec.x(), -center_vec.y()));
        let translation_back = Matrix3x3::translation_2d(&center_vec);
        Ok(translation_back * scale_matrix * translation_to_origin)
    }

    /// 複合変換パラメータ構造体（2D）
    pub struct CompositeTransform2D<T: Scalar> {
        pub translation: Vector2D<T>,
        pub rotation_center: Point2D<T>,
        pub rotation_angle: Angle<T>,
        pub scale_center: Point2D<T>,
        pub scale_x: T,
        pub scale_y: T,
    }

    /// 複合変換行列を生成（最も効率的な順序：Scale→Rotate→Translate）
    pub fn composite_transform_matrix_2d<T: Scalar>(
        params: &CompositeTransform2D<T>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let scale_matrix = scale_matrix_2d(&params.scale_center, params.scale_x, params.scale_y)?;
        let rotation_matrix = rotation_matrix_2d(&params.rotation_center, params.rotation_angle);
        let translation_matrix = translation_matrix_2d(&params.translation);

        Ok(translation_matrix * rotation_matrix * scale_matrix)
    }
}

/// Ray2D用AnalysisTransform2Dトレイト実装
impl<T: Scalar> AnalysisTransform2D<T> for Ray2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Ray2D<T>;

    /// Matrix3x3による直接変換
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        analysis_transform::transform_ray_2d(self, matrix)
            .expect("Ray transformation should be valid")
    }

    /// 平行移動
    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError> {
        // 高速化: 起点のみ平行移動、方向ベクトルは不変

        let new_origin = Point2D::new(
            self.origin().x() + translation.x(),
            self.origin().y() + translation.y(),
        );
        let direction_vec = Vector2D::new(self.direction().x(), self.direction().y());
        Ray2D::new(new_origin, direction_vec).ok_or_else(|| {
            TransformError::InvalidGeometry("Direction vector became zero".to_string())
        })
    }

    /// 中心点指定回転
    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::rotation_matrix_2d(&center.origin(), angle);
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    /// 中心点指定スケール変換
    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::scale_matrix_2d(&center.origin(), scale_x, scale_y)?;
        Ok(self.transform_point_matrix_2d(&matrix))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point2D, Vector2D};
    use analysis::linalg::vector::Vector2;
    use geo_foundation::Angle;

    fn create_test_ray() -> Ray2D<f64> {
        Ray2D::new(
            Point2D::new(1.0, 2.0),  // origin
            Vector2D::new(1.0, 0.0), // direction (unit x)
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let ray = create_test_ray();
        let translation = Vector2D::new(5.0, 3.0);

        let translation_vec = Vector2::new(translation.x(), translation.y());
        let result = ray.translate_analysis_2d(&translation_vec).unwrap();

        // 起点が移動することを確認
        let expected_origin = Point2D::new(6.0, 5.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);

        // 方向ベクトルは変わらない
        assert!((result.direction().x() - 1.0).abs() < f64::EPSILON);
        assert!((result.direction().y() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_rotation() {
        let ray = create_test_ray();
        let center = Point2D::new(0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let center_ray = Ray2D::new(center, Vector2D::new(1.0, 0.0)).unwrap();
        let result = ray.rotate_analysis_2d(&center_ray, angle).unwrap();

        // 90度回転後の起点確認
        let expected_origin = Point2D::new(-2.0, 1.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);

        // 90度回転後の方向ベクトル確認
        assert!((result.direction().x() - 0.0).abs() < f64::EPSILON);
        assert!((result.direction().y() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analysis_scale() {
        let ray = create_test_ray();
        let center = Point2D::new(0.0, 0.0);
        let scale_x = 2.0;
        let scale_y = 3.0;

        let center_ray = Ray2D::new(center, Vector2D::new(1.0, 0.0)).unwrap();
        let result = ray
            .scale_analysis_2d(&center_ray, scale_x, scale_y)
            .unwrap();

        // スケール変換後の起点確認
        let expected_origin = Point2D::new(2.0, 6.0);
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);

        // 方向ベクトルもスケール変換される（正規化前）
        let expected_dir_magnitude = (scale_x * scale_x).sqrt();
        // Direction2Dは常に正規化されている（norm = 1.0）
        assert!((result.direction().x() - 1.0).abs() < f64::EPSILON); // x方向は保持
        assert!((result.direction().y() - 0.0).abs() < f64::EPSILON); // y方向は0のまま
    }

    #[test]
    fn test_analysis_composite_transform() {
        let ray = create_test_ray();
        let translation = Vector2D::new(1.0, 1.0);
        let rotation_center = Point2D::new(0.0, 0.0);
        let rotation_angle = Angle::from_degrees(0.0); // 回転なし
        let scale_center = Point2D::new(0.0, 0.0);
        let scale_x = 2.0;
        let scale_y = 2.0;

        let _rotation_center_ray = Ray2D::new(rotation_center, Vector2D::new(1.0, 0.0)).unwrap();
        let params = analysis_transform::CompositeTransform2D {
            translation,
            rotation_center,
            rotation_angle,
            scale_center,
            scale_x,
            scale_y,
        };
        let matrix = analysis_transform::composite_transform_matrix_2d(&params).unwrap();
        let result = ray.transform_point_matrix_2d(&matrix);

        // 複合変換の結果を確認
        // Scale(2,2) -> Rotate(0) -> Translate(1,1)
        let expected_origin = Point2D::new(3.0, 5.0); // (1*2+1, 2*2+1)
        assert!((result.origin().x() - expected_origin.x()).abs() < f64::EPSILON);
        assert!((result.origin().y() - expected_origin.y()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_scale_error() {
        let ray = create_test_ray();
        let center = Point2D::new(0.0, 0.0);

        let center_ray = Ray2D::new(center, Vector2D::new(1.0, 0.0)).unwrap();
        let result = ray.scale_analysis_2d(&center_ray, 0.0, 1.0);
        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }
}
