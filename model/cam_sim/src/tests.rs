use geo_algorithms::{Aabb3D, Point3D};
use std::io::Cursor;
use std::sync::Arc;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ContourLevelPath, CuttingDirection, SegmentType, Tool,
    ToolPath, read_toolpath_artifact_v1, write_toolpath_payload_v1,
};
use job_runtime::{JobEvent, JobManager, JobRelation, JobSpec, JobStatus, JobType, RetryPolicy};

use crate::{
    CamJobExecutorAdapter, CamWorkflowError, CamWorkflowSubmitter, CuttingSimulationInput,
    CuttingSimulator, InMemoryCuttingSimulationInputStore, SimulationError, SnapshotInterval,
};

fn cam_spec(input: &str) -> JobSpec {
    JobSpec {
        job_type: JobType::CamProcess,
        input_ref: input.to_string(),
        retry_policy: RetryPolicy::default(),
    }
}

fn sim_spec(input: &str) -> JobSpec {
    JobSpec {
        job_type: JobType::CuttingSimulation,
        input_ref: input.to_string(),
        retry_policy: RetryPolicy::default(),
    }
}

/// テスト用の固定のツールパス（工具 ID `nc-post-src`）を切削する入力を、`input://sim/sample` に登録したアダプタ
fn adapter_with_sim_input() -> CamJobExecutorAdapter {
    let store = InMemoryCuttingSimulationInputStore::new();
    store
        .register(
            "input://sim/sample",
            CuttingSimulationInput {
                stock: Aabb3D::new(Point3D::new(-5.0, -5.0, -5.0), Point3D::new(5.0, 5.0, 1.0)),
                tool: Tool::flat_end_mill("nc-post-src".to_string(), 2.0, 10.0),
                precision: 0.5,
                snapshot_interval: SnapshotInterval::default(),
            },
        )
        .unwrap();
    CamJobExecutorAdapter::default().with_simulation_input_provider(Arc::new(store))
}

fn nc_post_from_cam_spec(input: &str) -> JobSpec {
    JobSpec {
        job_type: JobType::NcPostFromCam,
        input_ref: input.to_string(),
        retry_policy: RetryPolicy::default(),
    }
}

/// 点の列を切削の区間で結んだツールパス
fn cutting_toolpath(tool_id: &str, points: &[Point3D<f64>]) -> ToolPath<f64> {
    ToolPath::new(
        tool_id.to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            points
                .windows(2)
                .map(|pair| {
                    cam_core::PathSegment::new_line(
                        pair[0],
                        pair[1],
                        SegmentType::Cutting { feed_rate: 300.0 },
                    )
                })
                .collect(),
        )],
        vec![],
    )
}

/// 素材の精度（半径 5 の工具で格子の間隔が約 0.28 になる）
const PRECISION: f64 = 0.002;

fn unit_cube_stock() -> Aabb3D<f64> {
    Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    )
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
fn test_simulate_records_remaining_volume_before_and_after_cutting() {
    let mut simulator = CuttingSimulator::new(unit_cube_stock(), SnapshotInterval::default());
    let toolpath = cutting_toolpath(
        "EM10",
        &[
            Point3D::new(10.0, 10.0, 90.0),
            Point3D::new(90.0, 10.0, 90.0),
        ],
    );
    simulator
        .simulate(
            &toolpath,
            &Tool::flat_end_mill("EM10".to_string(), 10.0, 30.0),
            PRECISION,
        )
        .unwrap();

    let snapshots = simulator.snapshots();
    let initial = unit_cube_stock().volume();
    // 区間の始点は切削の前、終点は切削の後
    assert!((snapshots[0].remaining_volume - initial).abs() < 1e-6 * initial);
    let last = snapshots.last().unwrap().remaining_volume;
    // 深さ 10 の溝（幅 10・長さ 80 と両端の半円）を除去する
    let removed = (10.0 * 80.0 + std::f64::consts::PI * 25.0) * 10.0;
    assert!(
        ((initial - last) - removed).abs() < 0.02 * removed,
        "last = {last}"
    );
}

#[test]
fn test_auto_distance_interval_generates_snapshots() {
    let mut simulator = CuttingSimulator::new(
        unit_cube_stock(),
        SnapshotInterval::ByAutoDistance {
            target_count: 5,
            min_interval_mm: 1.0,
            include_segment_endpoints: false,
        },
    );
    let toolpath = cutting_toolpath(
        "EM4",
        &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(50.0, 0.0, 0.0)],
    );
    simulator
        .simulate(
            &toolpath,
            &Tool::flat_end_mill("EM4".to_string(), 4.0, 30.0),
            PRECISION,
        )
        .unwrap();
    assert!(!simulator.snapshots().is_empty());
}

#[test]
fn test_simulate_toolpath_flat_end_mill() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut simulator = CuttingSimulator::new(bounds, SnapshotInterval::default());

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
    let result = simulator.simulate(&toolpath, &tool, PRECISION);

    assert!(result.is_ok());
    assert!(!simulator.snapshots().is_empty());
}

#[test]
fn test_simulate_toolpath_ball_end_mill() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut simulator = CuttingSimulator::new(bounds, SnapshotInterval::default());

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
    let result = simulator.simulate(&toolpath, &tool, PRECISION);

    assert!(result.is_ok());
    assert!(!simulator.snapshots().is_empty());
}

#[test]
fn test_simulate_toolpath_radius_end_mill_is_rejected() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut simulator = CuttingSimulator::new(bounds, SnapshotInterval::default());

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
    let result = simulator.simulate(&toolpath, &tool, PRECISION);

    assert_eq!(result, Err(SimulationError::UnsupportedToolType));
}

#[test]
fn test_include_segment_endpoints_changes_snapshot_count() {
    let toolpath = cutting_toolpath(
        "EM6",
        &[
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(60.0, 0.0, 0.0),
            Point3D::new(100.0, 0.0, 0.0),
        ],
    );
    let tool = Tool::flat_end_mill("EM6".to_string(), 6.0, 30.0);
    let mut with_endpoints = CuttingSimulator::new(
        unit_cube_stock(),
        SnapshotInterval::ByDistance {
            interval_mm: 25.0,
            include_segment_endpoints: true,
        },
    );
    let mut without_endpoints = CuttingSimulator::new(
        unit_cube_stock(),
        SnapshotInterval::ByDistance {
            interval_mm: 25.0,
            include_segment_endpoints: false,
        },
    );
    with_endpoints
        .simulate(&toolpath, &tool, PRECISION)
        .unwrap();
    without_endpoints
        .simulate(&toolpath, &tool, PRECISION)
        .unwrap();

    assert!(with_endpoints.snapshots().len() > without_endpoints.snapshots().len());
}

#[test]
fn test_default_interval_is_the_shorter_of_the_tool_radius_and_the_grid_spacing_multiple() {
    let toolpath = cutting_toolpath(
        "EM10",
        &[
            Point3D::new(10.0, 50.0, 90.0),
            Point3D::new(90.0, 50.0, 90.0),
        ],
    );
    let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 30.0);
    // 精度 0.01 の格子の間隔は √(8 × 5 × 0.01) で、その 4 倍（約 2.53）は工具の半径 5 より短い。粗い精度 0.2 では
    // 間隔の 4 倍が 16 になり、半径になる
    for (precision, expected) in [(0.01, 4.0 * 0.4_f64.sqrt()), (0.2, 5.0)] {
        let mut simulator = CuttingSimulator::new(unit_cube_stock(), SnapshotInterval::default());
        simulator.simulate(&toolpath, &tool, precision).unwrap();
        let distances: Vec<f64> = simulator
            .snapshots()
            .iter()
            .map(|snapshot| snapshot.position.accumulated_distance)
            .collect();
        // 最後の時点（ツールパスの終わり）の前までは、間隔ごとに置く
        for pair in distances[..distances.len() - 1].windows(2) {
            assert!((pair[1] - pair[0] - expected).abs() < 1e-9, "{pair:?}");
        }
        assert!((distances.last().unwrap() - 80.0).abs() < 1e-9);
    }
}

#[test]
fn test_snapshots_are_placed_around_switches_and_breaks_of_the_path_and_at_the_end() {
    let segment =
        |start, end, segment_type| cam_core::PathSegment::new_line(start, end, segment_type);
    let cutting = SegmentType::Cutting { feed_rate: 300.0 };
    let [a, b, c, d, e, f] = [
        Point3D::new(10.0, 10.0, 90.0),
        Point3D::new(50.0, 10.0, 90.0),
        Point3D::new(50.0, 50.0, 90.0),
        Point3D::new(90.0, 50.0, 90.0),
        Point3D::new(90.0, 80.0, 90.0),
        Point3D::new(10.0, 80.0, 90.0),
    ];
    let toolpath = ToolPath::new(
        "EM10".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            90.0,
            vec![
                segment(a, b, cutting),
                segment(b, c, SegmentType::Rapid),
                segment(c, d, cutting),
                // 前の区間の終点から途切れた切削のパス
                segment(e, f, cutting),
            ],
        )],
        vec![],
    );
    let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 30.0);
    let simulate = |include_segment_endpoints| {
        let mut simulator = CuttingSimulator::new(
            unit_cube_stock(),
            SnapshotInterval::ByDistance {
                interval_mm: 1000.0,
                include_segment_endpoints,
            },
        );
        simulator.simulate(&toolpath, &tool, PRECISION).unwrap();
        simulator.snapshots().to_vec()
    };
    let sparse = simulate(false);
    let positions: Vec<(usize, f64)> = sparse
        .iter()
        .map(|snapshot| (snapshot.position.segment_index, snapshot.position.t))
        .collect();
    // 始まり・切り替わりと途切れたところの前後・ツールパスの終わり
    assert_eq!(
        positions,
        vec![
            (0, 0.0),
            (0, 1.0),
            (1, 0.0),
            (1, 1.0),
            (2, 0.0),
            (2, 1.0),
            (3, 0.0),
            (3, 1.0)
        ]
    );
    let last = sparse.last().unwrap().remaining_volume;
    let with_endpoints = simulate(true).last().unwrap().remaining_volume;
    assert!((last - with_endpoints).abs() < 1e-9 * with_endpoints);
    assert!(last < unit_cube_stock().volume());
}

#[test]
fn test_snapshot_exports_f64_maps_snapshot_fields() {
    let mut simulator = CuttingSimulator::new(unit_cube_stock(), SnapshotInterval::default());
    let toolpath = cutting_toolpath(
        "EM4",
        &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(40.0, 0.0, 0.0)],
    );
    simulator
        .simulate(
            &toolpath,
            &Tool::flat_end_mill("EM4".to_string(), 4.0, 30.0),
            PRECISION,
        )
        .unwrap();
    let exports = simulator.snapshot_exports_f64();

    assert_eq!(exports.len(), simulator.snapshots().len());
    for (export, snapshot) in exports.iter().zip(simulator.snapshots()) {
        assert_eq!(export.segment_index, snapshot.position.segment_index);
        assert_eq!(export.segment_t, snapshot.position.t);
        assert_eq!(
            export.accumulated_distance_mm,
            snapshot.position.accumulated_distance
        );
        assert_eq!(export.remaining_volume_mm3, snapshot.remaining_volume);
    }
}

#[test]
fn test_job_adapter_runs_cam_and_sim_jobs() {
    let mut manager = JobManager::new();
    let adapter = adapter_with_sim_input();

    let cam_id = manager.submit(JobSpec {
        job_type: JobType::CamProcess,
        input_ref: "input://cam/sample".to_string(),
        retry_policy: RetryPolicy::default(),
    });

    let sim_id = manager
        .submit_with_relation(
            JobSpec {
                job_type: JobType::CuttingSimulation,
                input_ref: "input://sim/sample".to_string(),
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

    // 成果物は切削シミュレーションの成果物として読め、スナップショットの数がログと一致する
    let output = &manager.output_history(sim_id).unwrap()[0];
    let (_, artifact) =
        crate::read_cutting_simulation_artifact_v1(&mut output.artifact_bytes.as_deref().unwrap())
            .unwrap();
    assert!(
        output
            .log_ref
            .as_deref()
            .unwrap()
            .ends_with(&format!("/ok/snapshots-{}", artifact.snapshots.len()))
    );

    let events = manager.take_events();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            JobEvent::Completed {
                job_id,
                status: JobStatus::Succeeded,
                log_ref: Some(log_ref),
                ..
            } if *job_id == sim_id && log_ref.contains("/ok/snapshots-")
        )
    }));
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
    let adapter = CamJobExecutorAdapter::default();

    let id = manager.submit(JobSpec {
        job_type: JobType::CuttingSimulation,
        input_ref: "input://cam/wrong".to_string(),
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
fn test_job_adapter_reports_no_solution_for_cam_process() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter::default();

    let id = manager.submit(cam_spec("input://cam/no-solution"));
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("no_solution")
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
            } if *job_id == id && log_ref.ends_with("/no_solution")
        )
    }));
}

#[test]
fn test_job_adapter_reports_convergence_failure_for_cam_process() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter::default();

    let id = manager.submit(cam_spec("input://cam/convergence-failure"));
    manager.execute_with(id, &adapter).unwrap();

    let job = manager.get(id).unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert!(
        job.last_error
            .as_deref()
            .unwrap_or_default()
            .contains("convergence_failure")
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
            } if *job_id == id && log_ref.ends_with("/convergence_failure")
        )
    }));
}

#[test]
fn test_job_adapter_rejects_unregistered_cutting_sim_input() {
    let mut manager = JobManager::new();
    let adapter = adapter_with_sim_input();

    let cam_id = manager.submit(cam_spec("input://cam/sample"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let id = manager
        .submit_with_relation(
            sim_spec("input://sim/unregistered"),
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
            .contains("cutting simulation input is not registered")
    );
}

#[test]
fn test_job_adapter_rejects_cutting_sim_with_mismatched_tool() {
    let store = InMemoryCuttingSimulationInputStore::new();
    store
        .register(
            "input://sim/other-tool",
            CuttingSimulationInput {
                stock: Aabb3D::new(Point3D::new(-5.0, -5.0, -5.0), Point3D::new(5.0, 5.0, 1.0)),
                tool: Tool::flat_end_mill("other-tool".to_string(), 2.0, 10.0),
                precision: 0.5,
                snapshot_interval: SnapshotInterval::default(),
            },
        )
        .unwrap();
    let adapter = CamJobExecutorAdapter::default().with_simulation_input_provider(Arc::new(store));
    let mut manager = JobManager::new();

    let cam_id = manager.submit(cam_spec("input://cam/sample"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let id = manager
        .submit_with_relation(
            sim_spec("input://sim/other-tool"),
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
            .contains("tool id does not match the toolpath")
    );
}

#[test]
fn test_job_adapter_surfaces_cutting_sim_artifact_read_failure_from_parent_artifact() {
    let mut manager = JobManager::new();
    let adapter = adapter_with_sim_input();

    let cam_id = manager.submit(cam_spec("input://cam/artifact-read-failed"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let sim_id = manager
        .submit_with_relation(
            sim_spec("input://sim/sample"),
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
fn test_job_adapter_rejects_cutting_sim_of_empty_toolpath_in_preflight() {
    let mut manager = JobManager::new();
    let adapter = adapter_with_sim_input();

    let cam_id = manager.submit(cam_spec("input://cam/sim-failure"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let sim_id = manager
        .submit_with_relation(
            sim_spec("input://sim/sample"),
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
            .contains("toolpath is empty")
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
            } if *job_id == sim_id && log_ref.ends_with("/preflight-not-ready")
        )
    }));
}

#[test]
fn test_job_adapter_rejects_cutting_sim_without_parent() {
    let mut manager = JobManager::new();
    let adapter = CamJobExecutorAdapter::default();

    let id = manager.submit(sim_spec("input://sim/sample"));
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
    let adapter = CamJobExecutorAdapter::default();

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
    let adapter = CamJobExecutorAdapter::default();

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
    let adapter = CamJobExecutorAdapter::default();

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
    let adapter = CamJobExecutorAdapter::default();

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
    let adapter = CamJobExecutorAdapter::default();

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
    let adapter = adapter_with_sim_input();

    let cam_id = manager.submit(cam_spec("input://cam/for-nc-post"));
    manager.execute_with(cam_id, &adapter).unwrap();

    let sim_id = manager
        .submit_with_relation(
            sim_spec("input://sim/sample"),
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
/// フラットは底が平らな円柱、ボールは先端が球の工具を掃引するため、残存体積に差が生じる。
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

    let initial_volume = bounds.volume();

    let mut sim_flat = CuttingSimulator::new(bounds, SnapshotInterval::default());
    sim_flat
        .simulate(
            &toolpath_flat,
            &Tool::flat_end_mill("flat".to_string(), 10.0, 30.0),
            PRECISION,
        )
        .unwrap();

    let mut sim_ball = CuttingSimulator::new(bounds, SnapshotInterval::default());
    sim_ball
        .simulate(
            &toolpath_ball,
            &Tool::ball_end_mill("ball".to_string(), 10.0, 30.0),
            PRECISION,
        )
        .unwrap();

    let flat_remaining = sim_flat.snapshots().last().unwrap().remaining_volume;
    let ball_remaining = sim_ball.snapshots().last().unwrap().remaining_volume;

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
        "フラット（底が平らな円柱）とボール（先端が球）は異なる形状で除去するため残存体積が異なるはず: \
         flat={flat_remaining}, ball={ball_remaining}"
    );
}

/// 経路の座標が工具の先端であることを検証する。
///
/// 先端基準の水平経路（Z=10）に対して、ボールは球の中心が Z=10+radius=15 にある。
/// フラットエンドミルとは異なる形を除去するため、残存体積が異なる。
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

    let mut sim_flat = CuttingSimulator::new(bounds, SnapshotInterval::default());
    sim_flat
        .simulate(
            &toolpath_flat,
            &Tool::flat_end_mill("flat-z".to_string(), 10.0, 30.0),
            PRECISION,
        )
        .unwrap();

    let mut sim_ball = CuttingSimulator::new(bounds, SnapshotInterval::default());
    sim_ball
        .simulate(
            &toolpath_ball,
            &Tool::ball_end_mill("ball-z".to_string(), 10.0, 30.0),
            PRECISION,
        )
        .unwrap();

    let flat_remaining = sim_flat.snapshots().last().unwrap().remaining_volume;
    let ball_remaining = sim_ball.snapshots().last().unwrap().remaining_volume;

    assert_ne!(
        flat_remaining, ball_remaining,
        "Z=10 の水平経路でフラット（底面 Z=10）とボール（球の中心 Z=15）は除去する形が異なるはず: \
         flat={flat_remaining}, ball={ball_remaining}"
    );
}
