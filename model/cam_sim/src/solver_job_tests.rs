//! CamProcess solver 導線（設計: CAM_ALGORITHMS_DESIGN.md §8.7）の Job Manager 経由テスト

use geo_algorithms::TriangleMesh3DConstructor;
use std::io::Cursor;
use std::sync::Arc;

use cam_algorithms::{
    CamSolverInput, OperationSpec, ScanlineParams, ScanlineToolRange, SolverGeometry,
};
use cam_core::{CoordinateFrame, LengthUnit, Tool, read_toolpath_artifact_v1};
use geo_algorithms::{Point3D, TessellationLimits, TriangleMesh3D};
use job_runtime::{
    JobEvent, JobExecutor, JobManager, JobRelation, JobSpec, JobStatus, JobType,
    RefValidationError, RetryPolicy,
};

use crate::{
    CamJobExecutorAdapter, CuttingSimulationInput, InMemoryCamSolverInputStore,
    InMemoryCuttingSimulationInputStore, SnapshotInterval,
};
use geo_algorithms::Aabb3D;

fn spec(job_type: JobType, input_ref: String) -> JobSpec {
    JobSpec {
        job_type,
        input_ref,
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
            tool_range: ScanlineToolRange::default(),
        }),
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: 0.01,
        tessellation_limits: TessellationLimits::default(),
        boundary: None,
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
    let simulation_inputs = InMemoryCuttingSimulationInputStore::new();
    simulation_inputs
        .register(
            "input://sim/pyramid",
            CuttingSimulationInput {
                stock: Aabb3D::new(
                    Point3D::new(0.0, 0.0, 0.0),
                    Point3D::new(100.0, 100.0, 100.0),
                ),
                tool: tool.clone(),
                precision: 6.25,
                snapshot_interval: SnapshotInterval::default(),
            },
        )
        .unwrap();
    let adapter = adapter_with(vec![(
        input_ref,
        scanline_input(SolverGeometry::TriangleMesh(pyramid_mesh()), tool),
    )])
    .with_simulation_input_provider(Arc::new(simulation_inputs));
    let mut manager = JobManager::new();

    let cam_id = manager.submit(spec(JobType::CamProcess, input_ref.to_string()));
    let sim_id = manager
        .submit_with_relation(
            spec(
                JobType::CuttingSimulation,
                "input://sim/pyramid".to_string(),
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
    // ラジアスエンドミルの逆オフセットは未対応
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

#[test]
fn template_resolved_input_runs_and_out_of_domain_rectangle_is_invalid_input() {
    use cam_algorithms::{
        MachiningBoundary, OperationDefinition, ProcessTemplate, RectangleBoundary, SolverDefaults,
        build_solver_input, resolve_operation,
    };

    let template = ProcessTemplate {
        template_id: "tpl-finish".to_string(),
        default_operation_type: Some("scanline".to_string()),
        default_tolerance_profile: Some("press_rough".to_string()),
    };
    let build = |boundary: RectangleBoundary| {
        let resolved = resolve_operation(
            &template,
            &OperationDefinition {
                operation_id: "op-scan-1".to_string(),
                machining_stage: "finish".to_string(),
                operation_type: None,
                tolerance_profile: None,
                boundary: Some(MachiningBoundary::Rectangle(boundary)),
            },
            SolverDefaults::default(),
        )
        .unwrap();
        build_solver_input(
            &resolved,
            ball_tool(),
            SolverGeometry::TriangleMesh(pyramid_mesh()),
            ScanlineParams {
                stepover: 4.0,
                sample_pitch: 2.0,
                feed_rate: 1200.0,
                clearance_height: 5.0,
                tool_range: ScanlineToolRange::default(),
            },
            TessellationLimits::default(),
        )
        .unwrap()
    };

    let adapter = adapter_with(vec![
        (
            "input://cam/pyramid-center",
            build(RectangleBoundary {
                rect_min: [30.0, 30.0],
                rect_max: [50.0, 50.0],
            }),
        ),
        (
            "input://cam/pyramid-far",
            build(RectangleBoundary {
                rect_min: [90.0, 90.0],
                rect_max: [95.0, 95.0],
            }),
        ),
    ]);

    let mut manager = JobManager::new();
    let id = manager.submit(spec(
        JobType::CamProcess,
        "input://cam/pyramid-center".to_string(),
    ));
    manager.execute_with(id, &adapter).unwrap();
    assert_eq!(manager.get(id).unwrap().status, JobStatus::Succeeded);

    assert_cam_failure(&adapter, "input://cam/pyramid-far", "invalid_input");
}

/// 平坦面（z = 0、10 × 10）の周りに工具の半径分の余裕を持つ素材を削り、平坦面より上に残る素材の体積を返す
/// 平坦面（z = 0、10 × 10）を工具 `tool` で走査線加工し、上面の高さ `stock_top` の素材の平坦面より上に残る体積
fn material_left_above_flat_face(
    tool_range: ScanlineToolRange,
    tool: Tool<f64>,
    stock_top: f64,
) -> f64 {
    let flat = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
            Point3D::new(0.0, 10.0, 0.0),
        ],
        vec![[0, 1, 2], [0, 2, 3]],
    )
    .unwrap();
    let r = tool.radius();
    let mut input = scanline_input(SolverGeometry::TriangleMesh(flat), tool.clone());
    let OperationSpec::Scanline(params) = &mut input.operation;
    params.tool_range = tool_range;
    let toolpath = cam_algorithms::solve_toolpath(&input).unwrap();

    // 素材の底は z = -2（平坦面より上の `stock_top` の厚さを削る）
    let stock = Aabb3D::new(
        Point3D::new(-r, -r, -2.0),
        Point3D::new(10.0 + r, 10.0 + r, stock_top),
    );
    let mut simulator = crate::CuttingSimulator::new(stock, SnapshotInterval::default());
    simulator.simulate(&toolpath, &tool).unwrap();
    let remaining = simulator.snapshots().last().unwrap().remaining_volume;
    let below_face = (10.0 + 2.0 * r) * (10.0 + 2.0 * r) * 2.0;
    remaining - below_face
}

#[test]
fn extending_the_tool_range_removes_the_material_around_a_flat_face() {
    let extended =
        material_left_above_flat_face(ScanlineToolRange::ExtendByToolRadius, ball_tool(), 2.0);
    let within = material_left_above_flat_face(ScanlineToolRange::GeometryExtent, ball_tool(), 2.0);
    let side = 10.0 + 2.0 * ball_tool().radius();
    // 工具の半径分広げると、平坦面より上には走査線の間の削り残し（カスプ）しか残らない
    assert!(extended < 0.1 * side * side, "extended: {extended}");
    // 形状の範囲では、平坦面の端の外側の素材が残る
    assert!(
        within > 2.0 * extended,
        "within: {within}, extended: {extended}"
    );
}

#[test]
fn a_tool_without_a_length_removes_the_material_above_its_length() {
    // 工具長 30 の工具より深い 50 の厚さを、平坦面まで削る
    let stock_top = 50.0;
    let mut unlimited = ball_tool();
    unlimited.cutting_length = None;
    let without_length =
        material_left_above_flat_face(ScanlineToolRange::ExtendByToolRadius, unlimited, stock_top);
    let with_length = material_left_above_flat_face(
        ScanlineToolRange::ExtendByToolRadius,
        ball_tool(),
        stock_top,
    );
    let side = 10.0 + 2.0 * ball_tool().radius();
    // 工具長を指定しない工具では、平坦面より上には走査線の間の削り残し（カスプ）しか残らない
    assert!(
        without_length < 0.1 * side * side,
        "without length: {without_length}"
    );
    // 工具長 30 の工具では、工具より高い位置（球の中心から 30 より上）の素材が残る
    let above_tool = side * side * (stock_top - 30.0 - ball_tool().radius());
    assert!(
        with_length > 0.9 * above_tool,
        "with length: {with_length}, above the tool: {above_tool}"
    );
}
