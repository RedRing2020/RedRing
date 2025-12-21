//! Vector3D Analysis Transform実装
//!
//! Analysis Matrix4x4を使用した効率的なVector3D変換
//! Point3D Transform実装パターンに準拠

use crate::Vector3D;
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{Angle, Scalar, TransformError};

/// Vector3D用Analysis Matrix4x4変換トレイト
pub trait AnalysisTransformVector3D<T: Scalar> {
    /// Matrix4x4でベクトルを変換（方向ベクトルとして、平行移動は無視）
    fn transform_vector_matrix(&self, matrix: &Matrix4x4<T>) -> Self;

    /// X軸周りの回転
    fn rotate_x_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// Y軸周りの回転
    fn rotate_y_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// Z軸周りの回転
    fn rotate_z_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 任意軸周りの回転
    fn rotate_analysis(&self, axis: &Vector3<T>, angle: Angle<T>) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// スケール変換（非均等）
    fn scale_analysis(&self, scale_x: T, scale_y: T, scale_z: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 均等スケール変換
    fn uniform_scale_analysis(&self, scale_factor: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 複合変換（回転→スケール）
    fn apply_composite_transform(
        &self,
        rotation_axis: Option<&Vector3<T>>,
        rotation_angle: Option<Angle<T>>,
        scale_x: Option<T>,
        scale_y: Option<T>,
        scale_z: Option<T>,
    ) -> Result<Self, TransformError>
    where
        Self: Sized;
}

// ============================================================================
// Analysis Vector3 型変換
// ============================================================================

impl<T: Scalar> From<Vector3D<T>> for Vector3<T> {
    fn from(vector: Vector3D<T>) -> Self {
        Vector3::new(vector.x(), vector.y(), vector.z())
    }
}

impl<T: Scalar> From<Vector3<T>> for Vector3D<T> {
    fn from(vector: Vector3<T>) -> Self {
        Vector3D::new(vector.x(), vector.y(), vector.z())
    }
}

// ============================================================================
// AnalysisTransformVector3D Implementation
// ============================================================================

impl<T: Scalar> AnalysisTransformVector3D<T> for Vector3D<T> {
    fn transform_vector_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        let vec: Vector3<T> = (*self).into();
        let transformed = matrix.transform_vector_3d(&vec);
        transformed.into()
    }

    fn rotate_x_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        let matrix = Matrix4x4::rotation_x_3d(angle.to_radians());
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn rotate_y_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        let matrix = Matrix4x4::rotation_y_3d(angle.to_radians());
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn rotate_z_analysis(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        let matrix = Matrix4x4::rotation_z_3d(angle.to_radians());
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn rotate_analysis(&self, axis: &Vector3<T>, angle: Angle<T>) -> Result<Self, TransformError> {
        let axis_length = (axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z()).sqrt();
        if axis_length <= T::EPSILON {
            return Err(TransformError::ZeroVector(
                "Rotation axis must be non-zero".to_string(),
            ));
        }

        let normalized_axis = Vector3::new(
            axis.x() / axis_length,
            axis.y() / axis_length,
            axis.z() / axis_length,
        );

        let matrix = Matrix4x4::rotation_axis_3d(normalized_axis, angle.to_radians());
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn scale_analysis(&self, scale_x: T, scale_y: T, scale_z: T) -> Result<Self, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let scale_vec = Vector3::new(scale_x, scale_y, scale_z);
        let matrix = Matrix4x4::scale_3d(&scale_vec);
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn uniform_scale_analysis(&self, scale_factor: T) -> Result<Self, TransformError> {
        if scale_factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Uniform scale factor cannot be zero".to_string(),
            ));
        }

        let matrix = Matrix4x4::uniform_scale_3d(scale_factor);
        Ok(self.transform_vector_matrix(&matrix))
    }

    fn apply_composite_transform(
        &self,
        rotation_axis: Option<&Vector3<T>>,
        rotation_angle: Option<Angle<T>>,
        scale_x: Option<T>,
        scale_y: Option<T>,
        scale_z: Option<T>,
    ) -> Result<Self, TransformError> {
        let mut composite = Matrix4x4::identity();

        // スケール変換（先に適用）
        if let (Some(sx), Some(sy), Some(sz)) = (scale_x, scale_y, scale_z) {
            if sx.is_zero() || sy.is_zero() || sz.is_zero() {
                return Err(TransformError::InvalidScaleFactor(
                    "Scale factors cannot be zero".to_string(),
                ));
            }
            let scale_vec = Vector3::new(sx, sy, sz);
            let scale_matrix = Matrix4x4::scale_3d(&scale_vec);
            composite = scale_matrix * composite;
        }

        // 回転変換（後に適用）
        if let (Some(axis), Some(angle)) = (rotation_axis, rotation_angle) {
            let axis_length =
                (axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z()).sqrt();
            if axis_length <= T::EPSILON {
                return Err(TransformError::ZeroVector(
                    "Rotation axis must be non-zero".to_string(),
                ));
            }

            let normalized_axis = Vector3::new(
                axis.x() / axis_length,
                axis.y() / axis_length,
                axis.z() / axis_length,
            );

            let rotation_matrix = Matrix4x4::rotation_axis_3d(normalized_axis, angle.to_radians());
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
    fn test_rotate_x_analysis() {
        let v = Vector3D::new(0.0, 1.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0); // 90度

        let rotated = v.rotate_x_analysis(angle).unwrap();

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_y_analysis() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0);

        let rotated = v.rotate_y_analysis(angle).unwrap();

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_z_analysis() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0);

        let rotated = v.rotate_z_analysis(angle).unwrap();

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_analysis() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        let scaled = v.scale_analysis(2.0, 3.0, 4.0).unwrap();

        assert_eq!(scaled.x(), 2.0);
        assert_eq!(scaled.y(), 6.0);
        assert_eq!(scaled.z(), 12.0);
    }

    #[test]
    fn test_uniform_scale_analysis() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        let scaled = v.uniform_scale_analysis(2.0).unwrap();

        assert_eq!(scaled.x(), 2.0);
        assert_eq!(scaled.y(), 4.0);
        assert_eq!(scaled.z(), 6.0);
    }

    #[test]
    fn test_composite_transform() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_radians(PI / 2.0);

        let transformed = v
            .apply_composite_transform(Some(&axis), Some(angle), Some(2.0), Some(2.0), Some(2.0))
            .unwrap();

        assert!((transformed.x() - 0.0).abs() < 1e-10);
        assert!((transformed.y() - 2.0).abs() < 1e-10);
        assert!((transformed.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling() {
        let v = Vector3D::new(1.0, 2.0, 3.0);

        // ゼロスケール
        assert!(v.scale_analysis(0.0, 1.0, 1.0).is_err());
        assert!(v.uniform_scale_analysis(0.0).is_err());

        // ゼロ軸
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0);
        assert!(v.rotate_analysis(&zero_axis, angle).is_err());
    }
}
