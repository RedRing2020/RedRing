//! 幾何要素の分類システムのテスト
//! DimensionClass, PrimitiveKindのテスト

#[cfg(test)]
use crate::traits::common::classification::{DimensionClass, PrimitiveKind};

#[test]
fn test_dimension_classification() {
    assert_eq!(PrimitiveKind::Point.dimension(), DimensionClass::Zero);
    assert_eq!(PrimitiveKind::LineSegment.dimension(), DimensionClass::One);
    assert_eq!(PrimitiveKind::Circle.dimension(), DimensionClass::Two);
    assert_eq!(PrimitiveKind::Sphere.dimension(), DimensionClass::Three);
}

#[test]
fn test_property_checks() {
    assert!(PrimitiveKind::BezierCurve.is_parametric());
    assert!(PrimitiveKind::Circle.is_analytical());
    assert!(PrimitiveKind::TriangleMesh.is_mesh());
}
