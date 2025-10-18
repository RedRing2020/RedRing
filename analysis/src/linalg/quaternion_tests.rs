use crate::linalg::quaternion::Quaternion;
use crate::linalg::vector::Vector3;
use std::f64::consts::PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_basic_creation() {
        let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.x(), 1.0);
        assert_eq!(q.y(), 2.0);
        assert_eq!(q.z(), 3.0);
        assert_eq!(q.w(), 4.0);
    }

    #[test]
    fn test_quaternion_constants() {
        let identity = Quaternion::<f64>::identity();
        assert_eq!(identity.x(), 0.0);
        assert_eq!(identity.y(), 0.0);
        assert_eq!(identity.z(), 0.0);
        assert_eq!(identity.w(), 1.0);
        assert!(identity.is_unit());

        let zero = Quaternion::<f64>::zero();
        assert!(zero.is_zero());
    }

    #[test]
    fn test_quaternion_norm_and_normalize() {
        let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
        let expected_norm = (1.0 + 4.0 + 9.0 + 16.0_f64).sqrt();
        assert!((q.norm() - expected_norm).abs() < 1e-10_f64);

        let normalized = q.normalize().unwrap();
        assert!((normalized.norm() - 1.0).abs() < 1e-10_f64);
    }

    #[test]
    fn test_quaternion_axis_angle_conversion() {
        // Z軸周りの90度回転
        let axis = Vector3::<f64>::new(0.0, 0.0, 1.0);
        let angle = PI / 2.0;
        let q = Quaternion::from_axis_angle(&axis, angle);

        // 軸角表現に戻す
        let (recovered_axis, recovered_angle) = q.to_axis_angle().unwrap();

        assert!((recovered_angle - angle).abs() < 1e-10_f64);
        assert!((recovered_axis.x() - axis.x()).abs() < 1e-10_f64);
        assert!((recovered_axis.y() - axis.y()).abs() < 1e-10_f64);
        assert!((recovered_axis.z() - axis.z()).abs() < 1e-10_f64);
    }

    #[test]
    fn test_quaternion_multiplication() {
        let q1 = Quaternion::<f64>::new(1.0, 0.0, 0.0, 0.0);
        let q2 = Quaternion::<f64>::new(0.0, 1.0, 0.0, 0.0);
        let result = q1 * q2;

        // 虚数単位の積: i * j = k
        assert_eq!(result.x(), 0.0);
        assert_eq!(result.y(), 0.0);
        assert_eq!(result.z(), 1.0);
        assert_eq!(result.w(), 0.0);
    }

    #[test]
    fn test_quaternion_rotation() {
        // X軸周りの90度回転のクォータニオン
        let q = Quaternion::from_axis_angle(&Vector3::<f64>::new(1.0, 0.0, 0.0), PI / 2.0);

        // Y軸の点を回転
        let point = Vector3::<f64>::new(0.0, 1.0, 0.0);
        let rotated = q.rotate_vector(&point);

        // Y軸がZ軸になることを確認
        assert!(rotated.x().abs() < 1e-10);
        assert!(rotated.y().abs() < 1e-10);
        assert!((rotated.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_quaternion_conjugate() {
        let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
        let conjugate = q.conjugate();

        assert_eq!(conjugate.x(), -1.0);
        assert_eq!(conjugate.y(), -2.0);
        assert_eq!(conjugate.z(), -3.0);
        assert_eq!(conjugate.w(), 4.0);
    }

    #[test]
    fn test_quaternion_inverse() {
        let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
        let inverse = q.inverse().unwrap();
        let product = q * inverse;

        // q * q^-1 = identity
        assert!((product.x() - 0.0).abs() < 1e-10);
        assert!((product.y() - 0.0).abs() < 1e-10);
        assert!((product.z() - 0.0).abs() < 1e-10);
        assert!((product.w() - 1.0).abs() < 1e-10);
    }
}
