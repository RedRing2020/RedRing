use crate::linalg::vector::Vector3;

#[cfg(test)]
mod tests {
    use super::*;
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

        // エイリアスのf32型でも動作することを確認
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
