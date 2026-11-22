//! 区間の切削シミュレーション（前の区間の終わりの素材から始める）の試験

use cam_algorithms::SolverGeometry;
use cam_core::{
    ContourLevelPath, CoordinateFrame, CuttingDirection, LengthUnit, PathSegment, SegmentType,
    Tool, ToolPath,
};
use geo_algorithms::{Aabb3D, Point3D, TriangleMesh3D, TriangleMesh3DConstructor};

use crate::simulation_artifact::read_cutting_simulation_artifact_v1;
use crate::stock_model::StockModel;
use crate::tri_dexel::{DexelGrid, Scratch, dexel_spacing, grid_layouts};
use crate::{
    CuttingSimulationInput, CuttingSimulationReplay, CuttingSimulationRun, DexelGridStates,
    SnapshotInterval, run_cutting_simulation,
};

/// 格子を決める工具の半径（2 つの工程の工具のうち小さいほう）
const GRID_TOOL_RADIUS: f64 = 1.0;
const PRECISION: f64 = 0.05;

fn stock() -> Aabb3D<f64> {
    Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(20.0, 20.0, 10.0))
}

fn rough_tool() -> Tool<f64> {
    Tool::flat_end_mill("ROUGH".to_string(), 4.0, 30.0)
}

fn finish_tool() -> Tool<f64> {
    Tool::ball_end_mill("FINISH".to_string(), 2.0, 30.0)
}

fn input(tool: Tool<f64>) -> CuttingSimulationInput {
    CuttingSimulationInput {
        stock: stock(),
        tool,
        precision: PRECISION,
        grid_tool_radius: GRID_TOOL_RADIUS,
        snapshot_interval: SnapshotInterval::default(),
        // 素材の底の正方形
        product: SolverGeometry::TriangleMesh(
            TriangleMesh3D::new(
                vec![
                    Point3D::new(0.0, 0.0, 0.0),
                    Point3D::new(20.0, 0.0, 0.0),
                    Point3D::new(20.0, 20.0, 0.0),
                    Point3D::new(0.0, 20.0, 0.0),
                ],
                vec![[0, 1, 2], [0, 2, 3]],
            )
            .unwrap(),
        ),
        tolerance: 0.01,
    }
}

/// 点の列を切削の区間で結んだツールパス
fn toolpath(tool: &Tool<f64>, points: &[[f64; 3]]) -> ToolPath<f64> {
    ToolPath::new(
        tool.id.clone(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            points
                .windows(2)
                .map(|pair| {
                    let [a, b] = [pair[0], pair[1]].map(|[x, y, z]| Point3D::new(x, y, z));
                    PathSegment::new_line(a, b, SegmentType::Cutting { feed_rate: 300.0 })
                })
                .collect(),
        )],
        vec![],
    )
}

/// 荒の工程の経路（X の向きの溝）
fn rough_path() -> ToolPath<f64> {
    toolpath(
        &rough_tool(),
        &[[2.0, 6.0, 8.0], [18.0, 6.0, 8.0], [18.0, 12.0, 8.0]],
    )
}

/// 仕上げの工程の経路（荒の溝を Y の向きに横切る、より深い溝）
fn finish_path() -> ToolPath<f64> {
    toolpath(&finish_tool(), &[[6.0, 2.0, 6.0], [6.0, 18.0, 6.0]])
}

fn toolpath_bytes(toolpath: &ToolPath<f64>) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut header = cam_core::ArtifactHeaderV1::new(cam_core::ArtifactKind::ToolPath, 0);
    let mut payload = Vec::new();
    cam_core::write_toolpath_payload_v1(&mut payload, toolpath).unwrap();
    header.payload_len = payload.len() as u64;
    header.unit = LengthUnit::Millimeter;
    header.frame = CoordinateFrame::WorldRightHandedZUp;
    header.write_to(&mut bytes).unwrap();
    bytes.extend_from_slice(&payload);
    bytes
}

fn run(
    tool: Tool<f64>,
    toolpath: &ToolPath<f64>,
    start: Option<&[u8]>,
) -> Result<CuttingSimulationRun, crate::JobRunFailure> {
    let mut start = start;
    run_cutting_simulation(
        &input(tool),
        &mut toolpath_bytes(toolpath).as_slice(),
        start.as_mut().map(|bytes| bytes as &mut dyn std::io::Read),
        &mut |_, _| {},
    )
}

/// 手順の格子で、素材の範囲から 2 つの工程の掃引体を加工の順に差し引いた状態
fn cut_in_one_go() -> [DexelGrid; 3] {
    let spacing = dexel_spacing(GRID_TOOL_RADIUS, PRECISION);
    let mut grids = grid_layouts(&stock(), spacing).map(|layout| DexelGrid::from_layout(&layout));
    let mut scratch = Scratch::default();
    for (path, tool) in [(rough_path(), rough_tool()), (finish_path(), finish_tool())] {
        let model = StockModel::from_toolpath(stock(), &path, &tool).unwrap();
        for primitive in model.primitives() {
            for grid in &mut grids {
                grid.subtract(primitive, &mut scratch);
            }
        }
    }
    grids
}

fn assert_states_eq(states: &DexelGridStates, grids: &[DexelGrid; 3]) {
    for (axis, grid) in grids.iter().enumerate() {
        for (index, intervals) in states.intervals[axis].iter().enumerate() {
            assert_eq!(intervals.as_slice(), grid.current(index), "{axis} {index}");
            assert_eq!(
                states.normals[axis][index].as_slice(),
                grid.current_normals(index),
                "{axis} {index}"
            );
        }
    }
}

#[test]
fn the_next_segment_starts_from_the_end_stock_of_the_previous_segment() {
    let rough = run(rough_tool(), &rough_path(), None).unwrap();
    let finish = run(finish_tool(), &finish_path(), Some(&rough.stock)).unwrap();

    let (_, rough_artifact) =
        read_cutting_simulation_artifact_v1(&mut &rough.artifact[..]).unwrap();
    let (_, artifact) = read_cutting_simulation_artifact_v1(&mut &finish.artifact[..]).unwrap();
    assert!(rough_artifact.start_stock.len() == 0);
    assert!(artifact.start_stock.len() > 0);
    assert_eq!(artifact.grids, rough_artifact.grids);

    // 2 つの区間に分けて削った素材は、続けて削った素材と同じ
    let last = artifact.keyframe_changes.keyframe_count() - 1;
    let end = artifact.dexel_states_at_keyframe(last);
    assert_states_eq(&end, &cut_in_one_go());

    // 仕上げの区間の始まりの残りの体積は、荒の区間の終わりの残りの体積
    let rough_end = rough_artifact
        .snapshots
        .last()
        .unwrap()
        .remaining_volume_mm3;
    let finish_start = artifact.snapshots[0].remaining_volume_mm3;
    assert!((rough_end - finish_start).abs() <= 1e-9 * rough_end);
    assert!(artifact.snapshots.last().unwrap().remaining_volume_mm3 < finish_start);
}

#[test]
fn replaying_a_segment_starts_and_rewinds_to_the_start_stock() {
    let rough = run(rough_tool(), &rough_path(), None).unwrap();
    let finish = run(finish_tool(), &finish_path(), Some(&rough.stock)).unwrap();
    let (_, rough_artifact) =
        read_cutting_simulation_artifact_v1(&mut &rough.artifact[..]).unwrap();
    let (_, artifact) = read_cutting_simulation_artifact_v1(&mut &finish.artifact[..]).unwrap();
    let rough_end = rough_artifact
        .dexel_states_at_keyframe(rough_artifact.keyframe_changes.keyframe_count() - 1);
    let last_keyframe = artifact.keyframe_changes.keyframe_count() - 1;
    let finish_end = artifact.dexel_states_at_keyframe(last_keyframe);
    assert_eq!(artifact.snapshot_applied[0], 0);

    let mut replay =
        CuttingSimulationReplay::new(artifact.clone(), &finish_path(), &finish_tool()).unwrap();
    assert_eq!(replay.states(), rough_end);
    replay.seek(artifact.snapshots.len() - 1).count();
    assert_eq!(replay.states(), finish_end);
    // 最初の時点へ飛ぶと、キーフレームの前（初期の状態）から作り直す
    replay.seek(0).count();
    assert_eq!(replay.states(), rough_end);
}

#[test]
fn a_start_stock_on_another_grid_is_invalid_input() {
    let rough = run(rough_tool(), &rough_path(), None).unwrap();
    let other_grid = CuttingSimulationInput {
        grid_tool_radius: 0.5,
        ..input(finish_tool())
    };
    let failure = run_cutting_simulation(
        &other_grid,
        &mut toolpath_bytes(&finish_path()).as_slice(),
        Some(&mut rough.stock.as_slice()),
        &mut |_, _| {},
    )
    .unwrap_err();
    assert_eq!(failure.code, "start-stock-mismatch");
    assert!(failure.is_invalid_input());

    let not_a_stock = run(finish_tool(), &finish_path(), Some(&rough.artifact)).unwrap_err();
    assert_eq!(not_a_stock.code, "kind-mismatch");
}
