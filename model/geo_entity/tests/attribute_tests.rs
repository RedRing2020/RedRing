use geo_entity::{
    AttributeCode, AttributeValue, Attributes, CAM_PATH_DRILL_FEED_RATE, EntityError,
};

#[test]
fn zero_code_is_invalid() {
    let result = AttributeCode::try_new(0);
    assert_eq!(result, Err(EntityError::InvalidAttributeCode));
}

#[test]
fn known_system_code_accepts_matching_type() {
    let mut attrs = Attributes::new();
    attrs
        .set(CAM_PATH_DRILL_FEED_RATE, AttributeValue::Float(120.0))
        .expect("system code with correct type should be accepted");
    assert!(attrs.contains(CAM_PATH_DRILL_FEED_RATE));
}

#[test]
fn known_system_code_rejects_type_mismatch() {
    let mut attrs = Attributes::new();
    let result = attrs.set(
        CAM_PATH_DRILL_FEED_RATE,
        AttributeValue::String("bad".into()),
    );
    assert!(matches!(
        result,
        Err(EntityError::AttributeTypeMismatch { .. })
    ));
}

#[test]
fn unknown_system_code_is_rejected() {
    let mut attrs = Attributes::new();
    let unknown = AttributeCode::try_new(-99_99_99_99).expect("negative non-zero code is valid");
    let result = attrs.set(unknown, AttributeValue::Float(1.0));
    assert!(matches!(
        result,
        Err(EntityError::UnknownSystemAttributeCode(_))
    ));
}

#[test]
fn user_code_is_accepted_without_system_definition() {
    let mut attrs = Attributes::new();
    let user_code = AttributeCode::try_new(10_01_02_02).expect("positive non-zero code is valid");
    attrs
        .set(user_code, AttributeValue::Float(180.0))
        .expect("user code should be accepted");
    assert!(attrs.contains(user_code));
}
