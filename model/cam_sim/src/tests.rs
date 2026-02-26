use cam_core::{ContourLevelPath, CuttingDirection, SegmentType, Tool, ToolPath};
use geo_algorithms::{Aabb3D, Point3D};
use geo_algorithms::{LineSegment3D, octree::VoxelOctree};

use crate::{CuttingSimulator, SimulationError, SnapshotInterval};

#[test]
fn test_simulate_line_segments_removes_material() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel = VoxelOctree::new(bounds, 5);
    let initial = voxel.remaining_volume();

    let mut simulator = CuttingSimulator::new(voxel, SnapshotInterval::default());

    let segment = LineSegment3D::new(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(90.0, 10.0, 10.0),
    )
    .unwrap();

    simulator.simulate_line_segments(&[segment], 5.0);

    let remaining = simulator.voxel_tree().remaining_volume();
    assert!(remaining < initial);
    assert!(!simulator.snapshots().is_empty());
}

#[test]
fn test_auto_distance_interval_generates_snapshots() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel = VoxelOctree::new(bounds, 4);

    let mut simulator = CuttingSimulator::new(
        voxel,
        SnapshotInterval::ByAutoDistance {
            target_count: 5,
            min_interval_mm: 1.0,
            include_segment_endpoints: false,
        },
    );

    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(50.0, 0.0, 0.0)).unwrap();

    simulator.simulate_line_segments(&[segment], 2.0);
    assert!(!simulator.snapshots().is_empty());
}

#[test]
fn test_simulate_toolpath_flat_end_mill() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel = VoxelOctree::new(bounds, 4);
    let mut simulator = CuttingSimulator::new(voxel, SnapshotInterval::default());

    let cutting = cam_core::PathSegment::new_line(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(80.0, 10.0, 10.0),
        SegmentType::Cutting { feed_rate: 300.0 },
    );

    let toolpath = ToolPath::new(
        "flat-tool".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 10.0, vec![cutting])],
        vec![],
    );

    let tool = Tool::flat_end_mill("flat-tool".to_string(), 10.0, 30.0);
    let result = simulator.simulate(&toolpath, &tool);

    assert!(result.is_ok());
    assert!(!simulator.snapshots().is_empty());
}

#[test]
fn test_simulate_toolpath_non_flat_is_rejected() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel = VoxelOctree::new(bounds, 4);
    let mut simulator = CuttingSimulator::new(voxel, SnapshotInterval::default());

    let cutting = cam_core::PathSegment::new_line(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(80.0, 10.0, 10.0),
        SegmentType::Cutting { feed_rate: 300.0 },
    );

    let toolpath = ToolPath::new(
        "ball-tool".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 10.0, vec![cutting])],
        vec![],
    );

    let tool = Tool::ball_end_mill("ball-tool".to_string(), 10.0, 30.0);
    let result = simulator.simulate(&toolpath, &tool);

    assert_eq!(result, Err(SimulationError::UnsupportedToolType));
}

#[test]
fn test_include_segment_endpoints_changes_snapshot_count() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let segment1 = LineSegment3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(60.0, 0.0, 0.0),
    )
    .unwrap();
    let segment2 = LineSegment3D::new(
        Point3D::new(60.0, 0.0, 0.0),
        Point3D::new(100.0, 0.0, 0.0),
    )
    .unwrap();

    let mut with_endpoints = CuttingSimulator::new(
        VoxelOctree::new(bounds, 4),
        SnapshotInterval::ByDistance {
            interval_mm: 25.0,
            include_segment_endpoints: true,
        },
    );

    let mut without_endpoints = CuttingSimulator::new(
        VoxelOctree::new(bounds, 4),
        SnapshotInterval::ByDistance {
            interval_mm: 25.0,
            include_segment_endpoints: false,
        },
    );

    with_endpoints.simulate_line_segments(&[segment1, segment2], 3.0);
    without_endpoints.simulate_line_segments(&[segment1, segment2], 3.0);

    assert!(with_endpoints.snapshots().len() > without_endpoints.snapshots().len());
}

#[test]
fn test_snapshot_exports_f64_maps_snapshot_fields() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel = VoxelOctree::new(bounds, 4);
    let mut simulator = CuttingSimulator::new(voxel, SnapshotInterval::default());

    let segment = LineSegment3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(40.0, 0.0, 0.0),
    )
    .unwrap();

    simulator.simulate_line_segments(&[segment], 2.0);
    let exports = simulator.snapshot_exports_f64();

    assert!(!exports.is_empty());
    let first = exports[0];
    assert_eq!(first.segment_index, 0);
    assert!((0.0..=1.0).contains(&first.segment_t));
    assert!(first.remaining_volume_mm3.is_finite());
}
