use super::*;
use crate::consts::test_constants::TOLERANCE_F64;

#[test]
fn test_length_unit_conversions() {
    // ミリメートル → 他の単位
    assert_eq!(
        LengthUnit::Millimeter.conversion_factor_to(LengthUnit::Meter),
        0.001
    );
    assert_eq!(
        LengthUnit::Millimeter.conversion_factor_to(LengthUnit::Centimeter),
        0.1
    );
    assert!((LengthUnit::Millimeter.conversion_factor_to(LengthUnit::Inch) - 1.0 / 25.4).abs() < TOLERANCE_F64);

    // メートル → 他の単位
    assert_eq!(
        LengthUnit::Meter.conversion_factor_to(LengthUnit::Millimeter),
        1000.0
    );
    assert_eq!(
        LengthUnit::Meter.conversion_factor_to(LengthUnit::Centimeter),
        100.0
    );

    // 同一単位
    assert_eq!(LengthUnit::Meter.conversion_factor_to(LengthUnit::Meter), 1.0);
    assert_eq!(LengthUnit::Inch.conversion_factor_to(LengthUnit::Inch), 1.0);
}

#[test]
fn test_tolerance_conversions() {
    let tol = Tolerance::new(1.0, LengthUnit::Millimeter);

    assert_eq!(tol.in_millimeters(), 1.0);
    assert_eq!(tol.in_unit(LengthUnit::Meter), 0.001);
    assert_eq!(tol.in_unit(LengthUnit::Centimeter), 0.1);
}

#[test]
fn test_tolerance_default() {
    let tol = Tolerance::default();
    assert_eq!(tol.value, 0.01);
    assert_eq!(tol.unit, LengthUnit::Millimeter);
    assert_eq!(tol.in_millimeters(), 0.01);
}

#[test]
fn test_unit_string_representation() {
    assert_eq!(LengthUnit::Millimeter.as_str(), "mm");
    assert_eq!(LengthUnit::Meter.as_str(), "m");
    assert_eq!(LengthUnit::Centimeter.as_str(), "cm");
    assert_eq!(LengthUnit::Inch.as_str(), "in");
}

#[test]
fn test_tolerance_cross_unit() {
    // 異なる単位で同じ物理量を表現
    let tol_mm = Tolerance::new(10.0, LengthUnit::Millimeter);
    let tol_cm = Tolerance::new(1.0, LengthUnit::Centimeter);
    let tol_m = Tolerance::new(0.01, LengthUnit::Meter);

    // 全て10mmと等価
    assert_eq!(tol_mm.in_millimeters(), 10.0);
    assert_eq!(tol_cm.in_millimeters(), 10.0);
    assert_eq!(tol_m.in_millimeters(), 10.0);

    // 同じ値をメートル単位で表現すると全て0.01
    assert!((tol_mm.in_unit(LengthUnit::Meter) - 0.01).abs() < TOLERANCE_F64);
    assert!((tol_cm.in_unit(LengthUnit::Meter) - 0.01).abs() < TOLERANCE_F64);
    assert!((tol_m.in_unit(LengthUnit::Meter) - 0.01).abs() < TOLERANCE_F64);
}
