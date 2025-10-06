use crate::linalg::vector::Vector3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_creation() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn test_vector3_cross_product() {
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 1.0, 0.0);
        let cross = v1.cross(&v2);

        assert_eq!(cross, Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_vector3_norm() {
        let v = Vector3::new(1.0, 2.0, 2.0);
        assert_eq!(v.norm(), 3.0); // sqrt(1 + 4 + 4) = 3
        assert_eq!(v.norm_squared(), 9.0);
    }

    #[test]
    fn test_vector3_normalize() {
        let v = Vector3::<f64>::new(3.0, 4.0, 0.0);
        let normalized = v.normalize().unwrap();
        assert!((normalized.norm() - 1.0_f64).abs() < 1e-10);
        assert!((normalized.x() - 0.6_f64).abs() < 1e-10);
        assert!((normalized.y() - 0.8_f64).abs() < 1e-10);
        assert_eq!(normalized.z(), 0.0);
    }

    #[test]
    fn test_vector3_arithmetic() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);

        let sum = v1 + v2;
        assert_eq!(sum, Vector3::new(5.0, 7.0, 9.0));

        let diff = v2 - v1;
        assert_eq!(diff, Vector3::new(3.0, 3.0, 3.0));

        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_vector3_axis_constants() {
        // 定数テスト（f64型）
        let x_axis = Vector3::<f64>::X_AXIS;
        let y_axis = Vector3::<f64>::Y_AXIS;
        let z_axis = Vector3::<f64>::Z_AXIS;
        let zero = Vector3::<f64>::ZERO;
        assert_eq!(x_axis.x(), 1.0);
        assert_eq!(x_axis.y(), 0.0);
        assert_eq!(x_axis.z(), 0.0);
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);
        assert_eq!(y_axis.z(), 0.0);
        assert_eq!(z_axis.x(), 0.0);
        assert_eq!(z_axis.y(), 0.0);
        assert_eq!(z_axis.z(), 1.0);
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);
        assert_eq!(zero.z(), 0.0);

        // メソッドテスト（エイリアス機能）
        let x_axis = Vector3::<f64>::x_axis();
        let y_axis = Vector3::<f64>::y_axis();
        let z_axis = Vector3::<f64>::z_axis();
        let zero = Vector3::<f64>::zero();
        assert_eq!(x_axis.x(), 1.0);
        assert_eq!(x_axis.y(), 0.0);
        assert_eq!(x_axis.z(), 0.0);
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);
        assert_eq!(y_axis.z(), 0.0);
        assert_eq!(z_axis.x(), 0.0);
        assert_eq!(z_axis.y(), 0.0);
        assert_eq!(z_axis.z(), 1.0);
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);
        assert_eq!(zero.z(), 0.0);

        // エイリアスはf32型でも動作することを確認
        let x_axis_f32 = Vector3::<f32>::x_axis();
        let zero_f32 = Vector3::<f32>::zero();
        assert_eq!(x_axis_f32.x(), 1.0f32);
        assert_eq!(x_axis_f32.y(), 0.0f32);
        assert_eq!(x_axis_f32.z(), 0.0f32);
        assert_eq!(zero_f32.x(), 0.0f32);
        assert_eq!(zero_f32.y(), 0.0f32);
        assert_eq!(zero_f32.z(), 0.0f32);
    }
}
