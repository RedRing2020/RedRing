use crate::linalg::scalar::*;
use std::f64::consts::PI;

#[test]
fn test_scalar_trait_f64() {
    let a: f64 = 3.0;
    let b: f64 = 4.0;

    assert_eq!(a + b, 7.0);
    assert!((a.sqrt() - 1.7320508075688772).abs() < f64::EPSILON);
    assert!(a.approx_eq(3.0000001, 1e-6));
    assert!(F64::ZERO.is_zero());
}

#[test]
fn test_scalar_trait_f32() {
    let a: f32 = 3.0;
    let b: f32 = 4.0;

    assert_eq!(a + b, 7.0);
    assert!((a.sqrt() - 1.7320508).abs() < f32::EPSILON * 10.0);
    assert!(a.approx_eq(3.0000001, 1e-6));
    assert!(F32::ZERO.is_zero());
}

#[test]
fn test_type_conversion() {
    let a: f64 = PI;
    let b: f32 = F32::from_f64(a);
    let c: f64 = F64::from_f32(b);

    // f32の精度でクランプされることを確認
    assert!((c - (PI as f32) as f64).abs() < 1e-6);
}
