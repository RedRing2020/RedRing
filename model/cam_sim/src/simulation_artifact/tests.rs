//! `simulation_artifact` のテスト

use cam_algorithms::ProductSurface;
use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
use geo_algorithms::{Aabb3D, Point3D, TriangleMesh3D, TriangleMesh3DConstructor};

use super::*;
use crate::{CuttingSimulator, SnapshotInterval};

/// 10 × 10 × 4 の素材を、直径 2 のボールエンドミルで斜めに掘り下げてから水平に削った結果
fn simulated() -> CuttingSimulator {
    let points = [
        Point3D::new(1.0, 1.0, 0.5),
        Point3D::new(4.0, 3.0, -1.5),
        Point3D::new(8.0, 3.0, -1.5),
        Point3D::new(8.0, 8.0, -0.5),
    ];
    let toolpath = ToolPath::new(
        "BEM2".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            points
                .windows(2)
                .map(|pair| {
                    PathSegment::new_line(
                        pair[0],
                        pair[1],
                        SegmentType::Cutting { feed_rate: 100.0 },
                    )
                })
                .collect(),
        )],
        vec![],
    );
    let stock = Aabb3D::new(Point3D::new(0.0, 0.0, -4.0), Point3D::new(10.0, 10.0, 0.0));
    let mut simulator = CuttingSimulator::new(
        stock,
        SnapshotInterval::ByDistance {
            interval_mm: 2.0,
            include_segment_endpoints: true,
        },
    );
    simulator
        .simulate(
            &toolpath,
            &Tool::ball_end_mill("BEM2".to_string(), 2.0, 6.0),
            0.001,
        )
        .unwrap();
    simulator
}

fn written(artifact: &CuttingSimulationArtifact) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_cutting_simulation_artifact_v1(
        &mut bytes,
        artifact,
        LengthUnit::Millimeter,
        CoordinateFrame::WorldRightHandedZUp,
    )
    .unwrap();
    bytes
}

#[test]
fn artifact_round_trips_through_the_binary_format() {
    let artifact = simulated().artifact().unwrap();
    let bytes = written(&artifact);
    let (header, read) = read_cutting_simulation_artifact_v1(&mut bytes.as_slice()).unwrap();
    assert_eq!(header.kind, ArtifactKind::CuttingSimulation);
    assert_eq!(read, artifact);
}

#[test]
fn product_deviations_round_trip_and_must_match_the_endpoints_of_the_last_state() {
    let mut simulator = simulated();
    let product = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, -1.0),
            Point3D::new(10.0, 0.0, -1.0),
            Point3D::new(10.0, 10.0, -1.0),
            Point3D::new(0.0, 10.0, -1.0),
        ],
        vec![[0, 1, 2], [0, 2, 3]],
    )
    .unwrap();
    simulator.compare_with_product(&ProductSurface::new(&product).unwrap(), 0.001);
    let artifact = simulator.artifact().unwrap();
    let deviations = artifact.product_deviations.as_ref().unwrap();
    let (_, read) =
        read_cutting_simulation_artifact_v1(&mut written(&artifact).as_slice()).unwrap();
    // NaN（削っていない端点）を含むため、ビット列で比べる
    let bits = |deviations: &ProductDeviations| {
        deviations
            .distances
            .each_ref()
            .map(|distances| distances.iter().map(|d| d.to_bits()).collect::<Vec<_>>())
    };
    let read_deviations = read.product_deviations.as_ref().unwrap();
    assert_eq!(read_deviations.tolerance, deviations.tolerance);
    assert_eq!(bits(read_deviations), bits(deviations));

    // 端点の数が最後の時点の区間の端点の数と違う比較は読まない
    let mut broken = artifact.clone();
    broken.product_deviations.as_mut().unwrap().distances[2].pop();
    assert!(matches!(
        read_cutting_simulation_artifact_v1(&mut written(&broken).as_slice()),
        Err(BinaryFormatError::InvalidPayload(
            "product_deviations.count"
        ))
    ));
}

#[test]
fn unknown_sections_are_skipped() {
    let artifact = simulated().artifact().unwrap();
    let bytes = written(&artifact);
    let header_len = ArtifactHeaderV1::FIXED_HEADER_LEN;
    // 先頭に知らない種類の節を足し、ヘッダのペイロード長を合わせる
    let mut payload = Vec::new();
    write_section(&mut payload, 0x7FFF, b"future");
    payload.extend_from_slice(&bytes[header_len..]);
    let mut extended = Vec::new();
    ArtifactHeaderV1::new(ArtifactKind::CuttingSimulation, payload.len() as u64)
        .write_to(&mut extended)
        .unwrap();
    extended.extend_from_slice(&payload);
    let (_, read) = read_cutting_simulation_artifact_v1(&mut extended.as_slice()).unwrap();
    assert_eq!(read, artifact);
}

#[test]
fn artifacts_of_the_older_format_are_rejected() {
    let mut bytes = written(&simulated().artifact().unwrap());
    // ヘッダの version_minor（magic 4 バイト・version_major 2 バイトの後）を時点ごとの変化の形式の版にする
    bytes[6..8].copy_from_slice(&2_u16.to_le_bytes());
    assert!(matches!(
        read_cutting_simulation_artifact_v1(&mut bytes.as_slice()),
        Err(BinaryFormatError::UnsupportedMinorVersion { found_minor: 2, .. })
    ));
}

#[test]
fn restored_states_match_the_dexels_after_the_simulation() {
    let simulator = simulated();
    let artifact = simulator.artifact().unwrap();
    let dexels = simulator.dexels.as_ref().unwrap();
    assert!(artifact.snapshots.len() > 2);
    // 最後の時点はいつもキーフレームで、シミュレーションの後のデクセルと一致する（途中の時点は再生のテストで確かめる）
    let last = artifact.keyframe_snapshots.len() - 1;
    assert_eq!(
        artifact.keyframe_snapshots[last],
        artifact.snapshots.len() - 1
    );
    let restored = artifact.dexel_states_at_keyframe(last);
    for grid in 0..3 {
        for (index, intervals) in restored.intervals[grid].iter().enumerate() {
            assert_eq!(
                intervals.as_slice(),
                dexels.current_intervals(grid, index),
                "grid {grid} dexel {index}"
            );
            assert_eq!(
                restored.normals[grid][index].as_slice(),
                dexels.current_normals(grid, index),
                "grid {grid} dexel {index}"
            );
        }
    }
}

#[test]
fn broken_or_foreign_bytes_are_rejected() {
    let bytes = written(&simulated().artifact().unwrap());
    // 途中で切れている
    let truncated = &bytes[..bytes.len() - 3];
    assert!(read_cutting_simulation_artifact_v1(&mut &truncated[..]).is_err());
    // 余分なバイトがある（ヘッダのペイロード長より後は読まないため、ペイロードの末尾を壊す）
    let mut corrupted = bytes.clone();
    let header_len = ArtifactHeaderV1::FIXED_HEADER_LEN;
    corrupted[header_len] = 0xFF;
    assert!(read_cutting_simulation_artifact_v1(&mut corrupted.as_slice()).is_err());
    // 種別が異なる
    let mut toolpath_bytes = Vec::new();
    let mut payload = Vec::new();
    cam_core::write_toolpath_payload_v1(
        &mut payload,
        &ToolPath::new(
            "T".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![],
            vec![],
        ),
    )
    .unwrap();
    ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64)
        .write_to(&mut toolpath_bytes)
        .unwrap();
    toolpath_bytes.extend_from_slice(&payload);
    assert!(matches!(
        read_cutting_simulation_artifact_v1(&mut toolpath_bytes.as_slice()),
        Err(BinaryFormatError::ArtifactKindMismatch { .. })
    ));
}

/// 成果物 `artifact` を書いたバイト列の、Z の格子の線の本数を `count` に書き換えたもの（書き込みは上限を超える
/// 格子を拒むため、書いた後に書き換える）
fn written_with_grid_count(artifact: &CuttingSimulationArtifact, count: [u32; 2]) -> Vec<u8> {
    let mut bytes = written(artifact);
    // 格子の節の Z の格子: 軸（u8）・原点（f64 × 2）・間隔（f64 × 2）・線の本数（u32 × 2）・初めの区間（f64 × 2）
    let grid = &artifact.grids[2];
    let mut record = vec![grid.axis];
    for value in grid.origin.into_iter().chain(grid.pitch) {
        record.extend_from_slice(&value.to_le_bytes());
    }
    for value in grid.count {
        record.extend_from_slice(&value.to_le_bytes());
    }
    let at = bytes
        .windows(record.len())
        .position(|window| window == record.as_slice())
        .expect("the written grid is found");
    let count_at = at + 1 + 4 * 8;
    bytes[count_at..count_at + 4].copy_from_slice(&count[0].to_le_bytes());
    bytes[count_at + 4..count_at + 8].copy_from_slice(&count[1].to_le_bytes());
    bytes
}

#[test]
fn grids_beyond_the_dexel_line_limit_are_rejected() {
    let artifact = simulated().artifact().unwrap();
    for count in [[u32::MAX, u32::MAX], [4_000, 1_001]] {
        let bytes = written_with_grid_count(&artifact, count);
        assert!(matches!(
            read_cutting_simulation_artifact_v1(&mut bytes.as_slice()),
            Err(BinaryFormatError::InvalidPayload("grid.count"))
        ));

        // 書き込みも、読み込みで拒む格子を書かない
        let mut beyond = artifact.clone();
        beyond.grids[2].count = count;
        assert!(matches!(
            write_cutting_simulation_artifact_v1(
                &mut Vec::new(),
                &beyond,
                LengthUnit::Millimeter,
                CoordinateFrame::WorldRightHandedZUp,
            ),
            Err(BinaryFormatError::InvalidPayload("grid.count"))
        ));
    }
}

/// 擬似乱数（xorshift。試験の再現のため、種を決めて使う）
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

#[test]
fn corrupted_cutting_simulation_artifacts_do_not_crash_the_reader() {
    let valid = written(&simulated().artifact().unwrap());
    let mut random = XorShift(3);
    for _ in 0..300 {
        let mut bytes = valid.clone();
        match random.below(3) {
            0 => {
                for _ in 0..=random.below(4) {
                    let at = random.below(bytes.len());
                    bytes[at] = random.next() as u8;
                }
            }
            1 => {
                // 個数・長さに当たりうる 4 バイトを、大きな値にする
                let at = random.below(bytes.len().saturating_sub(4).max(1));
                let end = (at + 4).min(bytes.len());
                bytes[at..end].fill(0xFF);
            }
            _ => bytes.truncate(random.below(bytes.len())),
        }
        // 読めた場合は、再生の初めの状態（キーフレームからの作り直し）も作れること（時点のない成果物は、
        // 再生の前に誤りにするため除く）
        if let Ok((_, artifact)) = read_cutting_simulation_artifact_v1(&mut bytes.as_slice())
            && !artifact.snapshots.is_empty()
        {
            let _ = artifact.dexel_states_at_keyframe(0);
        }
    }
}
