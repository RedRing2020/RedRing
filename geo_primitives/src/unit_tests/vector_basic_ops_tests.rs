//! Vector基礎演算のテスト
//!
//! 加算、減算、スカラー倍、内積などの基本的なベクトル演算の確認

#[cfg(test)]
mod tests {
    use crate::geometry2d::{Vector, Vector2D};
    use geo_foundation::abstract_types::geometry::common::normalization_operations::Normalizable;
    use geo_foundation::Scalar;

    #[test]
    fn test_vector2d_addition() {
        let v1 = Vector::new(1.0, 2.0);
        let v2 = Vector::new(3.0, 4.0);
        let result = v1 + v2;

        assert_eq!(result.x(), 4.0);
        assert_eq!(result.y(), 6.0);
    }

    #[test]
    fn test_vector2d_subtraction() {
        let v1 = Vector::new(5.0, 7.0);
        let v2 = Vector::new(2.0, 3.0);
        let result = v1 - v2;

        assert_eq!(result.x(), 3.0);
        assert_eq!(result.y(), 4.0);
    }

    #[test]
    fn test_vector2d_scalar_multiplication() {
        let v = Vector::new(2.0, 3.0);
        let result = v * 2.5;

        assert_eq!(result.x(), 5.0);
        assert_eq!(result.y(), 7.5);
    }

    #[test]
    fn test_vector2d_scalar_division() {
        let v = Vector::new(6.0, 9.0);
        let result = v / 3.0;

        assert_eq!(result.x(), 2.0);
        assert_eq!(result.y(), 3.0);
    }

    #[test]
    fn test_vector2d_negation() {
        let v = Vector::new(2.0, -3.0);
        let result = -v;

        assert_eq!(result.x(), -2.0);
        assert_eq!(result.y(), 3.0);
    }

    #[test]
    fn test_vector2d_dot_product() {
        let v1 = Vector::new(2.0, 3.0);
        let v2 = Vector::new(4.0, 5.0);
        let result = v1.dot(&v2);

        // 2*4 + 3*5 = 8 + 15 = 23
        assert_eq!(result, 23.0);
    }

    #[test]
    fn test_vector2d_length() {
        let v = Vector::new(3.0, 4.0);
        let length = v.length();

        // sqrt(3^2 + 4^2) = sqrt(9 + 16) = sqrt(25) = 5
        assert_eq!(length, 5.0);
    }

    #[test]
    fn test_vector2d_normalize() {
        let v = Vector::new(3.0, 4.0);
        let normalized = v.normalize().unwrap();

        // 正規化後の長さは1
        assert!((normalized.length() - 1.0).abs() < f64::EPSILON);

        // 方向は保持される
        assert_eq!(normalized.x(), 0.6); // 3/5
        assert_eq!(normalized.y(), 0.8); // 4/5
    }

    #[test]
    fn test_vector2d_zero_vector() {
        let zero = Vector::<f64>::zero();

        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);
        assert_eq!(zero.length(), 0.0);
    }

    #[test]
    fn test_vector2d_f32_compatibility() {
        let v1 = Vector::<f32>::new(1.0, 2.0);
        let v2 = Vector::<f32>::new(3.0, 4.0);
        let result = v1 + v2;

        assert_eq!(result.x(), 4.0f32);
        assert_eq!(result.y(), 6.0f32);
    }

    #[test]
    fn test_vector2d_cross_product_2d() {
        let v1 = Vector::new(2.0, 3.0);
        let v2 = Vector::new(4.0, 5.0);
        let cross = v1.cross_2d(&v2);

        // 2*5 - 3*4 = 10 - 12 = -2
        assert_eq!(cross, -2.0);
    }

    #[test]
    fn test_vector2d_perpendicular() {
        let v = Vector::new(3.0, 4.0);
        let perp = v.perpendicular();

        // 垂直ベクトルは (-y, x)
        assert_eq!(perp.x(), -4.0);
        assert_eq!(perp.y(), 3.0);

        // 元のベクトルと垂直であることを確認（内積=0）
        assert_eq!(v.dot(&perp), 0.0);
    }

    #[test]
    fn test_vector2d_type_alias_compatibility() {
        // Vector2D (= Vector<f64>) が期待通りに動作することを確認
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);
        let result = v1 + v2;

        assert_eq!(result.x(), 4.0);
        assert_eq!(result.y(), 6.0);
    }
}
