use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ContourLevelPath, CuttingDirection, SegmentType, Tool,
    ToolPath, read_toolpath_artifact_v1, write_toolpath_payload_v1,
};
use geo_algorithms::{Aabb3D, Point3D};
use geo_algorithms::{LineSegment3D, octree::VoxelOctree};
use job_runtime::{JobEvent, JobManager, JobRelation, JobSpec, JobStatus, JobType, RetryPolicy};

use crate::{
    CamJobExecutorAdapter, CamWorkflowError, CamWorkflowSubmitter, CuttingSimulator,
    ExactToolPrimitive, ExactWorkModel, PrimitiveSetExactWork, SimulationError, SnapshotInterval,
    collect_toolpath_line_segments,
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

mod demo_symbol_guard {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../test_helpers/demo_symbol_guard.rs"
    ));
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

#[derive(Debug, Clone, Copy)]
struct GateMetrics {
    removed_voxel: f64,
    removed_exact: f64,
    gap: f64,
    boundary_disagreement_rate: f64,
    boundary_sample_count: usize,
    boundary_exact_only_count: usize,
    boundary_voxel_only_count: usize,
    elapsed_voxel_ms: f64,
    elapsed_exact_ms: f64,
    elapsed_ratio: f64,
}

const GATE_SAMPLE_PITCH: f64 = 2.0;
const GATE_MAX_AXIS_SAMPLES: usize = 128;
const GATE_TARGET_GAP: f64 = 0.15;
const GATE_TARGET_BOUNDARY: f64 = 0.10;
const GATE_TARGET_ELAPSED_RATIO: f64 = 3.0;

fn gate_axis_sample_count(span: f64, sample_pitch: f64) -> usize {
    if !span.is_finite() || span <= 0.0 || !sample_pitch.is_finite() || sample_pitch <= 0.0 {
        return 1;
    }
    ((span / sample_pitch).ceil() as usize).clamp(1, GATE_MAX_AXIS_SAMPLES)
}

#[derive(Debug, Clone)]
struct SolidBoundsSpatialIndex {
    bucket_size: f64,
    buckets: HashMap<(i32, i32, i32), Vec<usize>>,
    solid_bounds: Vec<Aabb3D<f64>>,
}

impl SolidBoundsSpatialIndex {
    fn from_bounds(solid_bounds: Vec<Aabb3D<f64>>, bucket_size: f64) -> Self {
        let size = if bucket_size.is_finite() && bucket_size > 0.0 {
            bucket_size
        } else {
            1.0
        };

        let mut buckets: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();
        for (index, bounds) in solid_bounds.iter().enumerate() {
            let min = bounds.min();
            let max = bounds.max();
            let min_ix = floor_bucket(min.x(), size);
            let min_iy = floor_bucket(min.y(), size);
            let min_iz = floor_bucket(min.z(), size);
            let max_ix = floor_bucket(max.x(), size);
            let max_iy = floor_bucket(max.y(), size);
            let max_iz = floor_bucket(max.z(), size);

            for ix in min_ix..=max_ix {
                for iy in min_iy..=max_iy {
                    for iz in min_iz..=max_iz {
                        buckets.entry((ix, iy, iz)).or_default().push(index);
                    }
                }
            }
        }

        Self {
            bucket_size: size,
            buckets,
            solid_bounds,
        }
    }

    fn contains_material_at(&self, point: &Point3D<f64>) -> bool {
        let key = (
            floor_bucket(point.x(), self.bucket_size),
            floor_bucket(point.y(), self.bucket_size),
            floor_bucket(point.z(), self.bucket_size),
        );

        self.buckets.get(&key).is_some_and(|candidate_indexes| {
            candidate_indexes
                .iter()
                .any(|&idx| self.solid_bounds[idx].contains_point(point))
        })
    }
}

fn floor_bucket(value: f64, bucket_size: f64) -> i32 {
    (value / bucket_size).floor() as i32
}

fn is_voxel_boundary_point(
    bounds: &Aabb3D<f64>,
    voxel_index: &SolidBoundsSpatialIndex,
    point: &Point3D<f64>,
    center_state: bool,
    probe_offset: f64,
) -> bool {
    let offsets = [
        (probe_offset, 0.0, 0.0),
        (-probe_offset, 0.0, 0.0),
        (0.0, probe_offset, 0.0),
        (0.0, -probe_offset, 0.0),
        (0.0, 0.0, probe_offset),
        (0.0, 0.0, -probe_offset),
    ];

    offsets.into_iter().any(|(dx, dy, dz)| {
        let p = Point3D::new(point.x() + dx, point.y() + dy, point.z() + dz);
        bounds.contains_point(&p) && voxel_index.contains_material_at(&p) != center_state
    })
}

fn compute_boundary_disagreement_rate(
    bounds: &Aabb3D<f64>,
    sample_pitch: f64,
    boundary_band: f64,
    exact_conservative_margin: f64,
    exact_work: &PrimitiveSetExactWork,
    voxel_index: &SolidBoundsSpatialIndex,
) -> (f64, usize, usize, usize) {
    if sample_pitch <= 0.0 || !sample_pitch.is_finite() {
        return (0.0, 0, 0, 0);
    }

    let min = bounds.min();
    let max = bounds.max();
    let width = max.x() - min.x();
    let height = max.y() - min.y();
    let depth = max.z() - min.z();

    let x_samples = gate_axis_sample_count(width, sample_pitch);
    let y_samples = gate_axis_sample_count(height, sample_pitch);
    let z_samples = gate_axis_sample_count(depth, sample_pitch);

    let dx = width / (x_samples as f64);
    let dy = height / (y_samples as f64);
    let dz = depth / (z_samples as f64);

    let probe_offset = if boundary_band.is_finite() && boundary_band > 0.0 {
        boundary_band * 0.5
    } else {
        sample_pitch * 0.5
    };
    let mut boundary_points = 0usize;
    let mut disagreements = 0usize;
    let mut exact_only = 0usize;
    let mut voxel_only = 0usize;
    for ix in 0..x_samples {
        let x = min.x() + ((ix as f64) + 0.5) * dx;
        for iy in 0..y_samples {
            let y = min.y() + ((iy as f64) + 0.5) * dy;
            for iz in 0..z_samples {
                let z = min.z() + ((iz as f64) + 0.5) * dz;
                let point = Point3D::new(x, y, z);
                let dist = exact_work.nearest_removed_surface_distance(&point);
                let voxel_has_material = voxel_index.contains_material_at(&point);

                // Only probe voxel boundary neighbors for points outside the exact boundary band.
                if dist > boundary_band
                    && !is_voxel_boundary_point(
                        bounds,
                        voxel_index,
                        &point,
                        voxel_has_material,
                        probe_offset,
                    )
                {
                    continue;
                }

                boundary_points += 1;
                let exact_has_material =
                    exact_work.contains_material_at(&point) && dist > exact_conservative_margin;
                if exact_has_material != voxel_has_material {
                    disagreements += 1;
                    if exact_has_material {
                        exact_only += 1;
                    } else {
                        voxel_only += 1;
                    }
                }
            }
        }
    }

    if boundary_points == 0 {
        return (0.0, 0, 0, 0);
    }

    (
        disagreements as f64 / boundary_points as f64,
        boundary_points,
        exact_only,
        voxel_only,
    )
}

fn run_gate_case(toolpath: &ToolPath<f64>, tool: &Tool<f64>, sample_pitch: f64) -> GateMetrics {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let initial_volume = bounds.volume();

    let voxel_started = Instant::now();
    let mut simulator =
        CuttingSimulator::new(VoxelOctree::new(bounds, 5), SnapshotInterval::default());
    simulator
        .simulate(toolpath, tool)
        .expect("voxel simulation must succeed");
    let elapsed_voxel_ms = voxel_started.elapsed().as_secs_f64() * 1000.0;
    let voxel_pitch = simulator.voxel_tree().voxel_size_at_max_depth();
    let exact_sample_pitch = sample_pitch.max(voxel_pitch);
    let boundary_band = voxel_pitch;
    let removed_voxel = initial_volume - simulator.voxel_tree().remaining_volume();
    let solid_bounds = simulator.voxel_tree().collect_solid_voxel_bounds();
    let voxel_index = SolidBoundsSpatialIndex::from_bounds(solid_bounds, exact_sample_pitch);

    let exact_started = Instant::now();
    let mut exact_work = PrimitiveSetExactWork::new(bounds);
    let segments = collect_toolpath_line_segments(
        toolpath,
        geo_algorithms::DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM,
    );
    for (segment, is_cutting) in segments {
        if !is_cutting {
            continue;
        }

        if tool.is_flat_end_mill() {
            exact_work.apply_primitive(ExactToolPrimitive::Flat {
                segment,
                radius: tool.radius(),
            });
            continue;
        }

        if tool.is_ball_end_mill() {
            let start = Point3D::new(
                segment.start().x(),
                segment.start().y(),
                segment.start().z() + tool.radius(),
            );
            let end = Point3D::new(
                segment.end().x(),
                segment.end().y(),
                segment.end().z() + tool.radius(),
            );
            let ball_segment =
                LineSegment3D::new(start, end).expect("ball center segment must be valid");
            exact_work.apply_primitive(ExactToolPrimitive::Ball {
                segment: ball_segment,
                radius: tool.radius(),
            });
            continue;
        }
    }
    let removed_exact = initial_volume - exact_work.estimate_remaining_volume(exact_sample_pitch);
    let elapsed_exact_ms = exact_started.elapsed().as_secs_f64() * 1000.0;

    let gap = if removed_voxel.abs() <= f64::EPSILON {
        0.0
    } else {
        ((removed_exact - removed_voxel).abs()) / removed_voxel.abs()
    };
    let exact_conservative_margin = if tool.is_ball_end_mill() {
        exact_sample_pitch * 0.75
    } else if exact_work.primitive_count() > 1 {
        exact_sample_pitch * 0.5
    } else {
        exact_sample_pitch * 0.75
    };

    let (
        boundary_rate,
        boundary_sample_count,
        boundary_exact_only_count,
        boundary_voxel_only_count,
    ) = compute_boundary_disagreement_rate(
        &bounds,
        exact_sample_pitch,
        boundary_band,
        exact_conservative_margin,
        &exact_work,
        &voxel_index,
    );

    GateMetrics {
        removed_voxel,
        removed_exact,
        gap,
        boundary_disagreement_rate: boundary_rate,
        boundary_sample_count,
        boundary_exact_only_count,
        boundary_voxel_only_count,
        elapsed_voxel_ms,
        elapsed_exact_ms,
        elapsed_ratio: elapsed_exact_ms / elapsed_voxel_ms.max(1.0e-9),
    }
}

fn case_plane_cut_flat() -> (ToolPath<f64>, Tool<f64>) {
    let segment = cam_core::PathSegment::new_line(
        Point3D::new(10.0, 50.0, 50.0),
        Point3D::new(90.0, 50.0, 50.0),
        SegmentType::Cutting { feed_rate: 300.0 },
    );
    let toolpath = ToolPath::new(
        "plane-cut-flat".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 50.0, vec![segment])],
        vec![],
    );
    let tool = Tool::flat_end_mill("flat".to_string(), 10.0, 30.0);
    (toolpath, tool)
}

fn case_step_cut_flat() -> (ToolPath<f64>, Tool<f64>) {
    let level0 = cam_core::PathSegment::new_line(
        Point3D::new(10.0, 30.0, 40.0),
        Point3D::new(90.0, 30.0, 40.0),
        SegmentType::Cutting { feed_rate: 300.0 },
    );
    let level1 = cam_core::PathSegment::new_line(
        Point3D::new(10.0, 70.0, 60.0),
        Point3D::new(90.0, 70.0, 60.0),
        SegmentType::Cutting { feed_rate: 300.0 },
    );
    let toolpath = ToolPath::new(
        "step-cut-flat".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![
            ContourLevelPath::new(0, 40.0, vec![level0]),
            ContourLevelPath::new(1, 60.0, vec![level1]),
        ],
        vec![],
    );
    let tool = Tool::flat_end_mill("flat-step".to_string(), 10.0, 30.0);
    (toolpath, tool)
}

fn case_diagonal_cut_ball() -> (ToolPath<f64>, Tool<f64>) {
    let segment = cam_core::PathSegment::new_line(
        Point3D::new(10.0, 10.0, 20.0),
        Point3D::new(90.0, 90.0, 80.0),
        SegmentType::Cutting { feed_rate: 250.0 },
    );
    let toolpath = ToolPath::new(
        "diagonal-cut-ball".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(0, 20.0, vec![segment])],
        vec![],
    );
    let tool = Tool::ball_end_mill("ball".to_string(), 10.0, 30.0);
    (toolpath, tool)
}

#[test]
fn phase3_gate_metrics_are_measurable_for_reference_cases() {
    let cases = [
        case_plane_cut_flat(),
        case_step_cut_flat(),
        case_diagonal_cut_ball(),
    ];

    for (index, (toolpath, tool)) in cases.iter().enumerate() {
        let metrics = run_gate_case(toolpath, tool, GATE_SAMPLE_PITCH);
        println!(
            "case[{index}] removed_voxel={:.3}, removed_exact={:.3}, gap={:.4}, boundary_disagreement_rate={:.4}, boundary_samples={}, boundary_exact_only={}, boundary_voxel_only={}, elapsed_ratio={:.4} (voxel={:.2}ms, exact={:.2}ms)",
            metrics.removed_voxel,
            metrics.removed_exact,
            metrics.gap,
            metrics.boundary_disagreement_rate,
            metrics.boundary_sample_count,
            metrics.boundary_exact_only_count,
            metrics.boundary_voxel_only_count,
            metrics.elapsed_ratio,
            metrics.elapsed_voxel_ms,
            metrics.elapsed_exact_ms
        );
        assert!(
            metrics.removed_voxel.is_finite(),
            "case[{index}] removed_voxel must be finite"
        );
        assert!(
            metrics.removed_exact.is_finite(),
            "case[{index}] removed_exact must be finite"
        );
        assert!(metrics.gap.is_finite(), "case[{index}] gap must be finite");
        assert!(
            metrics.boundary_disagreement_rate.is_finite(),
            "case[{index}] boundary_disagreement_rate must be finite"
        );
        assert!(
            (0.0..=1.0).contains(&metrics.boundary_disagreement_rate),
            "case[{index}] boundary_disagreement_rate must be within [0, 1]"
        );
        assert!(
            metrics.boundary_sample_count > 0,
            "case[{index}] boundary_sample_count must be positive"
        );
        assert!(
            metrics.elapsed_voxel_ms.is_finite() && metrics.elapsed_voxel_ms >= 0.0,
            "case[{index}] elapsed_voxel_ms must be finite and non-negative"
        );
        assert!(
            metrics.elapsed_exact_ms.is_finite() && metrics.elapsed_exact_ms >= 0.0,
            "case[{index}] elapsed_exact_ms must be finite and non-negative"
        );
        assert!(
            metrics.elapsed_ratio.is_finite(),
            "case[{index}] elapsed_ratio must be finite"
        );
        assert!(
            metrics.removed_voxel > 0.0,
            "case[{index}] removed_voxel must be positive"
        );
        assert!(
            metrics.removed_exact > 0.0,
            "case[{index}] removed_exact must be positive"
        );
    }
}

#[test]
fn phase3_gate_reproducibility_parallel_matches_sequential_removed_volume() {
    let (toolpath, tool) = case_diagonal_cut_ball();

    let sequential = run_gate_case(&toolpath, &tool, GATE_SAMPLE_PITCH);

    let mut workers = Vec::new();
    for _ in 0..4 {
        let toolpath_cloned = toolpath.clone();
        let tool_cloned = tool.clone();
        workers.push(std::thread::spawn(move || {
            run_gate_case(&toolpath_cloned, &tool_cloned, GATE_SAMPLE_PITCH)
        }));
    }

    for worker in workers {
        let parallel = worker.join().expect("parallel worker must finish");
        assert!(
            (parallel.removed_voxel - sequential.removed_voxel).abs() <= 1.0e-9,
            "voxel removed volume mismatch: parallel={}, sequential={}",
            parallel.removed_voxel,
            sequential.removed_voxel
        );
        assert!(
            (parallel.removed_exact - sequential.removed_exact).abs() <= 1.0e-9,
            "exact removed volume mismatch: parallel={}, sequential={}",
            parallel.removed_exact,
            sequential.removed_exact
        );
    }
}

#[test]
fn phase3_gate_threshold_targets_are_met_for_reference_cases() {
    let cases = [
        ("plane_cut_flat", case_plane_cut_flat()),
        ("step_cut_flat", case_step_cut_flat()),
        ("diagonal_cut_ball", case_diagonal_cut_ball()),
    ];

    for (name, (toolpath, tool)) in &cases {
        let metrics = run_gate_case(toolpath, tool, GATE_SAMPLE_PITCH);
        assert!(
            metrics.gap <= GATE_TARGET_GAP,
            "{name}: gap {:.4} exceeds target {:.4}",
            metrics.gap,
            GATE_TARGET_GAP
        );
        assert!(
            metrics.boundary_disagreement_rate <= GATE_TARGET_BOUNDARY,
            "{name}: boundary_disagreement_rate {:.4} exceeds target {:.4}",
            metrics.boundary_disagreement_rate,
            GATE_TARGET_BOUNDARY
        );
        assert!(
            metrics.elapsed_ratio <= GATE_TARGET_ELAPSED_RATIO,
            "{name}: elapsed_ratio {:.4} exceeds target {:.4}",
            metrics.elapsed_ratio,
            GATE_TARGET_ELAPSED_RATIO
        );
    }
}

#[test]
fn test_cam_sim_source_does_not_reference_demo_symbols() {
    demo_symbol_guard::assert_layer_does_not_reference_demo_symbols("cam_sim");
}
