use std::io::Cursor;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ContourLevelPath, CuttingDirection, SegmentType, Tool,
    ToolPath, read_toolpath_artifact_v1, write_toolpath_payload_v1,
};
use geo_algorithms::{Aabb3D, Point3D};
use geo_algorithms::{LineSegment3D, octree::VoxelOctree};
use job_runtime::{JobManager, JobSpec, JobStatus, JobType, RetryPolicy};

use crate::{
    CamJobExecutorAdapter, CamWorkflowError, CamWorkflowSubmitter, CuttingSimulator,
    SimulationError, SnapshotInterval,
};

fn cam_spec(input: &str) -> JobSpec {
    JobSpec {
        job_type: JobType::CamProcess,
        input_ref: input.to_string(),
        timeout_secs: 30,
        retry_policy: RetryPolicy::default(),
    }
}

fn sim_spec(input: &str) -> JobSpec {
    JobSpec {
        job_type: JobType::CuttingSimulation,
        input_ref: input.to_string(),
        timeout_secs: 30,
        retry_policy: RetryPolicy::default(),
    }
}

fn roundtrip_toolpath_via_artifact(toolpath: &ToolPath<f64>) -> ToolPath<f64> {
    let mut payload_bytes = Vec::new();
    write_toolpath_payload_v1(&mut payload_bytes, toolpath).unwrap();

    let header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload_bytes.len() as u64);
    let mut bytes = Vec::new();
    header.write_to(&mut bytes).unwrap();
    bytes.extend_from_slice(&payload_bytes);

    let mut cursor = Cursor::new(bytes);
    let (read_header, read_toolpath) = read_toolpath_artifact_v1(&mut cursor).unwrap();
    assert_eq!(read_header.kind, ArtifactKind::ToolPath);
    read_toolpath
}

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

    let source_toolpath = ToolPath::new(
        "flat-tool".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 10.0, vec![cutting])],
        vec![],
    );
    let toolpath = roundtrip_toolpath_via_artifact(&source_toolpath);

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

    let source_toolpath = ToolPath::new(
        "ball-tool".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 10.0, vec![cutting])],
        vec![],
    );
    let toolpath = roundtrip_toolpath_via_artifact(&source_toolpath);

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

    let segment1 =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(60.0, 0.0, 0.0)).unwrap();
    let segment2 =
        LineSegment3D::new(Point3D::new(60.0, 0.0, 0.0), Point3D::new(100.0, 0.0, 0.0)).unwrap();

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

    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(40.0, 0.0, 0.0)).unwrap();

    simulator.simulate_line_segments(&[segment], 2.0);
    let exports = simulator.snapshot_exports_f64();

    assert!(!exports.is_empty());
    let first = exports[0];
    assert_eq!(first.segment_index, 0);
    assert!((0.0..=1.0).contains(&first.segment_t));
    assert!(first.remaining_volume_mm3.is_finite());
}

#[test]
fn test_job_adapter_runs_cam_and_sim_jobs() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(JobSpec {
        job_type: JobType::CamProcess,
        input_ref: "input://cam/sample".to_string(),
        timeout_secs: 30,
        retry_policy: RetryPolicy::default(),
    });

    let sim_id = manager.submit(JobSpec {
        job_type: JobType::CuttingSimulation,
        input_ref: "input://sim/sample".to_string(),
        timeout_secs: 30,
        retry_policy: RetryPolicy::default(),
    });

    manager.execute_with(cam_id, &adapter).unwrap();
    manager.execute_with(sim_id, &adapter).unwrap();

    let cam = manager.get(cam_id).unwrap();
    let sim = manager.get(sim_id).unwrap();

    assert_eq!(cam.status, JobStatus::Succeeded);
    assert_eq!(sim.status, JobStatus::Succeeded);
    assert!(
        manager
            .active_result_ref(cam_id)
            .unwrap()
            .unwrap_or_default()
            .starts_with("result://cam/")
    );
    assert!(
        manager
            .active_result_ref(sim_id)
            .unwrap()
            .unwrap_or_default()
            .starts_with("result://sim/")
    );
}

#[test]
fn test_workflow_rejects_second_simulation_under_same_cam() {
    let mut manager = JobManager::new();
    let mut workflow = CamWorkflowSubmitter::new(&mut manager);

    let cam_id = workflow
        .submit_cam_process(cam_spec("input://cam/sample"))
        .unwrap();
    workflow
        .submit_cutting_simulation(cam_id, sim_spec("input://sim/first"))
        .unwrap();

    let second = workflow.submit_cutting_simulation(cam_id, sim_spec("input://sim/second"));

    assert!(matches!(
        second,
        Err(CamWorkflowError::SimulationAlreadyExists { parent_cam_job_id }) if parent_cam_job_id == cam_id
    ));
}

#[test]
fn test_workflow_rejects_simulation_without_cam_parent() {
    let mut manager = JobManager::new();
    let mut workflow = CamWorkflowSubmitter::new(&mut manager);

    let non_cam_parent = workflow
        .submit_cam_process(cam_spec("input://cam/parent"))
        .unwrap();
    let sim_parent = workflow
        .submit_cutting_simulation(non_cam_parent, sim_spec("input://sim/parent"))
        .unwrap();

    let result = workflow.submit_cutting_simulation(sim_parent, sim_spec("input://sim/orphan"));

    assert!(matches!(
        result,
        Err(CamWorkflowError::InvalidParentType {
            expected: JobType::CamProcess,
            actual: JobType::CuttingSimulation,
        })
    ));
}

#[test]
fn test_workflow_rejects_child_under_simulation() {
    let mut manager = JobManager::new();
    let mut workflow = CamWorkflowSubmitter::new(&mut manager);

    let cam_id = workflow
        .submit_cam_process(cam_spec("input://cam/pipe"))
        .unwrap();
    let sim_id = workflow
        .submit_cutting_simulation(cam_id, sim_spec("input://sim/pipe"))
        .unwrap();

    let result = workflow.submit_child_under(sim_id, cam_spec("input://cam/after-sim"));

    assert!(matches!(
        result,
        Err(CamWorkflowError::SimulationCannotHaveChildren { simulation_job_id }) if simulation_job_id == sim_id
    ));
}

#[test]
fn test_job_adapter_rejects_invalid_input_ref() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let id = manager.submit(JobSpec {
        job_type: JobType::CuttingSimulation,
        input_ref: "input://cam/wrong".to_string(),
        timeout_secs: 30,
        retry_policy: RetryPolicy::default(),
    });

    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("invalid input_ref")
    );
}
