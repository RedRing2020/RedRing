/// Scalar型のユニットテスト

use crate::scalar::Scalar;
use crate::tolerance::{ToleranceContext, TolerantEq};

#[test]
fn test_scalar_creation() {
    let s1 = Scalar::new(3.14);
    let s2 = Scalar::with_tolerance(3.14, 0.01);

    assert_eq!(s1.value(), 3.14);
    assert_eq!(s2.value(), 3.14);
    assert_eq!(s1.tolerance(), None);
    assert_eq!(s2.tolerance(), Some(0.01));
}

#[test]
fn test_arithmetic_operations() {
    let a = Scalar::with_tolerance(1.0, 0.1);
    let b = Scalar::with_tolerance(2.0, 0.1);

    let sum = a + b;
    assert_eq!(sum.value(), 3.0);
    // √(0.1² + 0.1²) = √0.02 ≈ 0.1414
    assert!((sum.tolerance().unwrap() - 0.1414).abs() < 0.001);

    let product = a * b;
    assert_eq!(product.value(), 2.0);
    // √((2*0.1)² + (1*0.1)²) = √(0.04 + 0.01) = √0.05 ≈ 0.2236
    assert!((product.tolerance().unwrap() - 0.2236).abs() < 0.001);
}

#[test]
fn test_tolerant_comparison() {
    let context = ToleranceContext::standard();
    let a = Scalar::new(1.0);
    let b = Scalar::new(1.0 + context.linear * 0.5);
    let c = Scalar::new(1.0 + context.linear * 2.0);

    assert!(a.tolerant_eq(&b, &context));
    assert!(!a.tolerant_eq(&c, &context));
}

#[test]
fn test_transcendental_functions() {
    let x = Scalar::with_tolerance(1.0, 0.01);

    let sin_x = x.sin();
    let cos_x = x.cos();

    // sin²(x) + cos²(x) = 1 の検証
    let sum_squares = sin_x * sin_x + cos_x * cos_x;
    assert!((sum_squares.value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_safe_division() {
    let context = ToleranceContext::standard();
    let a = Scalar::new(1.0);
    let zero = Scalar::new(0.0);
    let small = Scalar::new(context.linear * 0.1);

    assert!(a.safe_div(&zero, &context).is_none());
    assert!(a.safe_div(&small, &context).is_none());

    let large = Scalar::new(1.0);
    assert!(a.safe_div(&large, &context).is_some());
}