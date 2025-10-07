//! Scalar型のユニットテスト
use crate::abstract_types::Scalar;
use std::f64::consts::PI;

#[test]
fn test_scalar_creation() {
    let s1 = Scalar::new(PI);
    let s2 = Scalar::new(2.71);

    assert_eq!(s1.value(), PI);
    assert_eq!(s2.value(), 2.71);
}

#[test]
fn test_arithmetic_operations() {
    let a = Scalar::new(1.0);
    let b = Scalar::new(2.0);

    let sum = a + b;
    let diff = a - b;
    let product = a * b;
    let quotient = b / a;

    assert_eq!(sum.value(), 3.0);
    assert_eq!(diff.value(), -1.0);
    assert_eq!(product.value(), 2.0);
    assert_eq!(quotient.value(), 2.0);
}

#[test]
fn test_scalar_constants() {
    let zero = Scalar::ZERO;
    let one = Scalar::ONE;
    
    assert_eq!(zero.value(), 0.0);
    assert_eq!(one.value(), 1.0);
}
