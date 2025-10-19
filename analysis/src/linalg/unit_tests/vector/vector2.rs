use crate::linalg::vector::Vector2;
use std::f64::consts::PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector2_creation() {
        let v = Vector2::new(1.0, 2.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
    }

    #[test]
    fn test_vector2_operations() {
        let v1 = Vector2::new(1.0, 2.0);
        let v2 = Vector2::new(3.0, 4.0);

        let dot = v1.dot(&v2);
        assert_eq!(dot, 11.0); // 1*3 + 2*4 = 11

        let lerp = v1.lerp(&v2, 0.5);
        assert_eq!(lerp, Vector2::new(2.0, 3.0));
    }

    #[test]
    fn test_vector2_norm() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.norm(), 5.0); // 3-4-5 直角三角形
        assert_eq!(v.norm_squared(), 25.0);
    }

    #[test]
    fn test_vector2_rotation() {
        let v = Vector2::new(1.0, 0.0);
        let rotated = v.rotate(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector2_arithmetic() {
        let v1 = Vector2::new(1.0, 2.0);
        let v2 = Vector2::new(3.0, 4.0);

        let sum = v1 + v2;
        assert_eq!(sum, Vector2::new(4.0, 6.0));

        let diff = v2 - v1;
        assert_eq!(diff, Vector2::new(2.0, 2.0));

        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector2::new(2.0, 4.0));
    }

    #[test]
    fn test_vector2_axis_constants() {
        // 定数テスト（f64型）
        let x_axis = Vector2::<f64>::X_AXIS;
        let y_axis = Vector2::<f64>::Y_AXIS;
        let zero = Vector2::<f64>::ZERO;
        assert_eq!(x_axis.x(), 1.0);
        assert_eq!(x_axis.y(), 0.0);
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);

        // メソッドテスト（エイリアス機能）
        let x_axis = Vector2::<f64>::x_axis();
        let y_axis = Vector2::<f64>::y_axis();
        let zero = Vector2::<f64>::zero();
        assert_eq!(x_axis.x(), 1.0);
        assert_eq!(x_axis.y(), 0.0);
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);

        // エイリアスはf32型でも動作することを確認
        let x_axis_f32 = Vector2::<f32>::x_axis();
        let y_axis_f32 = Vector2::<f32>::y_axis();
        let zero_f32 = Vector2::<f32>::zero();
        assert_eq!(x_axis_f32.x(), 1.0f32);
        assert_eq!(x_axis_f32.y(), 0.0f32);
        assert_eq!(y_axis_f32.x(), 0.0f32);
        assert_eq!(y_axis_f32.y(), 1.0f32);
        assert_eq!(zero_f32.x(), 0.0f32);
        assert_eq!(zero_f32.y(), 0.0f32);
    }
}
