//! CamProcess solver 導線（#684 §8.7）の Job Manager 経由テスト

use std::io::Cursor;
use std::sync::Arc;

use cam_algorithms::{
    CamSolverInput, OperationSpec, ScanlineParams, SolverGeometry, TessellationLimits,
};
use cam_core::{CoordinateFrame, LengthUnit, Tool, read_toolpath_artifact_v1};
use geo_algorithms::{Point3D, TriangleMesh3D};
use job_runtime::{
    JobEvent, JobExecutor, JobManager, JobRelation, JobSpec, JobStatus, JobType,
    RefValidationError, RetryPolicy,
};

use crate::{CamJobExecutorAdapter, InMemoryCamSolverInputStore};

fn spec(job_type: JobType, input_ref: String) -> JobSpec {
    JobSpec {
        job_type,
        input_ref,
        timeout_secs: 30,
        retry_policy: RetryPolicy::default(),
    }
}

/// 既定ワーク（0..100 立方体）内の四角錐（底面 z=80、頂点 z=90）
fn pyramid_mesh() -> TriangleMesh3D<f64> {
    let vertices = vec![
        Point3D::new(20.0, 20.0, 80.0),
        Point3D::new(60.0, 20.0, 80.0),
        Point3D::new(60.0, 60.0, 80.0),
        Point3D::new(20.0, 60.0, 80.0),
        Point3D::new(40.0, 40.0, 90.0),
    ];
    let indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
    TriangleMesh3D::new(vertices, indices).unwrap()
}

fn scanline_input(geometry: SolverGeometry, tool: Tool<f64>) -> CamSolverInput {
    CamSolverInput {
        operation_id: "op-scan-1".to_string(),
        tool,
        geometry,
        operation: OperationSpec::Scanline(ScanlineParams {
            stepover: 4.0,
            sample_pitch: 2.0,
            feed_rate: 1200.0,
            clearance_height: 5.0,
        }),
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: 0.01,
        tessellation_limits: TessellationLimits::default(),
    }
}

fn ball_tool() -> Tool<f64> {
    Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0)
}

fn adapter_with(entries: Vec<(&str, CamSolverInput)>) -> CamJobExecutorAdapter {
    let store = InMemoryCamSolverInputStore::new();
    for (input_ref, input) in entries {
        store.register(input_ref, input).unwrap();
    }
    CamJobExecutorAdapter::with_input_provider(Arc::new(store))
}

#[test]
fn registered_input_is_solved_into_toolpath_artifact() {
    let input_ref = "input://cam/pyramid";
    let adapter = adapter_with(vec![(
        input_ref,
        scanline_input(SolverGeometry::TriangleMesh(pyramid_mesh()), ball_tool()),
    )]);
    let mut manager = JobManager::new();
    let cam_id = manager.submit(spec(JobType::CamProcess, input_ref.to_string()));

    let result = adapter.execute(manager.get(cam_id).unwrap(), None);
    assert_eq!(result.status, JobStatus::Succeeded, "{:?}", result.error);
    assert_eq!(
        result.result_ref.as_deref(),
        Some(format!("result://cam/{}/ok", cam_id.0).as_str())
    );

    let bytes = result.artifact_bytes.expect("toolpath artifact bytes");
    let (header, toolpath) = read_toolpath_artifact_v1(&mut Cursor::new(bytes)).unwrap();
    assert_eq!(header.unit, LengthUnit::Millimeter);
    assert_eq!(toolpath.tool_id, "BEM6");
    assert!(toolpath.level_count() > 0);
    assert!(toolpath.total_cutting_length() > 0.0);
}

#[test]
fn cam_parent_to_cutting_simulation_child_succeeds_with_solver_output() {
    for tool in [
        ball_tool(),
        Tool::flat_end_mill("EM6".to_string(), 6.0, 30.0),
    ] {
        assert_cam_to_simulation_succeeds(tool);
    }
}

fn assert_cam_to_simulation_succeeds(tool: Tool<f64>) {
    let input_ref = "input://cam/pyramid";
    let adapter = adapter_with(vec![(
        input_ref,
        scanline_input(SolverGeometry::TriangleMesh(pyramid_mesh()), tool),
    )]);
    let mut manager = JobManager::new();

    let cam_id = manager.submit(spec(JobType::CamProcess, input_ref.to_string()));
    let sim_id = manager
        .submit_with_relation(
            spec(
                JobType::CuttingSimulation,
                format!("result://cam/{}/ok", cam_id.0),
            ),
            JobRelation {
                parent_job_id: Some(cam_id),
                group_id: None,
            },
        )
        .unwrap();

    manager.execute_with(cam_id, &adapter).unwrap();
    manager.execute_with(sim_id, &adapter).unwrap();

    assert_eq!(manager.get(cam_id).unwrap().status, JobStatus::Succeeded);
    let sim = manager.get(sim_id).unwrap();
    assert_eq!(sim.status, JobStatus::Succeeded, "{:?}", sim.last_error);
    assert_eq!(
        manager.active_result_ref(sim_id).unwrap(),
        Some(format!("result://sim/{}/ok", sim_id.0).as_str())
    );
}

fn assert_cam_failure(adapter: &CamJobExecutorAdapter, input_ref: &str, code: &str) {
    let mut manager = JobManager::new();
    let id = manager.submit(spec(JobType::CamProcess, input_ref.to_string()));
    manager.execute_with(id, adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error.as_deref().unwrap_or_default().contains(code),
        "expected {code}, got {:?}",
        job.last_error
    );
    let events = manager.take_events();
    assert!(events.iter().any(|event| matches!(
        event,
        JobEvent::Completed {
            job_id,
            status: JobStatus::Failed,
            log_ref: Some(log_ref),
            ..
        } if *job_id == id && log_ref.ends_with(&format!("/{code}"))
    )));
}

#[test]
fn solver_invalid_input_is_reported_as_invalid_input() {
    // ラジアスエンドミルの逆オフセットは未対応（#211）
    let input_ref = "input://cam/radius-tool";
    let radius_tool = Tool::radius_end_mill("REM6R1".to_string(), 6.0, 1.0, 30.0);
    let adapter = adapter_with(vec![(
        input_ref,
        scanline_input(SolverGeometry::TriangleMesh(pyramid_mesh()), radius_tool),
    )]);
    assert_cam_failure(&adapter, input_ref, "invalid_input");
}

#[test]
fn solver_no_solution_is_reported_as_no_solution() {
    let input_ref = "input://cam/degenerate";
    let degenerate = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap();
    let adapter = adapter_with(vec![(
        input_ref,
        scanline_input(SolverGeometry::TriangleMesh(degenerate), ball_tool()),
    )]);
    assert_cam_failure(&adapter, input_ref, "no_solution");
}

#[test]
fn malformed_input_ref_is_rejected_as_invalid_input() {
    let adapter = adapter_with(Vec::new());
    assert_cam_failure(&adapter, "input://sim/pyramid", "invalid_input");
    assert_cam_failure(&adapter, "input://cam", "invalid_input");
}

#[test]
fn input_store_rejects_non_cam_references() {
    let store = InMemoryCamSolverInputStore::new();
    let input = scanline_input(SolverGeometry::TriangleMesh(pyramid_mesh()), ball_tool());
    assert!(matches!(
        store.register("result://cam/1/ok", input.clone()),
        Err(RefValidationError::InvalidScheme { .. })
    ));
    assert!(matches!(
        store.register("input://nc-post/sample", input),
        Err(RefValidationError::DomainMismatch { .. })
    ));
}
