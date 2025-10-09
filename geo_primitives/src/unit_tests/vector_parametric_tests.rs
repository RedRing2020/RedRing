//! Vector の型パラメータ化機能のテスト
//! f32/f64 両方の動作を検証

#[cfg(test)]
pub mod tests {
    use crate::geometry2d::vector::{Vector, Vector2D, Vector2Df};
    use geo_foundation::Scalar;

    #[test]
    fn test_vector_f64_creation() {
        // f64版（デフォルト）
        let vec_f64: Vector<f64> = Vector::new(3.0, 4.0);
        assert_eq!(vec_f64.x(), 3.0f64);
        assert_eq!(vec_f64.y(), 4.0f64);

        // Vector2D型エイリアス使用
        let vec_alias: Vector2D = Vector::new(1.0, 2.0);
        assert_eq!(vec_alias.x(), 1.0f64);
        assert_eq!(vec_alias.y(), 2.0f64);
    }

    #[test]
    fn test_vector_f32_creation() {
        // f32版
        let vec_f32: Vector<f32> = Vector::new(3.0f32, 4.0f32);
        assert_eq!(vec_f32.x(), 3.0f32);
        assert_eq!(vec_f32.y(), 4.0f32);

        // Vector2Df型エイリアス使用
        let vec_alias: Vector2Df = Vector::new(1.0f32, 2.0f32);
        assert_eq!(vec_alias.x(), 1.0f32);
        assert_eq!(vec_alias.y(), 2.0f32);
    }

    #[test]
    fn test_vector_operations() {
        // f64でのベクトル演算
        let v1_f64: Vector<f64> = Vector::new(3.0, 4.0);
        let v2_f64: Vector<f64> = Vector::new(1.0, 2.0);

        let sum_f64 = v1_f64 + v2_f64;
        assert_eq!(sum_f64.x(), 4.0f64);
        assert_eq!(sum_f64.y(), 6.0f64);

        let dot_f64 = v1_f64.dot(&v2_f64);
        assert_eq!(dot_f64, 11.0f64); // 3*1 + 4*2 = 11

        // f32でのベクトル演算
        let v1_f32: Vector<f32> = Vector::new(3.0f32, 4.0f32);
        let v2_f32: Vector<f32> = Vector::new(1.0f32, 2.0f32);

        let sum_f32 = v1_f32 + v2_f32;
        assert_eq!(sum_f32.x(), 4.0f32);
        assert_eq!(sum_f32.y(), 6.0f32);

        let dot_f32 = v1_f32.dot(&v2_f32);
        assert_eq!(dot_f32, 11.0f32);
    }

    #[test]
    fn test_vector_length() {
        // f64でのベクトル長さ
        let v_f64: Vector<f64> = Vector::new(3.0, 4.0);
        let length_f64 = v_f64.length();
        assert!((length_f64 - 5.0).abs() < f64::EPSILON);

        // f32でのベクトル長さ
        let v_f32: Vector<f32> = Vector::new(3.0f32, 4.0f32);
        let length_f32 = v_f32.length();
        assert!((length_f32 - 5.0f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vector_normalize() {
        // f64でのベクトル正規化
        let v_f64: Vector<f64> = Vector::new(3.0, 4.0);
        let normalized_f64 = v_f64.normalize().unwrap();
        let expected_length = 1.0f64;
        assert!((normalized_f64.length() - expected_length).abs() < f64::EPSILON);

        // f32でのベクトル正規化
        let v_f32: Vector<f32> = Vector::new(3.0f32, 4.0f32);
        let normalized_f32 = v_f32.normalize().unwrap();
        let expected_length = 1.0f32;
        assert!((normalized_f32.length() - expected_length).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vector_scalar_constants() {
        // Scalarトレイトの定数を使用
        let zero_f64: Vector<f64> = Vector::zero();
        assert_eq!(zero_f64.x(), f64::ZERO);
        assert_eq!(zero_f64.y(), f64::ZERO);

        let unit_x_f32: Vector<f32> = Vector::unit_x();
        assert_eq!(unit_x_f32.x(), f32::ONE);
        assert_eq!(unit_x_f32.y(), f32::ZERO);
    }

    #[test]
    fn test_vector_cross_product_2d() {
        // f64での2D外積
        let v1_f64: Vector<f64> = Vector::new(1.0, 0.0);
        let v2_f64: Vector<f64> = Vector::new(0.0, 1.0);
        let cross_f64 = v1_f64.cross_2d(&v2_f64);
        assert_eq!(cross_f64, 1.0f64);

        // f32での2D外積
        let v1_f32: Vector<f32> = Vector::new(1.0f32, 0.0f32);
        let v2_f32: Vector<f32> = Vector::new(0.0f32, 1.0f32);
        let cross_f32 = v1_f32.cross_2d(&v2_f32);
        assert_eq!(cross_f32, 1.0f32);
    }

    #[test]
    fn test_vector_scalar_multiplication() {
        // f64でのスカラー乗算
        let v_f64: Vector<f64> = Vector::new(2.0, 3.0);
        let scaled_f64 = v_f64 * 2.0;
        assert_eq!(scaled_f64.x(), 4.0f64);
        assert_eq!(scaled_f64.y(), 6.0f64);

        // f32でのスカラー乗算
        let v_f32: Vector<f32> = Vector::new(2.0f32, 3.0f32);
        let scaled_f32 = v_f32 * 2.0f32;
        assert_eq!(scaled_f32.x(), 4.0f32);
        assert_eq!(scaled_f32.y(), 6.0f32);
    }

    #[test]
    fn test_vector_parametric_interoperability() {
        // Vector<f64> と Vector<f32> の相互運用性テスト
        let v_f64: Vector<f64> = Vector::new(1.5, 2.5);
        let v_f32: Vector<f32> = Vector::new(1.5f32, 2.5f32);

        // 型変換による比較
        let converted_f32 = Vector::<f32>::new(v_f64.x() as f32, v_f64.y() as f32);
        assert!((converted_f32.x() - v_f32.x()).abs() < f32::EPSILON);
        assert!((converted_f32.y() - v_f32.y()).abs() < f32::EPSILON);

        // 演算の一貫性テスト
        let result_f64 = v_f64 * 2.0;
        let result_f32 = v_f32 * 2.0f32;
        let converted_result = Vector::<f32>::new(result_f64.x() as f32, result_f64.y() as f32);

        assert!((converted_result.x() - result_f32.x()).abs() < f32::EPSILON);
        assert!((converted_result.y() - result_f32.y()).abs() < f32::EPSILON);
    }
}
