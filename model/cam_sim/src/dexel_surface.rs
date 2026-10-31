//! トリデクセルの表示用の面（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §4・§4.1）
//!
//! 3 方向の格子線は同じ格子点を通るため、格子点の間の辺はいずれかの方向のデクセル上にある。格子点の内外を
//! Z の向きのデクセルで判定し、内外が変わる辺の交点を、その辺上のデクセルの区間の端（正本から求めた厳密な点）と
//! その端の法線とする。面は、交点と法線を使う双対の形のデュアルコンツアリングで作る。
//!
//! - 頂点: セルの各面で「内側の頂点どうしは面の対角線をまたいでつながない」と決めて交点の輪を作り、輪ごとに
//!   頂点を 1 つ、輪の交点の法線の平面からのずれの 2 乗の和（QEF）が最小の点に置く（セルの中に収める）。
//!   稜線・角の頂点は、稜線・角の上に来る
//! - 面: 内外が変わる辺ごとに、辺を囲む 4 つのセルの、その辺を含む輪の頂点で四角形を作る。隣り合う四角形は
//!   頂点を共有するため、面に穴は開かない
//!
//! 格子を素材の外側へ 1 層広げ、そこを空として扱うことで、素材の外周の面も作る。面はセルのブロックごとに作り
//! （辺は、辺の小さい側の格子点を最小の格子点とするセルのブロックに属する）、スナップショットの変化から
//! 作り直すブロックを求める。
//!
//! 座標軸に垂直な同じ平面の上の面だけを持つセル（削っていない素材の外周や、平らな底など）の頂点だけでできた
//! 四角形は、ブロックの中で多角形にまとめて三角形に分ける。周りの四角形がすべてまとめた平面に属する頂点は、
//! 頂点の周りだけで決まる決まりで省くため、隣のブロックの多角形・四角形と頂点がそろい、面に穴は開かない。

mod polygon;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::constants::SURFACE_BLOCK_CELLS;
use crate::dexel_normal::DexelNormal;
use crate::simulation_artifact::{DexelGridLayout, DexelGridStates, DexelState};

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

/// QEF を解くときに切り捨てる成分の、最大の特異値に対する比（「Dual Contouring of Hermite Data」の値。長さの
/// 許容誤差ではなく、法線がほぼ平行な交点の頂点が暴れないようにする方式の値）
const QEF_SINGULAR_VALUE_RATIO: f64 = 0.1;

/// 対称な 3×3 行列の固有値を求めるヤコビ法の、回転の掃引の回数の上限（対角の外が 0 になれば先に終わる）
const JACOBI_MAX_SWEEPS: usize = 32;

/// スナップショットの時点の、トリデクセルの表示用の面
#[derive(Debug, Clone)]
pub struct DexelSurface {
    grids: [DexelGridLayout; 3],
    intervals: [Vec<Vec<(f64, f64)>>; 3],
    /// `intervals` と同じ並びの、区間の始まり・終わりの材料の外向きの法線
    normals: [Vec<Vec<[DexelNormal; 2]>>; 3],
    /// 各座標軸の、最初の格子点の座標・間隔・格子点の数
    origin: [f64; 3],
    pitch: [f64; 3],
    count: [usize; 3],
    /// 平らな部分をまとめるときに、同じ平面とみなす交点の座標の幅（素材の精度）
    flat_tolerance: f64,
    /// 頂点の陰影を分ける折れの角度（ラジアン。なめらかな曲面の隣り合う四角形の向きの違いの最大）
    crease_angle: f64,
}

impl DexelSurface {
    /// 格子 `grids` のデクセルが区間 `states` のときの面
    ///
    /// `flat_tolerance` は同じ平面とみなす交点の座標の幅（素材の精度）、`curvature_radius` は削った面の曲率半径の
    /// 最小（工具の半径）。陰影を分ける折れの角度は、隣り合うセルの頂点の距離の最大 √3·間隔 から
    /// √3·間隔 / `curvature_radius` とする。
    pub fn new(
        grids: [DexelGridLayout; 3],
        states: DexelGridStates,
        flat_tolerance: f64,
        curvature_radius: f64,
    ) -> Self {
        // Z の格子は X・Y、X の格子は Y・Z の座標を持つ
        let [z_grid, x_grid] = [grids[2], grids[0]];
        let pitch = [z_grid.pitch[0], z_grid.pitch[1], x_grid.pitch[1]];
        let spacing = pitch.into_iter().fold(0.0, f64::max);
        Self {
            grids,
            intervals: states.intervals,
            normals: states.normals,
            origin: [z_grid.origin[0], z_grid.origin[1], x_grid.origin[1]],
            pitch,
            count: [
                z_grid.count[0] as usize,
                z_grid.count[1] as usize,
                x_grid.count[1] as usize,
            ],
            flat_tolerance,
            crease_angle: 3.0_f64.sqrt() * spacing / curvature_radius,
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
    /// 変化後の区間と法線（`changes`。複数の時点の変化を続けて並べてよい）を順に当て、作り直すブロックをまとめて
    /// 返す。作り直すのは、区間の端か法線が変わった範囲（変化前と変化後で一致しない端の間）の格子点・辺に接するセルと、
    /// その周りの 2 層のセルを含むブロック（四角形は辺を囲む 4 つのセルの頂点を使い、平らな部分の頂点を省くかは、
    /// 頂点のセルと隣のセルの四角形で決まるため）。
    pub fn apply_changes<'a>(
        &mut self,
        changes: impl IntoIterator<Item = DexelState<'a>>,
    ) -> Vec<[usize; 3]> {
        let blocks = self.block_counts();
        let mut dirty = BTreeSet::new();
        for change in changes {
            let grid = change.grid as usize;
            let index = change.index as usize;
            let Some((low, high)) = changed_range(
                (&self.intervals[grid][index], &self.normals[grid][index]),
                (change.intervals, change.normals),
            ) else {
                continue;
            };
            let dexel = &mut self.intervals[grid][index];
            dexel.clear();
            dexel.extend_from_slice(change.intervals);
            let normals = &mut self.normals[grid][index];
            normals.clear();
            normals.extend_from_slice(change.normals);
            let plane_axes = plane_axes(grid);
            let count = self.grids[grid].count[1] as usize;
            let line = [change.index as usize / count, change.index as usize % count];
            let mut ranges = [0..0, 0..0, 0..0];
            // 線の向き: 変わった範囲の端点を挟む格子点（前後に 1 つの余裕）に接するセルと、その前後の 2 層
            // （外側へ広げたセルの番号は、格子点の番号 + 1 と + 0）
            let point =
                |value: f64| ((value - self.origin[grid]) / self.pitch[grid]).floor() as i64;
            let last_cell = self.count[grid] as i64;
            let (first, last) = (
                (point(low) - 3).clamp(0, last_cell) as usize,
                (point(high) + 5).clamp(0, last_cell) as usize,
            );
            ranges[grid] =
                first / SURFACE_BLOCK_CELLS..(last / SURFACE_BLOCK_CELLS + 1).min(blocks[grid]);
            // 線の周り: 線上の格子点・辺に接するセルと、その周りの 2 層
            for (index, &axis) in plane_axes.iter().enumerate() {
                let first = line[index].saturating_sub(2) / SURFACE_BLOCK_CELLS;
                let last = (line[index] + 3) / SURFACE_BLOCK_CELLS;
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
    /// ブロックを通るデクセルが、ブロックの範囲ですべて素材の範囲（初期の区間）のままなら、頂点の内外がそろわない
    /// セルを探さず、外側へ広げた層のセルだけで面を作る（セルごとに探した場合と同じ面になる）。
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
            return self.mesh_cells(&cells, true);
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
        self.mesh_cells(&cells, false)
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

    /// 最小の格子点が `cells` のセルを最小の格子点とする辺の面（ブロックの面は、ブロックのセルを渡す）
    ///
    /// 同じ平面に乗るセルの頂点だけでできた四角形は、平面ごとに多角形にまとめる。
    ///
    /// `uncut` の場合、`cells` は削っていないブロックの外側へ広げた層のセルで、セルが乗る平面は判定せずに素材の面とする
    /// （判定した場合と同じ平面になる）。
    fn mesh_cells(&self, cells: &[[i64; 3]], uncut: bool) -> SurfaceMesh {
        let mut context = BlockContext::default();
        if uncut {
            for &lower in cells {
                let planes = self
                    .outer_sides(lower)
                    .into_iter()
                    .map(|(axis, inside_below)| {
                        self.stock_face_key(axis, inside_below, lower[axis])
                    })
                    .collect();
                // 削っていないブロックの格子点は、外側へ広げた層でなければ材料の内部
                let inside = std::array::from_fn(|corner| {
                    (0..3).all(|axis| {
                        let index = lower[axis] + CORNERS[corner][axis] as i64;
                        index >= 0 && index < self.count[axis] as i64
                    })
                });
                context
                    .cells
                    .insert(lower, CellSurface::new(inside, planes));
            }
        }
        let mut planes: BTreeMap<PlaneKey, BTreeSet<[i64; 2]>> = BTreeMap::new();
        for &lower in cells {
            let inside = self.cell(lower, &mut context).inside;
            for axis in 0..3 {
                if inside[0] == inside[1 << axis] {
                    continue;
                }
                match self.quad_plane(lower, axis, &mut context) {
                    // 四角形は、平面の 2 方向で辺を囲むセルの範囲の正方形（最小のセルの番号）
                    Some(key) => {
                        let [u, v] = plane_axes(axis);
                        planes
                            .entry(key)
                            .or_default()
                            .insert([lower[u] - 1, lower[v] - 1]);
                    }
                    None => self.mesh_quad(lower, axis, &mut context),
                }
            }
        }
        for (key, squares) in planes {
            self.mesh_plane(key, &squares, &mut context);
        }
        context.builder.finish(self.crease_angle)
    }

    /// 最小の格子点が `lower` のセルの、交点の輪と、セルが乗る平面（`context` に覚える）
    fn cell<'a>(&self, lower: [i64; 3], context: &'a mut BlockContext) -> &'a CellSurface {
        context.cells.entry(lower).or_insert_with(|| {
            let inside = self.corner_inside(lower);
            let planes = self.planes_of_cell(lower, &inside, self.is_sharp_cell(lower));
            CellSurface::new(inside, planes)
        })
    }

    /// セル `lower` の輪 `index` の頂点の番号（頂点の位置は、使うときに初めて求める）
    fn loop_vertex_id(&self, lower: [i64; 3], index: u8, context: &mut BlockContext) -> u32 {
        if let Some(&id) = context.builder.ids.get(&(lower, index)) {
            return id;
        }
        let edge_loops = self.cell(lower, context).edge_loops;
        let position = self.loop_vertex(lower, &edge_loops, index);
        context.builder.vertex((lower, index), position)
    }

    /// 最小の格子点が `lower` のセルの、輪 `index` の頂点（QEF の最小点をセルの中に収めた点。輪の辺は `edge_loops`）
    ///
    /// 交点の重心 m を基準に、交点 pᵢ・法線 nᵢ の Σ (nᵢ·(x − pᵢ))² の最小点を求める。Σ nᵢnᵢᵀ の固有値（特異値の
    /// 2 乗）のうち、最大の `QEF_SINGULAR_VALUE_RATIO²` 倍より小さい成分は切り捨てる（m から動かさない）。平面が
    /// 1 枚なら m を平面に射影した点（法線がすべて同じなら m そのもの）、2 枚なら交線に射影した点になる。
    fn loop_vertex(&self, lower: [i64; 3], edge_loops: &[Option<u8>; 12], index: u8) -> [f64; 3] {
        let corner_point = |corner: usize| {
            let offset = CORNERS[corner];
            [0, 1, 2].map(|axis| lower[axis] + offset[axis] as i64)
        };
        let mut crossings = [([0.0; 3], [0.0; 3]); 12];
        let mut count = 0;
        for (edge, &owner) in edge_loops.iter().enumerate() {
            if owner == Some(index) {
                let (a, b) = EDGES[edge];
                crossings[count] = self.crossing_with_normal(corner_point(a), corner_point(b));
                count += 1;
            }
        }
        let crossings = &crossings[..count];
        let mass: [f64; 3] = std::array::from_fn(|axis| {
            crossings.iter().map(|(point, _)| point[axis]).sum::<f64>() / count as f64
        });
        let clamped = |vertex: [f64; 3]| -> [f64; 3] {
            std::array::from_fn(|axis| {
                vertex[axis].clamp(
                    self.coordinate(axis, lower[axis]),
                    self.coordinate(axis, lower[axis] + 1),
                )
            })
        };
        if crossings
            .iter()
            .all(|(_, normal)| *normal == crossings[0].1)
        {
            return clamped(mass);
        }
        let mut matrix = [[0.0; 3]; 3];
        let mut rhs = [0.0; 3];
        for (point, normal) in crossings {
            let offset = dot(*normal, sub(*point, mass));
            for row in 0..3 {
                rhs[row] += normal[row] * offset;
                for column in 0..3 {
                    matrix[row][column] += normal[row] * normal[column];
                }
            }
        }
        let (values, vectors) = symmetric_eigen(matrix);
        let largest = values.iter().copied().fold(0.0, f64::max);
        let threshold = QEF_SINGULAR_VALUE_RATIO * QEF_SINGULAR_VALUE_RATIO * largest;
        let mut vertex = mass;
        for (&value, vector) in values.iter().zip(&vectors) {
            if value > 0.0 && value >= threshold {
                let scale = dot(*vector, rhs) / value;
                for axis in 0..3 {
                    vertex[axis] += scale * vector[axis];
                }
            }
        }
        clamped(vertex)
    }

    /// 最小の格子点が `point` の座標軸 `axis` の向きの辺を囲む 4 つのセルの最小の格子点（辺の正の向きから見て左回り）
    fn cells_around_edge(&self, point: [i64; 3], axis: usize) -> [[i64; 3]; 4] {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        [(-1, -1), (0, -1), (0, 0), (-1, 0)].map(|(du, dv)| {
            let mut lower = point;
            lower[u] += du;
            lower[v] += dv;
            lower
        })
    }

    /// 最小の格子点が `point` の座標軸 `axis` の向きの辺を含む、セル `lower` の輪の頂点の番号
    fn edge_vertex(
        &self,
        lower: [i64; 3],
        point: [i64; 3],
        axis: usize,
        context: &mut BlockContext,
    ) -> u32 {
        let start: [usize; 3] = std::array::from_fn(|i| (point[i] - lower[i]) as usize);
        let mut end = start;
        end[axis] += 1;
        let corner = |offset: [usize; 3]| offset[0] + 2 * offset[1] + 4 * offset[2];
        let edge = edge_index(corner(start), corner(end));
        let index = self.cell(lower, context).edge_loops[edge]
            .expect("a crossing edge belongs to a loop of the cell");
        self.loop_vertex_id(lower, index, context)
    }

    /// 最小の格子点が `point` の座標軸 `axis` の向きの辺の四角形の頂点（外向きが表になる順）
    fn quad_vertices(&self, point: [i64; 3], axis: usize, context: &mut BlockContext) -> [u32; 4] {
        let mut cells = self.cells_around_edge(point, axis);
        // 辺の小さい側が材料なら外向きは座標軸の正の向きで、左回りのまま表になる
        if !self.cell(point, context).inside[0] {
            cells.reverse();
        }
        cells.map(|lower| self.edge_vertex(lower, point, axis, context))
    }

    /// 最小の格子点が `point` の座標軸 `axis` の向きの辺の四角形を、2 つの三角形で加える
    ///
    /// 対角線は、分けた 2 つの三角形の向きの違いが小さいほうを選ぶ。
    fn mesh_quad(&self, point: [i64; 3], axis: usize, context: &mut BlockContext) {
        let quad = self.quad_vertices(point, axis, context);
        let positions = quad.map(|index| context.builder.positions[index as usize]);
        let agreement = |[first, second]: [[usize; 3]; 2]| {
            let normal = |[a, b, c]: [usize; 3]| {
                cross(
                    sub(positions[b], positions[a]),
                    sub(positions[c], positions[a]),
                )
            };
            let (n1, n2) = (normal(first), normal(second));
            let lengths = dot(n1, n1).sqrt() * dot(n2, n2).sqrt();
            if lengths == 0.0 {
                1.0
            } else {
                dot(n1, n2) / lengths
            }
        };
        let splits = [[[0, 1, 2], [0, 2, 3]], [[0, 1, 3], [1, 2, 3]]];
        let split = if agreement(splits[1]) > agreement(splits[0]) {
            splits[1]
        } else {
            splits[0]
        };
        for triangle in split {
            context.builder.add_triangle(triangle.map(|i| quad[i]));
        }
    }

    /// 最小の格子点が `point` の座標軸 `axis` の向きの辺の四角形がまとめる平面（辺を囲む 4 つのセルがすべて乗る、
    /// その軸に垂直な平面。なければ `None`）
    fn quad_plane(
        &self,
        point: [i64; 3],
        axis: usize,
        context: &mut BlockContext,
    ) -> Option<PlaneKey> {
        if let Some(&plane) = context.quad_planes.get(&(point, axis)) {
            return plane;
        }
        let cells = self.cells_around_edge(point, axis);
        let candidates: Vec<PlaneKey> = self
            .cell(cells[0], context)
            .planes
            .iter()
            .copied()
            .filter(|key| key.axis == axis)
            .collect();
        let plane = candidates.into_iter().find(|key| {
            cells[1..]
                .iter()
                .all(|&lower| self.cell(lower, context).planes.contains(key))
        });
        context.quad_planes.insert((point, axis), plane);
        plane
    }

    /// 平らな部分の多角形の頂点（最小の格子点が `lower` のセルの頂点）を残すか
    ///
    /// 頂点を使う四角形（セルの交点を持つ辺の四角形）がすべてまとめた平面に属し、平面が 1 枚なら平面の 2 方向とも、
    /// 座標軸の違う 2 枚なら 2 枚の交線の向きで、ブロックの境の線（外側へ広げた格子の端を含む）の上にない頂点を
    /// 省く。判定はセルと隣のセルだけで決まるため、どのブロックから見ても同じになる。平面が 2 枚の頂点は交線の上に
    /// あるため、省いても形は変わらない。
    fn keeps_vertex(&self, lower: [i64; 3], context: &mut BlockContext) -> bool {
        if let Some(&kept) = context.kept_vertices.get(&lower) {
            return kept;
        }
        let kept = self.decides_to_keep_vertex(lower, context);
        context.kept_vertices.insert(lower, kept);
        kept
    }

    fn decides_to_keep_vertex(&self, lower: [i64; 3], context: &mut BlockContext) -> bool {
        let cell = self.cell(lower, context);
        if cell.loop_count != 1 {
            return true;
        }
        let edges: Vec<([i64; 3], usize)> = (0..12)
            .filter(|&edge| cell.edge_loops[edge].is_some())
            .map(|edge| {
                let (a, b) = EDGES[edge];
                let start = [0, 1, 2].map(|axis| lower[axis] + CORNERS[a][axis] as i64);
                let axis = (0..3)
                    .find(|&axis| CORNERS[a][axis] != CORNERS[b][axis])
                    .expect("an edge has one axis");
                (start, axis)
            })
            .collect();
        let mut planes: Vec<PlaneKey> = Vec::new();
        for (point, axis) in edges {
            match self.quad_plane(point, axis, context) {
                None => return true,
                Some(key) => {
                    if !planes.contains(&key) {
                        planes.push(key);
                    }
                }
            }
        }
        // ブロックの境の線: 四角形は辺を囲むセルの範囲の正方形のため、ブロックの最初のセルの 1 つ手前の線
        let fixed = |axis: usize| {
            let index = lower[axis];
            index == -1
                || index == self.count[axis] as i64 - 1
                || (index + 2).rem_euclid(SURFACE_BLOCK_CELLS as i64) == 0
        };
        match planes[..] {
            [key] => {
                let [u, v] = plane_axes(key.axis);
                fixed(u) && fixed(v)
            }
            [first, second] if first.axis != second.axis => fixed(3 - first.axis - second.axis),
            _ => true,
        }
    }

    /// 同じ平面 `key` にまとめる四角形（平面の 2 方向の、辺を囲むセルの範囲の正方形の最小のセルの番号）を、
    /// 多角形にまとめて面を加える
    ///
    /// 多角形の輪郭はセルの番号の整数の格子で求め、残す頂点（`keeps_vertex`）だけで耳刈り法で三角形に分ける。
    /// 三角形に分けられない場合は四角形ごとに描く。
    fn mesh_plane(&self, key: PlaneKey, squares: &BTreeSet<[i64; 2]>, context: &mut BlockContext) {
        let [u_axis, v_axis] = plane_axes(key.axis);
        let cell_at = |[u, v]: [i64; 2]| {
            let mut lower = [0; 3];
            lower[key.axis] = key.layer;
            lower[u_axis] = u;
            lower[v_axis] = v;
            lower
        };

        // 輪郭の輪（左回りが外周、右回りが穴）を、残す頂点の番号の列にする
        let mut points: Vec<[i64; 2]> = Vec::new();
        let mut ids: HashMap<[i64; 2], usize> = HashMap::new();
        let mut outers = Vec::new();
        let mut holes = Vec::new();
        for boundary in boundary_loops(squares) {
            let mut ring = Vec::new();
            for &point in &boundary.points {
                if !self.keeps_vertex(cell_at(point), context) {
                    continue;
                }
                ring.push(*ids.entry(point).or_insert_with(|| {
                    points.push(point);
                    points.len() - 1
                }));
            }
            if boundary.area > 0 {
                outers.push((boundary, ring));
            } else {
                holes.push((boundary, ring));
            }
        }
        // 穴は、穴に接するまとめた正方形の中心を含む外周のうち、最も小さいものに属する
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
                None => return self.mesh_plane_quads(key, squares, context),
            }
        }
        let mut triangles = Vec::new();
        for ((_, ring), holes) in outers.iter().zip(&outer_holes) {
            match polygon::triangulate(&points, ring, holes) {
                Some(part) => triangles.extend(part),
                None => return self.mesh_plane_quads(key, squares, context),
            }
        }

        // 平面の 2 方向で左回りの三角形の法線は、Y に垂直な平面だけ座標軸と逆を向く
        let counterclockwise_sign = if key.axis == 1 { -1.0 } else { 1.0 };
        let outward = if key.inside_below { 1.0 } else { -1.0 };
        let flip = counterclockwise_sign * outward < 0.0;
        // 平面に乗るセルの交点の輪は 1 つ
        let vertices: Vec<u32> = points
            .iter()
            .map(|&point| self.loop_vertex_id(cell_at(point), 0, context))
            .collect();
        for [a, b, c] in triangles {
            let (b, c) = if flip { (c, b) } else { (b, c) };
            context
                .builder
                .add_triangle([vertices[a], vertices[b], vertices[c]]);
        }
    }

    /// 同じ平面 `key` にまとめる四角形を、四角形ごとに描く
    fn mesh_plane_quads(
        &self,
        key: PlaneKey,
        squares: &BTreeSet<[i64; 2]>,
        context: &mut BlockContext,
    ) {
        let [u_axis, v_axis] = plane_axes(key.axis);
        for &[u, v] in squares {
            let mut point = [0; 3];
            point[key.axis] = key.layer;
            point[u_axis] = u + 1;
            point[v_axis] = v + 1;
            self.mesh_quad(point, key.axis, context);
        }
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
        self.mesh_cells(&cells, false)
    }

    /// 最小の格子点が `lower` のセルの頂点の内外（`CORNERS` の順）
    fn corner_inside(&self, lower: [i64; 3]) -> [bool; 8] {
        std::array::from_fn(|corner| {
            let offset = CORNERS[corner];
            self.is_inside([0, 1, 2].map(|axis| lower[axis] + offset[axis] as i64))
        })
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

    /// 内外の異なる 2 つの格子点の間の辺と面の交点（辺上のデクセルの区間の端）
    fn crossing(&self, a: [i64; 3], b: [i64; 3]) -> [f64; 3] {
        self.crossing_with_normal(a, b).0
    }

    /// 内外の異なる 2 つの格子点の間の辺と面の交点（辺上のデクセルの区間の端）と、その端の材料の外向きの法線
    ///
    /// 面が格子点を通る場合、格子点の内外（Z の向きのデクセルで判定する）と辺の上のデクセルの端は、丸め誤差で
    /// 食い違いうる。そのため、辺に最も近い端を辺の範囲に収めて使う。
    fn crossing_with_normal(&self, a: [i64; 3], b: [i64; 3]) -> ([f64; 3], [f64; 3]) {
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
                    .zip(&self.normals[axis][index])
                    .flat_map(|(&(start, end), &[start_normal, end_normal])| {
                        [(start, start_normal), (end, end_normal)]
                    })
                    .min_by(|(a, _), (b, _)| {
                        let gap = |value: f64| (low - value).max(value - high).max(0.0);
                        gap(*a).total_cmp(&gap(*b))
                    })
                    .map(|(value, normal)| (value.clamp(low, high), normal.to_unit()))
            })
            .flatten()
            // 辺の上のデクセルに端がない場合は、辺の中点の、材料から空への向きの面とする
            .unwrap_or_else(|| {
                let mut normal = [0.0; 3];
                normal[axis] = if self.is_inside(a) == (a[axis] < b[axis]) {
                    1.0
                } else {
                    -1.0
                };
                (0.5 * (low + high), normal)
            });
        let mut point = [0, 1, 2].map(|i| self.coordinate(i, a[i]));
        point[axis] = endpoint.0;
        (point, endpoint.1)
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

/// 区間と法線の列 `before` から `after` に変わる範囲（座標か法線が一致しない端の座標の最小と最大。変わらなければ
/// `None`）
fn changed_range(
    before: (&[(f64, f64)], &[[DexelNormal; 2]]),
    after: (&[(f64, f64)], &[[DexelNormal; 2]]),
) -> Option<(f64, f64)> {
    let ends =
        |(intervals, normals): (&[(f64, f64)], &[[DexelNormal; 2]])| -> Vec<(f64, DexelNormal)> {
            intervals
                .iter()
                .zip(normals)
                .flat_map(|(&(low, high), &[low_normal, high_normal])| {
                    [(low, low_normal), (high, high_normal)]
                })
                .collect()
        };
    let (before, after) = (ends(before), ends(after));
    let differing = before
        .iter()
        .filter(|end| !after.contains(end))
        .chain(after.iter().filter(|end| !before.contains(end)));
    differing.fold(None, |range, &(value, _)| match range {
        None => Some((value, value)),
        Some((low, high)) => Some((low.min(value), high.max(value))),
    })
}

/// 頂点の内外が `inside` のセルの、交点の輪（辺ごとの輪の番号と、輪の数）
///
/// セルの各面で交点を線分でつなぎ（内側の頂点どうしは面の対角線をまたいでつながない）、線分をつないで輪にする。
/// 交点を持つ辺は、ちょうど 1 つの輪に入る。
fn cell_loops(inside: &[bool; 8]) -> ([Option<u8>; 12], u8) {
    let mut edge_loops = [None; 12];
    if inside.iter().all(|&flag| flag == inside[0]) {
        return (edge_loops, 0);
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
    let mut count = 0;
    for start in 0..12 {
        if neighbor_count[start] == 0 || edge_loops[start].is_some() {
            continue;
        }
        edge_loops[start] = Some(count);
        let mut previous = start;
        let mut current = neighbors[start][0];
        while current != start {
            edge_loops[current] = Some(count);
            let next = if neighbors[current][0] == previous {
                neighbors[current][1]
            } else {
                neighbors[current][0]
            };
            previous = current;
            current = next;
        }
        count += 1;
    }
    (edge_loops, count)
}

/// 対称な 3×3 行列の固有値と単位固有ベクトル（ヤコビ法。`i` 番目の固有値の固有ベクトルは `vectors[i]`）
fn symmetric_eigen(mut matrix: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut rotation = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..JACOBI_MAX_SWEEPS {
        if matrix[0][1] == 0.0 && matrix[0][2] == 0.0 && matrix[1][2] == 0.0 {
            break;
        }
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            // 対角の外の成分が、対角の成分に足しても浮動小数点の値を変えないほど小さければ 0 とする
            // （Numerical Recipes のヤコビ法と同じ判定。丸め誤差で 0 にならない成分の回転を繰り返さない）
            let small = 100.0 * matrix[p][q].abs();
            if matrix[p][p].abs() + small == matrix[p][p].abs()
                && matrix[q][q].abs() + small == matrix[q][q].abs()
            {
                matrix[p][q] = 0.0;
                matrix[q][p] = 0.0;
            }
            if matrix[p][q] == 0.0 {
                continue;
            }
            // matrix[p][q] を 0 にする回転（cos c・sin s）
            let theta = (matrix[q][q] - matrix[p][p]) / (2.0 * matrix[p][q]);
            let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
            let c = 1.0 / (t * t + 1.0).sqrt();
            let s = t * c;
            for row in &mut matrix {
                let (a, b) = (row[p], row[q]);
                row[p] = c * a - s * b;
                row[q] = s * a + c * b;
            }
            let (upper, lower) = matrix.split_at_mut(q);
            for (a, b) in upper[p].iter_mut().zip(lower[0].iter_mut()) {
                let (x, y) = (*a, *b);
                *a = c * x - s * y;
                *b = s * x + c * y;
            }
            for row in &mut rotation {
                let (a, b) = (row[p], row[q]);
                row[p] = c * a - s * b;
                row[q] = s * a + c * b;
            }
        }
    }
    let values = [matrix[0][0], matrix[1][1], matrix[2][2]];
    let vectors = std::array::from_fn(|i| [rotation[0][i], rotation[1][i], rotation[2][i]]);
    (values, vectors)
}

/// セルの頂点の内外、交点の輪と、セルが乗る平面
struct CellSurface {
    /// 頂点の内外（`CORNERS` の順）
    inside: [bool; 8],
    /// 辺ごとの、その辺の交点を含む輪の番号（交点のない辺は `None`）
    edge_loops: [Option<u8>; 12],
    /// 輪の数
    loop_count: u8,
    planes: Vec<PlaneKey>,
}

impl CellSurface {
    /// 頂点の内外が `inside` で、平面 `planes` に乗るセル
    fn new(inside: [bool; 8], planes: Vec<PlaneKey>) -> Self {
        let (edge_loops, loop_count) = cell_loops(&inside);
        Self {
            inside,
            edge_loops,
            loop_count,
            planes,
        }
    }
}

/// ブロックの面を作る間に覚えるセル・四角形の平面・頂点を残すかと、組み立て中の面
#[derive(Default)]
struct BlockContext {
    cells: HashMap<[i64; 3], CellSurface>,
    /// 辺（最小の格子点と向き）ごとの、四角形がまとめる平面
    quad_planes: HashMap<([i64; 3], usize), Option<PlaneKey>>,
    /// セルごとの、平らな部分の多角形の頂点を残すか
    kept_vertices: HashMap<[i64; 3], bool>,
    builder: MeshBuilder,
}

/// 1 つの頂点を使う三角形のうち、陰影の頂点を共有する組（面積で重み付けた法線の和と、三角形と角の番号）
struct ShadingGroup {
    normal_sum: [f64; 3],
    corners: Vec<(usize, usize)>,
}

/// セルの輪ごとの頂点を共有して面を組み立て、折れの角度で陰影の頂点を分ける
#[derive(Default)]
struct MeshBuilder {
    positions: Vec<[f64; 3]>,
    /// セルの最小の格子点と輪の番号ごとの頂点の番号
    ids: HashMap<([i64; 3], u8), u32>,
    triangles: Vec<[u32; 3]>,
}

impl MeshBuilder {
    /// セルの輪 `key` の頂点の番号（なければ `position` で加える）
    fn vertex(&mut self, key: ([i64; 3], u8), position: [f64; 3]) -> u32 {
        *self.ids.entry(key).or_insert_with(|| {
            self.positions.push(position);
            (self.positions.len() - 1) as u32
        })
    }

    fn add_triangle(&mut self, triangle: [u32; 3]) {
        self.triangles.push(triangle);
    }

    /// 面にする（面積が 0 の三角形は除く）
    ///
    /// 頂点ごとに、頂点を使う三角形を、向きの違いが `crease_angle` 以内の組に分け、組ごとに別の頂点として面積で
    /// 重み付けた法線を持たせる。
    fn finish(self, crease_angle: f64) -> SurfaceMesh {
        let normals: Vec<[f64; 3]> = self
            .triangles
            .iter()
            .map(|triangle| {
                let [a, b, c] = triangle.map(|index| self.positions[index as usize]);
                cross(sub(b, a), sub(c, a))
            })
            .collect();
        let mut incident: Vec<Vec<(usize, usize)>> = vec![Vec::new(); self.positions.len()];
        for (triangle, corners) in self.triangles.iter().enumerate() {
            if normals[triangle] == [0.0; 3] {
                continue;
            }
            for (corner, &vertex) in corners.iter().enumerate() {
                incident[vertex as usize].push((triangle, corner));
            }
        }
        let cos_crease = crease_angle.cos();
        let mut mesh = SurfaceMesh::default();
        let mut corner_vertices = vec![[0_u32; 3]; self.triangles.len()];
        for (vertex, uses) in incident.iter().enumerate() {
            let mut groups: Vec<ShadingGroup> = Vec::new();
            for &(triangle, corner) in uses {
                let normal = normals[triangle];
                let length = dot(normal, normal).sqrt();
                let group = groups.iter_mut().find(|group| {
                    let sum = group.normal_sum;
                    dot(normal, sum) >= length * dot(sum, sum).sqrt() * cos_crease
                });
                match group {
                    Some(group) => {
                        group.normal_sum =
                            std::array::from_fn(|axis| group.normal_sum[axis] + normal[axis]);
                        group.corners.push((triangle, corner));
                    }
                    None => groups.push(ShadingGroup {
                        normal_sum: normal,
                        corners: vec![(triangle, corner)],
                    }),
                }
            }
            for ShadingGroup {
                normal_sum: sum,
                corners: members,
            } in groups
            {
                let index = mesh.positions.len() as u32;
                let length = dot(sum, sum).sqrt();
                mesh.positions.push(self.positions[vertex]);
                mesh.normals.push(sum.map(|value| value / length));
                for (triangle, corner) in members {
                    corner_vertices[triangle][corner] = index;
                }
            }
        }
        for (triangle, corners) in corner_vertices.iter().enumerate() {
            if normals[triangle] != [0.0; 3] {
                mesh.indices.extend(corners);
            }
        }
        mesh
    }
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
