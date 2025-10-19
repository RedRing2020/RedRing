//! Vector3D transform機能のテストモジュール

use crate::Vector3D;
use geo_foundation::{extensions::BasicTransform, Angle};

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_rotate_z() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let rotated = v.rotate_z(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_x() {
        let v = Vector3D::new(0.0, 1.0, 0.0);
        let rotated = v.rotate_x(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_y() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let rotated = v.rotate_y(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_basic_transform_translate() {
        let v1 = Vector3D::new(1.0, 2.0, 3.0);
        let v2 = Vector3D::new(0.5, 1.0, 1.5);
        let translated = BasicTransform::translate(&v1, v2);

        assert_eq!(translated.x(), 1.5);
        assert_eq!(translated.y(), 3.0);
        assert_eq!(translated.z(), 4.5);
    }

    #[test]
    fn test_basic_transform_rotate() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let center = crate::Point3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_radians(PI / 2.0);
        let rotated = BasicTransform::rotate(&v, center, angle);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_basic_transform_scale() {
        let v = Vector3D::new(2.0, 3.0, 4.0);
        let center = crate::Point3D::new(0.0, 0.0, 0.0);
        let scaled = BasicTransform::scale(&v, center, 2.0);

        assert_eq!(scaled.x(), 4.0);
        assert_eq!(scaled.y(), 6.0);
        assert_eq!(scaled.z(), 8.0);
    }

    #[test]
    fn test_transform_vector_trait() {
        // カスタム変換行列の例
        struct SimpleMatrix;

        impl crate::vector_3d_transform::TransformVector3D<f64> for SimpleMatrix {
            fn transform_vector_3d(&self, vector: &Vector3D<f64>) -> Vector3D<f64> {
                // 単純なスケール変換
                Vector3D::new(vector.x() * 2.0, vector.y() * 2.0, vector.z() * 2.0)
            }
        }

        let v = Vector3D::new(1.0, 2.0, 3.0);
        let matrix = SimpleMatrix;
        let transformed = v.transform_vector(&matrix);

        assert_eq!(transformed.x(), 2.0);
        assert_eq!(transformed.y(), 4.0);
        assert_eq!(transformed.z(), 6.0);
    }

    #[test]
    fn test_transform_point_trait() {
        // カスタム変換行列の例
        struct SimpleMatrix;

        impl crate::vector_3d_transform::TransformPoint3D<f64> for SimpleMatrix {
            fn transform_point_3d(&self, point: &crate::Point3D<f64>) -> crate::Point3D<f64> {
                // 単純な平行移動
                crate::Point3D::new(point.x() + 1.0, point.y() + 1.0, point.z() + 1.0)
            }
        }

        let v = Vector3D::new(1.0, 2.0, 3.0);
        let matrix = SimpleMatrix;
        let transformed = v.transform_point(&matrix);

        assert_eq!(transformed.x(), 2.0);
        assert_eq!(transformed.y(), 3.0);
        assert_eq!(transformed.z(), 4.0);
    }

    #[test]
    fn test_combined_rotations() {
        let v = Vector3D::new(1.0, 0.0, 0.0);

        // X軸周りの回転後、Z軸周りの回転
        let rotated = v.rotate_x(PI / 2.0).rotate_z(PI / 2.0);

        // X軸周りに90度回転: (1,0,0) → (1,0,0) (X軸ベクトルは変わらない)
        // その後Z軸周りに90度回転: (1,0,0) → (0,1,0)
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_identity() {
        let v = Vector3D::new(1.0, 2.0, 3.0);

        // 2π回転は元のベクトルと同じ
        let rotated = v.rotate_z(2.0 * PI);

        assert!((rotated.x() - v.x()).abs() < 1e-10);
        assert!((rotated.y() - v.y()).abs() < 1e-10);
        assert!((rotated.z() - v.z()).abs() < 1e-10);
    }
}
