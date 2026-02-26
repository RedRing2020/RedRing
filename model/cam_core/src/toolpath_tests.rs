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
