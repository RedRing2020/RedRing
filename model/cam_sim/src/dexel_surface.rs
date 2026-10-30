//! トリデクセルの表示用の面（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §4）
//!
//! 3 方向の格子線は同じ格子点を通るため、格子点の間の辺はいずれかの方向のデクセル上にある。格子点の内外を
//! Z の向きのデクセルで判定し、内外が変わる辺の交点をその辺上のデクセルの端点（正本から求めた厳密な点）とする
//! マーチングキューブで面を作る。セルの各面では「内側の頂点どうしは面の対角線をまたいでつながない」と決めて
//! 交点を線分でつなぎ、線分の輪から三角形を作る。隣り合うセルは共有する面で同じ線分を作るため、面に穴は開かない。
//!
//! 格子を素材の外側へ 1 層広げ、そこを空として扱うことで、素材の外周の面も作る。面はセルのブロックごとに作り、
//! スナップショットの変化から作り直すブロックを求める。
//!
//! 座標軸に垂直な同じ平面の上の面だけを持つセル（削っていない素材の外周や、平らな底など）は、ブロックの中で
//! 輪郭の多角形にまとめて三角形に分ける。多角形の頂点は、周りのセルが同じ平面にない輪郭の点とブロックの角だけと
//! し、どちらも点の周りだけで決まるため、隣のセル・ブロックと頂点がそろい、面に穴は開かない。

mod polygon;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::constants::SURFACE_BLOCK_CELLS;
use crate::simulation_artifact::{CuttingSimulationArtifact, DexelGridLayout, DexelState};

/// 表示用の面（頂点の位置・法線と三角形の頂点番号。三角形は外向きが表）
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SurfaceMesh {
    pub positions: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub indices: Vec<u32>,
}

/// セルの頂点（`dx + 2·dy + 4·dz`）
const CORNERS: [[usize; 3]; 8] = [
    [0, 0, 0],
    [1, 0, 0],
    [0, 1, 0],
    [1, 1, 0],
    [0, 0, 1],
    [1, 0, 1],
    [0, 1, 1],
    [1, 1, 1],
];

/// セルの 12 本の辺（両端の頂点の組。組の昇順）
const EDGES: [(usize, usize); 12] = [
    (0, 1),
    (0, 2),
    (0, 4),
    (1, 3),
    (1, 5),
    (2, 3),
    (2, 6),
    (3, 7),
    (4, 5),
    (4, 6),
    (5, 7),
    (6, 7),
];

/// 2 つの頂点を結ぶ辺の番号（`EDGES` の番号）
fn edge_index(a: usize, b: usize) -> usize {
    let pair = (a.min(b), a.max(b));
    EDGES
        .iter()
        .position(|&edge| edge == pair)
        .expect("adjacent corners of a cell form an edge")
}

/// セルの 6 つの面の頂点（面を一周する順）
const FACES: [[usize; 4]; 6] = [
    [0, 2, 6, 4],
    [1, 3, 7, 5],
    [0, 1, 5, 4],
    [2, 3, 7, 6],
    [0, 1, 3, 2],
    [4, 5, 7, 6],
];

/// スナップショットの時点の、トリデクセルの表示用の面
#[derive(Debug, Clone)]
pub struct DexelSurface {
    grids: [DexelGridLayout; 3],
    intervals: [Vec<Vec<(f64, f64)>>; 3],
    /// 各座標軸の、最初の格子点の座標・間隔・格子点の数
    origin: [f64; 3],
    pitch: [f64; 3],
    count: [usize; 3],
    /// 平らな部分をまとめるときに、同じ平面とみなす交点の座標の幅（素材の精度）
    flat_tolerance: f64,
}

impl DexelSurface {
    /// 成果物のスナップショット `snapshot` の時点の面（`flat_tolerance` は同じ平面とみなす交点の座標の幅）
    ///
    /// # Panics
    ///
    /// `snapshot` がスナップショットの数以上の場合。
    pub fn at_snapshot(
        artifact: &CuttingSimulationArtifact,
        snapshot: usize,
        flat_tolerance: f64,
    ) -> Self {
        let grids = artifact.grids;
        // Z の格子は X・Y、X の格子は Y・Z の座標を持つ
        let [z_grid, x_grid] = [grids[2], grids[0]];
        Self {
            grids,
            intervals: artifact.dexel_intervals_at(snapshot),
            origin: [z_grid.origin[0], z_grid.origin[1], x_grid.origin[1]],
            pitch: [z_grid.pitch[0], z_grid.pitch[1], x_grid.pitch[1]],
            count: [
                z_grid.count[0] as usize,
                z_grid.count[1] as usize,
                x_grid.count[1] as usize,
            ],
            flat_tolerance,
        }
    }

    /// 各座標軸のブロックの数
    pub fn block_counts(&self) -> [usize; 3] {
        // セルは、格子を外側へ 1 層広げた格子点の間（各軸 count + 1 個）
        self.count
            .map(|count| (count + 1).div_ceil(SURFACE_BLOCK_CELLS))
    }

    /// デクセルの区間を変え、作り直すブロックを返す（前の時点へ戻す区間も同じように当てられる）
    ///
    /// 変化後の区間（`changes`。複数の時点の変化を続けて並べてよい）を順に当て、作り直すブロックをまとめて返す。作り直すのは、区間が変わった範囲
    /// （変化前と変化後で一致しない端点の間）の格子点・辺に接するセルと、その周りの 1 層のセルを含むブロック
    /// （平らな部分の頂点は、頂点の周りのセルが同じ平面かで決まるため）。
    pub fn apply_changes<'a>(
        &mut self,
        changes: impl IntoIterator<Item = DexelState<'a>>,
    ) -> Vec<[usize; 3]> {
        let blocks = self.block_counts();
        let mut dirty = BTreeSet::new();
        for change in changes {
            let grid = change.grid as usize;
            let dexel = &mut self.intervals[grid][change.index as usize];
            let Some((low, high)) = changed_range(dexel, change.intervals) else {
                continue;
            };
            dexel.clear();
            dexel.extend_from_slice(change.intervals);
            let plane_axes = plane_axes(grid);
            let count = self.grids[grid].count[1] as usize;
            let line = [change.index as usize / count, change.index as usize % count];
            let mut ranges = [0..0, 0..0, 0..0];
            // 線の向き: 変わった範囲の端点を挟む格子点（前後に 1 つの余裕）に接するセルと、その前後の 1 層
            // （外側へ広げたセルの番号は、格子点の番号 + 1 と + 0）
            let point =
                |value: f64| ((value - self.origin[grid]) / self.pitch[grid]).floor() as i64;
            let last_cell = self.count[grid] as i64;
            let (first, last) = (
                (point(low) - 2).clamp(0, last_cell) as usize,
                (point(high) + 4).clamp(0, last_cell) as usize,
            );
            ranges[grid] =
                first / SURFACE_BLOCK_CELLS..(last / SURFACE_BLOCK_CELLS + 1).min(blocks[grid]);
            // 線の周り: 線上の格子点・辺に接するセルと、その周りの 1 層
            for (index, &axis) in plane_axes.iter().enumerate() {
                let first = line[index].saturating_sub(1) / SURFACE_BLOCK_CELLS;
                let last = (line[index] + 2) / SURFACE_BLOCK_CELLS;
                ranges[axis] = first..(last + 1).min(blocks[axis]);
            }
            for bx in ranges[0].clone() {
                for by in ranges[1].clone() {
                    for bz in ranges[2].clone() {
                        dirty.insert([bx, by, bz]);
                    }
                }
            }
        }
        dirty.into_iter().collect()
    }

    /// ブロックの列 `blocks` の面（複数のコアで並べて作る。並びは `blocks` と同じ）
    pub fn mesh_blocks(&self, blocks: &[[usize; 3]]) -> Vec<SurfaceMesh> {
        let threads = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        if threads <= 1 || blocks.len() <= 1 {
            return blocks.iter().map(|&block| self.mesh_block(block)).collect();
        }
        let chunk = blocks.len().div_ceil(threads);
        std::thread::scope(|scope| {
            let handles: Vec<_> = blocks
                .chunks(chunk)
                .map(|part| {
                    scope.spawn(move || {
                        part.iter()
                            .map(|&block| self.mesh_block(block))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| handle.join().expect("meshing a block does not panic"))
                .collect()
        })
    }

    /// ブロック `block` の面
    ///
    /// ブロックを通るデクセルが、ブロックの範囲ですべて素材の範囲（初期の区間）のままなら、セルごとの判定を省き、
    /// 外側へ広げた層のセルを素材の面に乗るセルとして面を作る（セルごとに処理した場合と同じ面になる）。
    pub fn mesh_block(&self, block: [usize; 3]) -> SurfaceMesh {
        if self.is_uncut_block(block) {
            let [x_cells, y_cells, z_cells] = [0, 1, 2].map(|axis| self.block_cells(block, axis));
            let mut cells = Vec::new();
            for x in x_cells {
                for y in y_cells.clone() {
                    for z in z_cells.clone() {
                        let lower = [x as i64 - 1, y as i64 - 1, z as i64 - 1];
                        let outer = (0..3).any(|axis| {
                            lower[axis] == -1 || lower[axis] == self.count[axis] as i64 - 1
                        });
                        if outer {
                            cells.push(lower);
                        }
                    }
                }
            }
            return self.mesh_cells(block, &cells, true);
        }
        self.mesh_block_by_cells(block)
    }

    /// ブロック `block` の面を、セルごとに判定して作る
    ///
    /// セルの頂点の内外は Z の向きのデクセルだけで決まるため、Z の向きのセルの列ごとに、列の 4 本の Z の線の
    /// 内外の区間から頂点の内外がそろわないセルを求め、そのセルだけを処理する（時間は面の大きさに比例する）。
    fn mesh_block_by_cells(&self, block: [usize; 3]) -> SurfaceMesh {
        let [x_cells, y_cells, z_cells] = [0, 1, 2].map(|axis| self.block_cells(block, axis));
        let mut cells = Vec::new();
        for x in x_cells {
            for y in y_cells.clone() {
                // 外側へ広げたセルの番号から、セルの最小の格子点の番号（-1 から）
                let (i, j) = (x as i64 - 1, y as i64 - 1);
                cells.extend(
                    self.mixed_cells_in_column(i, j, z_cells.clone())
                        .into_iter()
                        .map(|k| [i, j, k]),
                );
            }
        }
        self.mesh_cells(block, &cells, false)
    }

    /// ブロック `block` を通るデクセルが、ブロックの範囲ですべて素材の範囲（初期の区間）とちょうど一致するか
    fn is_uncut_block(&self, block: [usize; 3]) -> bool {
        let cells = [0, 1, 2].map(|axis| self.block_cells(block, axis));
        (0..3).all(|axis| {
            // 外側へ広げたセルの番号から、セルの最小の格子点の番号（-1 から）
            let first = cells[axis].start as i64 - 1;
            let last = cells[axis].end as i64 - 1;
            let span = (self.coordinate(axis, first), self.coordinate(axis, last));
            let (low, high) = self.grids[axis].initial;
            let expected = (low.max(span.0), high.min(span.1));
            let plane = plane_axes(axis);
            let lines = |i: usize| {
                let axis = plane[i];
                (cells[axis].start.saturating_sub(1))..cells[axis].end.min(self.count[axis])
            };
            lines(0).all(|u| {
                lines(1).all(|v| {
                    let mut found = false;
                    for &(start, end) in &self.intervals[axis][u * self.count[plane[1]] + v] {
                        let clipped = (start.max(span.0), end.min(span.1));
                        if clipped.0 < clipped.1 {
                            if found || clipped != expected {
                                return false;
                            }
                            found = true;
                        }
                    }
                    found == (expected.0 < expected.1)
                })
            })
        })
    }

    /// ブロック `block` の、最小の格子点が `cells` のセルの面（同じ平面の上の面だけを持つセルは多角形にまとめる）
    ///
    /// `uncut` の場合、`cells` は削っていないブロックの外側へ広げた層のセルで、セルが乗る平面は素材の面とする。
    fn mesh_cells(&self, block: [usize; 3], cells: &[[i64; 3]], uncut: bool) -> SurfaceMesh {
        if cells.is_empty() {
            return SurfaceMesh::default();
        }
        let mut builder =
            MeshBuilder::new(block.map(|index| (index * SURFACE_BLOCK_CELLS) as i64 - 1));
        let mut planes: BTreeMap<PlaneKey, BTreeSet<[i64; 2]>> = BTreeMap::new();
        let mut cell_planes = HashMap::new();
        let mut sharp_cells = Vec::new();
        for &lower in cells {
            let (sharp, keys) = if uncut {
                let sides = self.outer_sides(lower);
                let keys: Vec<PlaneKey> = sides
                    .iter()
                    .map(|&(axis, inside_below)| {
                        self.stock_face_key(axis, inside_below, lower[axis])
                    })
                    .collect();
                (sides.len() >= 2, keys)
            } else {
                let inside = self.corner_inside(lower);
                let sharp = self.is_sharp_cell(lower);
                let keys = self.planes_of_cell(lower, &inside, sharp);
                if !sharp && keys.is_empty() {
                    self.mesh_cell(lower, &inside, &mut builder);
                }
                (sharp, keys)
            };
            if sharp {
                sharp_cells.push(lower);
            }
            for &key in &keys {
                let [u, v] = plane_axes(key.axis);
                planes.entry(key).or_default().insert([lower[u], lower[v]]);
            }
            cell_planes.insert(lower, keys);
        }
        for (key, cells) in planes {
            self.mesh_plane(key, &cells, &mut cell_planes, &mut builder);
        }
        for lower in sharp_cells {
            self.mesh_sharp_cell_seams(lower, &mut builder);
        }
        builder.finish()
    }

    /// 最小の格子点が `lower` のセルが平面 `key` に乗るか（`cell_planes` に覚える）
    ///
    /// 外側へ広げた格子のさらに外のセルは、どの平面にも乗るとみなす（素材の辺の上の点の頂点の判定で、
    /// 格子の外を数えないため）。
    fn cell_has_plane(
        &self,
        lower: [i64; 3],
        key: PlaneKey,
        cell_planes: &mut HashMap<[i64; 3], Vec<PlaneKey>>,
    ) -> bool {
        if (0..3).any(|axis| lower[axis] < -1 || lower[axis] >= self.count[axis] as i64) {
            return true;
        }
        cell_planes
            .entry(lower)
            .or_insert_with(|| {
                self.planes_of_cell(lower, &self.corner_inside(lower), self.is_sharp_cell(lower))
            })
            .contains(&key)
    }

    /// 最小の格子点が `lower`、頂点の内外が `inside` のセルが乗る平面（鋭い角のセル `sharp` は接する素材の面すべて）
    fn planes_of_cell(&self, lower: [i64; 3], inside: &[bool; 8], sharp: bool) -> Vec<PlaneKey> {
        if inside.iter().all(|&flag| flag == inside[0]) {
            return Vec::new();
        }
        if sharp {
            return self
                .outer_sides(lower)
                .into_iter()
                .map(|(axis, inside_below)| self.stock_face_key(axis, inside_below, lower[axis]))
                .collect();
        }
        self.planar_cell(lower, inside).into_iter().collect()
    }

    /// セルが外側へ広げた層にある座標軸と、その向きで材料が小さい座標の側にあるか（外周の大きい側の層）
    fn outer_sides(&self, lower: [i64; 3]) -> Vec<(usize, bool)> {
        (0..3)
            .filter_map(|axis| {
                if lower[axis] == -1 {
                    Some((axis, false))
                } else if lower[axis] == self.count[axis] as i64 - 1 {
                    Some((axis, true))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 素材の辺か角を含み（外側へ広げた層に 2 つ以上の座標軸で入る）、削っていないセルか
    fn is_sharp_cell(&self, lower: [i64; 3]) -> bool {
        let outer_axes = (0..3)
            .filter(|&axis| lower[axis] == -1 || lower[axis] == self.count[axis] as i64 - 1)
            .count();
        outer_axes >= 2 && self.is_uncut_cell(lower)
    }

    /// セルの辺の上のデクセルが、素材の範囲（初期の区間）のまま、または違いが `flat_tolerance` 以内か
    fn is_uncut_cell(&self, lower: [i64; 3]) -> bool {
        (0..3).all(|axis| {
            let plane = plane_axes(axis);
            let span = (
                self.coordinate(axis, lower[axis]),
                self.coordinate(axis, lower[axis] + 1),
            );
            let initial = self.grids[axis].initial;
            [[0, 0], [1, 0], [0, 1], [1, 1]]
                .into_iter()
                .all(|[du, dv]| {
                    let line = [lower[plane[0]] + du, lower[plane[1]] + dv];
                    let on_grid =
                        (0..2).all(|i| line[i] >= 0 && line[i] < self.count[plane[i]] as i64);
                    if !on_grid {
                        return true;
                    }
                    let index = line[0] as usize * self.count[plane[1]] + line[1] as usize;
                    let actual = clip_intervals(&self.intervals[axis][index], span);
                    let expected = clip_intervals(&[initial], span);
                    actual.len() == expected.len()
                        && actual.iter().zip(&expected).all(|(a, e)| {
                            (a.0 - e.0).abs() <= self.flat_tolerance
                                && (a.1 - e.1).abs() <= self.flat_tolerance
                        })
                })
        })
    }

    /// 素材の面（座標軸 `axis` に垂直で、材料が `inside_below` の側）の、セルの層 `layer` の平面
    fn stock_face_key(&self, axis: usize, inside_below: bool, layer: i64) -> PlaneKey {
        let (low, high) = self.grids[axis].initial;
        PlaneKey {
            axis,
            inside_below,
            layer,
            value: self.section(if inside_below { high } else { low }),
        }
    }

    /// 座標を `flat_tolerance` の幅で区切った区画の番号
    fn section(&self, value: f64) -> i64 {
        (value / self.flat_tolerance).round() as i64
    }

    /// 外側へ広げた格子点を、素材の箱の上の点に寄せた位置（素材の辺・角の点）
    fn stock_point(&self, point: [i64; 3]) -> [f64; 3] {
        std::array::from_fn(|axis| {
            let (low, high) = self.grids[axis].initial;
            self.coordinate(axis, point[axis]).clamp(low, high)
        })
    }

    /// 格子点が外側へ広げた層にある座標軸の数
    fn outer_count(&self, point: [i64; 3]) -> usize {
        (0..3)
            .filter(|&axis| point[axis] < 0 || point[axis] >= self.count[axis] as i64)
            .count()
    }

    /// 鋭い角のセル `lower` と、面を共有する削ったセルの間の隙間を埋める
    ///
    /// 共有する面に素材の辺が 2 本とも通る（面と素材の交わりが L 字の輪郭を持つ）場合、鋭い角のセルの輪郭は
    /// 「辺の上の交点 A → 素材の辺の点 E → 辺の上の交点 B」、削ったセルの面は A と B を結ぶ線分になるため、
    /// 三角形 A・E・B を、削ったセルの側を向けて加える。
    fn mesh_sharp_cell_seams(&self, lower: [i64; 3], builder: &mut MeshBuilder) {
        let sides = self.outer_sides(lower);
        for face_axis in 0..3 {
            let others: Vec<(usize, bool)> = sides
                .iter()
                .copied()
                .filter(|&(axis, _)| axis != face_axis)
                .collect();
            let [(p, p_below), (q, q_below)] = others[..] else {
                continue;
            };
            for step in [-1, 1] {
                let mut neighbor = lower;
                neighbor[face_axis] += step;
                let beyond =
                    neighbor[face_axis] < -1 || neighbor[face_axis] >= self.count[face_axis] as i64;
                if beyond || self.is_sharp_cell(neighbor) {
                    continue;
                }
                // 共有する面の上の格子点（面の座標軸の番号）と、素材の側・外側の番号
                let mut face = lower;
                face[face_axis] = if step > 0 {
                    lower[face_axis] + 1
                } else {
                    lower[face_axis]
                };
                let inner = |axis: usize, below: bool| lower[axis] + i64::from(!below);
                let outer = |axis: usize, below: bool| lower[axis] + i64::from(below);
                let point = |p_index: i64, q_index: i64| {
                    let mut point = face;
                    point[p] = p_index;
                    point[q] = q_index;
                    point
                };
                let corner = point(inner(p, p_below), inner(q, q_below));
                let a_end = point(inner(p, p_below), outer(q, q_below));
                let b_end = point(outer(p, p_below), inner(q, q_below));
                let e = point(outer(p, p_below), outer(q, q_below));
                let a =
                    builder.edge_vertex(&EdgeKey::new(corner, a_end), self.crossing(corner, a_end));
                let b =
                    builder.edge_vertex(&EdgeKey::new(corner, b_end), self.crossing(corner, b_end));
                let e = builder.stock_vertex(e, face_axis, self.stock_point(e));
                let toward_neighbor = step as f64;
                let mut triangle = [a, e, b];
                let [pa, pe, pb] = triangle.map(|index| builder.mesh.positions[index as usize]);
                if cross(sub(pe, pa), sub(pb, pa))[face_axis] * toward_neighbor < 0.0 {
                    triangle.swap(1, 2);
                }
                builder.add_triangle(triangle);
            }
        }
    }

    /// 最小の格子点が `lower` のセルが、座標軸に垂直な 1 つの平面の上の面だけを持つ場合のその平面
    ///
    /// ある座標軸の向きの 4 本の辺だけが面と交わり、4 つの交点のその軸の座標が、座標を `flat_tolerance` の幅で
    /// 区切った同じ区画に入る場合とする（同じ区画の平面の中の座標の違いは `flat_tolerance` 以下になる）。
    fn planar_cell(&self, lower: [i64; 3], inside: &[bool; 8]) -> Option<PlaneKey> {
        let section = |value: f64| self.section(value);
        let corner_point = |corner: usize| {
            let offset = CORNERS[corner];
            [0, 1, 2].map(|axis| lower[axis] + offset[axis] as i64)
        };
        (0..3).find_map(|axis| {
            let bit = 1 << axis;
            let low_corners = (0..8).filter(move |corner| corner & bit == 0);
            let separated = low_corners
                .clone()
                .all(|corner| inside[corner] == inside[0] && inside[corner | bit] != inside[0]);
            if !separated {
                return None;
            }
            let value = section(self.crossing(corner_point(0), corner_point(bit))[axis]);
            low_corners
                .skip(1)
                .all(|corner| {
                    section(self.crossing(corner_point(corner), corner_point(corner | bit))[axis])
                        == value
                })
                .then_some(PlaneKey {
                    axis,
                    inside_below: inside[0],
                    layer: lower[axis],
                    value,
                })
        })
    }

    /// 同じ平面の上の面だけを持つセル（平面の 2 方向の最小の格子点の番号）を多角形にまとめて面を加える
    ///
    /// 多角形の頂点は、輪郭の格子点のうち、周りの 4 つのセルのどれかが同じ平面にない点と、ブロックの角とする。
    /// どちらも点の周りだけで決まるため、隣のブロックの多角形・セルの面と頂点がそろう。三角形に分けられない場合は
    /// セルごとに描く。
    fn mesh_plane(
        &self,
        key: PlaneKey,
        cells: &BTreeSet<[i64; 2]>,
        cell_planes: &mut HashMap<[i64; 3], Vec<PlaneKey>>,
        builder: &mut MeshBuilder,
    ) {
        let [u_axis, v_axis] = plane_axes(key.axis);
        let grid_point = |[u, v]: [i64; 2]| {
            let mut point = [0; 3];
            point[key.axis] = key.layer;
            point[u_axis] = u;
            point[v_axis] = v;
            point
        };
        // ブロックの境の線と、外側へ広げた格子点の線（素材の辺）は、どのブロックからも同じに見える線
        let fixed_line = |index: i64, axis: usize| {
            (index + 1).rem_euclid(SURFACE_BLOCK_CELLS as i64) == 0
                || index == -1
                || index == self.count[axis] as i64
        };
        let mut kept = |[u, v]: [i64; 2]| {
            (fixed_line(u, u_axis) && fixed_line(v, v_axis))
                || [[u - 1, v - 1], [u, v - 1], [u - 1, v], [u, v]]
                    .into_iter()
                    .any(|cell| !self.cell_has_plane(grid_point(cell), key, cell_planes))
        };

        // 輪郭の輪（左回りが外周、右回りが穴）を、残す頂点の番号の列にする
        let mut points: Vec<[i64; 2]> = Vec::new();
        let mut ids: HashMap<[i64; 2], usize> = HashMap::new();
        let mut outers = Vec::new();
        let mut holes = Vec::new();
        for boundary in boundary_loops(cells) {
            let ring: Vec<usize> = boundary
                .points
                .iter()
                .filter(|&&point| kept(point))
                .map(|&point| {
                    *ids.entry(point).or_insert_with(|| {
                        points.push(point);
                        points.len() - 1
                    })
                })
                .collect();
            if boundary.area > 0 {
                outers.push((boundary, ring));
            } else {
                holes.push((boundary, ring));
            }
        }
        // 穴は、穴に接する材料のセルの中心を含む外周のうち、最も小さいものに属する
        let mut outer_holes: Vec<Vec<Vec<usize>>> = vec![Vec::new(); outers.len()];
        for (hole, ring) in holes {
            let sample = hole.inner_cell_center();
            let owner = outers
                .iter()
                .enumerate()
                .filter(|(_, (outer, _))| outer.contains_doubled(sample))
                .min_by_key(|(_, (outer, _))| outer.area)
                .map(|(index, _)| index);
            match owner {
                Some(index) => outer_holes[index].push(ring),
                None => return self.mesh_plane_cells(key, cells, builder),
            }
        }
        let mut triangles = Vec::new();
        for ((_, ring), holes) in outers.iter().zip(&outer_holes) {
            match polygon::triangulate(&points, ring, holes) {
                Some(part) => triangles.extend(part),
                None => return self.mesh_plane_cells(key, cells, builder),
            }
        }

        // 平面の 2 方向で左回りの三角形の法線は、Y に垂直な平面だけ座標軸と逆を向く
        let counterclockwise_sign = if key.axis == 1 { -1.0 } else { 1.0 };
        let outward = if key.inside_below { 1.0 } else { -1.0 };
        let flip = counterclockwise_sign * outward < 0.0;
        // 頂点は、平面に垂直な辺と平面の交点。辺の材料のない側の端が 2 つ以上の座標軸で外側にある点は、
        // 素材の辺・角の点（外側へ広げた格子点を素材の箱に寄せた点）とし、平面ごとに別の頂点にする（角を鋭く描く）
        let vertices: Vec<u32> = points
            .iter()
            .map(|&point| {
                let start = grid_point(point);
                let mut end = start;
                end[key.axis] += 1;
                let empty_end = if key.inside_below { end } else { start };
                if self.outer_count(empty_end) >= 2 {
                    builder.stock_vertex(empty_end, key.axis, self.stock_point(empty_end))
                } else {
                    builder.edge_vertex(&EdgeKey::new(start, end), self.crossing(start, end))
                }
            })
            .collect();
        for [a, b, c] in triangles {
            let (b, c) = if flip { (c, b) } else { (b, c) };
            builder.add_triangle([vertices[a], vertices[b], vertices[c]]);
        }
    }

    /// 同じ平面の上の面だけを持つセルを、セルごとに描く
    fn mesh_plane_cells(
        &self,
        key: PlaneKey,
        cells: &BTreeSet<[i64; 2]>,
        builder: &mut MeshBuilder,
    ) {
        let [u_axis, v_axis] = plane_axes(key.axis);
        for &[u, v] in cells {
            let mut lower = [0; 3];
            lower[key.axis] = key.layer;
            lower[u_axis] = u;
            lower[v_axis] = v;
            self.mesh_cell(lower, &self.corner_inside(lower), builder);
        }
    }

    /// ブロックの、座標軸 `axis` の向きのセルの番号の範囲（外側へ広げたセルの番号）
    fn block_cells(&self, block: [usize; 3], axis: usize) -> std::ops::Range<usize> {
        let first = block[axis] * SURFACE_BLOCK_CELLS;
        first..(first + SURFACE_BLOCK_CELLS).min(self.count[axis] + 1)
    }

    /// 最小の格子点が `(i, j, k)` のセルのうち、頂点の内外がそろわないものの `k`（昇順）
    ///
    /// 列の 4 本の Z の線の内外は、区間の端でだけ変わる。内外の組が変わる格子点の前後のセルと、4 本の内外が
    /// そろわない範囲のセルが、頂点の内外がそろわないセルになる。`z_cells` は外側へ広げたセルの番号の範囲。
    fn mixed_cells_in_column(&self, i: i64, j: i64, z_cells: std::ops::Range<usize>) -> Vec<i64> {
        let lines: Vec<Vec<(i64, i64)>> = [(i, j), (i + 1, j), (i, j + 1), (i + 1, j + 1)]
            .into_iter()
            .map(|(x, y)| self.inside_ranges(x, y))
            .collect();
        // 内外の組が変わりうる格子点（区間の始まりと終わりの次）と、格子点の範囲の両端
        let last_point = self.count[2] as i64;
        let mut breakpoints: Vec<i64> = lines
            .iter()
            .flatten()
            .flat_map(|&(first, last)| [first, last + 1])
            .chain([-1, last_point + 1])
            .collect();
        breakpoints.sort_unstable();
        breakpoints.dedup();

        let state = |point: i64| -> [bool; 4] {
            std::array::from_fn(|line| {
                lines[line]
                    .iter()
                    .any(|&(first, last)| (first..=last).contains(&point))
            })
        };
        let mut mixed = BTreeSet::new();
        let mut previous: Option<[bool; 4]> = None;
        for pair in breakpoints.windows(2) {
            let (start, end) = (pair[0], pair[1]);
            let current = state(start);
            // 4 本の内外がそろわない範囲の格子点に接するセル
            if current.iter().any(|&flag| flag != current[0]) {
                mixed.extend((start - 1)..end);
            }
            // 内外の組が変わる格子点の手前のセル
            if previous.is_some_and(|previous| previous != current) {
                mixed.insert(start - 1);
            }
            previous = Some(current);
        }
        let (first, end) = (z_cells.start as i64 - 1, z_cells.end as i64 - 1);
        mixed.range(first..end).copied().collect()
    }

    /// Z の線 `(x, y)` が材料の内部にある格子点の番号の範囲（両端を含む。範囲外の線は空）
    ///
    /// `is_inside` と同じ座標の比較で決める。
    fn inside_ranges(&self, x: i64, y: i64) -> Vec<(i64, i64)> {
        if x < 0 || y < 0 || x >= self.count[0] as i64 || y >= self.count[1] as i64 {
            return Vec::new();
        }
        let count = self.count[2] as i64;
        let index = x as usize * self.count[1] + y as usize;
        self.intervals[2][index]
            .iter()
            .filter_map(|&(low, high)| {
                let estimate =
                    |value: f64| ((value - self.origin[2]) / self.pitch[2]).floor() as i64;
                let mut first = estimate(low).clamp(-1, count);
                while first < count && self.coordinate(2, first) < low {
                    first += 1;
                }
                while first > 0 && self.coordinate(2, first - 1) >= low {
                    first -= 1;
                }
                let mut last = estimate(high).clamp(-1, count - 1);
                while last >= 0 && self.coordinate(2, last) > high {
                    last -= 1;
                }
                while last + 1 < count && self.coordinate(2, last + 1) <= high {
                    last += 1;
                }
                let (first, last) = (first.max(0), last.min(count - 1));
                (first <= last).then_some((first, last))
            })
            .collect()
    }

    /// ブロック `block` の面を、すべてのセルを調べて作る（比較用）
    #[cfg(test)]
    fn mesh_block_by_scanning(&self, block: [usize; 3]) -> SurfaceMesh {
        let [x_cells, y_cells, z_cells] = [0, 1, 2].map(|axis| self.block_cells(block, axis));
        let mut cells = Vec::new();
        for x in x_cells {
            for y in y_cells.clone() {
                for z in z_cells.clone() {
                    cells.push([x as i64 - 1, y as i64 - 1, z as i64 - 1]);
                }
            }
        }
        self.mesh_cells(block, &cells, false)
    }

    /// 最小の格子点が `lower` のセルの頂点の内外（`CORNERS` の順）
    fn corner_inside(&self, lower: [i64; 3]) -> [bool; 8] {
        std::array::from_fn(|corner| {
            let offset = CORNERS[corner];
            self.is_inside([0, 1, 2].map(|axis| lower[axis] + offset[axis] as i64))
        })
    }

    /// 最小の格子点が `lower`、頂点の内外が `inside` のセルの面を加える
    fn mesh_cell(&self, lower: [i64; 3], inside: &[bool; 8], builder: &mut MeshBuilder) {
        let corner_point = |corner: usize| {
            let offset = CORNERS[corner];
            [0, 1, 2].map(|axis| lower[axis] + offset[axis] as i64)
        };
        if inside.iter().all(|&flag| flag == inside[0]) {
            return;
        }

        // 面ごとの線分（両端は交点を持つ辺）。各辺は 2 つの面の線分に 1 回ずつ現れる
        let mut neighbors = [[0_usize; 2]; 12];
        let mut neighbor_count = [0_usize; 12];
        let mut connect = |a: usize, b: usize| {
            neighbors[a][neighbor_count[a]] = b;
            neighbor_count[a] += 1;
            neighbors[b][neighbor_count[b]] = a;
            neighbor_count[b] += 1;
        };
        for face in FACES {
            let edges: [usize; 4] = std::array::from_fn(|i| edge_index(face[i], face[(i + 1) % 4]));
            let crossing: [bool; 4] =
                std::array::from_fn(|i| inside[face[i]] != inside[face[(i + 1) % 4]]);
            match crossing.iter().filter(|&&flag| flag).count() {
                2 => {
                    let mut ends = (0..4).filter(|&i| crossing[i]).map(|i| edges[i]);
                    let (first, second) = (ends.next().unwrap(), ends.next().unwrap());
                    connect(first, second);
                }
                // 内側の頂点ごとに、その頂点に接する 2 本の辺を結ぶ（対角線をまたいでつながない）
                4 => {
                    for i in (0..4).filter(|&i| inside[face[i]]) {
                        connect(edges[(i + 3) % 4], edges[i]);
                    }
                }
                _ => {}
            }
        }

        // 線分をつないで輪にし、外向きにそろえて三角形にする
        let mut visited = [false; 12];
        for start in 0..12 {
            if neighbor_count[start] == 0 || visited[start] {
                continue;
            }
            let mut loop_edges = [0_usize; 12];
            let mut length = 0;
            loop_edges[length] = start;
            length += 1;
            visited[start] = true;
            let mut previous = start;
            let mut current = neighbors[start][0];
            while current != start {
                visited[current] = true;
                loop_edges[length] = current;
                length += 1;
                let next = if neighbors[current][0] == previous {
                    neighbors[current][1]
                } else {
                    neighbors[current][0]
                };
                previous = current;
                current = next;
            }
            let loop_edges = &loop_edges[..length];
            let mut points = [[0.0; 3]; 12];
            let mut keys = [EdgeKey {
                lower: [0; 3],
                axis: 0,
            }; 12];
            for (index, &edge) in loop_edges.iter().enumerate() {
                let (a, b) = EDGES[edge];
                let (start_point, end_point) = (corner_point(a), corner_point(b));
                points[index] = self.crossing(start_point, end_point);
                keys[index] = EdgeKey::new(start_point, end_point);
            }
            let (points, keys) = (&mut points[..length], &mut keys[..length]);
            // 外向き: 内側の頂点から外側の頂点へ向かう辺の向きの和
            let mut outward = [0.0; 3];
            for &edge in loop_edges {
                let (a, b) = EDGES[edge];
                let (from, to) = if inside[a] { (a, b) } else { (b, a) };
                for axis in 0..3 {
                    outward[axis] += CORNERS[to][axis] as f64 - CORNERS[from][axis] as f64;
                }
            }
            if dot(polygon_normal(points), outward) < 0.0 {
                keys.reverse();
                points.reverse();
            }
            builder.add_polygon(keys, points);
        }
    }

    /// 格子点が材料の内部か（外側へ広げた格子点は空）
    fn is_inside(&self, point: [i64; 3]) -> bool {
        if (0..3).any(|axis| point[axis] < 0 || point[axis] >= self.count[axis] as i64) {
            return false;
        }
        let [x, y, z] = point.map(|value| value as usize);
        let z_coordinate = self.coordinate(2, z as i64);
        self.intervals[2][x * self.count[1] + y]
            .iter()
            .any(|&(low, high)| (low..=high).contains(&z_coordinate))
    }

    /// 内外の異なる 2 つの格子点の間の辺と面の交点（辺上のデクセルの端点）
    fn crossing(&self, a: [i64; 3], b: [i64; 3]) -> [f64; 3] {
        let axis = (0..3)
            .find(|&axis| a[axis] != b[axis])
            .expect("an edge has one axis");
        let lower = a[axis].min(b[axis]);
        let (low, high) = (
            self.coordinate(axis, lower),
            self.coordinate(axis, lower + 1),
        );
        let plane = plane_axes(axis);
        let line = [a[plane[0]], a[plane[1]]];
        let on_line = (0..2).all(|i| line[i] >= 0 && line[i] < self.count[plane[i]] as i64);
        let endpoint = on_line
            .then(|| {
                let index = line[0] as usize * self.count[plane[1]] + line[1] as usize;
                self.intervals[axis][index]
                    .iter()
                    .flat_map(|&(start, end)| [start, end])
                    .find(|value| (low..=high).contains(value))
            })
            .flatten()
            // 内外の判定と端点が食い違う場合（丸め誤差）は、辺の中点にする
            .unwrap_or(0.5 * (low + high));
        let mut point = [0, 1, 2].map(|i| self.coordinate(i, a[i]));
        point[axis] = endpoint;
        point
    }

    fn coordinate(&self, axis: usize, index: i64) -> f64 {
        self.origin[axis] + index as f64 * self.pitch[axis]
    }
}

/// 区間の列 `intervals` の、範囲 `span` の中の部分
fn clip_intervals(intervals: &[(f64, f64)], (start, end): (f64, f64)) -> Vec<(f64, f64)> {
    intervals
        .iter()
        .map(|&(low, high)| (low.max(start), high.min(end)))
        .filter(|(low, high)| low < high)
        .collect()
}

/// 区間の列 `before` から `after` に変わる範囲（一致しない端点の最小と最大。変わらなければ `None`）
fn changed_range(before: &[(f64, f64)], after: &[(f64, f64)]) -> Option<(f64, f64)> {
    let endpoints = |intervals: &[(f64, f64)]| -> Vec<f64> {
        intervals
            .iter()
            .flat_map(|&(low, high)| [low, high])
            .collect()
    };
    let (before, after) = (endpoints(before), endpoints(after));
    let differing = before
        .iter()
        .filter(|value| !after.contains(value))
        .chain(after.iter().filter(|value| !before.contains(value)));
    differing.fold(None, |range, &value| match range {
        None => Some((value, value)),
        Some((low, high)) => Some((low.min(value), high.max(value))),
    })
}

/// デクセルの向き `axis` の格子の 2 方向（`axis` 以外の座標軸の昇順）
fn plane_axes(axis: usize) -> [usize; 2] {
    match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    }
}

/// 平面の上のセルの集まりの輪郭の輪（格子点の列。材料のセルを左に見る向き）
struct Boundary {
    points: Vec<[i64; 2]>,
    /// 2 倍の符号付き面積（外周は正、穴は負）
    area: i64,
}

impl Boundary {
    /// 輪の最初の辺の左にある材料のセルの中心（座標を 2 倍した値）
    fn inner_cell_center(&self) -> [i64; 2] {
        let (start, end) = (self.points[0], self.points[1]);
        let direction = [end[0] - start[0], end[1] - start[1]];
        [
            2 * start[0] + direction[0] - direction[1],
            2 * start[1] + direction[1] + direction[0],
        ]
    }

    /// 座標を 2 倍した点 `point`（輪の上にない点）が輪の中にあるか
    fn contains_doubled(&self, point: [i64; 2]) -> bool {
        let mut inside = false;
        for (index, &current) in self.points.iter().enumerate() {
            let next = self.points[(index + 1) % self.points.len()];
            let (a, b) = (current.map(|value| 2 * value), next.map(|value| 2 * value));
            if (a[1] > point[1]) != (b[1] > point[1]) {
                // 辺は座標軸に平行で、Y の向きの辺だけが横切る
                if point[0] < a[0] {
                    inside = !inside;
                }
            }
        }
        inside
    }
}

/// セルの集まり `cells` の輪郭の輪（2 つの輪が 1 点で接する場合は、左に曲がる向きを選んで分ける）
fn boundary_loops(cells: &BTreeSet<[i64; 2]>) -> Vec<Boundary> {
    let mut outgoing: BTreeMap<[i64; 2], Vec<[i64; 2]>> = BTreeMap::new();
    let mut add = |start: [i64; 2], end: [i64; 2]| outgoing.entry(start).or_default().push(end);
    for &[u, v] in cells {
        if !cells.contains(&[u, v - 1]) {
            add([u, v], [u + 1, v]);
        }
        if !cells.contains(&[u + 1, v]) {
            add([u + 1, v], [u + 1, v + 1]);
        }
        if !cells.contains(&[u, v + 1]) {
            add([u + 1, v + 1], [u, v + 1]);
        }
        if !cells.contains(&[u - 1, v]) {
            add([u, v + 1], [u, v]);
        }
    }
    let mut loops = Vec::new();
    while let Some(&start) = outgoing.keys().next() {
        let mut points = vec![start];
        let mut current = start;
        let mut direction: Option<[i64; 2]> = None;
        loop {
            let ends = outgoing
                .get_mut(&current)
                .expect("each point has an outgoing edge");
            let turn = |end: &[i64; 2]| {
                direction.map_or(0, |[du, dv]| {
                    let [eu, ev] = [end[0] - current[0], end[1] - current[1]];
                    du * ev - dv * eu
                })
            };
            let index = (0..ends.len())
                .max_by_key(|&index| turn(&ends[index]))
                .expect("outgoing edges are not empty");
            let end = ends.swap_remove(index);
            if ends.is_empty() {
                outgoing.remove(&current);
            }
            direction = Some([end[0] - current[0], end[1] - current[1]]);
            current = end;
            if current == start {
                break;
            }
            points.push(current);
        }
        let area = (0..points.len())
            .map(|index| {
                let (a, b) = (points[index], points[(index + 1) % points.len()]);
                a[0] * b[1] - a[1] * b[0]
            })
            .sum();
        loops.push(Boundary { points, area });
    }
    loops
}

/// 座標軸 `axis` に垂直な平面（`layer` はセルのその軸の最小の格子点の番号、`value` は平面の座標を
/// 同じ平面とみなす幅で区切った区画の番号）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PlaneKey {
    axis: usize,
    /// 平面より小さい座標の側が材料か
    inside_below: bool,
    layer: i64,
    value: i64,
}

/// 格子の辺（小さい側の格子点と向き）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeKey {
    lower: [i64; 3],
    axis: usize,
}

impl EdgeKey {
    fn new(a: [i64; 3], b: [i64; 3]) -> Self {
        let axis = (0..3)
            .find(|&axis| a[axis] != b[axis])
            .expect("an edge has one axis");
        Self {
            lower: if a[axis] < b[axis] { a } else { b },
            axis,
        }
    }
}

/// 辺ごとに頂点を共有して面を組み立てる
struct MeshBuilder {
    mesh: SurfaceMesh,
    /// ブロックの格子点の最小の番号（辺の小さい側の格子点は、ここから各軸 `SURFACE_BLOCK_CELLS + 1` まで）
    origin: [i64; 3],
    /// 辺（格子点と向き）ごとの頂点の番号（`u32::MAX` は未登録）
    vertices: Vec<u32>,
    /// 素材の辺・角の点（外側へ広げた格子点）と、その点を使う面の座標軸ごとの頂点の番号
    stock_vertices: HashMap<([i64; 3], usize), u32>,
}

impl MeshBuilder {
    /// 最小の格子点が `origin` のブロックの面を組み立てる
    fn new(origin: [i64; 3]) -> Self {
        let side = SURFACE_BLOCK_CELLS + 1;
        Self {
            mesh: SurfaceMesh::default(),
            origin,
            vertices: vec![u32::MAX; side * side * side * 3],
            stock_vertices: HashMap::new(),
        }
    }

    /// 素材の辺・角の点 `point` の、座標軸 `axis` に垂直な面の頂点の番号（なければ `position` で加える）
    fn stock_vertex(&mut self, point: [i64; 3], axis: usize, position: [f64; 3]) -> u32 {
        if let Some(&index) = self.stock_vertices.get(&(point, axis)) {
            return index;
        }
        self.mesh.positions.push(position);
        self.mesh.normals.push([0.0; 3]);
        let index = (self.mesh.positions.len() - 1) as u32;
        self.stock_vertices.insert((point, axis), index);
        index
    }

    fn slot(&self, key: &EdgeKey) -> usize {
        let side = (SURFACE_BLOCK_CELLS + 1) as i64;
        let [x, y, z] = [0, 1, 2].map(|axis| key.lower[axis] - self.origin[axis]);
        debug_assert!([x, y, z].iter().all(|&value| (0..side).contains(&value)));
        (((x * side + y) * side + z) * 3) as usize + key.axis
    }

    /// 外向きにそろえた多角形を、最初の頂点からの扇形の三角形で加える
    fn add_polygon(&mut self, keys: &[EdgeKey], points: &[[f64; 3]]) {
        let mut indices = [0_u32; 12];
        for (index, (key, point)) in keys.iter().zip(points).enumerate() {
            indices[index] = self.edge_vertex(key, *point);
        }
        for i in 1..keys.len().saturating_sub(1) {
            self.add_triangle([indices[0], indices[i], indices[i + 1]]);
        }
    }

    /// 辺 `key` の頂点の番号（なければ `point` で加える）
    fn edge_vertex(&mut self, key: &EdgeKey, point: [f64; 3]) -> u32 {
        let slot = self.slot(key);
        if self.vertices[slot] == u32::MAX {
            self.mesh.positions.push(point);
            self.mesh.normals.push([0.0; 3]);
            self.vertices[slot] = (self.mesh.positions.len() - 1) as u32;
        }
        self.vertices[slot]
    }

    /// 三角形を加え、面積で重み付けた法線を頂点に足す
    fn add_triangle(&mut self, triangle: [u32; 3]) {
        let [a, b, c] = triangle.map(|index| self.mesh.positions[index as usize]);
        let normal = cross(sub(b, a), sub(c, a));
        for index in triangle {
            let accumulated = &mut self.mesh.normals[index as usize];
            for axis in 0..3 {
                accumulated[axis] += normal[axis];
            }
        }
        self.mesh.indices.extend(triangle);
    }

    fn finish(mut self) -> SurfaceMesh {
        for normal in &mut self.mesh.normals {
            let length = dot(*normal, *normal).sqrt();
            if length > 0.0 {
                *normal = normal.map(|value| value / length);
            }
        }
        self.mesh
    }
}

/// 多角形の法線（Newell の方法）
fn polygon_normal(points: &[[f64; 3]]) -> [f64; 3] {
    let mut normal = [0.0; 3];
    for (i, current) in points.iter().enumerate() {
        let next = points[(i + 1) % points.len()];
        normal[0] += (current[1] - next[1]) * (current[2] + next[2]);
        normal[1] += (current[2] - next[2]) * (current[0] + next[0]);
        normal[2] += (current[0] - next[0]) * (current[1] + next[1]);
    }
    normal
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
