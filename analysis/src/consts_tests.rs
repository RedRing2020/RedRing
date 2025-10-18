//! 統合された定数機能のテスト

#[cfg(test)]
mod tests {
    use crate::consts::{MathConstants, ToleranceConstants};

    #[test]
    fn test_math_constants() {
        // 黄金比のテスト
        let golden_ratio_f64: f64 = MathConstants::golden_ratio();
        let expected_golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!((golden_ratio_f64 - expected_golden_ratio).abs() < 1e-10);

        let golden_ratio_f32: f32 = MathConstants::golden_ratio();
        let expected_golden_ratio_f32 = (1.0 + 5.0_f32.sqrt()) / 2.0;
        assert!((golden_ratio_f32 - expected_golden_ratio_f32).abs() < 1e-6);

        // ln(2)のテスト
        let ln_2_f64: f64 = MathConstants::ln_2();
        assert!((ln_2_f64 - std::f64::consts::LN_2).abs() < 1e-10);

        // √3のテスト
        let sqrt_3_f64: f64 = MathConstants::sqrt_3();
        assert!((sqrt_3_f64 - 3.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_tolerance_constants() {
        // 幾何計算用許容誤差のテスト
        let geometric_f64: f64 = ToleranceConstants::geometric();
        assert_eq!(geometric_f64, 1e-10);

        let geometric_f32: f32 = ToleranceConstants::geometric();
        assert_eq!(geometric_f32, 1e-6);

        // 角度計算用許容誤差のテスト
        let angular_f64: f64 = ToleranceConstants::angular();
        assert_eq!(angular_f64, 1e-8);

        // 距離計算用許容誤差のテスト
        let distance_f64: f64 = ToleranceConstants::distance();
        assert_eq!(distance_f64, 1e-12);

        // 面積計算用許容誤差のテスト
        let area_f32: f32 = ToleranceConstants::area();
        assert_eq!(area_f32, 1e-5);
    }
}
