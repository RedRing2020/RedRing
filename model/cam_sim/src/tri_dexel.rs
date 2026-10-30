//! 素材の派生表現のトリデクセル（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §4・§5）
//!
//! X・Y・Z の 3 方向の格子の各線（デクセル）上の、材料のある区間の列と、区間の端の材料の外向きの法線。
//! 正本（`StockModel`）の掃引体を時系列の順に差し引き、今の区間だけを持つ。削った端の法線は、端を決めた
//! 掃引体の面の法線の逆向き（工具の外向きの逆が材料の外向き）とする。スナップショットの時点の境で、前の時点から変化したデクセルの
//! 区間を記録する（掃引体ごとの変化の履歴は持たない）。

mod ray_interval;
#[cfg(test)]
mod tests;

use geo_algorithms::{Aabb3D, Point3D};

use crate::dexel_normal::DexelNormal;
use crate::simulation_artifact::{DexelChanges, DexelGridLayout};
use crate::stock_model::StockModel;
use ray_interval::{AxisLine, ToolInterval, swept_primitive_intervals};

/// 1 本のデクセルの区間の、格子の区間の配列の中の位置
#[derive(Debug, Clone, Copy)]
struct Slot {
    /// 最初の区間の位置
    start: u32,
    /// 区間の数
    len: u32,
    /// 区間を書ける数（これを超えたら配列の末尾に置き直す）
    capacity: u32,
}

/// 1 方向のデクセルの格子
#[derive(Debug, Clone)]
struct DexelGrid {
    /// デクセルの向き（0: X、1: Y、2: Z）
    axis: usize,
    /// 格子の 2 方向（`axis` 以外の座標軸。昇順）
    plane_axes: [usize; 2],
    /// 格子の各方向の、最初の線の座標・間隔・本数
    origin: [f64; 2],
    pitch: [f64; 2],
    count: [usize; 2],
    /// 初期の区間（素材の範囲）
    initial: (f64, f64),
    /// デクセルごとの、`intervals` の中の今の区間の位置
    slots: Vec<Slot>,
    /// すべてのデクセルの今の区間（デクセルごとに連続して並ぶ。置き直した後の古い位置は使わない）
    intervals: Vec<(f64, f64)>,
    /// `intervals` と同じ位置の、区間の始まり・終わりの法線
    normals: Vec<[DexelNormal; 2]>,
    /// 前のスナップショットの時点から変化したか（デクセルごと）
    changed: Vec<bool>,
    /// 前のスナップショットの時点から変化したデクセルの番号
    changed_indices: Vec<usize>,
}

impl DexelGrid {
    fn new(stock: &Aabb3D<f64>, axis: usize, spacing: f64) -> Self {
        let min = coordinates(stock.min());
        let max = coordinates(stock.max());
        let plane_axes = match axis {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        };
        let mut origin = [0.0; 2];
        let mut pitch = [0.0; 2];
        let mut count = [0; 2];
        for (index, &plane_axis) in plane_axes.iter().enumerate() {
            let extent = max[plane_axis] - min[plane_axis];
            // 間隔が `spacing` 以下になる最小の本数で、各セルの中心に線を置く
            let lines = ((extent / spacing).ceil() as usize).max(1);
            pitch[index] = extent / lines as f64;
            origin[index] = min[plane_axis] + 0.5 * pitch[index];
            count[index] = lines;
        }
        let dexel_count = count[0] * count[1];
        let initial = (min[axis], max[axis]);
        Self {
            axis,
            plane_axes,
            origin,
            pitch,
            count,
            initial,
            slots: (0..dexel_count)
                .map(|index| Slot {
                    start: slot_position(index),
                    len: 1,
                    capacity: 1,
                })
                .collect(),
            intervals: vec![initial; dexel_count],
            normals: vec![initial_normals(axis); dexel_count],
            changed: vec![false; dexel_count],
            changed_indices: Vec::new(),
        }
    }

    /// 格子の位置 `(i, j)` の線
    fn line(&self, i: usize, j: usize) -> AxisLine {
        let mut point = [0.0; 3];
        point[self.plane_axes[0]] = self.origin[0] + i as f64 * self.pitch[0];
        point[self.plane_axes[1]] = self.origin[1] + j as f64 * self.pitch[1];
        AxisLine {
            axis: self.axis,
            point: Point3D::new(point[0], point[1], point[2]),
        }
    }

    /// デクセル `index` の今の区間
    fn current(&self, index: usize) -> &[(f64, f64)] {
        let Slot { start, len, .. } = self.slots[index];
        &self.intervals[start as usize..(start + len) as usize]
    }

    /// デクセル `index` の今の区間の、始まり・終わりの法線
    fn current_normals(&self, index: usize) -> &[[DexelNormal; 2]] {
        let Slot { start, len, .. } = self.slots[index];
        &self.normals[start as usize..(start + len) as usize]
    }

    /// デクセル `index` の今の区間と法線を `intervals`・`normals` にする（書ける数を超える場合は配列の末尾に置き直す）
    fn set(&mut self, index: usize, intervals: &[(f64, f64)], normals: &[[DexelNormal; 2]]) {
        let len = u32::try_from(intervals.len()).expect("dexel interval count fits in u32");
        let slot = &mut self.slots[index];
        if len > slot.capacity {
            // 置き直すたびに書ける数を倍にするため、使わない古い位置の合計は今の書ける数の合計を超えない
            slot.start = slot_position(self.intervals.len());
            slot.capacity = len.max(2 * slot.capacity);
            self.intervals
                .resize(self.intervals.len() + slot.capacity as usize, (0.0, 0.0));
            self.normals
                .resize(self.intervals.len(), [DexelNormal([0, 0]); 2]);
        }
        slot.len = len;
        let start = slot.start as usize;
        self.intervals[start..start + intervals.len()].copy_from_slice(intervals);
        self.normals[start..start + normals.len()].copy_from_slice(normals);
        if !self.changed[index] {
            self.changed[index] = true;
            self.changed_indices.push(index);
        }
    }

    /// 座標の範囲 `[low, high]` に入る線の番号の範囲（格子の方向 `index`）
    fn index_range(&self, index: usize, low: f64, high: f64) -> std::ops::Range<usize> {
        let first = ((low - self.origin[index]) / self.pitch[index])
            .ceil()
            .max(0.0) as usize;
        let last = ((high - self.origin[index]) / self.pitch[index]).floor();
        if last < 0.0 {
            return 0..0;
        }
        first..(last as usize + 1).min(self.count[index])
    }

    /// 掃引体を差し引き、除去した区間の長さの和を返す
    ///
    /// `scratch` は区間の計算に使う作業用の列（呼び出しの間で使い回し、確保を減らす）。
    fn subtract(
        &mut self,
        primitive: &crate::stock_model::SweptToolPrimitive,
        scratch: &mut Scratch,
    ) -> f64 {
        let bounds = primitive.bounds();
        let (low, high) = (coordinates(bounds.min()), coordinates(bounds.max()));
        let [u, v] = self.plane_axes;
        let (along_low, along_high) = (low[self.axis], high[self.axis]);
        let mut removed_length = 0.0;
        for i in self.index_range(0, low[u], high[u]) {
            for j in self.index_range(1, low[v], high[v]) {
                let index = i * self.count[1] + j;
                let current = self.current(index);
                // 除去する区間は掃引体の外接する箱の中にあるため、箱の中に材料がない線は変わらない
                if !current
                    .iter()
                    .any(|&(low, high)| along_low < high && low < along_high)
                {
                    continue;
                }
                let line = self.line(i, j);
                let removed = swept_primitive_intervals(primitive, line);
                // 除去する区間が材料の区間と重ならない線は変わらない
                let overlaps = removed.iter().flatten().any(|removed| {
                    let (start, end) = removed.range;
                    current.iter().any(|&(low, high)| start < high && low < end)
                });
                if !overlaps {
                    continue;
                }
                let Scratch {
                    intervals: [next, other],
                    normals: [next_normals, other_normals],
                } = scratch;
                next.clear();
                next.extend_from_slice(current);
                next_normals.clear();
                next_normals.extend_from_slice(self.current_normals(index));
                for removed in removed.iter().flatten() {
                    subtract_interval(next, next_normals, removed, line, other, other_normals);
                    std::mem::swap(next, other);
                    std::mem::swap(next_normals, other_normals);
                }
                if next.as_slice() != current {
                    removed_length += total_length(current) - total_length(next);
                    self.set(index, next, next_normals);
                }
            }
        }
        removed_length
    }

    /// 前のスナップショットの時点から変化したデクセルの今の区間（番号の昇順）を `changes` に加え、変化の記録を空にする
    fn take_changes(&mut self, grid: u8, changes: &mut DexelChanges) {
        let mut indices = std::mem::take(&mut self.changed_indices);
        indices.sort_unstable();
        for index in indices {
            self.changed[index] = false;
            changes.push(
                grid,
                u32::try_from(index).expect("dexel count is bounded by the display resolution"),
                self.current(index),
                self.current_normals(index),
            );
        }
    }

    /// 素材の範囲の、材料の区間の長さの和
    fn initial_length(&self) -> f64 {
        let (low, high) = self.initial;
        (high - low) * self.slots.len() as f64
    }
}

/// トリデクセル
#[derive(Debug, Clone)]
pub struct TriDexelModel {
    /// X・Y・Z の向きの格子
    grids: [DexelGrid; 3],
    /// `remaining_volumes[applied]` は、`applied` 個の掃引体を差し引いた時点の残りの体積（Z の向きのデクセルから）
    remaining_volumes: Vec<f64>,
    /// 時点 `k` の変化は、スナップショット `k - 1`（`k = 0` では初期の状態）からスナップショット `k` への変化
    snapshot_changes: DexelChanges,
}

impl TriDexelModel {
    /// 正本の素材から、格子の間隔が `spacing` 以下のトリデクセルを作り、掃引体を時系列の順に差し引く
    ///
    /// `snapshot_applied`（昇順）は、各スナップショットの時点までに差し引く掃引体の数。時点の境ごとに、
    /// 前の時点から変化したデクセルの区間を記録する。最後の時点より後の変化は記録しない。
    ///
    /// # Panics
    ///
    /// `spacing` が正の有限値でない場合。`snapshot_applied` が昇順でないか、掃引体の数を超える場合。
    pub fn from_stock_model(model: &StockModel, spacing: f64, snapshot_applied: &[usize]) -> Self {
        assert!(
            spacing.is_finite() && spacing > 0.0,
            "dexel spacing must be finite and positive"
        );
        let primitives = model.primitives();
        assert!(
            snapshot_applied.is_sorted() && snapshot_applied.last() <= Some(&primitives.len()),
            "snapshot times must be sorted and within the primitives"
        );
        let stock = model.stock();
        let mut grids = [0, 1, 2].map(|axis| DexelGrid::new(&stock, axis, spacing));
        // 残りの体積は、Z の向きのデクセルの区間の長さの和 × 格子のセルの面積。除去した長さを時系列に足し込む
        let cell_area = grids[2].pitch[0] * grids[2].pitch[1];
        let mut length = grids[2].initial_length();
        let mut remaining_volumes = Vec::with_capacity(primitives.len() + 1);
        remaining_volumes.push(length * cell_area);
        let mut snapshot_changes = DexelChanges::default();
        let mut scratch = Scratch::default();
        record_snapshot_changes(&mut grids, snapshot_applied, 0, &mut snapshot_changes);
        for (index, primitive) in primitives.iter().enumerate() {
            if snapshot_changes.snapshot_count() == snapshot_applied.len() {
                break;
            }
            let [x_grid, y_grid, z_grid] = &mut grids;
            x_grid.subtract(primitive, &mut scratch);
            y_grid.subtract(primitive, &mut scratch);
            length -= z_grid.subtract(primitive, &mut scratch);
            remaining_volumes.push(length * cell_area);
            record_snapshot_changes(
                &mut grids,
                snapshot_applied,
                index + 1,
                &mut snapshot_changes,
            );
        }
        Self {
            grids,
            remaining_volumes,
            snapshot_changes,
        }
    }

    /// `applied` 個の掃引体を差し引いた時点の残りの体積（Z の向きのデクセルの区間から求める）
    ///
    /// # Panics
    ///
    /// `applied` が、最後のスナップショットの時点までに差し引いた掃引体の数より大きい場合。
    pub fn remaining_volume_at(&self, applied: usize) -> f64 {
        self.remaining_volumes[applied]
    }

    /// 格子 `grid` のデクセル `index` の、最後のスナップショットの時点の区間
    #[cfg(test)]
    pub(crate) fn current_intervals(&self, grid: usize, index: usize) -> &[(f64, f64)] {
        self.grids[grid].current(index)
    }

    /// 格子 `grid` のデクセル `index` の、最後のスナップショットの時点の区間の始まり・終わりの法線
    #[cfg(test)]
    pub(crate) fn current_normals(&self, grid: usize, index: usize) -> &[[DexelNormal; 2]] {
        self.grids[grid].current_normals(index)
    }

    /// 格子の配置（X・Y・Z の順）
    pub(crate) fn layouts(&self) -> [DexelGridLayout; 3] {
        self.grids.each_ref().map(|grid| DexelGridLayout {
            axis: grid.axis as u8,
            origin: grid.origin,
            pitch: grid.pitch,
            count: grid.count.map(|count| {
                u32::try_from(count).expect("dexel line count is bounded by the display resolution")
            }),
            initial: grid.initial,
        })
    }

    /// スナップショットの時点ごとの、前の時点からのデクセルの変化（`snapshot_applied` と同じ並び）
    pub(crate) fn snapshot_changes(&self) -> &DexelChanges {
        &self.snapshot_changes
    }
}

/// `applied` 個の掃引体を差し引いた時点にあるスナップショットに、前の時点から変化したデクセルを記録する
fn record_snapshot_changes(
    grids: &mut [DexelGrid; 3],
    snapshot_applied: &[usize],
    applied: usize,
    snapshot_changes: &mut DexelChanges,
) {
    while snapshot_applied.get(snapshot_changes.snapshot_count()) == Some(&applied) {
        for (index, grid) in grids.iter_mut().enumerate() {
            grid.take_changes(index as u8, snapshot_changes);
        }
        snapshot_changes.end_snapshot();
    }
}

/// 区間の差し引きに使う作業用の列（区間と法線を 2 組。差し引くたびに入れ替える）
#[derive(Debug, Default)]
struct Scratch {
    intervals: [Vec<(f64, f64)>; 2],
    normals: [Vec<[DexelNormal; 2]>; 2],
}

/// 向き `axis` のデクセルの、素材の箱の区間の始まり・終わりの法線（座標軸の負・正の向き）
pub(crate) fn initial_normals(axis: usize) -> [DexelNormal; 2] {
    let unit = |sign: f64| {
        let mut unit = [0.0; 3];
        unit[axis] = sign;
        DexelNormal::from_unit(unit)
    };
    [unit(-1.0), unit(1.0)]
}

/// 区間の配列の中の位置
fn slot_position(position: usize) -> u32 {
    u32::try_from(position).expect("dexel interval storage fits in u32")
}

/// 昇順で互いに素な区間の列 `intervals`（法線 `normals`）から、線 `line` の上の工具の区間 `removed` を差し引き、
/// `result`・`result_normals` に書く（削った端の法線は、工具の外向きの法線の逆向き）
fn subtract_interval(
    intervals: &[(f64, f64)],
    normals: &[[DexelNormal; 2]],
    removed: &ToolInterval,
    line: AxisLine,
    result: &mut Vec<(f64, f64)>,
    result_normals: &mut Vec<[DexelNormal; 2]>,
) {
    result.clear();
    result_normals.clear();
    let (low, high) = removed.range;
    let cut =
        |end: usize| DexelNormal::from_unit(removed.normal(end, line).map(|component| -component));
    for (&(start, end), &[start_normal, end_normal]) in intervals.iter().zip(normals) {
        if end <= low || high <= start {
            result.push((start, end));
            result_normals.push([start_normal, end_normal]);
            continue;
        }
        if start < low {
            result.push((start, low));
            result_normals.push([start_normal, cut(0)]);
        }
        if high < end {
            result.push((high, end));
            result_normals.push([cut(1), end_normal]);
        }
    }
}

/// 区間の長さの和
fn total_length(intervals: &[(f64, f64)]) -> f64 {
    intervals.iter().map(|(low, high)| high - low).sum()
}

fn coordinates(point: Point3D<f64>) -> [f64; 3] {
    [point.x(), point.y(), point.z()]
}
