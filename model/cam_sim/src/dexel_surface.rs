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

use std::collections::{BTreeSet, HashMap};

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
    pub fn mesh_block(&self, block: [usize; 3]) -> SurfaceMesh {
        let mut builder = MeshBuilder::default();
        let cell_range = |axis: usize| {
            let first = block[axis] * SURFACE_BLOCK_CELLS;
            first..(first + SURFACE_BLOCK_CELLS).min(self.count[axis] + 1)
        };
        for x in cell_range(0) {
            for y in cell_range(1) {
                for z in cell_range(2) {
                    // 外側へ広げたセルの番号から、セルの最小の格子点の番号（-1 から）
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

        // 面ごとの線分（両端は交点を持つ辺。辺は 2 つの頂点の組で表す）
        let mut neighbors: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();
        let edge = |a: usize, b: usize| (a.min(b), a.max(b));
        for face in FACES {
            let crossing: Vec<(usize, usize)> = (0..4)
                .map(|i| edge(face[i], face[(i + 1) % 4]))
                .filter(|&(a, b)| inside[a] != inside[b])
                .collect();
            let segments: Vec<((usize, usize), (usize, usize))> = match crossing.len() {
                2 => vec![(crossing[0], crossing[1])],
                4 => (0..4)
                    .filter(|&i| inside[face[i]])
                    .map(|i| {
                        let corner = face[i];
                        (
                            edge(face[(i + 3) % 4], corner),
                            edge(corner, face[(i + 1) % 4]),
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            };
            for (a, b) in segments {
                neighbors.entry(a).or_default().push(b);
                neighbors.entry(b).or_default().push(a);
            }
        }

        // 線分をつないで輪にし、外向きにそろえて三角形にする
        let mut visited = BTreeSet::new();
        let mut starts: Vec<(usize, usize)> = neighbors.keys().copied().collect();
        starts.sort_unstable();
        for start in starts {
            if visited.contains(&start) {
                continue;
            }
            let mut loop_edges = vec![start];
            visited.insert(start);
            let mut previous = start;
            let mut current = neighbors[&start][0];
            while current != start {
                visited.insert(current);
                loop_edges.push(current);
                let next = neighbors[&current]
                    .iter()
                    .copied()
                    .find(|&candidate| candidate != previous)
                    .unwrap_or(previous);
                previous = current;
                current = next;
            }
            let points: Vec<[f64; 3]> = loop_edges
                .iter()
                .map(|&(a, b)| self.crossing(corner_point(a), corner_point(b)))
                .collect();
            // 外向き: 内側の頂点から外側の頂点へ向かう辺の向きの和
            let mut outward = [0.0; 3];
            for &(a, b) in &loop_edges {
                let (from, to) = if inside[a] { (a, b) } else { (b, a) };
                for axis in 0..3 {
                    outward[axis] += CORNERS[to][axis] as f64 - CORNERS[from][axis] as f64;
                }
            }
            let mut keys: Vec<EdgeKey> = loop_edges
                .iter()
                .map(|&(a, b)| EdgeKey::new(corner_point(a), corner_point(b)))
                .collect();
            let mut points = points;
            if dot(polygon_normal(&points), outward) < 0.0 {
                keys.reverse();
                points.reverse();
            }
            builder.add_polygon(&keys, &points);
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
#[derive(Default)]
struct MeshBuilder {
    mesh: SurfaceMesh,
    vertices: HashMap<EdgeKey, u32>,
}

impl MeshBuilder {
    /// 外向きにそろえた多角形を、最初の頂点からの扇形の三角形で加える
    fn add_polygon(&mut self, keys: &[EdgeKey], points: &[[f64; 3]]) {
        let indices: Vec<u32> = keys
            .iter()
            .zip(points)
            .map(|(key, point)| {
                *self.vertices.entry(*key).or_insert_with(|| {
                    self.mesh.positions.push(*point);
                    self.mesh.normals.push([0.0; 3]);
                    (self.mesh.positions.len() - 1) as u32
                })
            })
            .collect();
        for i in 1..indices.len().saturating_sub(1) {
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
