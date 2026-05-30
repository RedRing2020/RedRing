use std::io::Cursor;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ContourLevelPath, CuttingDirection, SegmentType, Tool,
    ToolPath, read_toolpath_artifact_v1, write_toolpath_payload_v1,
};
use geo_algorithms::{Aabb3D, Point3D};
use geo_algorithms::{LineSegment3D, octree::VoxelOctree};
use job_runtime::{JobEvent, JobManager, JobRelation, JobSpec, JobStatus, JobType, RetryPolicy};

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

fn nc_post_from_cam_spec(input: &str) -> JobSpec {
    JobSpec {
        job_type: JobType::NcPostFromCam,
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
fn test_simulate_toolpath_ball_end_mill() {
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

    assert!(result.is_ok());
    assert!(!simulator.snapshots().is_empty());
}

#[test]
fn test_simulate_toolpath_radius_end_mill_is_rejected() {
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
        "radius-tool".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 10.0, vec![cutting])],
        vec![],
    );
    let toolpath = roundtrip_toolpath_via_artifact(&source_toolpath);

    let tool = Tool::radius_end_mill("radius-tool".to_string(), 10.0, 1.0, 30.0);
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

    let sim_id = manager
        .submit_with_relation(
            JobSpec {
                job_type: JobType::CuttingSimulation,
                input_ref: format!("result://cam/{}/ok", cam_id.0),
                timeout_secs: 30,
                retry_policy: RetryPolicy::default(),
            },
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();

    manager.execute_with(cam_id, &adapter).unwrap();
    manager.execute_with(sim_id, &adapter).unwrap();

    let cam = manager.get(cam_id).unwrap();
    let sim = manager.get(sim_id).unwrap();

    assert_eq!(cam.status, JobStatus::Succeeded);
    assert_eq!(sim.status, JobStatus::Succeeded);
    assert_eq!(manager.active_result_ref(cam_id).unwrap(), None);
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
fn test_workflow_rejects_nc_post_submission_with_wrong_job_type() {
    let mut manager = JobManager::new();
    let mut workflow = CamWorkflowSubmitter::new(&mut manager);

    let cam_id = workflow
        .submit_cam_process(cam_spec("input://cam/for-nc-post"))
        .unwrap();

    let result = workflow.submit_nc_post_from_cam(cam_id, sim_spec("input://sim/wrong-type"));

    assert!(matches!(
        result,
        Err(CamWorkflowError::InvalidJobType {
            expected: JobType::NcPostFromCam,
            actual: JobType::CuttingSimulation,
        })
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

#[test]
fn test_job_adapter_rejects_cutting_sim_parent_mismatch() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/sample"));

    let id = manager
        .submit_with_relation(
            sim_spec("result://cam/999/ok"),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("input_ref cam job id mismatch")
    );
}

#[test]
fn test_job_adapter_rejects_cutting_sim_non_ok_suffix() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/sample"));

    let id = manager
        .submit_with_relation(
            sim_spec(&format!("result://cam/{}/artifact-read-failed", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("invalid cam result_ref suffix for cutting simulation")
    );
}

#[test]
fn test_job_adapter_surfaces_cutting_sim_artifact_read_failure_from_parent_artifact() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/artifact-read-failed"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let sim_id = manager
        .submit_with_relation(
            sim_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(sim_id, &adapter).unwrap();

    let sim = manager.get(sim_id).unwrap();
    assert_eq!(sim.status, JobStatus::Failed);
    assert!(
        sim.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("failed to read cutting simulation input artifact")
    );

    let events = manager.take_events();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            JobEvent::Completed {
                job_id,
                status: JobStatus::Failed,
                log_ref: Some(log_ref),
                ..
            } if *job_id == sim_id && log_ref.ends_with("/artifact-read-failed")
        )
    }));
}

#[test]
fn test_job_adapter_surfaces_cutting_sim_execution_failure_from_parent_artifact() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/sim-failure"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let sim_id = manager
        .submit_with_relation(
            sim_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(sim_id, &adapter).unwrap();

    let sim = manager.get(sim_id).unwrap();
    assert_eq!(sim.status, JobStatus::Failed);
    assert!(
        sim.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("failed to run cutting simulation")
    );

    let events = manager.take_events();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            JobEvent::Completed {
                job_id,
                status: JobStatus::Failed,
                log_ref: Some(log_ref),
                ..
            } if *job_id == sim_id && log_ref.ends_with("/sim-failure")
        )
    }));
}

#[test]
fn test_job_adapter_rejects_cutting_sim_without_parent() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let id = manager.submit(sim_spec("result://cam/42/ok"));
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("requires parent_job_id")
    );
}

#[test]
fn test_job_adapter_runs_nc_post_from_cam_with_toolpath_artifact() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/for-nc-post"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let id = manager
        .submit_with_relation(
            nc_post_from_cam_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
    assert!(
        manager
            .active_result_ref(id)
            .unwrap()
            .unwrap_or_default()
            .starts_with("result://nc-post/")
    );
}

#[test]
fn test_job_adapter_rejects_nc_post_from_cam_kind_mismatch() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/kind-mismatch"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let id = manager
        .submit_with_relation(
            nc_post_from_cam_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("artifact kind mismatch")
    );
}

#[test]
fn test_job_adapter_surfaces_nc_post_from_cam_version_incompatibility() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/version-mismatch"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let id = manager
        .submit_with_relation(
            nc_post_from_cam_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("failed to read artifact binary")
    );
}

#[test]
fn test_job_adapter_surfaces_nc_post_from_cam_artifact_read_failure() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/artifact-read-failed"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let nc_post_id = manager
        .submit_with_relation(
            nc_post_from_cam_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(nc_post_id, &adapter).unwrap();

    let nc_post = manager.get(nc_post_id).unwrap();
    assert_eq!(nc_post.status, JobStatus::Failed);
    assert!(
        nc_post
            .last_error
            .as_deref()
            .unwrap_or_default()
            .contains("failed to read artifact binary")
    );

    let events = manager.take_events();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            JobEvent::Completed {
                job_id,
                status: JobStatus::Failed,
                log_ref: Some(log_ref),
                ..
            } if *job_id == nc_post_id && log_ref.ends_with("/artifact-read-failed")
        )
    }));
}

#[test]
fn test_job_adapter_rejects_nc_post_from_cam_parent_mismatch() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/for-nc-post"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let id = manager
        .submit_with_relation(
            nc_post_from_cam_spec("result://cam/999/ok"),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("input_ref cam job id mismatch")
    );
}

#[test]
fn test_job_adapter_nc_post_can_use_superseded_cam_output_history() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter;

    let cam_id = manager.submit(cam_spec("input://cam/for-nc-post"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let sim_id = manager
        .submit_with_relation(
            sim_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(sim_id, &adapter).unwrap();

    let nc_post_id = manager
        .submit_with_relation(
            nc_post_from_cam_spec(&format!("result://cam/{}/ok", cam_id.0)),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();
    manager.execute_with(nc_post_id, &adapter).unwrap();

    let job = manager.get(nc_post_id).unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
}

// --- フラット vs ボール除去比較テスト ---

/// 同一経路・同一径で、フラットとボールが異なる体積を除去することを検証する。
///
/// - フラット: `remove_material_swept_cylinder`（掃引円柱）
/// - ボール: Z+radius 補正後に `remove_material_capsule`（掃引カプセル）
///
/// 形状が異なるため、VoxelOctree 上の残存体積に差が生じる。
#[test]
fn test_flat_vs_ball_end_mill_remaining_volume_differs() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let cutting_flat = cam_core::PathSegment::new_line(
        Point3D::new(10.0, 50.0, 50.0),
        Point3D::new(90.0, 50.0, 50.0),
        SegmentType::Cutting { feed_rate: 300.0 },
    );
    let cutting_ball = cutting_flat.clone();

    let toolpath_flat = roundtrip_toolpath_via_artifact(&ToolPath::new(
        "flat".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 50.0, vec![cutting_flat])],
        vec![],
    ));
    let toolpath_ball = roundtrip_toolpath_via_artifact(&ToolPath::new(
        "ball".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 50.0, vec![cutting_ball])],
        vec![],
    ));

    let initial_volume = VoxelOctree::<f64>::new(bounds, 5).remaining_volume();

    let mut sim_flat =
        CuttingSimulator::new(VoxelOctree::new(bounds, 5), SnapshotInterval::default());
    sim_flat
        .simulate(
            &toolpath_flat,
            &Tool::flat_end_mill("flat".to_string(), 10.0, 30.0),
        )
        .unwrap();

    let mut sim_ball =
        CuttingSimulator::new(VoxelOctree::new(bounds, 5), SnapshotInterval::default());
    sim_ball
        .simulate(
            &toolpath_ball,
            &Tool::ball_end_mill("ball".to_string(), 10.0, 30.0),
        )
        .unwrap();

    let flat_remaining = sim_flat.voxel_tree().remaining_volume();
    let ball_remaining = sim_ball.voxel_tree().remaining_volume();

    assert!(
        flat_remaining < initial_volume,
        "フラットエンドミルが材料を除去していない: remaining={flat_remaining}, initial={initial_volume}"
    );
    assert!(
        ball_remaining < initial_volume,
        "ボールエンドミルが材料を除去していない: remaining={ball_remaining}, initial={initial_volume}"
    );
    assert_ne!(
        flat_remaining, ball_remaining,
        "フラット(swept_cylinder)とボール(capsule+Z補正)は異なる形状で除去するため残存体積が異なるはず: \
         flat={flat_remaining}, ball={ball_remaining}"
    );
}

/// ボールエンドミルの Z+radius 補正が正しく機能することを検証する。
///
/// 先端基準の水平経路（Z=10）に対して、ボールは中心経路を Z=10+radius=15 に補正して
/// カプセル除去を行う。フラットエンドミルとは異なる位置を除去するため、残存体積が異なる。
#[test]
fn test_ball_end_mill_z_offset_removes_material_at_shifted_z() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let segment_at_low_z = |seg_type: SegmentType<f64>| {
        cam_core::PathSegment::new_line(
            Point3D::new(10.0, 50.0, 10.0),
            Point3D::new(90.0, 50.0, 10.0),
            seg_type,
        )
    };

    let cutting_flat = segment_at_low_z(SegmentType::Cutting { feed_rate: 300.0 });
    let cutting_ball = segment_at_low_z(SegmentType::Cutting { feed_rate: 300.0 });

    let toolpath_flat = roundtrip_toolpath_via_artifact(&ToolPath::new(
        "flat-z".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 10.0, vec![cutting_flat])],
        vec![],
    ));
    let toolpath_ball = roundtrip_toolpath_via_artifact(&ToolPath::new(
        "ball-z".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 10.0, vec![cutting_ball])],
        vec![],
    ));

    let mut sim_flat =
        CuttingSimulator::new(VoxelOctree::new(bounds, 5), SnapshotInterval::default());
    sim_flat
        .simulate(
            &toolpath_flat,
            &Tool::flat_end_mill("flat-z".to_string(), 10.0, 30.0),
        )
        .unwrap();

    let mut sim_ball =
        CuttingSimulator::new(VoxelOctree::new(bounds, 5), SnapshotInterval::default());
    sim_ball
        .simulate(
            &toolpath_ball,
            &Tool::ball_end_mill("ball-z".to_string(), 10.0, 30.0),
        )
        .unwrap();

    let flat_remaining = sim_flat.voxel_tree().remaining_volume();
    let ball_remaining = sim_ball.voxel_tree().remaining_volume();

    assert_ne!(
        flat_remaining, ball_remaining,
        "Z=10 の水平経路でフラット(Z=10 で円柱)とボール(Z=15 でカプセル)は除去位置が異なるはず: \
         flat={flat_remaining}, ball={ball_remaining}"
    );
}
