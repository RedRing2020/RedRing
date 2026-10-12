use super::*;

#[test]
fn test_flat_end_mill_creation() {
    let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);

    assert_eq!(tool.id, "EM10");
    assert_eq!(tool.tool_type(), ToolType::FlatEndMill);
    assert_eq!(tool.diameter(), 10.0);
    assert_eq!(tool.radius(), 5.0);
    assert_eq!(tool.corner_radius(), 0.0);
    assert_eq!(tool.cutting_length, 50.0);
    assert_eq!(tool.tip_offset(), 0.0);
}

#[test]
fn test_ball_end_mill_creation() {
    let tool = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);

    assert_eq!(tool.id, "BEM6");
    assert_eq!(tool.tool_type(), ToolType::BallEndMill);
    assert_eq!(tool.diameter(), 6.0);
    assert_eq!(tool.radius(), 3.0);
    assert_eq!(tool.corner_radius(), 3.0);
    assert_eq!(tool.cutting_length, 30.0);
    assert_eq!(tool.tip_offset(), 3.0);
}

#[test]
fn test_radius_end_mill_creation() {
    let tool = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);

    assert_eq!(tool.id, "REM10R1");
    assert_eq!(tool.tool_type(), ToolType::RadiusEndMill);
    assert_eq!(tool.diameter(), 10.0);
    assert_eq!(tool.radius(), 5.0);
    assert_eq!(tool.corner_radius(), 1.0);
    assert_eq!(tool.cutting_length, 50.0);
    assert_eq!(tool.tip_offset(), 1.0);
}

#[test]
fn test_new_with_parameters() {
    let flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 50.0);
    assert_eq!(flat.tool_type(), ToolType::FlatEndMill);
    assert_eq!(flat.corner_radius(), 0.0);

    let ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 3.0, 30.0);
    assert_eq!(ball.tool_type(), ToolType::BallEndMill);
    assert_eq!(ball.corner_radius(), 3.0);
    assert_eq!(ball.radius(), 3.0);

    let radius = Tool::new(
        "REM10R1".to_string(),
        ToolType::RadiusEndMill,
        10.0,
        1.0,
        50.0,
    );
    assert_eq!(radius.tool_type(), ToolType::RadiusEndMill);
    assert_eq!(radius.corner_radius(), 1.0);
    assert!(radius.corner_radius() > 0.0);
    assert!(radius.corner_radius() < radius.radius());
}

#[test]
fn test_radius_and_diameter() {
    let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    assert_eq!(tool.radius(), 5.0);
    assert_eq!(tool.diameter(), 10.0);
}

#[test]
fn test_tool_type_detection() {
    let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    assert!(flat.is_flat_end_mill());
    assert!(!flat.is_radius_end_mill());
    assert!(!flat.is_ball_end_mill());

    let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    assert!(!ball.is_flat_end_mill());
    assert!(!ball.is_radius_end_mill());
    assert!(ball.is_ball_end_mill());

    let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    assert!(!radius.is_flat_end_mill());
    assert!(radius.is_radius_end_mill());
    assert!(!radius.is_ball_end_mill());
}

#[test]
fn test_tip_offset() {
    let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    assert_eq!(flat.tip_offset(), 0.0);

    let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    assert_eq!(ball.tip_offset(), 3.0);

    let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    assert_eq!(radius.tip_offset(), 1.0);
}

#[test]
fn test_convert_z() {
    let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);

    let tip_z = ball.convert_z(10.0, ToolReferencePoint::Center, ToolReferencePoint::Tip);
    assert_eq!(tip_z, 7.0);

    let center_z = ball.convert_z(7.0, ToolReferencePoint::Tip, ToolReferencePoint::Center);
    assert_eq!(center_z, 10.0);

    let same_z = ball.convert_z(10.0, ToolReferencePoint::Center, ToolReferencePoint::Center);
    assert_eq!(same_z, 10.0);
}

#[test]
fn test_flat_end_mill_reference_points() {
    let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);

    let z = 10.0;
    let tip_z = flat.convert_z(z, ToolReferencePoint::Center, ToolReferencePoint::Tip);
    assert_eq!(tip_z, z);
}

#[test]
fn test_corner_radius_consistency() {
    let flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 50.0);
    assert_eq!(flat.corner_radius(), 0.0);
    assert_eq!(flat.tool_type(), ToolType::FlatEndMill);

    let ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 3.0, 30.0);
    assert_eq!(ball.corner_radius(), ball.radius());
    assert_eq!(ball.tool_type(), ToolType::BallEndMill);

    let radius = Tool::new(
        "REM8R2".to_string(),
        ToolType::RadiusEndMill,
        8.0,
        2.0,
        40.0,
    );
    assert_eq!(radius.corner_radius(), 2.0);
    assert!(radius.corner_radius() > 0.0);
    assert!(radius.corner_radius() < radius.radius());
    assert_eq!(radius.tool_type(), ToolType::RadiusEndMill);
}

#[test]
fn test_validate_parameters() {
    let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    assert!(flat.validate_parameters());

    let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    assert!(ball.validate_parameters());

    let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    assert!(radius.validate_parameters());

    let invalid_flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 1.0, 50.0);
    assert!(!invalid_flat.validate_parameters());

    let invalid_ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 2.0, 30.0);
    assert!(!invalid_ball.validate_parameters());

    let invalid_radius1 = Tool::new(
        "REM10R0".to_string(),
        ToolType::RadiusEndMill,
        10.0,
        0.0,
        50.0,
    );
    assert!(!invalid_radius1.validate_parameters());

    let invalid_radius2 = Tool::new(
        "REM10R5".to_string(),
        ToolType::RadiusEndMill,
        10.0,
        5.0,
        50.0,
    );
    assert!(!invalid_radius2.validate_parameters());
}
