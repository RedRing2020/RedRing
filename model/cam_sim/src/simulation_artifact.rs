//! 切削シミュレーションの成果物（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §5.2）
//!
//! 共通のヘッダ（種別 `ArtifactKind::CuttingSimulation`・版・単位・座標系）は cam_core の成果物の形式を使い、
//! 中身（スナップショットの時点・表示用のトリデクセルの格子・差分のキーフレーム）は本モジュールが定義する。
//! 時点ごとの変化は持たず、キーフレームのあいだの時点は正本から作り直す（`CuttingSimulationReplay`）。
//!
//! 中身は節の列で、各節は種類 `u16` と長さ `u64` に続く中身からなる（数値はすべてリトルエンディアン）。
//! 読み込みは、知らない種類の節を読み飛ばす。
//! 1. 時点の表: スナップショットの数 `u32` と、各スナップショットの区間の番号 `u32`・区間のパラメータ・累積距離・
//!    残りの体積（`f64`）・時点までに差し引いた掃引体の数 `u32`
//! 2. 格子: 3 方向（X・Y・Z の順）の、向き `u8`、格子の 2 方向の最初の線の座標・間隔（`f64`×4）、本数（`u32`×2）、
//!    初期の区間（`f64`×2）
//! 3. キーフレームの目次: キーフレームの数 `u32` と、各キーフレームの時点の番号 `u32`・キーフレームの節の中の
//!    位置と長さ（`u64`×2）
//! 4. キーフレーム: キーフレームごとに、変化の数 `u32` と、各変化の格子の番号 `u8`・デクセルの番号 `u32`・区間の数
//!    `u32`・区間ごとの始まり・終わりの座標（`f64`×2）と法線（量子化した値 `u16`×2 を 2 つ）

use std::io::{Read, Write};
use std::ops::Range;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ArtifactPayload, BinaryFormatError, CoordinateFrame,
    FORMAT_VERSION_MINOR_V1, LengthUnit, read_artifact_v1,
};

use crate::SimulationSnapshotExport;
use crate::dexel_normal::DexelNormal;
use crate::tri_dexel::initial_normals;

/// 中身を差分のキーフレームの節の列にした版より前の版は読まない
const SUPPORTED_MINORS: &[u16] = &[FORMAT_VERSION_MINOR_V1];

/// 節の種類
const SECTION_SNAPSHOTS: u16 = 1;
const SECTION_GRIDS: u16 = 2;
const SECTION_KEYFRAME_INDEX: u16 = 3;
const SECTION_KEYFRAMES: u16 = 4;

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

/// 1 本のデクセルの区間
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DexelState<'a> {
    /// 格子の番号（`DexelGridLayout` の並びの番号）
    pub grid: u8,
    /// 格子の中のデクセルの番号
    pub index: u32,
    pub intervals: &'a [(f64, f64)],
    /// `intervals` と同じ並びの、区間の始まり・終わりの材料の外向きの法線
    pub normals: &'a [[DexelNormal; 2]],
}

/// キーフレームごとのデクセルの変化（全キーフレームの変化と区間を、キーフレームの順にそれぞれ 1 つの配列にまとめて持つ）
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DexelChanges {
    changes: Vec<DexelChange>,
    intervals: Vec<(f64, f64)>,
    /// `intervals` と同じ並びの、区間の始まり・終わりの法線
    normals: Vec<[DexelNormal; 2]>,
    /// `keyframe_ends[k]` は、キーフレーム `k` までの変化の件数
    keyframe_ends: Vec<usize>,
}

impl DexelChanges {
    /// キーフレームの数
    pub fn keyframe_count(&self) -> usize {
        self.keyframe_ends.len()
    }

    /// キーフレーム `keyframe` の、前のキーフレーム（最初は初期の状態）から変わったデクセルの区間
    pub fn keyframe(&self, keyframe: usize) -> impl ExactSizeIterator<Item = DexelState<'_>> {
        self.states(self.numbers(keyframe))
    }

    /// 次に区切るキーフレームの変化に、デクセルの変化後の区間と法線を加える
    pub(crate) fn push(
        &mut self,
        grid: u8,
        index: u32,
        intervals: &[(f64, f64)],
        normals: &[[DexelNormal; 2]],
    ) {
        debug_assert_eq!(intervals.len(), normals.len());
        let start = slot(self.intervals.len());
        self.intervals.extend_from_slice(intervals);
        self.normals.extend_from_slice(normals);
        self.changes.push(DexelChange {
            grid,
            index,
            intervals: [start, slot(self.intervals.len())],
        });
    }

    /// それまでに加えた変化を、1 つのキーフレームの変化として区切る
    pub(crate) fn end_keyframe(&mut self) {
        self.keyframe_ends.push(self.changes.len());
    }

    /// 変化の件数
    pub(crate) fn len(&self) -> usize {
        self.changes.len()
    }

    /// 変化 `number` のキーフレームの番号
    pub(crate) fn keyframe_of(&self, number: usize) -> usize {
        self.keyframe_ends.partition_point(|&end| end <= number)
    }

    /// キーフレーム `keyframe` の変化の番号の範囲
    pub(crate) fn numbers(&self, keyframe: usize) -> Range<usize> {
        let start = keyframe
            .checked_sub(1)
            .map_or(0, |before| self.keyframe_ends[before]);
        start..self.keyframe_ends[keyframe]
    }

    /// 変化 `number` のデクセルの格子と番号
    pub(crate) fn dexel(&self, number: usize) -> (u8, u32) {
        let change = &self.changes[number];
        (change.grid, change.index)
    }

    fn states(&self, numbers: Range<usize>) -> impl ExactSizeIterator<Item = DexelState<'_>> {
        self.changes[numbers]
            .iter()
            .map(|change| self.state(change))
    }

    /// 変化 `number` の変化後の区間
    pub(crate) fn state_of(&self, number: usize) -> DexelState<'_> {
        self.state(&self.changes[number])
    }

    fn state(&self, change: &DexelChange) -> DexelState<'_> {
        let [start, end] = change.intervals.map(|position| position as usize);
        DexelState {
            grid: change.grid,
            index: change.index,
            intervals: &self.intervals[start..end],
            normals: &self.normals[start..end],
        }
    }
}

/// 区間の配列の中の位置
fn slot(position: usize) -> u32 {
    u32::try_from(position).expect("dexel change intervals fit in u32 positions")
}

/// 切削シミュレーションの成果物の中身
#[derive(Debug, Clone, PartialEq)]
pub struct CuttingSimulationArtifact {
    pub snapshots: Vec<SimulationSnapshotExport>,
    /// 各スナップショットの時点までに差し引いた掃引体の数（`snapshots` と同じ並び。昇順）
    pub snapshot_applied: Vec<usize>,
    /// X・Y・Z の向きの格子
    pub grids: [DexelGridLayout; 3],
    /// 各キーフレームのスナップショットの番号（昇順。最後はいつも最後のスナップショット）
    pub keyframe_snapshots: Vec<usize>,
    /// キーフレーム `k` の変化は、キーフレーム `k - 1`（`k = 0` では初期の状態）からキーフレーム `k` への変化
    pub keyframe_changes: DexelChanges,
}

impl CuttingSimulationArtifact {
    /// キーフレーム `keyframe` の時点の、各格子のデクセルの区間と区間の端の法線（初期の状態に変化を順に当てる）
    ///
    /// # Panics
    ///
    /// `keyframe` がキーフレームの数以上の場合。
    pub fn dexel_states_at_keyframe(&self, keyframe: usize) -> DexelGridStates {
        assert!(
            keyframe < self.keyframe_changes.keyframe_count(),
            "keyframe index out of range"
        );
        let mut states = DexelGridStates {
            intervals: self
                .grids
                .map(|grid| vec![vec![grid.initial]; dexel_count(&grid)]),
            normals: std::array::from_fn(|axis| {
                vec![vec![initial_normals(axis)]; dexel_count(&self.grids[axis])]
            }),
        };
        for keyframe in 0..=keyframe {
            for change in self.keyframe_changes.keyframe(keyframe) {
                let (grid, index) = (change.grid as usize, change.index as usize);
                let intervals = &mut states.intervals[grid][index];
                intervals.clear();
                intervals.extend_from_slice(change.intervals);
                let normals = &mut states.normals[grid][index];
                normals.clear();
                normals.extend_from_slice(change.normals);
            }
        }
        states
    }

    /// スナップショット `snapshot` 以前の、最も新しいキーフレーム（なければ `None`。初期の状態から作り直す）
    pub fn keyframe_before(&self, snapshot: usize) -> Option<usize> {
        self.keyframe_snapshots
            .partition_point(|&keyframe| keyframe <= snapshot)
            .checked_sub(1)
    }
}

/// 各格子（X・Y・Z の順）のデクセルごとの区間と、区間の始まり・終わりの法線
#[derive(Debug, Clone, PartialEq)]
pub struct DexelGridStates {
    pub intervals: [Vec<Vec<(f64, f64)>>; 3],
    pub normals: [Vec<Vec<[DexelNormal; 2]>>; 3],
}

/// 格子のデクセルの数
pub(crate) fn dexel_count(grid: &DexelGridLayout) -> usize {
    grid.count[0] as usize * grid.count[1] as usize
}

/// 節を書き出す
fn write_section(payload: &mut Vec<u8>, kind: u16, content: &[u8]) {
    payload.extend_from_slice(&kind.to_le_bytes());
    payload.extend_from_slice(&(content.len() as u64).to_le_bytes());
    payload.extend_from_slice(content);
}

/// 成果物（共通のヘッダと中身）を書き出す
///
/// # Errors
///
/// 書き出しの失敗、数が `u32` に収まらない場合、時点・キーフレームの数が食い違う場合。
pub fn write_cutting_simulation_artifact_v1<W: Write>(
    writer: &mut W,
    artifact: &CuttingSimulationArtifact,
    unit: LengthUnit,
    frame: CoordinateFrame,
) -> Result<(), BinaryFormatError> {
    if artifact.snapshot_applied.len() != artifact.snapshots.len() {
        return Err(BinaryFormatError::InvalidPayload("snapshot_applied"));
    }
    if artifact.keyframe_snapshots.len() != artifact.keyframe_changes.keyframe_count() {
        return Err(BinaryFormatError::InvalidPayload("keyframe_changes"));
    }

    let mut snapshots = Vec::new();
    write_count(&mut snapshots, artifact.snapshots.len(), "snapshots")?;
    for (snapshot, &applied) in artifact.snapshots.iter().zip(&artifact.snapshot_applied) {
        write_count(&mut snapshots, snapshot.segment_index, "segment_index")?;
        for value in [
            snapshot.segment_t,
            snapshot.accumulated_distance_mm,
            snapshot.remaining_volume_mm3,
        ] {
            snapshots.extend_from_slice(&value.to_le_bytes());
        }
        write_count(&mut snapshots, applied, "snapshot_applied")?;
    }

    let mut grids = Vec::new();
    for grid in &artifact.grids {
        grids.push(grid.axis);
        for value in [grid.origin[0], grid.origin[1], grid.pitch[0], grid.pitch[1]] {
            grids.extend_from_slice(&value.to_le_bytes());
        }
        for count in grid.count {
            grids.extend_from_slice(&count.to_le_bytes());
        }
        grids.extend_from_slice(&grid.initial.0.to_le_bytes());
        grids.extend_from_slice(&grid.initial.1.to_le_bytes());
    }

    let mut keyframes = Vec::new();
    let mut index = Vec::new();
    write_count(&mut index, artifact.keyframe_snapshots.len(), "keyframes")?;
    for (keyframe, &snapshot) in artifact.keyframe_snapshots.iter().enumerate() {
        let start = keyframes.len();
        let states = artifact.keyframe_changes.keyframe(keyframe);
        write_count(&mut keyframes, states.len(), "changes")?;
        for change in states {
            keyframes.push(change.grid);
            keyframes.extend_from_slice(&change.index.to_le_bytes());
            write_count(&mut keyframes, change.intervals.len(), "intervals")?;
            for (&(low, high), normals) in change.intervals.iter().zip(change.normals) {
                keyframes.extend_from_slice(&low.to_le_bytes());
                keyframes.extend_from_slice(&high.to_le_bytes());
                for value in normals.iter().flat_map(|normal| normal.0) {
                    keyframes.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        write_count(&mut index, snapshot, "keyframe_snapshot")?;
        index.extend_from_slice(&(start as u64).to_le_bytes());
        index.extend_from_slice(&((keyframes.len() - start) as u64).to_le_bytes());
    }

    let mut payload = Vec::new();
    write_section(&mut payload, SECTION_SNAPSHOTS, &snapshots);
    write_section(&mut payload, SECTION_GRIDS, &grids);
    write_section(&mut payload, SECTION_KEYFRAME_INDEX, &index);
    write_section(&mut payload, SECTION_KEYFRAMES, &keyframes);

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
    if !SUPPORTED_MINORS.contains(&header.version_minor) {
        return Err(BinaryFormatError::UnsupportedMinorVersion {
            major: header.version_major,
            found_minor: header.version_minor,
            supported_minors: SUPPORTED_MINORS,
        });
    }

    let mut sections: [Option<&[u8]>; 4] = [None; 4];
    let mut cursor = bytes.as_slice();
    while !cursor.is_empty() {
        let kind = read_u16(&mut cursor)?;
        let length = usize::try_from(read_u64(&mut cursor)?)
            .ok()
            .filter(|&length| length <= cursor.len())
            .ok_or(BinaryFormatError::InvalidPayload("section.length"))?;
        let (content, rest) = cursor.split_at(length);
        cursor = rest;
        // 知らない種類の節は読み飛ばす
        if let Some(slot) = (SECTION_SNAPSHOTS..=SECTION_KEYFRAMES)
            .position(|known| known == kind)
            .map(|position| &mut sections[position])
            && slot.replace(content).is_some()
        {
            return Err(BinaryFormatError::InvalidPayload("section.duplicate"));
        }
    }
    let [Some(snapshots), Some(grids), Some(index), Some(keyframes)] = sections else {
        return Err(BinaryFormatError::InvalidPayload("section.missing"));
    };

    let (snapshots, snapshot_applied) = read_snapshots(snapshots)?;
    let grids = read_grids(grids)?;
    let (keyframe_snapshots, keyframe_changes) =
        read_keyframes(index, keyframes, &grids, snapshots.len())?;

    Ok((
        header,
        CuttingSimulationArtifact {
            snapshots,
            snapshot_applied,
            grids,
            keyframe_snapshots,
            keyframe_changes,
        },
    ))
}

fn read_snapshots(
    mut reader: &[u8],
) -> Result<(Vec<SimulationSnapshotExport>, Vec<usize>), BinaryFormatError> {
    let reader = &mut reader;
    let count = read_u32(reader)? as usize;
    let mut snapshots = Vec::with_capacity(count.min(reader.len()));
    let mut applied = Vec::with_capacity(count.min(reader.len()));
    for _ in 0..count {
        snapshots.push(SimulationSnapshotExport {
            segment_index: read_u32(reader)? as usize,
            segment_t: read_f64(reader)?,
            accumulated_distance_mm: read_f64(reader)?,
            remaining_volume_mm3: read_f64(reader)?,
        });
        applied.push(read_u32(reader)? as usize);
    }
    if !reader.is_empty() {
        return Err(BinaryFormatError::InvalidPayload("snapshots.trailing"));
    }
    if !applied.is_sorted() {
        return Err(BinaryFormatError::InvalidPayload("snapshot_applied"));
    }
    Ok((snapshots, applied))
}

fn read_grids(mut reader: &[u8]) -> Result<[DexelGridLayout; 3], BinaryFormatError> {
    let reader = &mut reader;
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
    if !reader.is_empty() {
        return Err(BinaryFormatError::InvalidPayload("grids.trailing"));
    }
    Ok(grids)
}

/// キーフレームの目次 `index` とキーフレームの節 `keyframes` を読む（目次の位置で各キーフレームを読む）
fn read_keyframes(
    mut index: &[u8],
    keyframes: &[u8],
    grids: &[DexelGridLayout; 3],
    snapshot_count: usize,
) -> Result<(Vec<usize>, DexelChanges), BinaryFormatError> {
    let index = &mut index;
    let count = read_u32(index)? as usize;
    let mut keyframe_snapshots = Vec::with_capacity(count.min(index.len()));
    let mut changes = DexelChanges::default();
    let mut intervals = Vec::new();
    let mut normals = Vec::new();
    for _ in 0..count {
        let snapshot = read_u32(index)? as usize;
        let start = usize::try_from(read_u64(index)?).ok();
        let length = usize::try_from(read_u64(index)?).ok();
        let content = start
            .zip(length)
            .and_then(|(start, length)| keyframes.get(start..start.checked_add(length)?))
            .ok_or(BinaryFormatError::InvalidPayload("keyframe.range"))?;
        if snapshot >= snapshot_count
            || keyframe_snapshots
                .last()
                .is_some_and(|&previous| previous >= snapshot)
        {
            return Err(BinaryFormatError::InvalidPayload("keyframe.snapshot"));
        }
        keyframe_snapshots.push(snapshot);

        let reader = &mut &content[..];
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
            normals.clear();
            for _ in 0..interval_count {
                intervals.push((read_f64(reader)?, read_f64(reader)?));
                let mut normal = || -> Result<DexelNormal, BinaryFormatError> {
                    Ok(DexelNormal([read_u16(reader)?, read_u16(reader)?]))
                };
                normals.push([normal()?, normal()?]);
            }
            changes.push(grid, index, &intervals, &normals);
        }
        if !reader.is_empty() {
            return Err(BinaryFormatError::InvalidPayload("keyframe.trailing"));
        }
        changes.end_keyframe();
    }
    if !index.is_empty() {
        return Err(BinaryFormatError::InvalidPayload("keyframe_index.trailing"));
    }
    if keyframe_snapshots.last() != snapshot_count.checked_sub(1).as_ref() {
        return Err(BinaryFormatError::InvalidPayload("keyframe.last"));
    }
    Ok((keyframe_snapshots, changes))
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

fn read_u16(reader: &mut &[u8]) -> Result<u16, BinaryFormatError> {
    let mut bytes = [0_u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32(reader: &mut &[u8]) -> Result<u32, BinaryFormatError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut &[u8]) -> Result<u64, BinaryFormatError> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
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
}
