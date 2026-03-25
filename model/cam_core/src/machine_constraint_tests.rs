use super::*;

#[test]
fn test_rotary_axis_limit_contains_deg() {
    let limit = RotaryAxisLimit::new(RotaryAxisLabel::B, -30.0, 120.0);

    assert!(limit.contains_deg(-30.0));
    assert!(limit.contains_deg(0.0));
    assert!(limit.contains_deg(120.0));
    assert!(!limit.contains_deg(-30.1));
    assert!(!limit.contains_deg(120.1));
}

#[test]
fn test_machine_constraint_rotary_limit_for_axis() {
    let constraint = MachineConstraint::new(vec![
        RotaryAxisLimit::new(RotaryAxisLabel::B, -30.0, 120.0),
        RotaryAxisLimit::new(RotaryAxisLabel::C, -360.0, 360.0),
    ]);

    let b_limit = constraint.rotary_limit_for_axis(RotaryAxisLabel::B);
    assert_eq!(
        b_limit,
        Some(&RotaryAxisLimit::new(RotaryAxisLabel::B, -30.0, 120.0))
    );

    let a_limit = constraint.rotary_limit_for_axis(RotaryAxisLabel::A);
    assert_eq!(a_limit, None);
}

#[test]
fn test_machine_constraint_empty() {
    let constraint = MachineConstraint::<f64>::empty();

    assert!(constraint.rotary_axis_limits.is_empty());
    assert_eq!(constraint.rotary_limit_for_axis(RotaryAxisLabel::B), None);
}
