use analysis::abstract_types::Scalar;
use analysis::GeometricTolerance;

#[test]
fn test_scalar_trait_f32_f64_compatibility() {
    let a_f32 = 3.0f32;
    let b_f32 = 4.0f32;
    let hypotenuse_f32 = (a_f32 * a_f32 + b_f32 * b_f32).sqrt();
    assert!((hypotenuse_f32 - 5.0f32).abs() < <f32 as GeometricTolerance>::TOLERANCE);

    let a_f64 = 3.0f64;
    let b_f64 = 4.0f64;
    let hypotenuse_f64 = (a_f64 * a_f64 + b_f64 * b_f64).sqrt();
    assert!((hypotenuse_f64 - 5.0f64).abs() < <f64 as GeometricTolerance>::TOLERANCE);

    let f32_val = f32::PI;
    let converted_to_f64 = f32_val.to_f64();
    let back_to_f32 = f32::from_f64(converted_to_f64);
    assert!((f32_val - back_to_f32).abs() < <f32 as GeometricTolerance>::TOLERANCE);
}

#[test]
fn test_mathematical_constants_consistency() {
    assert!((f32::PI.to_f64() - f64::PI).abs() < <f32 as GeometricTolerance>::TOLERANCE as f64);
    assert!((f32::TAU.to_f64() - f64::TAU).abs() < <f32 as GeometricTolerance>::TOLERANCE as f64);

    let degrees_f32 = 180.0f32;
    let radians_f32 = degrees_f32 * f32::DEG_TO_RAD;
    assert!((radians_f32 - f32::PI).abs() < <f32 as GeometricTolerance>::TOLERANCE);

    let degrees_f64 = 180.0f64;
    let radians_f64 = degrees_f64 * f64::DEG_TO_RAD;
    assert!((radians_f64 - f64::PI).abs() < <f64 as GeometricTolerance>::TOLERANCE);
}

#[test]
fn test_type_conversions() {
    let value_f32: f32 = 42.5;
    let value_f64: f64 = 42.5;

    assert!((value_f32.to_f64() - value_f64).abs() < <f32 as GeometricTolerance>::TOLERANCE as f64);
    assert!((f32::from_f64(value_f64) - value_f32).abs() < <f32 as GeometricTolerance>::TOLERANCE);

    assert!(value_f32.approx_eq(42.5));
    assert!(value_f64.approx_eq(42.5));
}

#[test]
fn test_precision_boundaries() {
    let small_f32 = <f32 as GeometricTolerance>::TOLERANCE * 0.1;
    let large_f32 = <f32 as GeometricTolerance>::TOLERANCE * 10.0;

    assert!(small_f32.abs() < <f32 as GeometricTolerance>::TOLERANCE);
    assert!(large_f32.abs() > <f32 as GeometricTolerance>::TOLERANCE);

    let small_f64 = <f64 as GeometricTolerance>::TOLERANCE * 0.1;
    let large_f64 = <f64 as GeometricTolerance>::TOLERANCE * 10.0;

    assert!(small_f64.abs() < <f64 as GeometricTolerance>::TOLERANCE);
    assert!(large_f64.abs() > <f64 as GeometricTolerance>::TOLERANCE);
}
