//! ToleranceContext型のユニットテスト
use crate::abstract_types::{ToleranceContext, TolerantEq};

#[test]
fn test_tolerance_context_creation() {
    let context1 = ToleranceContext::new(1e-6);
    let context2 = ToleranceContext::default();

    assert_eq!(context1.tolerance(), 1e-6);
    assert!(context2.tolerance() > 0.0);
}

#[test]
fn test_tolerant_equality() {
    let context = ToleranceContext::new(1e-6);

    let a = 1.0;
    let b = 1.0000005; // 許容誤差内
    let c = 1.1; // 許容誤差外

    assert!(a.tolerant_eq(&b, &context));
    assert!(!a.tolerant_eq(&c, &context));
}
