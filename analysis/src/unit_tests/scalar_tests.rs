//! Scalar trait の統合テスト
//!
//! Scalar trait 実装の統合動作確認を行います。
//! f32/f64両方での動作と相互運用性をテストします。

use crate::abstract_types::Scalar;

#[cfg(test)]
mod scalar_tests {
    use super::*;

    #[test]
    fn test_scalar_trait_f32_f64_compatibility() {
        // f32 Scalar trait テスト
        let a_f32 = 3.0f32;
        let b_f32 = 4.0f32;
        let hypotenuse_f32 = (a_f32 * a_f32 + b_f32 * b_f32).sqrt();
        assert!((hypotenuse_f32 - 5.0f32).abs() < f32::TOLERANCE);

        // f64 Scalar trait テスト
        let a_f64 = 3.0f64;
        let b_f64 = 4.0f64;
        let hypotenuse_f64 = (a_f64 * a_f64 + b_f64 * b_f64).sqrt();
        assert!((hypotenuse_f64 - 5.0f64).abs() < f64::TOLERANCE);

        // 型変換テスト
        let f32_val = f32::PI;
        let converted_to_f64 = f32_val.to_f64();
        let back_to_f32 = f32::from_f64(converted_to_f64);
        assert!((f32_val - back_to_f32).abs() < f32::TOLERANCE);
    }

    #[test]
    fn test_mathematical_constants_consistency() {
        // PI定数の一貫性
        assert!((f32::PI.to_f64() - f64::PI).abs() < 1e-6);
        assert!((f32::TAU.to_f64() - f64::TAU).abs() < 1e-6);

        // 角度変換定数の一貫性
        let degrees_f32 = 180.0f32;
        let radians_f32 = degrees_f32 * f32::DEG_TO_RAD;
        assert!((radians_f32 - f32::PI).abs() < 1e-6);

        let degrees_f64 = 180.0f64;
        let radians_f64 = degrees_f64 * f64::DEG_TO_RAD;
        assert!((radians_f64 - f64::PI).abs() < 1e-10);
    }

    #[test]
    fn test_type_conversions() {
        // Scalar trait のメソッドテスト
        let value_f32: f32 = 42.5;
        let value_f64: f64 = 42.5;

        // 基本的な変換
        assert!((value_f32.to_f64() - value_f64).abs() < 1e-6);
        assert!((f32::from_f64(value_f64) - value_f32).abs() < f32::TOLERANCE);

        // 近似等価
        assert!(value_f32.approx_eq(42.5));
        assert!(value_f64.approx_eq(42.5));
    }

    #[test]
    fn test_precision_boundaries() {
        // 精度境界でのテスト
        let small_f32 = f32::TOLERANCE * 0.1;
        let large_f32 = f32::TOLERANCE * 10.0;

        assert!(small_f32.abs() < f32::TOLERANCE);
        assert!(large_f32.abs() > f32::TOLERANCE);

        let small_f64 = f64::TOLERANCE * 0.1;
        let large_f64 = f64::TOLERANCE * 10.0;

        assert!(small_f64.abs() < f64::TOLERANCE);
        assert!(large_f64.abs() > f64::TOLERANCE);
    }
}
