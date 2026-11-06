use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use cam_algorithms::ProductSurface;
use cam_core::{
    ArtifactHeaderV1, ArtifactKind, CoordinateFrame, LengthUnit, write_toolpath_payload_v1,
};
use cam_sim::{CuttingSimulator, SnapshotInterval, write_cutting_simulation_artifact_v1};
use geo_algorithms::{Aabb3D, Point3D, TriangleMesh3D};
use geo_contracts::TriangleMesh3DConstructor;

use super::*;
use crate::toolpath::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType};

const PRECISION: f64 = 0.01;

fn tool() -> Tool<f64> {
    Tool::ball_end_mill("BEM2".to_string(), 2.0, 10.0)
}

/// 素材の上面を 3 本の往復の線で削るツールパスと切削シミュレーションの成果物（テストの間で 1 回だけ作る）
fn artifacts() -> &'static (Vec<u8>, Vec<u8>) {
    static ARTIFACTS: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();
    ARTIFACTS.get_or_init(|| {
        let points = [
            Point3D::new(-2.0, 1.0, 1.6),
            Point3D::new(32.0, 1.0, 1.6),
            Point3D::new(32.0, 2.0, 1.6),
            Point3D::new(-2.0, 2.0, 1.6),
            Point3D::new(-2.0, 3.0, 1.2),
            Point3D::new(32.0, 3.0, 1.2),
        ];
        let toolpath = ToolPath::new(
            "BEM2".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(
                0,
                1.6,
                points
                    .windows(2)
                    .map(|pair| {
                        PathSegment::new_line(
                            pair[0],
                            pair[1],
                            SegmentType::Cutting { feed_rate: 1200.0 },
                        )
                    })
                    .collect(),
            )],
            vec![],
        );
        let mut payload = Vec::new();
        write_toolpath_payload_v1(&mut payload, &toolpath).unwrap();
        let mut toolpath_artifact = Vec::new();
        ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64)
            .write_to(&mut toolpath_artifact)
            .unwrap();
        toolpath_artifact.extend_from_slice(&payload);

        let stock = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(30.0, 4.0, 2.0));
        let mut simulator = CuttingSimulator::new(stock, SnapshotInterval::default());
        simulator.simulate(&toolpath, &tool(), PRECISION).unwrap();
        // 製品形状は高さ 1.6 の平面（y = 3 の線は 0.4 深く削り込む）
        let product = TriangleMesh3D::new(
            vec![
                Point3D::new(-5.0, -5.0, 1.6),
                Point3D::new(35.0, -5.0, 1.6),
                Point3D::new(35.0, 9.0, 1.6),
                Point3D::new(-5.0, 9.0, 1.6),
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        )
        .unwrap();
        simulator.compare_with_product(&ProductSurface::new(&product).unwrap(), 0.001);
        let mut simulation_artifact = Vec::new();
        write_cutting_simulation_artifact_v1(
            &mut simulation_artifact,
            &simulator.artifact().unwrap(),
            LengthUnit::Millimeter,
            CoordinateFrame::WorldRightHandedZUp,
        )
        .unwrap();
        (toolpath_artifact, simulation_artifact)
    })
}

fn playback() -> CuttingSimulationPlayback {
    let (toolpath, simulation) = artifacts();
    CuttingSimulationPlayback::from_artifacts(toolpath, simulation, &tool(), PRECISION).unwrap()
}

/// ブロックごとの素材の面の頂点の位置
fn positions(blocks: &HashMap<[usize; 3], SurfaceMesh>) -> HashMap<[usize; 3], Vec<[f64; 3]>> {
    blocks
        .iter()
        .filter(|(_, mesh)| !mesh.indices.is_empty())
        .map(|(&block, mesh)| (block, mesh.positions.clone()))
        .collect()
}

fn by_block(blocks: Vec<([usize; 3], SurfaceMesh)>) -> HashMap<[usize; 3], SurfaceMesh> {
    blocks.into_iter().collect()
}

#[test]
fn the_playback_holds_the_toolpath_segments_and_snapshots_of_the_artifacts() {
    let playback = playback();
    assert!(playback.snapshots.len() > 2);
    assert_eq!(playback.stock.frame_count(), playback.snapshots.len());
    // 3 本の往復の線と、その間の 2 本の移動
    assert_eq!(playback.segments.len(), 5);
    assert!((playback.stock_top - 2.0).abs() < PRECISION);
    assert!(!playback.stock.stock_blocks().is_empty());
}

#[test]
fn only_the_last_frame_is_split_by_the_comparison_when_asked() {
    let only = |blocks: &[([usize; 3], SurfaceMesh)], class: DeviationClass| {
        blocks
            .iter()
            .all(|(_, mesh)| mesh.classes.iter().all(|&found| found == class))
    };
    let mut stock = playback().stock;
    let last = stock.frame_count() - 1;
    let all = stock.all_blocks().len();
    // 頼まなければ、最後の時点も分けない
    assert!(only(
        &stock.seek(last, false).1,
        DeviationClass::NotCompared
    ));
    // 頼むと、すべてのブロックを作り直して分ける
    let (frame, compared) = stock.seek(last, true);
    assert_eq!(frame, last);
    assert_eq!(compared.len(), all);
    let classes: HashSet<DeviationClass> = compared
        .iter()
        .flat_map(|(_, mesh)| mesh.classes.iter().copied())
        .collect();
    assert!(classes.contains(&DeviationClass::Gouge), "{classes:?}");
    assert!(classes.contains(&DeviationClass::Uncut), "{classes:?}");
    for (_, mesh) in &compared {
        assert_eq!(mesh.classes.len() * 3, mesh.indices.len());
    }
    // 最後の時点のほかは、頼んでも分けず、分けた面からはすべてのブロックを作り直す
    let (_, before) = stock.seek(last - 1, true);
    assert_eq!(before.len(), all);
    assert!(only(&before, DeviationClass::NotCompared));
}

#[test]
fn updating_changed_blocks_forward_and_backward_matches_the_state_at_each_frame() {
    let mut stock = playback().stock;
    let mut shown = by_block(stock.stock_blocks());
    let last = stock.frame_count() - 1;
    for frame in [1, last / 2, last, last - 1, last / 2, 1, 0, last] {
        shown.extend(stock.seek(frame, false).1);
        assert_eq!(stock.replay.snapshot(), frame);
        assert_eq!(
            positions(&shown),
            positions(&by_block(stock.stock_blocks())),
            "frame {frame}"
        );
    }
}

#[test]
fn frames_out_of_range_move_to_the_last_frame() {
    let mut stock = playback().stock;
    let last = stock.frame_count() - 1;
    assert_eq!(stock.seek(last + 10, false).0, last);
}

#[test]
fn broken_artifacts_are_rejected() {
    let (toolpath, simulation) = artifacts();
    let truncated = &simulation[..simulation.len() - 1];
    assert!(matches!(
        CuttingSimulationPlayback::from_artifacts(toolpath, truncated, &tool(), PRECISION),
        Err(CuttingSimulationPlaybackError::Artifact(_))
    ));
    assert!(matches!(
        CuttingSimulationPlayback::from_artifacts(simulation, simulation, &tool(), PRECISION),
        Err(CuttingSimulationPlaybackError::Artifact(_))
    ));
}
