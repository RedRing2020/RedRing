//! 統合された定数機能のテスト

#[cfg(test)]
mod tests {
    use crate::consts::{special, GeometricTolerance};

    #[test]
    fn test_special_constants() {
        // 黄金比のテスト
        let golden_ratio_f64 = special::GOLDEN_RATIO_F64;
        let expected_golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!((golden_ratio_f64 - expected_golden_ratio).abs() < 1e-10);

        let golden_ratio_f32 = special::GOLDEN_RATIO_F32;
        let expected_golden_ratio_f32 = (1.0 + 5.0_f32.sqrt()) / 2.0;
        assert!((golden_ratio_f32 - expected_golden_ratio_f32).abs() < 1e-6);

        // ln(2)のテスト
        let ln_2_f64 = special::LN_2_F64;
        assert!((ln_2_f64 - std::f64::consts::LN_2).abs() < 1e-15);

        // ln(10)のテスト
        let ln_10_f64 = special::LN_10_F64;
        assert!((ln_10_f64 - std::f64::consts::LN_10).abs() < 1e-15);

        // √3のテスト
        let sqrt_3_f64 = special::SQRT_3_F64;
        assert!((sqrt_3_f64 - 3.0_f64.sqrt()).abs() < 1e-15);
    }

    #[test]
    fn test_tolerance_constants() {
        // GeometricToleranceトレイトのテスト
        let geometric_f64 = <f64 as GeometricTolerance>::TOLERANCE;
        let geometric_f32 = <f32 as GeometricTolerance>::TOLERANCE;
        
        // f64は高精度、f32は低精度であることを確認
        assert!(geometric_f64 < geometric_f32);
        assert_eq!(geometric_f64, 1e-10);
        assert_eq!(geometric_f32, 1e-6);

        // 角度・距離許容誤差もテスト
        let angle_f64 = <f64 as GeometricTolerance>::ANGLE_TOLERANCE;
        let distance_f64 = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;
        assert_eq!(angle_f64, 1e-12);
        assert_eq!(distance_f64, 1e-10);
        assert_eq!(area_f32, 1e-5);
    }
}
