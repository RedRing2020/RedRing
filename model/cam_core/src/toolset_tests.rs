use super::*;
use crate::{Tool, ToolType};

#[test]
fn test_holder_segment_creation() {
    let cyl = HolderSegment::cylinder(20.0, 30.0, 1.0, 1.0);
    assert_eq!(cyl.kind, HolderSegmentKind::Cylinder);
    assert_eq!(cyl.top_diameter, 30.0);
    assert_eq!(cyl.bottom_diameter, 30.0);
    assert_eq!(cyl.top_corner_radius, 1.0);
    assert_eq!(cyl.bottom_corner_radius, 1.0);

    let taper = HolderSegment::taper(15.0, 30.0, 20.0, 0.5, 0.25);
    assert_eq!(taper.kind, HolderSegmentKind::Taper);
    assert_eq!(taper.top_diameter, 30.0);
    assert_eq!(taper.bottom_diameter, 20.0);
}

#[test]
fn test_holder_validation() {
    let holder = Holder::new(
        "HOLDER-A".to_string(),
        vec![
            HolderSegment::cylinder(20.0, 30.0, 1.0, 1.0),
            HolderSegment::taper(15.0, 30.0, 20.0, 0.5, 0.25),
        ],
    )
    .with_interference_offset(HolderInterferenceOffset::new(10.0, 0.0));

    assert!(holder.validate_parameters());
    assert_eq!(holder.total_length(), 35.0);
}

#[test]
fn test_holder_segment_kind_validation() {
    let invalid_cylinder = HolderSegment::taper(10.0, 20.0, 20.0, 0.0, 0.0);
    assert!(!invalid_cylinder.validate_parameters());

    let invalid_taper = HolderSegment::cylinder(10.0, 20.0, 0.0, 0.0);
    let mut invalid_taper = invalid_taper;
    invalid_taper.kind = HolderSegmentKind::Taper;
    assert!(!invalid_taper.validate_parameters());

    let invalid_corners = HolderSegment::cylinder(10.0, 20.0, 6.0, 5.0);
    assert!(!invalid_corners.validate_parameters());
}

#[test]
fn test_holder_interference_shape_applies_bottom_to_last_segment_only() {
    let holder = Holder::new(
        "HOLDER-B".to_string(),
        vec![
            HolderSegment::cylinder(20.0, 30.0, 1.0, 1.0),
            HolderSegment::taper(15.0, 30.0, 20.0, 0.5, 0.25),
        ],
    )
    .with_interference_offset(HolderInterferenceOffset::new(10.0, 2.0));

    let offset_shape = holder.to_interference_shape();

    assert_eq!(offset_shape.segments.len(), 2);
    assert_eq!(offset_shape.segments[0].top_diameter, 50.0);
    assert_eq!(offset_shape.segments[0].bottom_diameter, 50.0);
    assert_eq!(offset_shape.segments[1].top_diameter, 50.0);
    assert_eq!(offset_shape.segments[1].bottom_diameter, 40.0);

    assert_eq!(offset_shape.segments[0].length, 20.0);
    assert_eq!(offset_shape.segments[1].length, 17.0);
}

#[test]
fn test_toolset_validation() {
    let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
    let holder = Holder::new(
        "HOLDER-C".to_string(),
        vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
    );

    let toolset = ToolSet::new(
        "TS-001".to_string(),
        "Flat 10 Set".to_string(),
        tool,
        holder,
        75.0,
        40.0,
    )
    .with_shank_diameter(10.0)
    .with_shank_length(25.0)
    .with_reference_point(ToolSetReferencePoint::Gauge);

    assert!(toolset.validate_parameters());
    assert_eq!(toolset.reference_point, ToolSetReferencePoint::Gauge);
}

#[test]
fn test_toolset_validation_rejects_invalid_lengths() {
    let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
    let holder = Holder::new(
        "HOLDER-D".to_string(),
        vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
    );

    let toolset = ToolSet::new(
        "TS-002".to_string(),
        "Invalid Set".to_string(),
        tool,
        holder,
        40.0,
        50.0,
    )
    .with_shank_diameter(10.0)
    .with_shank_length(25.0);

    assert!(!toolset.validate_parameters());
}

#[test]
fn test_toolset_validation_rejects_invalid_shank_length() {
    let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
    let holder = Holder::new(
        "HOLDER-E".to_string(),
        vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
    );

    let valid_unspecified = ToolSet::new(
        "TS-003".to_string(),
        "Valid Unspecified Shank Length".to_string(),
        tool.clone(),
        holder.clone(),
        70.0,
        40.0,
    )
    .with_shank_diameter(10.0)
    .with_shank_length(0.0);
    assert!(valid_unspecified.validate_parameters());

    let invalid_equal_stickout = ToolSet::new(
        "TS-004".to_string(),
        "Invalid Equal Stickout Shank Length".to_string(),
        tool.clone(),
        holder.clone(),
        70.0,
        40.0,
    )
    .with_shank_diameter(10.0)
    .with_shank_length(40.0);
    assert!(!invalid_equal_stickout.validate_parameters());

    let invalid_too_long = ToolSet::new(
        "TS-005".to_string(),
        "Invalid Too Long Shank Length".to_string(),
        tool,
        holder,
        70.0,
        40.0,
    )
    .with_shank_diameter(10.0)
    .with_shank_length(45.0);
    assert!(!invalid_too_long.validate_parameters());
}
