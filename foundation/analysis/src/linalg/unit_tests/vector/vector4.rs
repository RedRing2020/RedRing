use crate::linalg::vector::Vector4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector4_creation() {
        let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
        assert_eq!(v.w(), 4.0);
    }

    #[test]
    fn test_vector4_homogeneous_conversion() {
        let point = Vector4::from_point(2.0, 4.0, 6.0);
        let euclidean = point.to_euclidean().unwrap();

        assert_eq!(euclidean.x(), 2.0);
        assert_eq!(euclidean.y(), 4.0);
        assert_eq!(euclidean.z(), 6.0);
    }

    #[test]
    fn test_vector4_perspective_division() {
        let v = Vector4::new(4.0, 8.0, 12.0, 2.0);
        let euclidean = v.to_euclidean().unwrap();

        assert_eq!(euclidean.x(), 2.0);
        assert_eq!(euclidean.y(), 4.0);
        assert_eq!(euclidean.z(), 6.0);
    }

    #[test]
    fn test_vector4_point_direction_check() {
        let point = Vector4::from_point(1.0, 2.0, 3.0);
        let direction = Vector4::from_direction(1.0, 2.0, 3.0);

        assert!(point.is_point());
        assert!(!point.is_direction());

        assert!(!direction.is_point());
        assert!(direction.is_direction());
    }

    #[test]
    fn test_vector4_arithmetic() {
        let v1 = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let v2 = Vector4::new(5.0, 6.0, 7.0, 8.0);

        let sum = v1 + v2;
        assert_eq!(sum, Vector4::new(6.0, 8.0, 10.0, 12.0));

        let diff = v2 - v1;
        assert_eq!(diff, Vector4::new(4.0, 4.0, 4.0, 4.0));

        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector4::new(2.0, 4.0, 6.0, 8.0));
    }

    #[test]
    fn test_vector4_dot_product() {
        let v1 = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let v2 = Vector4::new(5.0, 6.0, 7.0, 8.0);

        let dot = v1.dot(&v2);
        assert_eq!(dot, 70.0); // 1*5 + 2*6 + 3*7 + 4*8 = 70
    }

    #[test]
    fn test_vector4_axis_constants() {
        // 定数テスト（f64型）
        let x_axis = Vector4::<f64>::X_AXIS;
        let y_axis = Vector4::<f64>::Y_AXIS;
        let z_axis = Vector4::<f64>::Z_AXIS;
        let w_axis = Vector4::<f64>::W_AXIS;
        let zero = Vector4::<f64>::ZERO;
        assert_eq!(x_axis.x(), 1.0);
        assert_eq!(x_axis.y(), 0.0);
        assert_eq!(x_axis.z(), 0.0);
        assert_eq!(x_axis.w(), 0.0);
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);
        assert_eq!(y_axis.z(), 0.0);
        assert_eq!(y_axis.w(), 0.0);
        assert_eq!(z_axis.x(), 0.0);
        assert_eq!(z_axis.y(), 0.0);
        assert_eq!(z_axis.z(), 1.0);
        assert_eq!(z_axis.w(), 0.0);
        assert_eq!(w_axis.x(), 0.0);
        assert_eq!(w_axis.y(), 0.0);
        assert_eq!(w_axis.z(), 0.0);
        assert_eq!(w_axis.w(), 1.0);
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);
        assert_eq!(zero.z(), 0.0);
        assert_eq!(zero.w(), 0.0);

        // メソッドテスト（エイリアス機能）
        let x_axis = Vector4::<f64>::x_axis();
        let y_axis = Vector4::<f64>::y_axis();
        let z_axis = Vector4::<f64>::z_axis();
        let w_axis = Vector4::<f64>::w_axis();
        let zero = Vector4::<f64>::zero();
        assert_eq!(x_axis.x(), 1.0);
        assert_eq!(x_axis.y(), 0.0);
        assert_eq!(x_axis.z(), 0.0);
        assert_eq!(x_axis.w(), 0.0);
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);
        assert_eq!(y_axis.z(), 0.0);
        assert_eq!(y_axis.w(), 0.0);
        assert_eq!(z_axis.x(), 0.0);
        assert_eq!(z_axis.y(), 0.0);
        assert_eq!(z_axis.z(), 1.0);
        assert_eq!(z_axis.w(), 0.0);
        assert_eq!(w_axis.x(), 0.0);
        assert_eq!(w_axis.y(), 0.0);
        assert_eq!(w_axis.z(), 0.0);
        assert_eq!(w_axis.w(), 1.0);
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);
        assert_eq!(zero.z(), 0.0);
        assert_eq!(zero.w(), 0.0);

        // エイリアスのf32型でも動作することを確認
        let x_axis_f32 = Vector4::<f32>::x_axis();
        let zero_f32 = Vector4::<f32>::zero();
        assert_eq!(x_axis_f32.x(), 1.0f32);
        assert_eq!(x_axis_f32.y(), 0.0f32);
        assert_eq!(x_axis_f32.z(), 0.0f32);
        assert_eq!(x_axis_f32.w(), 0.0f32);
        assert_eq!(zero_f32.x(), 0.0f32);
        assert_eq!(zero_f32.y(), 0.0f32);
        assert_eq!(zero_f32.z(), 0.0f32);
        assert_eq!(zero_f32.w(), 0.0f32);
    }
}
