use super::*;

#[test]
fn test_line_segment_creation() {
    let segment = PathSegment::new_line(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 0.0),
        SegmentType::Cutting { feed_rate: 500.0 },
    );

    assert!(segment.is_cutting());
    assert!(!segment.is_rapid());
    assert_eq!(segment.length(), 10.0);
    assert_eq!(segment.end_point(), Point3D::new(10.0, 0.0, 0.0));
}

#[test]
fn test_arc_segment_creation() {
    let segment = PathSegment::new_arc(
        Point3D::new(5.0, 0.0, 0.0),
        Point3D::new(0.0, 5.0, 0.0),
        Point3D::new(0.0, 0.0, 0.0),
        ArcDirection::CounterClockwise,
        SegmentType::Rapid,
    );

    assert!(segment.is_rapid());
    assert_eq!(segment.end_point(), Point3D::new(0.0, 5.0, 0.0));

    let expected_length = 5.0 * std::f64::consts::PI / 2.0;
    assert!((segment.length() - expected_length).abs() < 0.001);
}

#[test]
fn test_arc_segment_180_degrees() {
    let segment = PathSegment::new_arc(
        Point3D::new(10.0, 0.0, 0.0),
        Point3D::new(-10.0, 0.0, 0.0),
        Point3D::new(0.0, 0.0, 0.0),
        ArcDirection::Clockwise,
        SegmentType::Cutting { feed_rate: 500.0 },
    );

    let expected_length = 10.0 * std::f64::consts::PI;
    assert!((segment.length() - expected_length).abs() < 0.001);
}

#[test]
fn test_retract_and_pass_retract() {
    let retract = PathSegment::new_line(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(0.0, 0.0, 5.0),
        SegmentType::Retract { feed_rate: 300.0 },
    );
    assert_eq!(retract.length(), 5.0);

    let pass_retract = PathSegment::new_line(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(0.0, 0.0, 1.0),
        SegmentType::PassRetract { feed_rate: 200.0 },
    );
    assert_eq!(pass_retract.length(), 1.0);
}

#[test]
fn test_contour_level_path() {
    let segments = vec![
        PathSegment::new_line(
            Point3D::new(0.0, 0.0, -5.0),
            Point3D::new(10.0, 0.0, -5.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        PathSegment::new_line(
            Point3D::new(10.0, 0.0, -5.0),
            Point3D::new(10.0, 10.0, -5.0),
            SegmentType::Rapid,
        ),
    ];

    let contour = ContourLevelPath::new(0, -5.0, segments);

    assert_eq!(contour.level_index, 0);
    assert_eq!(contour.z_level, -5.0);
    assert_eq!(contour.cutting_segments().len(), 1);
    assert_eq!(contour.rapid_segments().len(), 1);
    assert_eq!(contour.total_length(), 20.0);
    assert_eq!(contour.cutting_length(), 10.0);
}

#[test]
fn test_tool_path() {
    let contours = vec![
        ContourLevelPath::new(0, -5.0, vec![]),
        ContourLevelPath::new(1, -10.0, vec![]),
    ];

    let toolpath = ToolPath::new(
        "tool1".to_string(),
        CuttingDirection::Down,
        vec![],
        contours,
        vec![],
    );

    assert_eq!(toolpath.tool_id, "tool1");
    assert_eq!(toolpath.cutting_direction, CuttingDirection::Down);
    assert_eq!(toolpath.level_count(), 2);
}

#[test]
fn test_tool_pose_and_machine_axis_value_creation() {
    let machine_axes = vec![
        MachineAxisValue::new("C".to_string(), MachineAxisKind::Rotary, 30.0),
        MachineAxisValue::new("U".to_string(), MachineAxisKind::Linear, 12.5),
    ];
    let pose = ToolPose::new(
        Point3D::new(1.0, 2.0, 3.0),
        Vector3D::new(0.0, 0.0, -1.0),
        Some(machine_axes.clone()),
    );

    assert_eq!(pose.position, Point3D::new(1.0, 2.0, 3.0));
    assert_eq!(pose.process_axis, Vector3D::new(0.0, 0.0, -1.0));
    assert_eq!(pose.machine_axes, Some(machine_axes));
}

#[test]
fn test_toolpath_kinematic_meta_classification() {
    let three_axis = ToolPathKinematicMeta::three_axis_position_only();
    assert_eq!(
        three_axis,
        ToolPathKinematicMeta::new(
            MachineConfigurationClass::ThreeAxis,
            KinematicMode::PureThreeAxis,
            PoseDataPolicy::PositionOnlyCompatible,
        )
    );

    let continuous_four_axis = ToolPathKinematicMeta::new(
        MachineConfigurationClass::FourAxis,
        KinematicMode::ContinuousFourAxis,
        PoseDataPolicy::PoseLayerOptional,
    );
    assert_eq!(
        continuous_four_axis.configuration_class,
        MachineConfigurationClass::FourAxis
    );
    assert_eq!(
        continuous_four_axis.kinematic_mode,
        KinematicMode::ContinuousFourAxis
    );
}

#[test]
fn test_three_axis_position_only_policy_behavior() {
    let three_axis = ToolPathKinematicMeta::three_axis_position_only();

    assert!(three_axis.is_position_only_compatible());
    assert!(three_axis.allows_optional_pose_layer());
    assert!(!three_axis.requires_pose_layer());
}

#[test]
fn test_toolpath_kinematic_meta_matrix() {
    let cases = [
        (
            "3-axis",
            ToolPathKinematicMeta::new(
                MachineConfigurationClass::ThreeAxis,
                KinematicMode::PureThreeAxis,
                PoseDataPolicy::PositionOnlyCompatible,
            ),
            true,
            true,
            false,
        ),
        (
            "4-axis continuous",
            ToolPathKinematicMeta::new(
                MachineConfigurationClass::FourAxis,
                KinematicMode::ContinuousFourAxis,
                PoseDataPolicy::PoseLayerOptional,
            ),
            false,
            true,
            false,
        ),
        (
            "3+2 indexed",
            ToolPathKinematicMeta::new(
                MachineConfigurationClass::FiveAxisOrMore,
                KinematicMode::IndexedMultiAxis,
                PoseDataPolicy::PoseLayerOptional,
            ),
            false,
            true,
            false,
        ),
        (
            "5-axis continuous",
            ToolPathKinematicMeta::new(
                MachineConfigurationClass::FiveAxisOrMore,
                KinematicMode::ContinuousFiveAxis,
                PoseDataPolicy::PoseLayerRequired,
            ),
            false,
            false,
            true,
        ),
    ];

    for (name, meta, expected_position_only, expected_optional, expected_required) in cases {
        assert_eq!(
            meta.is_position_only_compatible(),
            expected_position_only,
            "unexpected position_only result for {name}"
        );
        assert_eq!(
            meta.allows_optional_pose_layer(),
            expected_optional,
            "unexpected optional_pose result for {name}"
        );
        assert_eq!(
            meta.requires_pose_layer(),
            expected_required,
            "unexpected required_pose result for {name}"
        );
    }
}

#[test]
fn test_pose_annotated_segment_creation() {
    let segment = PathSegment::new_line(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(5.0, 0.0, 0.0),
        SegmentType::Cutting { feed_rate: 400.0 },
    );
    let start_pose = ToolPose::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, -1.0),
        None,
    );
    let end_pose = ToolPose::new(
        Point3D::new(5.0, 0.0, 0.0),
        Vector3D::new(0.1, 0.0, -0.9),
        Some(vec![MachineAxisValue::new(
            "A".to_string(),
            MachineAxisKind::Rotary,
            12.0,
        )]),
    );
    let pose_span = ToolPoseSpan::new(
        start_pose.clone(),
        end_pose.clone(),
        PoseInterpolationPolicy::MachineConstrained,
    );
    let annotated = PoseAnnotatedSegment::new(segment.clone(), pose_span.clone());

    assert_eq!(annotated.segment, segment);
    assert_eq!(annotated.pose_span, pose_span);
    assert_eq!(annotated.pose_span.start_pose, start_pose);
    assert_eq!(annotated.pose_span.end_pose, end_pose);
}
