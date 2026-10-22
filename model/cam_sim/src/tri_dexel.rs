//! 素材の派生表現のトリデクセル（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §4・§5）
//!
//! X・Y・Z の 3 方向の格子の各線（デクセル）上の、材料のある区間の列。正本（`StockModel`）の掃引体を
//! 時系列の順に差し引き、デクセルごとに変化の履歴（何番目の掃引体で変わったかと、変化後の区間）を記録する。
//! 任意の時点（適用した掃引体の数）の状態を、履歴から復元する。

mod ray_interval;
#[cfg(test)]
mod tests;

use geo_algorithms::{Aabb3D, Point3D};

use crate::simulation_artifact::{DexelChange, DexelGridLayout};
use crate::stock_model::StockModel;
use ray_interval::{AxisLine, swept_primitive_intervals};

/// 1 本のデクセル（線上の材料の区間と、その変化の履歴）
#[derive(Debug, Clone)]
struct Dexel {
    /// 初期の区間（素材の範囲）
    initial: (f64, f64),
    /// 変化の履歴（掃引体の番号の昇順。変化後の区間の列）
    history: Vec<(usize, Vec<(f64, f64)>)>,
}

impl Dexel {
    /// `applied` 個の掃引体を差し引いた時点の区間
    fn intervals_at(&self, applied: usize) -> &[(f64, f64)] {
        let changed = self
            .history
            .partition_point(|(primitive, _)| *primitive < applied);
        match changed {
            0 => std::slice::from_ref(&self.initial),
            count => &self.history[count - 1].1,
        }
    }
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
    dexels: Vec<Dexel>,
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
        let dexel = Dexel {
            initial: (min[axis], max[axis]),
            history: Vec::new(),
        };
        Self {
            axis,
            plane_axes,
            origin,
            pitch,
            count,
            dexels: vec![dexel; count[0] * count[1]],
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

    /// `index` 番目の掃引体を差し引き、変化したデクセルの履歴に記録する
    fn subtract(&mut self, index: usize, primitive: &crate::stock_model::SweptToolPrimitive) {
        let bounds = primitive.bounds();
        let (low, high) = (coordinates(bounds.min()), coordinates(bounds.max()));
        let [u, v] = self.plane_axes;
        for i in self.index_range(0, low[u], high[u]) {
            for j in self.index_range(1, low[v], high[v]) {
                let removed = swept_primitive_intervals(primitive, self.line(i, j));
                if removed.is_empty() {
                    continue;
                }
                let dexel = &mut self.dexels[i * self.count[1] + j];
                let current = dexel.intervals_at(index + 1).to_vec();
                let mut next = current.clone();
                for &interval in &removed {
                    next = subtract_interval(&next, interval);
                }
                if next != current {
                    dexel.history.push((index, next));
                }
            }
        }
    }

    /// `applied` 個の掃引体を差し引いた時点の、材料の区間の長さの和 × 格子のセルの面積
    fn volume_at(&self, applied: usize) -> f64 {
        let length: f64 = self
            .dexels
            .iter()
            .flat_map(|dexel| dexel.intervals_at(applied))
            .map(|(low, high)| high - low)
            .sum();
        length * self.pitch[0] * self.pitch[1]
    }
}

/// トリデクセル
#[derive(Debug, Clone)]
pub struct TriDexelModel {
    /// X・Y・Z の向きの格子
    grids: [DexelGrid; 3],
    primitive_count: usize,
}

impl TriDexelModel {
    /// 正本の素材から、格子の間隔が `spacing` 以下のトリデクセルを作り、掃引体を時系列の順に差し引く
    ///
    /// # Panics
    ///
    /// `spacing` が正の有限値でない場合。
    pub fn from_stock_model(model: &StockModel, spacing: f64) -> Self {
        assert!(
            spacing.is_finite() && spacing > 0.0,
            "dexel spacing must be finite and positive"
        );
        let stock = model.stock();
        let mut grids = [0, 1, 2].map(|axis| DexelGrid::new(&stock, axis, spacing));
        for (index, primitive) in model.primitives().iter().enumerate() {
            for grid in &mut grids {
                grid.subtract(index, primitive);
            }
        }
        Self {
            grids,
            primitive_count: model.primitives().len(),
        }
    }

    /// 掃引体の数（時点の上限）
    pub fn primitive_count(&self) -> usize {
        self.primitive_count
    }

    /// `applied` 個の掃引体を差し引いた時点の残りの体積（Z の向きのデクセルの区間から求める）
    pub fn remaining_volume_at(&self, applied: usize) -> f64 {
        self.grids[2].volume_at(applied)
    }

    /// 格子 `grid` のデクセル `index` の、`applied` 個の掃引体を差し引いた時点の区間
    #[cfg(test)]
    pub(crate) fn intervals_at(&self, grid: usize, index: usize, applied: usize) -> &[(f64, f64)] {
        self.grids[grid].dexels[index].intervals_at(applied)
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
            initial: grid.dexels[0].initial,
        })
    }

    /// スナップショットの時点（各時点までに差し引いた掃引体の数。昇順）ごとの、前の時点からのデクセルの変化
    ///
    /// 掃引体ごとの履歴を、最初に `applied[k]` 個に達する時点 `k` にまとめ、時点ごとに最後の区間を変化とする。
    /// 最後の時点より後の変化は含めない。
    pub(crate) fn snapshot_changes(&self, applied: &[usize]) -> Vec<Vec<DexelChange>> {
        let mut changes = vec![Vec::new(); applied.len()];
        for (grid_index, grid) in self.grids.iter().enumerate() {
            for (dexel_index, dexel) in grid.dexels.iter().enumerate() {
                let mut pending: Option<(usize, &Vec<(f64, f64)>)> = None;
                for (primitive, intervals) in &dexel.history {
                    let snapshot = applied.partition_point(|&count| count <= *primitive);
                    if snapshot >= applied.len() {
                        break;
                    }
                    if let Some((previous, previous_intervals)) = pending
                        && previous != snapshot
                    {
                        changes[previous].push(change(grid_index, dexel_index, previous_intervals));
                    }
                    pending = Some((snapshot, intervals));
                }
                if let Some((snapshot, intervals)) = pending {
                    changes[snapshot].push(change(grid_index, dexel_index, intervals));
                }
            }
        }
        changes
    }
}

fn change(grid: usize, index: usize, intervals: &[(f64, f64)]) -> DexelChange {
    DexelChange {
        grid: grid as u8,
        index: u32::try_from(index).expect("dexel count is bounded by the display resolution"),
        intervals: intervals.to_vec(),
    }
}

/// 昇順で互いに素な区間の列から、区間 `[low, high]` を差し引く
fn subtract_interval(intervals: &[(f64, f64)], (low, high): (f64, f64)) -> Vec<(f64, f64)> {
    let mut result = Vec::with_capacity(intervals.len() + 1);
    for &(start, end) in intervals {
        if end <= low || high <= start {
            result.push((start, end));
            continue;
        }
        if start < low {
            result.push((start, low));
        }
        if high < end {
            result.push((high, end));
        }
    }
    result
}

fn coordinates(point: Point3D<f64>) -> [f64; 3] {
    [point.x(), point.y(), point.z()]
}
