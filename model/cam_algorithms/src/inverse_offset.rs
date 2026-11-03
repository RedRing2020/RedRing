//! 逆オフセット法の工具の形と、製品形状の三角形メッシュの準備（面積のない三角形を除く・XY の升目の索引）
//!
//! 逆オフセット形状は `inverse_offset_shape`、製品形状への問い合わせは `product_surface` が持つ。

use geo_algorithms::TriangleMesh3D;
use geo_contracts::{
    ToleranceSettings, TriangleMesh3DProperties, default_kernel_numerical_zero_tolerance,
};

use crate::solver::CamSolverError;

/// 逆オフセットに用いる工具形状
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CutterShape {
    /// ボールエンドミル
    Ball { radius: f64 },
    /// フラットエンドミル
    Flat { radius: f64 },
}

impl CutterShape {
    pub fn radius(self) -> f64 {
        match self {
            Self::Ball { radius } | Self::Flat { radius } => radius,
        }
    }

    /// 工具先端から逆オフセットの基準点までの高さ
    ///
    /// ボールは球中心（先端 + r）、フラットは底面中心（先端と同じ）を基準点とする。
    pub fn reference_offset(self) -> f64 {
        match self {
            Self::Ball { radius } => radius,
            Self::Flat { .. } => 0.0,
        }
    }
}

/// メッシュの、面積のある三角形と、その XY 範囲
pub(crate) struct MeshTriangles {
    pub(crate) vertices: Vec<[f64; 3]>,
    pub(crate) triangles: Vec<[usize; 3]>,
    pub(crate) xy_min: [f64; 2],
    pub(crate) xy_max: [f64; 2],
}

impl MeshTriangles {
    /// 面積のない三角形を除く（面積のある三角形がなければ `no_solution`）
    pub(crate) fn new(mesh: &TriangleMesh3D<f64>) -> Result<Self, CamSolverError> {
        let vertices: Vec<[f64; 3]> = mesh
            .vertices()
            .iter()
            .map(|p| [p.x(), p.y(), p.z()])
            .collect();
        let triangles: Vec<[usize; 3]> = mesh
            .indices()
            .iter()
            .copied()
            .filter(|tri| !is_degenerate(&vertices, tri))
            .collect();

        if triangles.is_empty() {
            return Err(CamSolverError::NoSolution(
                "geometry has no non-degenerate triangles".to_string(),
            ));
        }

        let mut xy_min = [f64::INFINITY; 2];
        let mut xy_max = [f64::NEG_INFINITY; 2];
        for tri in &triangles {
            for &vi in tri {
                let v = vertices[vi];
                xy_min = [xy_min[0].min(v[0]), xy_min[1].min(v[1])];
                xy_max = [xy_max[0].max(v[0]), xy_max[1].max(v[1])];
            }
        }
        Ok(Self {
            vertices,
            triangles,
            xy_min,
            xy_max,
        })
    }
}

pub(crate) fn contains_xy(a: [f64; 3], b: [f64; 3], c: [f64; 3], x: f64, y: f64) -> bool {
    let det = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
    let zero = default_kernel_numerical_zero_tolerance::<f64>();
    if det.abs() < zero {
        return false;
    }
    let l1 = ((b[1] - c[1]) * (x - c[0]) + (c[0] - b[0]) * (y - c[1])) / det;
    let l2 = ((c[1] - a[1]) * (x - c[0]) + (a[0] - c[0]) * (y - c[1])) / det;
    let l3 = 1.0 - l1 - l2;
    l1 >= -zero && l2 >= -zero && l3 >= -zero
}

fn is_degenerate(vertices: &[[f64; 3]], tri: &[usize; 3]) -> bool {
    let [a, b, c] = tri.map(|vi| vertices[vi]);
    let n = cross(sub(b, a), sub(c, a));
    let area = 0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    area <= ToleranceSettings::<f64>::standard().area_tolerance
}

pub(crate) fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// 三角形の XY 範囲（工具半径で拡張）を登録する一様バケット索引
#[derive(Debug, Clone)]
pub(crate) struct XyBucketIndex {
    origin: [f64; 2],
    cell_size: f64,
    columns: usize,
    rows: usize,
    buckets: Vec<Vec<usize>>,
}

impl XyBucketIndex {
    const MAX_CELLS_PER_AXIS: usize = 256;

    /// 三角形の XY 範囲を `radius` だけ広げて登録する
    pub(crate) fn build(
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
        radius: f64,
        xy_min: [f64; 2],
        xy_max: [f64; 2],
    ) -> Self {
        let bounds: Vec<([f64; 2], [f64; 2])> = triangles
            .iter()
            .map(|tri| {
                let [a, b, c] = tri.map(|vi| vertices[vi]);
                (
                    [
                        a[0].min(b[0]).min(c[0]) - radius,
                        a[1].min(b[1]).min(c[1]) - radius,
                    ],
                    [
                        a[0].max(b[0]).max(c[0]) + radius,
                        a[1].max(b[1]).max(c[1]) + radius,
                    ],
                )
            })
            .collect();
        Self::from_bounds(&bounds, radius, xy_min, xy_max)
    }

    /// XY 範囲 `bounds`（min, max）の列を登録する。升目の幅は `radius` 以上とし、索引の範囲は形状の XY 範囲を
    /// `radius` だけ広げた範囲とする
    pub(crate) fn from_bounds(
        bounds: &[([f64; 2], [f64; 2])],
        radius: f64,
        xy_min: [f64; 2],
        xy_max: [f64; 2],
    ) -> Self {
        let origin = [xy_min[0] - radius, xy_min[1] - radius];
        let span_x = xy_max[0] - xy_min[0] + 2.0 * radius;
        let span_y = xy_max[1] - xy_min[1] + 2.0 * radius;
        let cell_size = (span_x.max(span_y) / Self::MAX_CELLS_PER_AXIS as f64).max(radius);
        let columns = ((span_x / cell_size).ceil() as usize).max(1);
        let rows = ((span_y / cell_size).ceil() as usize).max(1);
        let mut index = Self {
            origin,
            cell_size,
            columns,
            rows,
            buckets: vec![Vec::new(); columns * rows],
        };

        for (item, (min, max)) in bounds.iter().enumerate() {
            let (c0, r0) = index.cell_of(min[0], min[1]);
            let (c1, r1) = index.cell_of(max[0], max[1]);
            for row in r0..=r1 {
                for column in c0..=c1 {
                    index.buckets[row * columns + column].push(item);
                }
            }
        }

        index
    }

    fn cell_of(&self, x: f64, y: f64) -> (usize, usize) {
        let column = ((x - self.origin[0]) / self.cell_size).floor();
        let row = ((y - self.origin[1]) / self.cell_size).floor();
        (
            (column.max(0.0) as usize).min(self.columns - 1),
            (row.max(0.0) as usize).min(self.rows - 1),
        )
    }

    pub(crate) fn candidates(&self, x: f64, y: f64) -> &[usize] {
        let (column, row) = self.cell_of(x, y);
        &self.buckets[row * self.columns + column]
    }

    /// XY 範囲 `min`〜`max` にかかる升目の候補（重複を除き、番号の小さい順）
    pub(crate) fn candidates_within(&self, min: [f64; 2], max: [f64; 2]) -> Vec<usize> {
        let (c0, r0) = self.cell_of(min[0], min[1]);
        let (c1, r1) = self.cell_of(max[0], max[1]);
        let mut items: Vec<usize> = (r0..=r1)
            .flat_map(|row| (c0..=c1).map(move |column| row * self.columns + column))
            .flat_map(|cell| self.buckets[cell].iter().copied())
            .collect();
        items.sort_unstable();
        items.dedup();
        items
    }

    /// (x, y) を含む升目（索引の外も含めて数える）から、升目の番号の差が `ring` の升目を、内側から外側へ順に返す
    ///
    /// 環ごとに、索引の中の升目の中身と、環の升目と (x, y) の XY の距離の下限（`(ring - 1)` × 升目の幅）を返す。
    /// 索引のすべての升目を返した後は終わる。
    pub(crate) fn rings(&self, x: f64, y: f64) -> impl Iterator<Item = (f64, Vec<&[usize]>)> {
        let column = ((x - self.origin[0]) / self.cell_size).floor() as i64;
        let row = ((y - self.origin[1]) / self.cell_size).floor() as i64;
        let (columns, rows) = (self.columns as i64, self.rows as i64);
        // 索引のいちばん遠い升目までの、升目の番号の差
        let last = [column, columns - 1 - column, row, rows - 1 - row]
            .map(i64::abs)
            .into_iter()
            .max()
            .unwrap_or(0);
        (0..=last).map(move |ring| {
            let bucket = |c: i64, r: i64| {
                ((0..columns).contains(&c) && (0..rows).contains(&r))
                    .then(|| self.buckets[(r * columns + c) as usize].as_slice())
            };
            let cells: Vec<&[usize]> = if ring == 0 {
                bucket(column, row).into_iter().collect()
            } else {
                // 環の上下の辺（角を含む）と左右の辺（角を除く）
                let horizontal = (column - ring..=column + ring)
                    .flat_map(|c| [(c, row - ring), (c, row + ring)]);
                let vertical = (row - ring + 1..row + ring)
                    .flat_map(|r| [(column - ring, r), (column + ring, r)]);
                horizontal
                    .chain(vertical)
                    .filter_map(|(c, r)| bucket(c, r))
                    .collect()
            };
            ((ring - 1).max(0) as f64 * self.cell_size, cells)
        })
    }
}
