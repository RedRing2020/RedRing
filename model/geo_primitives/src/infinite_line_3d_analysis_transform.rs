//! InfiniteLine3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D無限直線変換
//! Point3D/LineSegment3D Analysis Transform パターンを基盤とする統一実装
//! 3D無限直線の特性（点と方向による表現）を考慮したMatrix変換

use crate::{InfiniteLine3D, Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// InfiniteLine3D用Analysis Matrix4x4変換モジュール
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

    /// 無限直線の行列変換（Matrix4x4）
    ///
    /// 直線上の点と方向ベクトルをMatrix変換し、新しい無限直線を構築
    pub fn transform_infinite_line_3d<T: Scalar>(
        infinite_line: &InfiniteLine3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<InfiniteLine3D<T>, TransformError> {
        // 直線上の点を変換
        let point_vec = point_to_analysis_vector(infinite_line.point());
        let transformed_point_vec = matrix.transform_point_3d(&point_vec);
        let new_point = analysis_vector_to_point(transformed_point_vec);

        // 方向ベクトルを変換（平行移動成分を除去するため方向ベクトル専用変換）
        let direction_vec = vector_to_analysis_vector(*infinite_line.direction());
        let transformed_direction_vec = matrix.transform_vector_3d(&direction_vec);
        let new_direction_vector = analysis_vector_to_vector(transformed_direction_vec);

        // 変換後の無限直線を構築
        InfiniteLine3D::new(new_point, new_direction_vector).ok_or_else(|| {
            TransformError::InvalidGeometry("Transformed direction vector is zero".to_string())
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
        // Analysis Matrix4x4にはrotation_around_axis_and_pointがないので、手動で計算
        let translation_to_origin =
            Matrix4x4::translation(-center_vec.x(), -center_vec.y(), -center_vec.z());
        let rotation = Matrix4x4::rotation_axis_3d(normalized_axis, angle.to_radians());
        let translation_back =
            Matrix4x4::translation(center_vec.x(), center_vec.y(), center_vec.z());
        Ok(translation_back * rotation * translation_to_origin)
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
        // Analysis Matrix4x4にはscale_around_pointがないので、手動で計算
        let translation_to_origin =
            Matrix4x4::translation(-center_vec.x(), -center_vec.y(), -center_vec.z());
        let scale_vec = Vector3::new(scale_x, scale_y, scale_z);
        let scale = Matrix4x4::scale_3d(&scale_vec);
        let translation_back =
            Matrix4x4::translation(center_vec.x(), center_vec.y(), center_vec.z());
        Ok(translation_back * scale * translation_to_origin)
    }

    /// 均等スケール行列を生成（中心点指定）
    pub fn uniform_scale_matrix<T: Scalar>(
        center: &Point3D<T>,
        scale_factor: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        scale_matrix(center, scale_factor, scale_factor, scale_factor)
    }
}

// ============================================================================
// AnalysisTransform3D Trait Implementation for InfiniteLine3D
// ============================================================================

/// InfiniteLine3DでのAnalysisTransform3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform3D<T> for InfiniteLine3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = InfiniteLine3D<T>;

    /// Matrix4x4による汎用変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        // Analysis Transformではエラー処理をデフォルト値で対応
        analysis_transform::transform_infinite_line_3d(self, matrix)
            .unwrap_or_else(|_| InfiniteLine3D::default())
    }

    /// Analysis統合平行移動
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::translation_matrix(translation);
        analysis_transform::transform_infinite_line_3d(self, &matrix)
    }

    /// Analysis統合軸回転（中心点指定）
    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.point();
        let matrix = analysis_transform::rotation_matrix(&center_point, axis, angle)?;
        analysis_transform::transform_infinite_line_3d(self, &matrix)
    }

    /// Analysis統合スケール（中心点指定）
    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.point();
        let matrix = analysis_transform::scale_matrix(&center_point, scale_x, scale_y, scale_z)?;
        analysis_transform::transform_infinite_line_3d(self, &matrix)
    }

    /// Analysis統合均等スケール（中心点指定）
    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.point();
        let matrix = analysis_transform::uniform_scale_matrix(&center_point, scale_factor)?;
        analysis_transform::transform_infinite_line_3d(self, &matrix)
    }

    /// 複合変換適用（平行移動・回転・スケールの組み合わせ）
    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, TransformError> {
        let mut result = self.clone();

        // 平行移動を適用
        if let Some(t) = translation {
            result = result.translate_analysis(t)?;
        }

        // 回転を適用
        if let Some((center, axis, angle)) = rotation {
            result = result.rotate_analysis(center, axis, angle)?;
        }

        // スケールを適用
        if let Some((sx, sy, sz)) = scale {
            let center = result; // 自己中心スケール
            result = center.scale_analysis(&center, sx, sy, sz)?;
        }

        Ok(result)
    }

    /// 複合変換適用（均等スケール版）
    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, TransformError> {
        let mut result = self.clone();

        // 平行移動を適用
        if let Some(t) = translation {
            result = result.translate_analysis(t)?;
        }

        // 回転を適用
        if let Some((center, axis, angle)) = rotation {
            result = result.rotate_analysis(center, axis, angle)?;
        }

        // 均等スケールを適用
        if let Some(scale_factor) = scale {
            let center = result.clone(); // 自己中心スケール
            result = result.uniform_scale_analysis(&center, scale_factor)?;
        }

        Ok(result)
    }
}

// ============================================================================
// Default Implementation for InfiniteLine3D
// ============================================================================

impl<T: Scalar> Default for InfiniteLine3D<T> {
    fn default() -> Self {
        // デフォルト無限直線: X軸方向の原点を通る直線
        InfiniteLine3D::new(Point3D::origin(), Vector3D::new(T::ONE, T::ZERO, T::ZERO))
            .expect("Default InfiniteLine3D construction should not fail")
    }
}

// ============================================================================
// Analysis Transform Support Marker
// ============================================================================

impl<T: Scalar> geo_foundation::AnalysisTransformSupport for InfiniteLine3D<T> {
    const HAS_ANALYSIS_INTEGRATION: bool = true;
    const PERFORMANCE_OPTIMIZED: bool = true;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    const TOLERANCE: f64 = 1e-10;

    #[test]
    fn test_default_infinite_line_3d() {
        let line = InfiniteLine3D::<f64>::default();
        assert_eq!(line.point(), Point3D::origin());
        assert_eq!(line.direction().x(), 1.0);
        assert_eq!(line.direction().y(), 0.0);
        assert_eq!(line.direction().z(), 0.0);
    }

    #[test]
    fn test_analysis_transform_translate() {
        let line =
            InfiniteLine3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let translation = Vector3::new(1.0, 2.0, 3.0);

        let result = line.translate_analysis(&translation).unwrap();

        assert_eq!(result.point().x(), 1.0);
        assert_eq!(result.point().y(), 2.0);
        assert_eq!(result.point().z(), 3.0);
        // 方向ベクトルは変わらない
        assert_eq!(result.direction().x(), 1.0);
        assert_eq!(result.direction().y(), 0.0);
        assert_eq!(result.direction().z(), 0.0);
    }

    #[test]
    fn test_analysis_transform_rotate() {
        let line =
            InfiniteLine3D::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center_line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸回転
        let angle = Angle::from_degrees(90.0);

        let result = line.rotate_analysis(&center_line, &axis, angle).unwrap();

        // 90度Z軸回転後、点 (1,0,0) は (0,1,0) になる
        assert!((result.point().x() - 0.0).abs() < TOLERANCE);
        assert!((result.point().y() - 1.0).abs() < TOLERANCE);
        assert!((result.point().z() - 0.0).abs() < TOLERANCE);
        // 方向ベクトル (1,0,0) は (0,1,0) になる
        assert!((result.direction().x() - 0.0).abs() < TOLERANCE);
        assert!((result.direction().y() - 1.0).abs() < TOLERANCE);
        assert!((result.direction().z() - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_analysis_transform_scale() {
        let line =
            InfiniteLine3D::new(Point3D::new(2.0, 1.0, 1.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center_line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = line.scale_analysis(&center_line, 2.0, 3.0, 4.0).unwrap();

        // 点 (2,1,1) が (4,3,4) になる
        assert_eq!(result.point().x(), 4.0);
        assert_eq!(result.point().y(), 3.0);
        assert_eq!(result.point().z(), 4.0);
        // 方向ベクトル (1,0,0) が (2,0,0) になるが正規化されて (1,0,0)
        assert_eq!(result.direction().x(), 1.0);
        assert_eq!(result.direction().y(), 0.0);
        assert_eq!(result.direction().z(), 0.0);
    }

    #[test]
    fn test_analysis_transform_uniform_scale() {
        let line =
            InfiniteLine3D::new(Point3D::new(1.0, 1.0, 1.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center_line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = line.uniform_scale_analysis(&center_line, 2.0).unwrap();

        // 点 (1,1,1) が (2,2,2) になる
        assert_eq!(result.point().x(), 2.0);
        assert_eq!(result.point().y(), 2.0);
        assert_eq!(result.point().z(), 2.0);
        // 方向ベクトルは変わらない（均等スケール）
        assert_eq!(result.direction().x(), 1.0);
        assert_eq!(result.direction().y(), 0.0);
        assert_eq!(result.direction().z(), 0.0);
    }

    #[test]
    fn test_composite_transform() {
        let line =
            InfiniteLine3D::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center_line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let translation = Vector3::new(1.0, 1.0, 1.0);
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let result = line
            .apply_composite_transform_uniform(
                Some(&translation),
                Some((&center_line, &axis, angle)),
                Some(2.0),
            )
            .unwrap();

        // 複合変換の結果を検証（詳細は実装依存）
        assert!(result.point().x().is_finite());
        assert!(result.point().y().is_finite());
        assert!(result.point().z().is_finite());
    }

    #[test]
    fn test_transform_zero_scale_error() {
        let line =
            InfiniteLine3D::new(Point3D::new(1.0, 1.0, 1.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center_line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = line.scale_analysis(&center_line, 0.0, 1.0, 1.0);
        assert!(result.is_err());

        let result = line.uniform_scale_analysis(&center_line, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_zero_axis_error() {
        let line =
            InfiniteLine3D::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let center_line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = line.rotate_analysis(&center_line, &zero_axis, angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_point_matrix() {
        let line =
            InfiniteLine3D::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let matrix = Matrix4x4::translation(2.0, 3.0, 4.0);

        let result = line.transform_point_matrix(&matrix);

        assert_eq!(result.point().x(), 3.0);
        assert_eq!(result.point().y(), 3.0);
        assert_eq!(result.point().z(), 4.0);
        assert_eq!(result.direction().x(), 1.0);
        assert_eq!(result.direction().y(), 0.0);
        assert_eq!(result.direction().z(), 0.0);
    }
}
