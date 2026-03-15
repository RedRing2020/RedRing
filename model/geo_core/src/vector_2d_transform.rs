//! Vector2D Analysis Transform実装
//!
//! Analysis Matrix3x3を使用した効率的なVector2D変換
//! Point2D Transform実装パターンに準拠

use crate::Vector2D;
use crate::TransformError;
use analysis::abstract_types::{Angle, Scalar};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};

/// Vector2D用Analysis Matrix3x3変換トレイト
pub trait AnalysisTransformVector2D<T: Scalar> {
    /// Matrix3x3でベクトルを変換（方向ベクトルとして、平行移動は無視）
    fn transform_vector_matrix(&self, matrix: &Matrix3x3<T>) -> Self;

    /// 回転変換（2D平面内）
    fn rotate_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// スケール変換（非均等）
    fn scale_analysis(&self, scale_x: T, scale_y: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 均等スケール変換
    fn uniform_scale_analysis(&self, scale_factor: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 複合変換（回転→スケール）
    fn apply_composite_transform(
        &self,
        rotation: Option<Angle<T>>,
        scale_x: Option<T>,
        scale_y: Option<T>,
    ) -> Result<Self, TransformError>
    where
        Self: Sized;
}

// ============================================================================
// AnalysisTransformVector2D Implementation
// ============================================================================
//
// Note: Vector2D <-> Vector2 の From実装は point_2d_transform.rs に存在します

impl<T: Scalar> AnalysisTransformVector2D<T> for Vector2D<T> {
    fn transform_vector_matrix(&self, matrix: &Matrix3x3<T>) -> Self {
        let vec: Vector2<T> = (*self).into();
        let transformed = matrix.transform_vector_2d(&vec);
        transformed.into()
    }

    fn rotate_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        let matrix = Matrix3x3::rotation_2d(angle.to_radians());
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn scale_analysis(&self, scale_x: T, scale_y: T) -> Result<Self, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let scale_vec = Vector2::new(scale_x, scale_y);
        let matrix = Matrix3x3::scale_2d(&scale_vec);
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn uniform_scale_analysis(&self, scale_factor: T) -> Result<Self, TransformError> {
        if scale_factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Uniform scale factor cannot be zero".to_string(),
            ));
        }

        let matrix = Matrix3x3::uniform_scale_2d(scale_factor);
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn apply_composite_transform(
        &self,
        rotation: Option<Angle<T>>,
        scale_x: Option<T>,
        scale_y: Option<T>,
    ) -> Result<Self, TransformError> {
        let mut composite = Matrix3x3::identity();

        // スケール変換（先に適用）
        if let (Some(sx), Some(sy)) = (scale_x, scale_y) {
            if sx.is_zero() || sy.is_zero() {
                return Err(TransformError::InvalidScaleFactor(
                    "Scale factors cannot be zero".to_string(),
                ));
            }
            let scale_vec = Vector2::new(sx, sy);
            let scale_matrix = Matrix3x3::scale_2d(&scale_vec);
            composite = scale_matrix * composite;
        }

        // 回転変換（後に適用）
        if let Some(angle) = rotation {
            let rotation_matrix = Matrix3x3::rotation_2d(angle.to_radians());
            composite = rotation_matrix * composite;
        }

        Ok(self.transform_vector_matrix(&composite))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_analysis_rotation_2d() {
        let v = Vector2D::new(1.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0); // 90度

        let rotated = v.rotate_analysis(angle).unwrap();

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale_2d() {
        let v = Vector2D::new(2.0, 3.0);
        let scaled = v.scale_analysis(2.0, 3.0).unwrap();

        assert_eq!(scaled.x(), 4.0);
        assert_eq!(scaled.y(), 9.0);
    }

    #[test]
    fn test_uniform_scale_2d() {
        let v = Vector2D::new(1.0, 2.0);
        let scaled = v.uniform_scale_analysis(3.0).unwrap();

        assert_eq!(scaled.x(), 3.0);
        assert_eq!(scaled.y(), 6.0);
    }

    #[test]
    fn test_composite_transform_2d() {
        let v = Vector2D::new(1.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0);

        let transformed = v
            .apply_composite_transform(Some(angle), Some(2.0), Some(2.0))
            .unwrap();

        assert!((transformed.x() - 0.0).abs() < 1e-10);
        assert!((transformed.y() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling_2d() {
        let v = Vector2D::new(1.0, 2.0);

        // ゼロスケール
        assert!(v.scale_analysis(0.0, 1.0).is_err());
        assert!(v.uniform_scale_analysis(0.0).is_err());
    }
}
