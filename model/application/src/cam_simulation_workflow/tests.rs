//! `CamSimulationWorkflow` のテスト

use std::time::Instant;

use cam_algorithms::SolverGeometry;
use cam_core::{
    MachiningStage, OperationSpec, ScanlineParams, ToleranceProfile, Tool,
    read_toolpath_artifact_v1,
};
use geo_algorithms::{Point3D, TriangleMesh3D, TriangleMesh3DConstructor};
use geo_entity::FaceId;

use super::*;
use crate::machining_sequence::{MachiningOperationContent, MachiningStock};
use crate::product_shape::ProductShapeError;

/// 確かめた入力を、投入元なしで投入する（加工工程を通さない投入。ジョブの実行と取り込みを確かめる）
fn submit(
    workflow: &mut CamSimulationWorkflow,
    request: CamSimulationWorkflowRequest,
) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
    let errors = condition_errors(&request);
    if !errors.is_empty() {
        return Err(CamSimulationWorkflowError::Conditions(errors));
    }
    workflow.enqueue(request, None)
}

/// 取り込める（正常終了した）切削シミュレーションのジョブ（番号の順）
fn succeeded_simulations(workflow: &CamSimulationWorkflow) -> Vec<SimulationJob> {
    workflow
        .jobs
        .simulation_jobs()
        .unwrap()
        .into_iter()
        .filter(|job| job.status == CamJobStatus::Succeeded)
        .collect()
}

/// 投入の ID `submission_id` の切削シミュレーションのジョブ
fn submitted(workflow: &CamSimulationWorkflow, submission_id: &str) -> SimulationJob {
    workflow
        .jobs
        .simulation_jobs()
        .unwrap()
        .into_iter()
        .find(|job| job.submission_id == submission_id)
        .unwrap()
}

/// 底面 z = 80・頂点 z = 90 の四角錐を、直径 6 のボールエンドミルで粗い走査線で加工する入力
fn request(precision: f64) -> CamSimulationWorkflowRequest {
    let pyramid = TriangleMesh3D::new(
        vec![
            Point3D::new(20.0, 20.0, 80.0),
            Point3D::new(60.0, 20.0, 80.0),
            Point3D::new(60.0, 60.0, 80.0),
            Point3D::new(20.0, 60.0, 80.0),
            Point3D::new(40.0, 40.0, 90.0),
        ],
        vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
    )
    .unwrap();
    CamSimulationWorkflowRequest {
        solver_input: CamSolverInput {
            operation_id: "op-scan-1".to_string(),
            tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
            geometry: SolverGeometry::TriangleMesh(pyramid),
            operation: OperationSpec::Scanline(ScanlineParams {
                stepover: 6.0,
                feed_rate: 1200.0,
                tool_range: cam_core::ScanlineToolRange::default(),
                cut_direction: cam_core::ScanlineCutDirection::default(),
                angle_degrees: 0.0,
            }),
            units: LengthUnit::Millimeter,
            coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
            chord_tolerance: 0.01,
            tessellation_limits: TessellationLimits::default(),
            boundary: None,
        },
        stock: Aabb3D::new(
            Point3D::new(10.0, 10.0, 75.0),
            Point3D::new(70.0, 70.0, 92.0),
        ),
        precision,
        snapshot_interval: SnapshotInterval::default(),
    }
}

#[test]
fn submitting_only_enqueues_the_jobs_without_waiting_for_the_computation() {
    let mut workflow = CamSimulationWorkflow::in_process();
    let started = Instant::now();
    let submission = submit(&mut workflow, request(0.01)).unwrap();
    let submitting = started.elapsed();
    // 投入しただけでは実行されず、取り込めない
    assert!(succeeded_simulations(&workflow).is_empty());
    assert_eq!(
        workflow
            .import(submitted(&workflow, &submission.submission_id).job_id)
            .unwrap_err(),
        CamSimulationWorkflowError::NotSucceededSimulation(
            submitted(&workflow, &submission.submission_id).job_id
        )
    );

    let started = Instant::now();
    assert_eq!(workflow.run_queued_jobs().unwrap(), 2);
    let running = started.elapsed();
    assert!(
        submitting * 10 < running,
        "submitting {submitting:?}, running {running:?}"
    );
}

#[test]
fn succeeded_simulations_are_imported_with_their_toolpaths() {
    let mut workflow = CamSimulationWorkflow::in_process();
    let submissions: Vec<CamSimulationSubmission> = (0..2)
        .map(|_| submit(&mut workflow, request(0.01)).unwrap())
        .collect();
    assert_eq!(workflow.run_queued_jobs().unwrap(), 4);
    let succeeded: Vec<u64> = succeeded_simulations(&workflow)
        .iter()
        .map(|job| job.job_id)
        .collect();
    assert_eq!(
        succeeded,
        submissions
            .iter()
            .map(|submission| submitted(&workflow, &submission.submission_id).job_id)
            .collect::<Vec<_>>()
    );
    for submission in submissions {
        let result = workflow
            .import(submitted(&workflow, &submission.submission_id).job_id)
            .unwrap();
        let (_, toolpath) =
            read_toolpath_artifact_v1(&mut result.toolpath_artifact.as_slice()).unwrap();
        assert_eq!(toolpath.tool_id, "BEM6");
        let (_, simulation) = cam_sim::read_cutting_simulation_artifact_v1(
            &mut result.simulation_artifact.as_slice(),
        )
        .unwrap();
        assert!(simulation.snapshots.len() > 1);
        let first = simulation.snapshots[0].remaining_volume_mm3;
        let last = simulation.snapshots.last().unwrap().remaining_volume_mm3;
        assert!(last < first, "first = {first}, last = {last}");
    }
}

#[test]
fn failed_jobs_are_not_imported() {
    use cam_core::{MachiningBoundary, RectangleBoundary};

    let mut workflow = CamSimulationWorkflow::in_process();
    // 工具が形状に接触しない加工範囲では、ツールパス生成が失敗し、子の切削シミュレーションは実行されずに失敗する
    let mut outside = request(0.01);
    outside.solver_input.boundary = Some(MachiningBoundary::Rectangle(RectangleBoundary {
        rect_min: [200.0, 200.0],
        rect_max: [210.0, 210.0],
    }));
    let failed = submit(&mut workflow, outside).unwrap();
    let succeeded = submit(&mut workflow, request(0.01)).unwrap();
    workflow.run_queued_jobs().unwrap();

    let simulations: Vec<u64> = succeeded_simulations(&workflow)
        .iter()
        .map(|job| job.job_id)
        .collect();
    assert_eq!(
        simulations,
        vec![submitted(&workflow, &succeeded.submission_id).job_id]
    );
    assert_eq!(
        workflow
            .import(submitted(&workflow, &failed.submission_id).job_id)
            .unwrap_err(),
        CamSimulationWorkflowError::NotSucceededSimulation(
            submitted(&workflow, &failed.submission_id).job_id
        )
    );
    // ツールパス生成のジョブ（切削シミュレーションの 1 つ前の番号）は切削シミュレーションとしては取り込めない
    assert_eq!(
        workflow
            .import(submitted(&workflow, &succeeded.submission_id).job_id - 1)
            .unwrap_err(),
        CamSimulationWorkflowError::NotSucceededSimulation(
            submitted(&workflow, &succeeded.submission_id).job_id - 1
        )
    );
}

#[test]
fn precision_finer_than_the_toolpath_tolerance_is_not_submitted() {
    let mut workflow = CamSimulationWorkflow::in_process();
    assert_eq!(
        submit(&mut workflow, request(0.005)).unwrap_err(),
        CamSimulationWorkflowError::Conditions(vec![
            CamConditionError::PrecisionFinerThanTolerance {
                precision: 0.005,
                tolerance: 0.01,
            }
        ])
    );
    assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
}

/// 加工工程の中身（四角錐を、直径 6 のボールエンドミルで粗い走査線で仕上げる）
fn operation_content() -> MachiningOperationContent {
    MachiningOperationContent {
        name: "仕上げ".to_string(),
        stage: MachiningStage::Finish,
        spec: OperationSpec::Scanline(ScanlineParams {
            stepover: 6.0,
            feed_rate: 1200.0,
            tool_range: cam_core::ScanlineToolRange::default(),
            cut_direction: cam_core::ScanlineCutDirection::default(),
            angle_degrees: 0.0,
        }),
        tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
        boundary: None,
        tolerance_profile: ToleranceProfile::PressRough,
    }
}

/// XY の周りと上に 5 の余裕の素材
fn stock() -> MachiningStock {
    MachiningStock {
        margins: StockMargins {
            minus_x: 5.0,
            plus_x: 5.0,
            minus_y: 5.0,
            plus_y: 5.0,
            bottom: 0.0,
            top: 5.0,
        },
        precision: 0.01,
    }
}

/// 加工手順の保管先と、そこに置いた加工手順・加工工程の ID
struct Machining {
    store: MachiningSequenceStore,
    sequence: MachiningSequenceId,
    operation: MachiningOperationId,
}

impl Machining {
    /// 面 `face_ids` を加工の対象とし、素材 `stock()` で、加工工程を 1 つ持つ加工手順
    fn new(face_ids: Vec<FaceId>) -> Self {
        let mut store = MachiningSequenceStore::new();
        let sequence = store.create_sequence("手順".to_string(), face_ids, stock());
        let operation = store.add_operation(sequence, operation_content()).unwrap();
        Self {
            store,
            sequence,
            operation,
        }
    }

    fn check(&self, shapes: &ProductShapeStore) -> Result<(), CamSimulationWorkflowError> {
        CamSimulationWorkflow::check_operation(shapes, &self.store, self.sequence, self.operation)
    }

    fn submit(
        &self,
        workflow: &mut CamSimulationWorkflow,
        shapes: &ProductShapeStore,
    ) -> Result<CamSimulationSubmission, CamSimulationWorkflowError> {
        workflow.submit_operation(shapes, &self.store, self.sequence, self.operation)
    }

    fn latest(&self, workflow: &CamSimulationWorkflow) -> Option<u64> {
        workflow
            .latest_revision_simulation(&self.store, self.sequence, self.operation)
            .unwrap()
    }

    fn revision(&self) -> u32 {
        self.store
            .operation(self.sequence, self.operation)
            .unwrap()
            .revision()
    }

    fn set_stock(&mut self, stock: MachiningStock) {
        self.store.set_stock(self.sequence, stock).unwrap();
    }
}

/// 四角錐の 1 面だけの製品形状
fn pyramid_shape() -> (ProductShapeStore, FaceId) {
    use crate::product_shape::ProductFaceGeometry;

    let SolverGeometry::TriangleMesh(pyramid) = request(0.01).solver_input.geometry else {
        panic!("the test request uses a mesh");
    };
    let mut shapes = ProductShapeStore::new();
    let face = shapes.add_face(ProductFaceGeometry::Mesh(pyramid)).face_id;
    (shapes, face)
}

/// 切削シミュレーションの成果物の、最初の時点の素材の体積
fn initial_volume(result: &CamSimulationWorkflowResult) -> f64 {
    let (_, simulation) =
        cam_sim::read_cutting_simulation_artifact_v1(&mut result.simulation_artifact.as_slice())
            .unwrap();
    simulation.snapshots[0].remaining_volume_mm3
}

#[test]
fn operations_are_submitted_and_imported_with_the_submitted_inputs() {
    let (shapes, face) = pyramid_shape();
    let machining = Machining::new(vec![face]);
    let mut workflow = CamSimulationWorkflow::in_process();
    let submission = machining.submit(&mut workflow, &shapes).unwrap();
    assert_eq!(workflow.run_queued_jobs().unwrap(), 2);

    let result = workflow
        .import(submitted(&workflow, &submission.submission_id).job_id)
        .unwrap();
    let content = operation_content();
    assert_eq!(result.solver_input.tool, content.tool);
    assert_eq!(result.solver_input.operation, content.spec);
    assert_eq!(
        result.solver_input.operation_id,
        machining.operation.to_string()
    );
    // 精度プロファイル press_rough のトレランス
    assert_eq!(result.solver_input.chord_tolerance, 0.001);
    assert_eq!(result.simulation_input.tool, content.tool);
    assert_eq!(result.simulation_input.precision, stock().precision);
    assert_eq!(
        result.simulation_input.stock,
        Aabb3D::new(
            Point3D::new(15.0, 15.0, 80.0),
            Point3D::new(65.0, 65.0, 95.0)
        )
    );
    // 素材は四角錐（20〜60, 20〜60, 80〜90）を囲む箱に、XY の周りと上に 5 を足したもの（底の余裕は 0）
    let initial = initial_volume(&result);
    let expected = 50.0 * 50.0 * 15.0;
    assert!((initial - expected).abs() < 1e-6 * expected, "{initial}");
    let (_, simulation) =
        cam_sim::read_cutting_simulation_artifact_v1(&mut result.simulation_artifact.as_slice())
            .unwrap();
    assert!(simulation.snapshots.last().unwrap().remaining_volume_mm3 < initial);
}

#[test]
fn jobs_submitted_from_an_operation_have_its_id_and_revision() {
    let (shapes, face) = pyramid_shape();
    let machining = Machining::new(vec![face]);
    let mut workflow = CamSimulationWorkflow::in_process();
    // 工程から投入していないジョブは、工程から引かない
    let unrelated = submit(&mut workflow, request(0.01)).unwrap();
    let submission = machining.submit(&mut workflow, &shapes).unwrap();

    let expected = CamJobSource {
        source_id: machining.operation.to_string(),
        revision: 1,
    };
    assert_eq!(
        submitted(&workflow, &submission.submission_id).source,
        Some(expected)
    );
    // 実行するまでは、取り込める結果がない
    assert!(
        workflow
            .operation_simulations(machining.operation)
            .unwrap()
            .iter()
            .all(|job| job.status == CamJobStatus::Queued)
    );
    assert_eq!(machining.latest(&workflow), None);
    workflow.run_queued_jobs().unwrap();
    let simulations: Vec<u64> = workflow
        .operation_simulations(machining.operation)
        .unwrap()
        .iter()
        .map(|job| job.job_id)
        .collect();
    assert_eq!(
        simulations,
        vec![submitted(&workflow, &submission.submission_id).job_id]
    );
    assert!(!simulations.contains(&submitted(&workflow, &unrelated.submission_id).job_id));
}

#[test]
fn the_latest_revision_is_imported_and_the_previous_revisions_are_kept() {
    let (shapes, face) = pyramid_shape();
    let mut machining = Machining::new(vec![face]);
    let mut workflow = CamSimulationWorkflow::in_process();
    let first = machining.submit(&mut workflow, &shapes).unwrap();
    workflow.run_queued_jobs().unwrap();
    assert_eq!(
        machining.latest(&workflow),
        Some(submitted(&workflow, &first.submission_id).job_id)
    );

    // 素材の上の余裕を変えると、工程の改訂の番号が増え、前の改訂の結果は今の改訂の結果ではなくなる
    let mut changed = stock();
    changed.margins.top = 10.0;
    machining.set_stock(changed);
    assert_eq!(machining.revision(), 2);
    assert_eq!(machining.latest(&workflow), None);

    let second = machining.submit(&mut workflow, &shapes).unwrap();
    workflow.run_queued_jobs().unwrap();
    let latest = machining.latest(&workflow).unwrap();
    assert_eq!(latest, submitted(&workflow, &second.submission_id).job_id);
    assert_eq!(
        submitted(&workflow, &second.submission_id)
            .source
            .unwrap()
            .revision,
        2
    );
    // 新しい改訂の結果は、上に 10 を足した素材から削る
    let initial = initial_volume(&workflow.import(latest).unwrap());
    let expected = 50.0 * 50.0 * 20.0;
    assert!((initial - expected).abs() < 1e-6 * expected, "{initial}");

    // 前の改訂の結果も引けて、取り込める
    let simulations: Vec<u64> = workflow
        .operation_simulations(machining.operation)
        .unwrap()
        .iter()
        .map(|job| job.job_id)
        .collect();
    assert_eq!(
        simulations,
        vec![
            submitted(&workflow, &first.submission_id).job_id,
            submitted(&workflow, &second.submission_id).job_id
        ]
    );
    let initial = initial_volume(
        &workflow
            .import(submitted(&workflow, &first.submission_id).job_id)
            .unwrap(),
    );
    let expected = 50.0 * 50.0 * 15.0;
    assert!((initial - expected).abs() < 1e-6 * expected, "{initial}");
}

#[test]
fn inverse_offset_shapes_are_built_from_the_operation() {
    let (shapes, face) = pyramid_shape();
    let machining = Machining::new(vec![face]);
    let shape = CamSimulationWorkflow::inverse_offset_shape(
        &shapes,
        &machining.store,
        machining.sequence,
        machining.operation,
    )
    .unwrap();
    // 工程の工具（半径 3 のボール）で、四角錐（頂点 (40, 40, 90)）の頂点の上の工具先端は頂点の高さ
    assert_eq!(
        shape.shape(),
        cam_algorithms::CutterShape::Ball { radius: 3.0 }
    );
    assert!((shape.tip_height_at(40.0, 40.0).unwrap() - 90.0).abs() < 1e-9);
    // 投入前の確認の誤りは、そのまま返す
    let empty = Machining::new(Vec::new());
    assert_eq!(
        CamSimulationWorkflow::inverse_offset_shape(
            &shapes,
            &empty.store,
            empty.sequence,
            empty.operation
        )
        .unwrap_err(),
        CamSimulationWorkflowError::ProductShape(ProductShapeError::NoFaces)
    );
}

#[test]
fn inverse_offset_shapes_are_split_into_display_parts() {
    let (shapes, face) = pyramid_shape();
    let machining = Machining::new(vec![face]);
    let parts = |angle_step: f64| {
        CamSimulationWorkflow::inverse_offset_shape_parts(
            &shapes,
            &machining.store,
            machining.sequence,
            machining.operation,
            angle_step,
        )
    };
    let split = parts(10f64.to_radians()).unwrap();
    assert!(!split.is_empty());
    assert!(split.iter().all(|part| !part.triangles.is_empty()));
    // 表示用のきざみの範囲の外は、表示の誤り
    assert!(matches!(
        parts(0.0),
        Err(CamSimulationWorkflowError::InverseOffsetDisplay(_))
    ));
}

#[test]
fn operations_that_cannot_be_machined_are_not_submitted() {
    let (shapes, face) = pyramid_shape();
    let mut workflow = CamSimulationWorkflow::in_process();
    let empty = Machining::new(Vec::new());
    assert_eq!(
        empty.submit(&mut workflow, &shapes).unwrap_err(),
        CamSimulationWorkflowError::ProductShape(ProductShapeError::NoFaces)
    );
    // 加工の対象の面が製品形状にない（参照が切れた）
    let missing = FaceId::new_random();
    let broken = Machining::new(vec![face, missing]);
    assert_eq!(
        broken.submit(&mut workflow, &shapes).unwrap_err(),
        CamSimulationWorkflowError::ProductShape(ProductShapeError::UnknownFace(missing))
    );
    // 加工手順にない加工工程と、保管していない加工手順
    let other = Machining::new(vec![face]);
    assert_eq!(
        workflow
            .submit_operation(&shapes, &other.store, other.sequence, empty.operation)
            .unwrap_err(),
        CamSimulationWorkflowError::Sequence(MachiningSequenceError::UnknownOperation(
            empty.operation
        ))
    );
    assert_eq!(
        workflow
            .submit_operation(&shapes, &other.store, empty.sequence, empty.operation)
            .unwrap_err(),
        CamSimulationWorkflowError::Sequence(MachiningSequenceError::UnknownSequence(
            empty.sequence
        ))
    );
    assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
}

#[test]
fn stock_without_xy_margins_has_the_xy_size_of_the_product_shape() {
    let (shapes, face) = pyramid_shape();
    let mut machining = Machining::new(vec![face]);
    machining.set_stock(MachiningStock {
        margins: StockMargins {
            top: 5.0,
            ..StockMargins::default()
        },
        ..stock()
    });
    let mut workflow = CamSimulationWorkflow::in_process();
    let submission = machining.submit(&mut workflow, &shapes).unwrap();
    assert_eq!(workflow.run_queued_jobs().unwrap(), 2);

    // 素材は四角錐（20〜60, 20〜60, 80〜90）を囲む箱の XY の範囲のまま、上に 5 を足したもの
    let initial = initial_volume(
        &workflow
            .import(submitted(&workflow, &submission.submission_id).job_id)
            .unwrap(),
    );
    let expected = 40.0 * 40.0 * 15.0;
    assert!((initial - expected).abs() < 1e-6 * expected, "{initial}");
}

#[test]
fn negative_stock_margins_are_not_submitted() {
    let (shapes, face) = pyramid_shape();
    let mut machining = Machining::new(vec![face]);
    let mut negative = stock();
    negative.margins.minus_x = -1.0;
    machining.set_stock(negative);
    let mut workflow = CamSimulationWorkflow::in_process();
    assert_eq!(
        machining.submit(&mut workflow, &shapes).unwrap_err(),
        CamSimulationWorkflowError::Conditions(vec![CamConditionError::InvalidStockMargins(
            negative.margins
        )])
    );
    assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
}

#[test]
fn all_condition_errors_are_reported_before_submitting() {
    let (shapes, face) = pyramid_shape();
    let mut machining = Machining::new(vec![face]);
    let mut invalid = stock();
    invalid.margins.top = f64::NAN;
    // press_rough のトレランス 0.001 より細かい精度
    invalid.precision = 0.0005;
    machining.set_stock(invalid);
    let OperationSpec::Scanline(mut scanline) = operation_content().spec;
    // ピックフィードが工具径 6 を超える
    scanline.stepover = 7.0;
    machining
        .store
        .set_operation_content(
            machining.sequence,
            machining.operation,
            MachiningOperationContent {
                spec: OperationSpec::Scanline(scanline),
                // 工具長が 0
                tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 0.0),
                ..operation_content()
            },
        )
        .unwrap();
    let Err(CamSimulationWorkflowError::Conditions(errors)) = machining.check(&shapes) else {
        panic!("the conditions must be rejected");
    };
    let [
        CamConditionError::InvalidStockMargins(rejected_margins),
        CamConditionError::PrecisionFinerThanTolerance {
            precision,
            tolerance,
        },
        CamConditionError::StepoverExceedsToolDiameter { tool_diameter },
        CamConditionError::InvalidTool,
    ] = &errors[..]
    else {
        panic!("unexpected errors: {errors:?}");
    };
    assert!(rejected_margins.top.is_nan() && rejected_margins.minus_x == invalid.margins.minus_x);
    assert_eq!((*precision, *tolerance), (0.0005, 0.001));
    // 確かめた工具（直径 6）の径
    assert_eq!(*tool_diameter, 6.0);
    let fields: Vec<CamConditionField> = errors.iter().map(CamConditionError::field).collect();
    assert_eq!(
        fields,
        [
            CamConditionField::Stock,
            CamConditionField::Precision,
            CamConditionField::Machining,
            CamConditionField::Tool,
        ]
    );

    // 投入も同じ誤りで拒み、キューに入れない
    let mut workflow = CamSimulationWorkflow::in_process();
    let Err(CamSimulationWorkflowError::Conditions(rejected)) =
        machining.submit(&mut workflow, &shapes)
    else {
        panic!("the conditions must be rejected");
    };
    assert_eq!(rejected.len(), errors.len());
    assert_eq!(workflow.run_queued_jobs().unwrap(), 0);
}

#[test]
fn precision_beyond_the_line_limit_is_reported_with_the_finest_precision() {
    let (shapes, face) = pyramid_shape();
    let mut machining = Machining::new(vec![face]);
    let mut wide = stock();
    wide.margins.minus_x = 1000.0;
    wide.margins.plus_x = 1000.0;
    wide.margins.minus_y = 1000.0;
    wide.margins.plus_y = 1000.0;
    wide.precision = 0.001;
    machining.set_stock(wide);
    let Err(CamSimulationWorkflowError::Conditions(errors)) = machining.check(&shapes) else {
        panic!("the precision must be rejected");
    };
    let [CamConditionError::TooManyDexelLines { finest_precision }] = errors[..] else {
        panic!("unexpected errors: {errors:?}");
    };
    assert!(finest_precision > wide.precision);
    wide.precision = finest_precision;
    machining.set_stock(wide);
    assert_eq!(machining.check(&shapes), Ok(()));
}

#[test]
fn the_status_of_the_latest_revision_follows_its_jobs() {
    let (shapes, face) = pyramid_shape();
    let mut machining = Machining::new(vec![face]);
    let mut workflow = CamSimulationWorkflow::in_process();
    let status = |machining: &Machining, workflow: &CamSimulationWorkflow| {
        workflow
            .latest_revision_job_status(&machining.store, machining.sequence, machining.operation)
            .unwrap()
    };
    assert_eq!(status(&machining, &workflow), None);
    machining.submit(&mut workflow, &shapes).unwrap();
    assert_eq!(
        status(&machining, &workflow),
        Some(OperationJobStatus::Queued)
    );
    workflow.run_queued_jobs().unwrap();
    assert_eq!(
        status(&machining, &workflow),
        Some(OperationJobStatus::Succeeded)
    );
    // 条件を変えると、今の改訂の計算はまだない
    let mut changed = stock();
    changed.margins.top = 10.0;
    machining.set_stock(changed);
    assert_eq!(status(&machining, &workflow), None);
}
