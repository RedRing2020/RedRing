//! 切削シミュレーションの成果物（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §5）
//!
//! 共通のヘッダ（種別 `ArtifactKind::CuttingSimulation`・版・単位・座標系）は cam_core の成果物の形式を使い、
//! 中身（スナップショットの時点・表示用のトリデクセルの格子・スナップショットごとの変化）は本モジュールが定義する。
//!
//! 中身の並び（数値はすべてリトルエンディアン）:
//! 1. スナップショットの数 `u32` と、各スナップショットの区間の番号 `u32`・区間のパラメータ・累積距離・残りの体積（`f64`）
//! 2. 3 方向（X・Y・Z の順）の格子: 向き `u8`、格子の 2 方向の最初の線の座標・間隔（`f64`×4）、本数（`u32`×2）、
//!    初期の区間（`f64`×2）
//! 3. スナップショットごとの変化: 変化の数 `u32` と、各変化の格子の番号 `u8`・デクセルの番号 `u32`・区間の数 `u32`・区間（`f64`×2）

use std::io::{Read, Write};
use std::ops::{Range, RangeInclusive};

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ArtifactPayload, BinaryFormatError, CoordinateFrame,
    LengthUnit, read_artifact_v1,
};

use crate::SimulationSnapshotExport;

/// 1 方向のデクセルの格子の配置
///
/// 格子の 2 方向は、向き `axis` 以外の座標軸を昇順に並べたもの。番号 `(i, j)` の線は、格子の 2 方向の座標が
/// `origin[0] + i·pitch[0]`・`origin[1] + j·pitch[1]` の位置にあり、デクセルの番号は `i·count[1] + j`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DexelGridLayout {
    /// デクセルの向き（0: X、1: Y、2: Z）
    pub axis: u8,
    pub origin: [f64; 2],
    pub pitch: [f64; 2],
    pub count: [u32; 2],
    /// 初期の区間（素材の範囲）
    pub initial: (f64, f64),
}

/// 1 本のデクセルの変化（変化後の区間は、`DexelChanges` の区間の配列の中にある）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DexelChange {
    grid: u8,
    index: u32,
    /// 変化後の区間の、区間の配列の中の範囲（始まりと終わり）
    intervals: [u32; 2],
}

/// 1 本のデクセルの区間（変化後、または戻した後）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DexelState<'a> {
    /// 格子の番号（`DexelGridLayout` の並びの番号）
    pub grid: u8,
    /// 格子の中のデクセルの番号
    pub index: u32,
    pub intervals: &'a [(f64, f64)],
}

/// 時点ごとのデクセルの変化（全時点の変化と区間を、時点の順にそれぞれ 1 つの配列にまとめて持つ）
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DexelChanges {
    changes: Vec<DexelChange>,
    intervals: Vec<(f64, f64)>,
    /// `snapshot_ends[k]` は、時点 `k` までの変化の件数
    snapshot_ends: Vec<usize>,
}

impl DexelChanges {
    /// 時点の数
    pub fn snapshot_count(&self) -> usize {
        self.snapshot_ends.len()
    }

    /// 時点 `snapshot` までの変化の件数
    pub fn count_until(&self, snapshot: usize) -> usize {
        self.snapshot_ends[snapshot]
    }

    /// 時点 `snapshot` の変化の、変化後の区間
    pub fn snapshot(&self, snapshot: usize) -> impl ExactSizeIterator<Item = DexelState<'_>> {
        self.states(self.numbers(snapshot..=snapshot))
    }

    /// 次に区切る時点の変化に、デクセルの変化後の区間を加える
    pub(crate) fn push(&mut self, grid: u8, index: u32, intervals: &[(f64, f64)]) {
        let start = slot(self.intervals.len());
        self.intervals.extend_from_slice(intervals);
        self.changes.push(DexelChange {
            grid,
            index,
            intervals: [start, slot(self.intervals.len())],
        });
    }

    /// それまでに加えた変化を、1 つの時点の変化として区切る
    pub(crate) fn end_snapshot(&mut self) {
        self.snapshot_ends.push(self.changes.len());
    }

    /// 時点の範囲 `snapshots` の変化の番号の範囲（時点の範囲が空なら空）
    fn numbers(&self, snapshots: RangeInclusive<usize>) -> Range<usize> {
        let (first, last) = snapshots.into_inner();
        let start = first
            .checked_sub(1)
            .map_or(0, |before| self.count_until(before));
        start..self.count_until(last).max(start)
    }

    fn states(
        &self,
        numbers: Range<usize>,
    ) -> impl ExactSizeIterator<Item = DexelState<'_>> + DoubleEndedIterator {
        self.changes[numbers]
            .iter()
            .map(|change| self.state(change))
    }

    fn state(&self, change: &DexelChange) -> DexelState<'_> {
        let [start, end] = change.intervals.map(|position| position as usize);
        DexelState {
            grid: change.grid,
            index: change.index,
            intervals: &self.intervals[start..end],
        }
    }
}

/// 区間の配列の中の位置
fn slot(position: usize) -> u32 {
    u32::try_from(position).expect("dexel change intervals fit in u32 positions")
}

/// 同じデクセルの前の変化がないことを表す番号（戻すと初期の区間になる）
const NO_PREVIOUS_CHANGE: u32 = u32::MAX;

/// 切削シミュレーションの成果物の中身
#[derive(Debug, Clone, PartialEq)]
pub struct CuttingSimulationArtifact {
    pub snapshots: Vec<SimulationSnapshotExport>,
    /// X・Y・Z の向きの格子
    pub grids: [DexelGridLayout; 3],
    /// 時点 `k` の変化は、時点 `k - 1`（`k = 0` では初期の状態）から時点 `k` への変化
    pub snapshot_changes: DexelChanges,
}

impl CuttingSimulationArtifact {
    /// スナップショット `snapshot` の時点の、各格子のデクセルの区間（初期の状態に変化を順に当てる）
    ///
    /// # Panics
    ///
    /// `snapshot` がスナップショットの数以上の場合。
    pub fn dexel_intervals_at(&self, snapshot: usize) -> [Vec<Vec<(f64, f64)>>; 3] {
        assert!(
            snapshot < self.snapshot_changes.snapshot_count(),
            "snapshot index out of range"
        );
        let mut grids = self.initial_intervals();
        for change in self.forward_changes(0..=snapshot) {
            let dexel = &mut grids[change.grid as usize][change.index as usize];
            dexel.clear();
            dexel.extend_from_slice(change.intervals);
        }
        grids
    }

    /// 時点の範囲 `snapshots` の変化を、時点の順に並べた変化後の区間
    pub fn forward_changes(
        &self,
        snapshots: RangeInclusive<usize>,
    ) -> impl Iterator<Item = DexelState<'_>> {
        self.snapshot_changes
            .states(self.snapshot_changes.numbers(snapshots))
    }

    /// 変化ごとの、同じデクセルの 1 つ前の変化の番号（`backward_changes` に渡す）
    ///
    /// # Panics
    ///
    /// 変化の件数が `u32` の番号に収まらない場合。
    pub fn previous_changes(&self) -> Vec<u32> {
        let mut last = self
            .grids
            .map(|grid| vec![NO_PREVIOUS_CHANGE; grid.count[0] as usize * grid.count[1] as usize]);
        self.snapshot_changes
            .changes
            .iter()
            .enumerate()
            .map(|(number, change)| {
                let number = u32::try_from(number).expect("dexel change count fits in u32");
                std::mem::replace(
                    &mut last[change.grid as usize][change.index as usize],
                    number,
                )
            })
            .collect()
    }

    /// 時点の範囲 `snapshots` の変化を後ろから順に戻した区間（`previous` は `previous_changes` の結果）
    ///
    /// すべて当てると、`snapshots` の最初の時点の 1 つ前の時点（最初の時点が 0 なら初期の状態）になる。
    pub fn backward_changes<'a>(
        &'a self,
        previous: &'a [u32],
        snapshots: RangeInclusive<usize>,
    ) -> impl Iterator<Item = DexelState<'a>> {
        let changes = &self.snapshot_changes;
        changes.numbers(snapshots).rev().map(move |number| {
            let change = &changes.changes[number];
            let intervals = match previous[number] {
                NO_PREVIOUS_CHANGE => {
                    std::slice::from_ref(&self.grids[change.grid as usize].initial)
                }
                before => changes.state(&changes.changes[before as usize]).intervals,
            };
            DexelState {
                grid: change.grid,
                index: change.index,
                intervals,
            }
        })
    }

    /// 初期の状態の、各格子のデクセルの区間
    fn initial_intervals(&self) -> [Vec<Vec<(f64, f64)>>; 3] {
        self.grids
            .map(|grid| vec![vec![grid.initial]; grid.count[0] as usize * grid.count[1] as usize])
    }
}

/// 成果物（共通のヘッダと中身）を書き出す
///
/// # Errors
///
/// 書き出しの失敗、または数が `u32` に収まらない場合。
pub fn write_cutting_simulation_artifact_v1<W: Write>(
    writer: &mut W,
    artifact: &CuttingSimulationArtifact,
    unit: LengthUnit,
    frame: CoordinateFrame,
) -> Result<(), BinaryFormatError> {
    let mut payload = Vec::new();
    write_count(&mut payload, artifact.snapshots.len(), "snapshots")?;
    for snapshot in &artifact.snapshots {
        write_count(&mut payload, snapshot.segment_index, "segment_index")?;
        for value in [
            snapshot.segment_t,
            snapshot.accumulated_distance_mm,
            snapshot.remaining_volume_mm3,
        ] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
    }
    for grid in &artifact.grids {
        payload.push(grid.axis);
        for value in [grid.origin[0], grid.origin[1], grid.pitch[0], grid.pitch[1]] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        for count in grid.count {
            payload.extend_from_slice(&count.to_le_bytes());
        }
        payload.extend_from_slice(&grid.initial.0.to_le_bytes());
        payload.extend_from_slice(&grid.initial.1.to_le_bytes());
    }
    let changes = &artifact.snapshot_changes;
    if changes.snapshot_count() != artifact.snapshots.len() {
        return Err(BinaryFormatError::InvalidPayload("snapshot_changes"));
    }
    for snapshot in 0..changes.snapshot_count() {
        let states = changes.snapshot(snapshot);
        write_count(&mut payload, states.len(), "changes")?;
        for change in states {
            payload.push(change.grid);
            payload.extend_from_slice(&change.index.to_le_bytes());
            write_count(&mut payload, change.intervals.len(), "intervals")?;
            for &(low, high) in change.intervals {
                payload.extend_from_slice(&low.to_le_bytes());
                payload.extend_from_slice(&high.to_le_bytes());
            }
        }
    }

    let mut header = ArtifactHeaderV1::new(ArtifactKind::CuttingSimulation, payload.len() as u64);
    header.unit = unit;
    header.frame = frame;
    header.write_to(writer)?;
    writer.write_all(&payload)?;
    Ok(())
}

/// 成果物を読み込む（種別・版・破損を検査する）
///
/// # Errors
///
/// 種別が切削シミュレーションでない、版に対応していない、途中で切れている、値が契約に違反している場合。
pub fn read_cutting_simulation_artifact_v1<R: Read>(
    reader: &mut R,
) -> Result<(ArtifactHeaderV1, CuttingSimulationArtifact), BinaryFormatError> {
    let (header, payload) = read_artifact_v1(reader)?;
    let ArtifactPayload::CuttingSimulation(bytes) = payload else {
        return Err(BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::CuttingSimulation,
            found: header.kind,
        });
    };
    let mut cursor = bytes.as_slice();
    let reader = &mut cursor;

    let snapshot_count = read_u32(reader)? as usize;
    let mut snapshots = Vec::with_capacity(snapshot_count.min(bytes.len()));
    for _ in 0..snapshot_count {
        snapshots.push(SimulationSnapshotExport {
            segment_index: read_u32(reader)? as usize,
            segment_t: read_f64(reader)?,
            accumulated_distance_mm: read_f64(reader)?,
            remaining_volume_mm3: read_f64(reader)?,
        });
    }

    let mut grids = [DexelGridLayout {
        axis: 0,
        origin: [0.0; 2],
        pitch: [0.0; 2],
        count: [0; 2],
        initial: (0.0, 0.0),
    }; 3];
    for (expected_axis, grid) in grids.iter_mut().enumerate() {
        let axis = read_u8(reader)?;
        if usize::from(axis) != expected_axis {
            return Err(BinaryFormatError::InvalidPayload("grid.axis"));
        }
        *grid = DexelGridLayout {
            axis,
            origin: [read_f64(reader)?, read_f64(reader)?],
            pitch: [read_f64(reader)?, read_f64(reader)?],
            count: [read_u32(reader)?, read_u32(reader)?],
            initial: (read_f64(reader)?, read_f64(reader)?),
        };
    }

    let mut snapshot_changes = DexelChanges::default();
    let mut intervals = Vec::new();
    for _ in 0..snapshot_count {
        let change_count = read_u32(reader)? as usize;
        for _ in 0..change_count {
            let grid = read_u8(reader)?;
            let index = read_u32(reader)?;
            let layout = grids
                .get(usize::from(grid))
                .ok_or(BinaryFormatError::InvalidPayload("change.grid"))?;
            if u64::from(index) >= u64::from(layout.count[0]) * u64::from(layout.count[1]) {
                return Err(BinaryFormatError::InvalidPayload("change.index"));
            }
            let interval_count = read_u32(reader)? as usize;
            intervals.clear();
            for _ in 0..interval_count {
                intervals.push((read_f64(reader)?, read_f64(reader)?));
            }
            snapshot_changes.push(grid, index, &intervals);
        }
        snapshot_changes.end_snapshot();
    }
    if !reader.is_empty() {
        return Err(BinaryFormatError::InvalidPayload("trailing bytes"));
    }

    Ok((
        header,
        CuttingSimulationArtifact {
            snapshots,
            grids,
            snapshot_changes,
        },
    ))
}

fn write_count(
    payload: &mut Vec<u8>,
    count: usize,
    field: &'static str,
) -> Result<(), BinaryFormatError> {
    let count = u32::try_from(count).map_err(|_| BinaryFormatError::LengthOverflow(field))?;
    payload.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn read_u8(reader: &mut &[u8]) -> Result<u8, BinaryFormatError> {
    let mut bytes = [0_u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

fn read_u32(reader: &mut &[u8]) -> Result<u32, BinaryFormatError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_f64(reader: &mut &[u8]) -> Result<f64, BinaryFormatError> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
    use geo_algorithms::{Aabb3D, Point3D};

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
    fn restored_states_match_the_dexels_after_the_simulation() {
        let simulator = simulated();
        let artifact = simulator.artifact().unwrap();
        let dexels = simulator.dexels.as_ref().unwrap();
        assert!(artifact.snapshots.len() > 2);
        // 最後の時点の復元は、シミュレーションの後のデクセルと一致する（途中の時点はトリデクセルのテストで確かめる）
        let last = artifact.snapshots.len() - 1;
        let restored = artifact.dexel_intervals_at(last);
        for (grid, restored_grid) in restored.iter().enumerate() {
            for (index, intervals) in restored_grid.iter().enumerate() {
                assert_eq!(
                    intervals.as_slice(),
                    dexels.current_intervals(grid, index),
                    "grid {grid} dexel {index}"
                );
            }
        }
        // 最初の時点は切削の前で、変化がない
        assert_eq!(artifact.snapshot_changes.snapshot(0).len(), 0);
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
}
