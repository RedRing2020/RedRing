//! Vector3D のテスト

use crate::{Point3D, Vector3D};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3d_creation() {
        let vec = Vector3D::new(1.0, 2.0, 3.0);
        assert_eq!(vec.x(), 1.0);
        assert_eq!(vec.y(), 2.0);
        assert_eq!(vec.z(), 3.0);
    }

    #[test]
    fn test_vector3d_zero() {
        let zero = Vector3D::<f64>::zero();
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);
        assert_eq!(zero.z(), 0.0);
        assert!(zero.is_zero());
    }

    #[test]
    fn test_vector3d_unit_vectors() {
        let unit_x = Vector3D::<f64>::unit_x();
        let unit_y = Vector3D::<f64>::unit_y();
        let unit_z = Vector3D::<f64>::unit_z();

        assert_eq!(unit_x.components(), [1.0, 0.0, 0.0]);
        assert_eq!(unit_y.components(), [0.0, 1.0, 0.0]);
        assert_eq!(unit_z.components(), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_vector3d_length() {
        let vec = Vector3D::new(3.0, 4.0, 0.0);
        assert_eq!(vec.length(), 5.0); // 3-4-5直角三角形
    }

    #[test]
    fn test_vector3d_normalize() {
        let vec = Vector3D::new(3.0_f64, 4.0, 0.0);
        let normalized = vec.normalize();
        assert!((normalized.length() - 1.0).abs() < 1e-10);
        assert!((normalized.x() - 0.6).abs() < 1e-10);
        assert!((normalized.y() - 0.8).abs() < 1e-10);
        assert!((normalized.z() - 0.0).abs() < 1e-10);

        // ゼロベクトルの正規化はゼロベクトルを返す
        let zero = Vector3D::<f64>::zero();
        let normalized_zero = zero.normalize();
        assert!(normalized_zero.length() <= f64::EPSILON);
    }

    #[test]
    fn test_vector3d_operations() {
        let v1 = Vector3D::new(1.0, 2.0, 3.0);
        let v2 = Vector3D::new(4.0, 5.0, 6.0);

        // 内積
        assert_eq!(v1.dot(&v2), 32.0); // 1*4 + 2*5 + 3*6 = 32

        // 外積
        let cross = v1.cross(&v2);
        assert_eq!(cross.x(), -3.0); // 2*6 - 3*5 = -3
        assert_eq!(cross.y(), 6.0); // 3*4 - 1*6 = 6
        assert_eq!(cross.z(), -3.0); // 1*5 - 2*4 = -3

        // 反転
        let negated = v1.negate();
        assert_eq!(negated.components(), [-1.0, -2.0, -3.0]);
    }

    #[test]
    fn test_vector3d_from_points() {
        let p1 = Point3D::new(1.0, 2.0, 3.0);
        let p2 = Point3D::new(4.0, 6.0, 9.0);
        let vec = Vector3D::from_points(&p1, &p2);
        assert_eq!(vec.components(), [3.0, 4.0, 6.0]);
    }

    #[test]
    fn test_vector3d_relationships() {
        let v1 = Vector3D::new(2.0, 0.0, 0.0);
        let v2 = Vector3D::new(4.0, 0.0, 0.0);
        let v3 = Vector3D::new(0.0, 3.0, 0.0);

        // 平行
        assert!(v1.is_parallel(&v2));
        assert!(!v1.is_parallel(&v3));

        // 垂直
        assert!(v1.is_perpendicular(&v3));
        assert!(!v1.is_perpendicular(&v2));
    }

    // === foundation トレイトテスト ===

    #[test]
    fn test_geometry_foundation() {
        let vec = Vector3D::new(3.0, 4.0, 5.0);
        let bbox = vec.bounding_box();

        // ベクトルの境界ボックスは原点と終点を含む
        assert_eq!(bbox.min(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 5.0));
    }

    #[test]
    fn test_basic_metrics() {
        let vec = Vector3D::new(3.0_f64, 4.0, 0.0);
        // 長さの計算
        assert_eq!(vec.length(), 5.0);
        assert_eq!(vec.length_squared(), 25.0);
    }

    #[test]
    fn test_basic_operations() {
        let vec = Vector3D::new(3.0_f64, 4.0, 0.0);
        let normalized = vec.normalize();
        assert!((normalized.length() - 1.0).abs() < 1e-10);

        let negated = -vec;
        assert_eq!(negated.components(), [-3.0, -4.0, 0.0]);
    }

    #[test]
    fn test_vector3d_f32_compatibility() {
        let vec_f32 = Vector3D::new(1.0f32, 2.0f32, 3.0f32);
        let vec_f64 = Vector3D::new(1.0f64, 2.0f64, 3.0f64);

        assert_eq!(vec_f32.length(), 3.7416575f32);
        assert_eq!(vec_f64.length(), 3.7416573867739413f64);
    }

    // ============================================================================
    // Transform テスト (vector_3d_transform_tests.rs から統合)
    // ============================================================================

    #[test]
    fn test_rotate_z() {
        use std::f64::consts::PI;
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let rotated = v.rotate_z(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_x() {
        use std::f64::consts::PI;
        let v = Vector3D::new(0.0, 1.0, 0.0);
        let rotated = v.rotate_x(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_y() {
        use std::f64::consts::PI;
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let rotated = v.rotate_y(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 0.0).abs() < 1e-10);
        assert!((rotated.z() + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_basic_transform_translate() {
        use geo_foundation::extensions::BasicTransform;
        let v1 = Vector3D::new(1.0, 2.0, 3.0);
        let v2 = Vector3D::new(0.5, 1.0, 1.5);
        let translated = BasicTransform::translate(&v1, v2);

        assert_eq!(translated.x(), 1.5);
        assert_eq!(translated.y(), 3.0);
        assert_eq!(translated.z(), 4.5);
    }

    #[test]
    fn test_basic_transform_rotate() {
        use geo_foundation::{extensions::BasicTransform, Angle};
        use std::f64::consts::PI;

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
        use geo_foundation::extensions::BasicTransform;

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
        use std::f64::consts::PI;

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
        use std::f64::consts::PI;

        let v = Vector3D::new(1.0, 2.0, 3.0);

        // 2π回転は元のベクトルと同じ
        let rotated = v.rotate_z(2.0 * PI);

        assert!((rotated.x() - v.x()).abs() < 1e-10);
        assert!((rotated.y() - v.y()).abs() < 1e-10);
        assert!((rotated.z() - v.z()).abs() < 1e-10);
    }
}
