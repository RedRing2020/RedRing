use crate::abstract_types::Scalar;
use std::f64::consts::PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_trait_f64() {
        let a: f64 = 3.0;
        let b: f64 = 4.0;

        assert_eq!(a + b, 7.0);
        assert!((a.sqrt() - 1.7320508075688772).abs() < f64::EPSILON);
        assert!(a.approx_eq_with_tolerance(3.0000001, 1e-6));
        assert!(f64::ZERO.is_zero());
    }

    #[test]
    fn test_scalar_trait_f32() {
        let a: f32 = 3.0;
        let b: f32 = 4.0;

        assert_eq!(a + b, 7.0);
        assert!((a.sqrt() - 1.7320508).abs() < f32::EPSILON * 10.0);
        assert!(a.approx_eq_with_tolerance(3.0000001, 1e-6));
        assert!(f32::ZERO.is_zero());
    }

    #[test]
    fn test_type_conversion() {
        let a: f64 = PI;
        let b: f32 = f32::from_f64(a);
        let c: f64 = f64::from_f32(b);

        // f32の精度でクランプされることを確認
        assert!((c - (PI as f32) as f64).abs() < 1e-6);
    }

    #[test]
    fn test_scalar_constants() {
        // f64定数テスト
        assert_eq!(f64::ZERO, 0.0);
        assert_eq!(f64::ONE, 1.0);
        assert!(f64::ZERO.is_zero());
        assert!(!f64::ONE.is_zero());

        // f32定数テスト
        assert_eq!(f32::ZERO, 0.0);
        assert_eq!(f32::ONE, 1.0);
        assert!(f32::ZERO.is_zero());
        assert!(!f32::ONE.is_zero());
    }

    #[test]
    fn test_scalar_arithmetic() {
        let a: f64 = 2.0;
        let b: f64 = 3.0;

        assert_eq!(a.powf(b), 8.0);
        assert_eq!(a.max(b), 3.0);
        assert_eq!(a.min(b), 2.0);
        assert!((a.sin().powi(2) + a.cos().powi(2) - 1.0).abs() < f64::EPSILON);
    }
}
