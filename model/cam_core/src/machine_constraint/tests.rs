use super::*;

#[test]
fn test_linear_axis_limit_contains_mm() {
    let limit = LinearAxisLimit::new(LinearAxisLabel::X, -500.0, 500.0);

    assert!(limit.contains_mm(-500.0));
    assert!(limit.contains_mm(0.0));
    assert!(limit.contains_mm(500.0));
    assert!(!limit.contains_mm(-500.1));
    assert!(!limit.contains_mm(500.1));
}

#[test]
fn test_machine_constraint_linear_limit_for_axis() {
    let constraint = MachineConstraint {
        linear_axis_limits: vec![
            LinearAxisLimit::new(LinearAxisLabel::X, -500.0, 500.0),
            LinearAxisLimit::new(LinearAxisLabel::Z, -100.0, 0.0),
        ],
        rotary_axis_limits: vec![RotaryAxisLimit::new(RotaryAxisLabel::B, -30.0, 120.0)],
        linear_speed_limit: None,
        rotary_speed_limit: None,
        linear_acceleration_limit: None,
        rotary_acceleration_limit: None,
    };

    let x_limit = constraint.linear_limit_for_axis(LinearAxisLabel::X);
    assert_eq!(
        x_limit,
        Some(&LinearAxisLimit::new(LinearAxisLabel::X, -500.0, 500.0))
    );

    let y_limit = constraint.linear_limit_for_axis(LinearAxisLabel::Y);
    assert_eq!(y_limit, None);
}

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

    assert!(constraint.linear_axis_limits.is_empty());
    assert!(constraint.rotary_axis_limits.is_empty());
    assert_eq!(constraint.linear_speed_limit, None);
    assert_eq!(constraint.rotary_speed_limit, None);
    assert_eq!(constraint.linear_acceleration_limit, None);
    assert_eq!(constraint.rotary_acceleration_limit, None);
    assert_eq!(constraint.linear_limit_for_axis(LinearAxisLabel::X), None);
    assert_eq!(constraint.rotary_limit_for_axis(RotaryAxisLabel::B), None);
}

#[test]
fn test_machine_constraint_new_with_rotary_only() {
    let rotary_limits = vec![
        RotaryAxisLimit::new(RotaryAxisLabel::A, 0.0, 360.0),
        RotaryAxisLimit::new(RotaryAxisLabel::B, -90.0, 90.0),
    ];

    let constraint = MachineConstraint::new(rotary_limits.clone());

    assert_eq!(constraint.rotary_axis_limits, rotary_limits);
    assert!(constraint.linear_axis_limits.is_empty());
    assert_eq!(constraint.linear_speed_limit, None);
    assert_eq!(constraint.rotary_speed_limit, None);
    assert_eq!(constraint.linear_acceleration_limit, None);
    assert_eq!(constraint.rotary_acceleration_limit, None);
}

#[test]
fn test_machine_constraint_mixed_axes() {
    let constraint = MachineConstraint {
        linear_axis_limits: vec![
            LinearAxisLimit::new(LinearAxisLabel::X, -500.0, 500.0),
            LinearAxisLimit::new(LinearAxisLabel::Y, -400.0, 400.0),
            LinearAxisLimit::new(LinearAxisLabel::Z, -100.0, 0.0),
        ],
        rotary_axis_limits: vec![
            RotaryAxisLimit::new(RotaryAxisLabel::A, 0.0, 360.0),
            RotaryAxisLimit::new(RotaryAxisLabel::B, -30.0, 120.0),
        ],
        linear_speed_limit: Some(LinearSpeedLimit::new(12000.0)),
        rotary_speed_limit: Some(RotarySpeedLimit::new(100.0)),
        linear_acceleration_limit: Some(LinearAccelerationLimit::new(10000.0)),
        rotary_acceleration_limit: Some(RotaryAccelerationLimit::new(500.0)),
    };

    // 直動軸確認
    assert!(
        constraint
            .linear_limit_for_axis(LinearAxisLabel::X)
            .is_some()
    );
    assert!(
        constraint
            .linear_limit_for_axis(LinearAxisLabel::U)
            .is_none()
    );

    // 回転軸確認（既存互換性）
    assert!(
        constraint
            .rotary_limit_for_axis(RotaryAxisLabel::A)
            .is_some()
    );
    assert!(
        constraint
            .rotary_limit_for_axis(RotaryAxisLabel::C)
            .is_none()
    );

    assert_eq!(
        constraint.linear_speed_limit,
        Some(LinearSpeedLimit::new(12000.0))
    );
    assert_eq!(
        constraint.rotary_speed_limit,
        Some(RotarySpeedLimit::new(100.0))
    );
    assert_eq!(
        constraint.linear_acceleration_limit,
        Some(LinearAccelerationLimit::new(10000.0))
    );
    assert_eq!(
        constraint.rotary_acceleration_limit,
        Some(RotaryAccelerationLimit::new(500.0))
    );
}

#[test]
fn test_speed_and_acceleration_limits_allow_values() {
    let linear_speed = LinearSpeedLimit::new(12000.0);
    assert!(linear_speed.allows_mm_per_min(12000.0));
    assert!(!linear_speed.allows_mm_per_min(12000.1));

    let rotary_speed = RotarySpeedLimit::new(100.0);
    assert!(rotary_speed.allows_deg_per_sec(99.9));
    assert!(!rotary_speed.allows_deg_per_sec(100.1));

    let linear_acceleration = LinearAccelerationLimit::new(10000.0);
    assert!(linear_acceleration.allows_mm_per_sec2(10000.0));
    assert!(!linear_acceleration.allows_mm_per_sec2(10000.1));

    let rotary_acceleration = RotaryAccelerationLimit::new(500.0);
    assert!(rotary_acceleration.allows_deg_per_sec2(500.0));
    assert!(!rotary_acceleration.allows_deg_per_sec2(500.1));
}
