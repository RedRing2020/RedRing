//! Scalar traitのユニットテスト（新実装用）
//! f32/f64に対するScalar traitの基本機能テスト

use crate::Scalar;

#[test]
fn test_scalar_trait_f32() {
    let a = 3.0f32;
    let b = 4.0f32;
    
    // 基本演算
    let sum = a + b;
    assert_eq!(sum, 7.0f32);
    
    // Scalar trait メソッド
    let hypotenuse = (a * a + b * b).sqrt();
    assert_eq!(hypotenuse, 5.0f32);
    
    // 定数アクセス
    assert_eq!(f32::ZERO, 0.0f32);
    assert_eq!(f32::ONE, 1.0f32);
    assert!((f32::PI - std::f32::consts::PI).abs() < f32::TOLERANCE);
}

#[test]
fn test_scalar_trait_f64() {
    let a = 3.0f64;
    let b = 4.0f64;
    
    // 基本演算
    let sum = a + b;
    assert_eq!(sum, 7.0f64);
    
    // Scalar trait メソッド
    let hypotenuse = (a * a + b * b).sqrt();
    assert_eq!(hypotenuse, 5.0f64);
    
    // 定数アクセス
    assert_eq!(f64::ZERO, 0.0f64);
    assert_eq!(f64::ONE, 1.0f64);
    assert!((f64::PI - std::f64::consts::PI).abs() < f64::TOLERANCE);
}

#[test]
fn test_type_conversion() {
    let f32_val = 3.14159f32;
    let f64_val = f32_val.to_f64();
    let back_to_f32 = f32::from_f64(f64_val);
    
    assert!((f32_val - back_to_f32).abs() < f32::TOLERANCE);
}