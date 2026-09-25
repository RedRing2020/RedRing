//! 逆オフセット法による工具位置（CL）算出
//!
//! 三角形メッシュの各要素（頂点・辺・面）を工具形状で逆オフセットし、
//! 鉛直線 (x, y) との交点の上側包絡から工具先端高さ（ToolPath 座標）を求める。
//!
//! ボールエンドミル（半径 r、工具中心 = 先端 + r）:
//!
//! - 頂点 → 半径 r の球
//! - 辺 → 半径 r の円筒（辺に沿った円弧掃引面）
//! - 面 → 法線方向に r オフセットした平面（接触点が三角形内に入る範囲）
//!
//! フラットエンドミル（半径 r、底面 = 先端）:
//!
//! - 頂点 → 半径 r の円板（頂点の高さで水平）
//! - 辺 → 円板の辺沿い掃引（水平距離 r 以内の辺上の最高点）
//! - 面 → 底面円周上の接触（面の最大傾斜方向へ r 進んだ点が三角形内に入る範囲）
//!
//! 凹辺のオフセットは他要素の包絡に覆われるため、全辺を評価しても結果は変わらない。
//! ラジアスエンドミル（トーラス）の要素オフセットは未対応（辺で数値解法を要するため後続で扱う）。

use geo_algorithms::{Point3D, TriangleMesh3D};
use geo_contracts::{
    ToleranceSettings, default_kernel_numerical_zero_tolerance,
    default_orthogonality_dot_error_tolerance, default_parallel_cross_error_tolerance,
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

/// 逆オフセット法による drop-cutter
#[derive(Debug, Clone)]
pub struct DropCutter {
    shape: CutterShape,
    vertices: Vec<[f64; 3]>,
    triangles: Vec<[usize; 3]>,
    index: XyBucketIndex,
    xy_min: [f64; 2],
    xy_max: [f64; 2],
}

impl DropCutter {
    /// メッシュと工具形状から drop-cutter を構築する。
    pub fn new(mesh: &TriangleMesh3D<f64>, shape: CutterShape) -> Result<Self, CamSolverError> {
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

        let index = XyBucketIndex::build(&vertices, &triangles, shape.radius(), xy_min, xy_max);

        Ok(Self {
            shape,
            vertices,
            triangles,
            index,
            xy_min,
            xy_max,
        })
    }

    pub fn shape(&self) -> CutterShape {
        self.shape
    }

    /// 形状の XY 範囲（min, max）
    pub fn xy_bounds(&self) -> ([f64; 2], [f64; 2]) {
        (self.xy_min, self.xy_max)
    }

    /// (x, y) における工具先端高さ。工具が形状に接触しない場合は `None`。
    pub fn tip_height_at(&self, x: f64, y: f64) -> Option<f64> {
        let mut best: Option<f64> = None;
        let mut update = |z: Option<f64>| {
            if let Some(z) = z {
                best = Some(best.map_or(z, |current: f64| current.max(z)));
            }
        };

        for &tri_index in self.index.candidates(x, y) {
            let [a, b, c] = self.triangles[tri_index].map(|vi| self.vertices[vi]);
            match self.shape {
                CutterShape::Ball { radius: r } => {
                    let to_tip = |center: Option<f64>| center.map(|z| z - r);
                    update(to_tip(ball_face_contact(a, b, c, x, y, r)));
                    for p in [a, b, c] {
                        update(to_tip(ball_vertex_contact(p, x, y, r)));
                    }
                    for (p, q) in [(a, b), (b, c), (c, a)] {
                        update(to_tip(ball_edge_contact(p, q, x, y, r)));
                    }
                }
                CutterShape::Flat { radius: r } => {
                    update(flat_face_contact(a, b, c, x, y, r));
                    for p in [a, b, c] {
                        update(flat_vertex_contact(p, x, y, r));
                    }
                    for (p, q) in [(a, b), (b, c), (c, a)] {
                        update(flat_edge_contact(p, q, x, y, r));
                    }
                }
            }
        }

        best
    }

    /// (x, y) における工具先端位置（ToolPath 座標）
    pub fn tip_point_at(&self, x: f64, y: f64) -> Option<Point3D<f64>> {
        self.tip_height_at(x, y).map(|z| Point3D::new(x, y, z))
    }
}

/// 頂点 → 球
fn ball_vertex_contact(p: [f64; 3], x: f64, y: f64, r: f64) -> Option<f64> {
    let dx = x - p[0];
    let dy = y - p[1];
    let d2 = dx * dx + dy * dy;
    let r2 = r * r;
    (d2 <= r2).then(|| p[2] + (r2 - d2).sqrt())
}

/// 辺 → 円筒（接触点が線分内にある場合のみ）
fn ball_edge_contact(p: [f64; 3], q: [f64; 3], x: f64, y: f64, r: f64) -> Option<f64> {
    let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
    let horizontal = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let length = (horizontal * horizontal + d[2] * d[2]).sqrt();
    // |単位辺ベクトル × Z| = horizontal / length が 0 とみなせる辺は鉛直辺。
    // 鉛直辺は端点の球で代表される
    if horizontal < length * default_parallel_cross_error_tolerance::<f64>() {
        return None;
    }
    let u = [d[0] / horizontal, d[1] / horizontal];

    // 辺の水平方向 u に沿った座標 s0 と、水平面内で u に直交する距離 b に分解する。
    // 工具中心 C と辺直線の距離² = b² + (辺を含む鉛直面内の距離)² = r² から
    //   z_c = p.z + m s0 + sqrt((r² - b²)(1 + m²))   (m = 辺の勾配 dz / horizontal)
    // 二次方程式の判別式による解法は微小半径で桁落ちするため、この分解形を用いる。
    let wx = x - p[0];
    let wy = y - p[1];
    let s0 = wx * u[0] + wy * u[1];
    let b = wx * u[1] - wy * u[0];
    let remaining = r * r - b * b;
    if remaining < 0.0 {
        return None;
    }
    let m = d[2] / horizontal;
    let rise = (remaining * (1.0 + m * m)).sqrt();
    let center_z = p[2] + m * s0 + rise;

    // 鉛直面内で C から辺直線へ下ろした垂線の足が線分内にあること
    let foot = (s0 + m * (center_z - p[2])) / (1.0 + m * m);
    (0.0..=horizontal).contains(&foot).then_some(center_z)
}

/// 面 → 工具半径オフセット平面（接触点が三角形内にある場合のみ）
fn ball_face_contact(a: [f64; 3], b: [f64; 3], c: [f64; 3], x: f64, y: f64, r: f64) -> Option<f64> {
    let n = upward_unit_normal(a, b, c)?;

    // 接触点 Q = C - r n が三角形内にあるかを XY 投影で判定する
    let qx = x - r * n[0];
    let qy = y - r * n[1];
    if !contains_xy(a, b, c, qx, qy) {
        return None;
    }

    let plane_d = n[0] * a[0] + n[1] * a[1] + n[2] * a[2];
    Some((plane_d + r - n[0] * x - n[1] * y) / n[2])
}

/// 頂点 → 円板（水平距離 r 以内なら頂点高さで接触）
fn flat_vertex_contact(p: [f64; 3], x: f64, y: f64, r: f64) -> Option<f64> {
    let dx = x - p[0];
    let dy = y - p[1];
    (dx * dx + dy * dy <= r * r).then_some(p[2])
}

/// 辺 → 円板の掃引（水平距離 r 以内にある辺上の最高点で接触）
fn flat_edge_contact(p: [f64; 3], q: [f64; 3], x: f64, y: f64, r: f64) -> Option<f64> {
    let d = sub(q, p);
    let horizontal = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let length = (horizontal * horizontal + d[2] * d[2]).sqrt();
    if horizontal < length * default_parallel_cross_error_tolerance::<f64>() {
        // 鉛直辺は端点の円板で代表される
        return None;
    }
    let u = [d[0] / horizontal, d[1] / horizontal];

    // 辺の水平方向座標 s0 と直交距離 b に分解し、円板内に入る区間 [lo, hi] を求める
    let wx = x - p[0];
    let wy = y - p[1];
    let s0 = wx * u[0] + wy * u[1];
    let b = wx * u[1] - wy * u[0];
    let remaining = r * r - b * b;
    if remaining < 0.0 {
        return None;
    }
    let half = remaining.sqrt();
    let lo = (s0 - half).max(0.0);
    let hi = (s0 + half).min(horizontal);
    if lo > hi {
        return None;
    }

    // 辺上の高さは s に線形なので、区間端のうち高い側が接触点
    let m = d[2] / horizontal;
    Some(p[2] + m * if m >= 0.0 { hi } else { lo })
}

/// 面 → 底面円周上の接触
///
/// 円板と面の交わりで最も高い点は、円板中心から面の最大傾斜（上り）方向へ r 進んだ円周上にある。
/// その点が三角形外なら最高点は三角形の辺上にあり、辺・頂点のオフセットで代表される。
/// 水平面は円板中心が三角形内にある場合のみ扱う（それ以外は辺で代表される）。
fn flat_face_contact(a: [f64; 3], b: [f64; 3], c: [f64; 3], x: f64, y: f64, r: f64) -> Option<f64> {
    let n = upward_unit_normal(a, b, c)?;
    let slope = (n[0] * n[0] + n[1] * n[1]).sqrt();

    // |単位法線 × Z| = slope が 0 とみなせる面は水平面
    let (qx, qy) = if slope < default_parallel_cross_error_tolerance::<f64>() {
        (x, y)
    } else {
        (x - r * n[0] / slope, y - r * n[1] / slope)
    };
    if !contains_xy(a, b, c, qx, qy) {
        return None;
    }

    let plane_d = n[0] * a[0] + n[1] * a[1] + n[2] * a[2];
    Some((plane_d - n[0] * qx - n[1] * qy) / n[2])
}

/// 上向き（z >= 0 側）の単位法線。鉛直面は `None`。
fn upward_unit_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<[f64; 3]> {
    let n = cross(sub(b, a), sub(c, a));
    let norm = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    // 工具は上方から接近するため上向き法線側で評価する
    let sign = if n[2] < 0.0 { -1.0 } else { 1.0 };
    let n = [sign * n[0] / norm, sign * n[1] / norm, sign * n[2] / norm];
    // 単位法線と Z の内積が 0 とみなせる鉛直面は辺・頂点のオフセットで代表される
    (n[2] >= default_orthogonality_dot_error_tolerance::<f64>()).then_some(n)
}

fn contains_xy(a: [f64; 3], b: [f64; 3], c: [f64; 3], x: f64, y: f64) -> bool {
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

/// 三角形の XY 範囲（工具半径で拡張）を登録する一様バケット索引
#[derive(Debug, Clone)]
struct XyBucketIndex {
    origin: [f64; 2],
    cell_size: f64,
    columns: usize,
    rows: usize,
    buckets: Vec<Vec<usize>>,
}

impl XyBucketIndex {
    const MAX_CELLS_PER_AXIS: usize = 256;

    fn build(
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
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

        for (tri_index, tri) in triangles.iter().enumerate() {
            let [a, b, c] = tri.map(|vi| vertices[vi]);
            let min_x = a[0].min(b[0]).min(c[0]) - radius;
            let max_x = a[0].max(b[0]).max(c[0]) + radius;
            let min_y = a[1].min(b[1]).min(c[1]) - radius;
            let max_y = a[1].max(b[1]).max(c[1]) + radius;
            let (c0, r0) = index.cell_of(min_x, min_y);
            let (c1, r1) = index.cell_of(max_x, max_y);
            for row in r0..=r1 {
                for column in c0..=c1 {
                    index.buckets[row * columns + column].push(tri_index);
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

    fn candidates(&self, x: f64, y: f64) -> &[usize] {
        let (column, row) = self.cell_of(x, y);
        &self.buckets[row * self.columns + column]
    }
}
