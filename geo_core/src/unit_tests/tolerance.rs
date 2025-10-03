/// ToleranceContext型のユニットテスト

use crate::tolerance::{ToleranceContext, TolerantEq, TolerantOrd, TopologyChecker};

#[test]
fn test_tolerance_contexts() {
    let standard = ToleranceContext::standard();
    let high_prec = ToleranceContext::high_precision();
    let low_prec = ToleranceContext::low_precision();

    assert!(high_prec.linear < standard.linear);
    assert!(standard.linear < low_prec.linear);
}

#[test]
fn test_scaling() {
    let context = ToleranceContext::standard();
    let scaled = context.scaled(1000.0); // mm to m conversion

    assert!((scaled.linear - context.linear * 1000.0).abs() < 1e-15);
    assert!((scaled.angular - context.angular).abs() < 1e-15); // angle unchanged
    assert!((scaled.curvature - context.curvature / 1000.0).abs() < 1e-15);
}

#[test]
fn test_f64_tolerant_comparison() {
    let context = ToleranceContext::standard();

    let a = 1.0;
    let b = 1.0 + context.linear * 0.5; // within tolerance
    let c = 1.0 + context.linear * 2.0; // outside tolerance

    assert!(a.tolerant_eq(&b, &context));
    assert!(!a.tolerant_eq(&c, &context));
    assert!(a.tolerant_lt(&c, &context));
}

#[test]
fn test_topology_checker() {
    let context = ToleranceContext::standard();
    let checker = TopologyChecker::new(context);

    // テトラヘドロン: V=4, E=6, F=4
    assert!(checker.verify_euler_characteristic(4, 6, 4));

    // 立方体: V=8, E=12, F=6
    assert!(checker.verify_euler_characteristic(8, 12, 6));

    // 不正な組み合わせ
    assert!(!checker.verify_euler_characteristic(3, 3, 1));
}