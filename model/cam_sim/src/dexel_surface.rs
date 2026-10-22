//! トリデクセルの表示用の面（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §4）
//!
//! 3 方向の格子線は同じ格子点を通るため、格子点の間の辺はいずれかの方向のデクセル上にある。格子点の内外を
//! Z の向きのデクセルで判定し、内外が変わる辺の交点をその辺上のデクセルの端点（正本から求めた厳密な点）とする
//! マーチングキューブで面を作る。セルの各面では「内側の頂点どうしは面の対角線をまたいでつながない」と決めて
//! 交点を線分でつなぎ、線分の輪から三角形を作る。隣り合うセルは共有する面で同じ線分を作るため、面に穴は開かない。
//!
//! 格子を素材の外側へ 1 層広げ、そこを空として扱うことで、素材の外周の面も作る。面はセルのブロックごとに作り、
//! スナップショットの変化から作り直すブロックを求める。

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use crate::constants::SURFACE_BLOCK_CELLS;
use crate::simulation_artifact::{CuttingSimulationArtifact, DexelChange, DexelGridLayout};

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
}

impl DexelSurface {
    /// 成果物のスナップショット `snapshot` の時点の面
    ///
    /// # Panics
    ///
    /// `snapshot` がスナップショットの数以上の場合。
    pub fn at_snapshot(artifact: &CuttingSimulationArtifact, snapshot: usize) -> Self {
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
        }
    }

    /// 各座標軸のブロックの数
    pub fn block_counts(&self) -> [usize; 3] {
        // セルは、格子を外側へ 1 層広げた格子点の間（各軸 count + 1 個）
        self.count
            .map(|count| (count + 1).div_ceil(SURFACE_BLOCK_CELLS))
    }

    /// 次のスナップショットへの変化を当て、作り直すブロックを返す
    pub fn apply_changes(&mut self, changes: &[DexelChange]) -> Vec<[usize; 3]> {
        let blocks = self.block_counts();
        let mut dirty = BTreeSet::new();
        for change in changes {
            let grid = change.grid as usize;
            self.intervals[grid][change.index as usize].clone_from(&change.intervals);
            let plane_axes = plane_axes(grid);
            let count = self.grids[grid].count[1] as usize;
            let line = [change.index as usize / count, change.index as usize % count];
            // 線上の格子点・辺に接するセル（外側へ広げたセルの番号は格子点の番号 + 1 と + 0）
            let mut ranges = [0..0, 0..0, 0..0];
            ranges[grid] = 0..blocks[grid];
            for (index, &axis) in plane_axes.iter().enumerate() {
                let first = line[index] / SURFACE_BLOCK_CELLS;
                let last = (line[index] + 1) / SURFACE_BLOCK_CELLS;
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

    /// ブロック `block` の面
    ///
    /// セルの頂点の内外は Z の向きのデクセルだけで決まるため、Z の向きのセルの列ごとに、列の 4 本の Z の線の
    /// 内外の区間から頂点の内外がそろわないセルを求め、そのセルだけを処理する（時間は面の大きさに比例する）。
    pub fn mesh_block(&self, block: [usize; 3]) -> SurfaceMesh {
        let mut builder =
            MeshBuilder::new(block.map(|index| (index * SURFACE_BLOCK_CELLS) as i64 - 1));
        let [x_cells, y_cells, z_cells] = [0, 1, 2].map(|axis| self.block_cells(block, axis));
        for x in x_cells {
            for y in y_cells.clone() {
                // 外側へ広げたセルの番号から、セルの最小の格子点の番号（-1 から）
                let (i, j) = (x as i64 - 1, y as i64 - 1);
                for k in self.mixed_cells_in_column(i, j, z_cells.clone()) {
                    self.mesh_cell([i, j, k], &mut builder);
                }
            }
        }
        builder.finish()
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
        let mut builder =
            MeshBuilder::new(block.map(|index| (index * SURFACE_BLOCK_CELLS) as i64 - 1));
        let [x_cells, y_cells, z_cells] = [0, 1, 2].map(|axis| self.block_cells(block, axis));
        for x in x_cells {
            for y in y_cells.clone() {
                for z in z_cells.clone() {
                    let lower = [x as i64 - 1, y as i64 - 1, z as i64 - 1];
                    self.mesh_cell(lower, &mut builder);
                }
            }
        }
        builder.finish()
    }

    /// 最小の格子点が `lower` のセルの面を加える
    fn mesh_cell(&self, lower: [i64; 3], builder: &mut MeshBuilder) {
        let corner_point = |corner: usize| {
            let offset = CORNERS[corner];
            [0, 1, 2].map(|axis| lower[axis] + offset[axis] as i64)
        };
        let inside: [bool; 8] = std::array::from_fn(|corner| self.is_inside(corner_point(corner)));
        if inside.iter().all(|&flag| flag) || inside.iter().all(|&flag| !flag) {
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

/// デクセルの向き `axis` の格子の 2 方向（`axis` 以外の座標軸の昇順）
fn plane_axes(axis: usize) -> [usize; 2] {
    match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    }
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
}

impl MeshBuilder {
    /// 最小の格子点が `origin` のブロックの面を組み立てる
    fn new(origin: [i64; 3]) -> Self {
        let side = SURFACE_BLOCK_CELLS + 1;
        Self {
            mesh: SurfaceMesh::default(),
            origin,
            vertices: vec![u32::MAX; side * side * side * 3],
        }
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
            let slot = self.slot(key);
            if self.vertices[slot] == u32::MAX {
                self.mesh.positions.push(*point);
                self.mesh.normals.push([0.0; 3]);
                self.vertices[slot] = (self.mesh.positions.len() - 1) as u32;
            }
            indices[index] = self.vertices[slot];
        }
        for i in 1..keys.len().saturating_sub(1) {
            let triangle = [indices[0], indices[i], indices[i + 1]];
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
