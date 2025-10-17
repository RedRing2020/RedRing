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
}
