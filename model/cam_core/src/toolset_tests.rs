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
    .with_shank_segments(vec![
        ShankSegment::cylinder(15.0, 10.0),
        ShankSegment::taper(10.0, 10.0, 12.0),
    ])
    .with_shank_interference_offset(ShankInterferenceOffset::new(0.1, 0.0))
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
    .with_shank_segments(vec![ShankSegment::cylinder(10.0, 10.0)]);

    assert!(!toolset.validate_parameters());
}

#[test]
fn test_toolset_validation_rejects_invalid_shank_definition() {
    let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
    let holder = Holder::new(
        "HOLDER-E".to_string(),
        vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
    );

    let valid = ToolSet::new(
        "TS-003".to_string(),
        "Valid Shank".to_string(),
        tool.clone(),
        holder.clone(),
        70.0,
        40.0,
    )
    .with_shank_segments(vec![ShankSegment::cylinder(20.0, 10.0)]);
    assert!(valid.validate_parameters());

    let invalid_missing_segments = ToolSet::new(
        "TS-004".to_string(),
        "Invalid Missing Shank".to_string(),
        tool.clone(),
        holder.clone(),
        70.0,
        40.0,
    );
    assert!(!invalid_missing_segments.validate_parameters());

    let invalid_too_long = ToolSet::new(
        "TS-005".to_string(),
        "Invalid Too Long Shank".to_string(),
        tool,
        holder,
        70.0,
        40.0,
    )
    .with_shank_segments(vec![
        ShankSegment::cylinder(30.0, 10.0),
        ShankSegment::cylinder(10.0, 12.0),
    ]);
    assert!(!invalid_too_long.validate_parameters());

    let valid_taper_with_bottom_disabled = ToolSet::new(
        "TS-006".to_string(),
        "Valid Taper Bottom Clearance Disabled".to_string(),
        Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0),
        Holder::new(
            "HOLDER-F".to_string(),
            vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
        ),
        70.0,
        40.0,
    )
    .with_shank_segments(vec![
        ShankSegment::cylinder(12.0, 10.0),
        ShankSegment::taper(8.0, 10.0, 12.0),
    ])
    .with_shank_interference_offset(ShankInterferenceOffset::new(0.1, 0.5));
    assert!(valid_taper_with_bottom_disabled.validate_parameters());
}

#[test]
fn test_shank_segment_kind_validation() {
    let invalid_cylinder = ShankSegment::taper(10.0, 10.0, 10.0);
    assert!(!invalid_cylinder.validate_parameters());

    let invalid_taper = ShankSegment::cylinder(10.0, 10.0);
    let mut invalid_taper = invalid_taper;
    invalid_taper.kind = ShankSegmentKind::Taper;
    assert!(!invalid_taper.validate_parameters());
}

#[test]
fn test_shank_interference_shape_applies_bottom_to_last_segment_only() {
    let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
    let holder = Holder::new(
        "HOLDER-F".to_string(),
        vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
    );

    let toolset = ToolSet::new(
        "TS-007".to_string(),
        "Shank Interference Shape".to_string(),
        tool,
        holder,
        70.0,
        40.0,
    )
    .with_shank_segments(vec![
        ShankSegment::cylinder(12.0, 10.0),
        ShankSegment::cylinder(8.0, 12.0),
    ])
    .with_shank_interference_offset(ShankInterferenceOffset::new(0.1, 0.5));

    let offset_shape = toolset.shank_interference_shape();
    assert_eq!(offset_shape.len(), 2);
    assert_eq!(offset_shape[0].top_diameter, 10.2);
    assert_eq!(offset_shape[0].bottom_diameter, 10.2);
    assert_eq!(offset_shape[0].length, 12.0);
    assert_eq!(offset_shape[1].top_diameter, 12.2);
    assert_eq!(offset_shape[1].bottom_diameter, 12.2);
    assert_eq!(offset_shape[1].length, 8.5);
}

#[test]
fn test_shank_interference_shape_ignores_bottom_when_taper_is_present() {
    let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
    let holder = Holder::new(
        "HOLDER-G".to_string(),
        vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
    );

    let toolset = ToolSet::new(
        "TS-008".to_string(),
        "Shank Interference Shape Taper".to_string(),
        tool,
        holder,
        70.0,
        40.0,
    )
    .with_shank_segments(vec![
        ShankSegment::cylinder(12.0, 10.0),
        ShankSegment::taper(8.0, 10.0, 12.0),
    ])
    .with_shank_interference_offset(ShankInterferenceOffset::new(0.1, 0.5));

    let offset_shape = toolset.shank_interference_shape();
    assert_eq!(offset_shape.len(), 2);
    assert_eq!(offset_shape[0].length, 12.0);
    assert_eq!(offset_shape[1].length, 8.0);
}

#[test]
fn test_shank_interference_shape_ignores_bottom_when_tool_diameter_exceeds_last_shank_diameter() {
    let tool = Tool::new("EM12".to_string(), ToolType::FlatEndMill, 12.0, 0.0, 30.0);
    let holder = Holder::new(
        "HOLDER-H".to_string(),
        vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
    );

    let toolset = ToolSet::new(
        "TS-009".to_string(),
        "Shank Interference Shape Diameter Guard".to_string(),
        tool,
        holder,
        70.0,
        40.0,
    )
    .with_shank_segments(vec![
        ShankSegment::cylinder(12.0, 12.0),
        ShankSegment::cylinder(8.0, 10.0),
    ])
    .with_shank_interference_offset(ShankInterferenceOffset::new(0.1, 0.5));

    assert!(toolset.validate_parameters());

    let offset_shape = toolset.shank_interference_shape();
    assert_eq!(offset_shape.len(), 2);
    assert_eq!(offset_shape[0].length, 12.0);
    assert_eq!(offset_shape[1].length, 8.0);
}
