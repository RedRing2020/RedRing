//! 逆オフセット包絡面（CL 面）の格子サンプリング
//!
//! drop-cutter を等間隔格子で評価し、工具先端高さの格子を返す。
//! 経路生成とは独立した検査・可視化用途の入口であり、ToolPath は生成しない。
//!
//! 接触あり/なしが切り替わる格子辺では、二分法で接触境界の位置を求める。
//! 包絡面の境界（形状の角で半径 r の円弧になる等）を格子間隔より細かく表すために用いる。

use geo_contracts::default_distance_tolerance;

use crate::inverse_offset::DropCutter;
use crate::solver::CamSolverError;

/// 1 格子あたりの最大サンプル数（検査用途での過大な評価を防ぐ上限）
pub const MAX_CL_GRID_SAMPLES: usize = 4_000_000;

/// 工具先端高さの等間隔格子（行 = Y、列 = X の行優先）
#[derive(Debug, Clone, PartialEq)]
pub struct ClGrid {
    /// 格子原点（最小 X, 最小 Y）
    pub origin: [f64; 2],
    /// 格子間隔
    pub pitch: f64,
    /// X 方向サンプル数
    pub columns: usize,
    /// Y 方向サンプル数
    pub rows: usize,
    /// 各格子点の工具先端高さ。工具が形状に接触しない点は `None`
    pub tip_heights: Vec<Option<f64>>,
    /// X 方向の格子辺 (column, row)-(column + 1, row) 上の接触境界点
    /// （`(columns - 1) * rows` 件、行優先）。接触が切り替わらない辺は `None`
    pub x_edge_boundaries: Vec<Option<[f64; 3]>>,
    /// Y 方向の格子辺 (column, row)-(column, row + 1) 上の接触境界点
    /// （`columns * (rows - 1)` 件、行優先）。接触が切り替わらない辺は `None`
    pub y_edge_boundaries: Vec<Option<[f64; 3]>>,
}

impl ClGrid {
    /// 格子点 (column, row) の XY 座標
    pub fn xy(&self, column: usize, row: usize) -> [f64; 2] {
        [
            self.origin[0] + self.pitch * column as f64,
            self.origin[1] + self.pitch * row as f64,
        ]
    }

    /// 格子点 (column, row) の工具先端高さ
    pub fn tip_height(&self, column: usize, row: usize) -> Option<f64> {
        self.tip_heights[row * self.columns + column]
    }

    /// X 方向格子辺 (column, row)-(column + 1, row) 上の接触境界点（工具先端座標）
    pub fn x_edge_boundary(&self, column: usize, row: usize) -> Option<[f64; 3]> {
        self.x_edge_boundaries[row * (self.columns - 1) + column]
    }

    /// Y 方向格子辺 (column, row)-(column, row + 1) 上の接触境界点（工具先端座標）
    pub fn y_edge_boundary(&self, column: usize, row: usize) -> Option<[f64; 3]> {
        self.y_edge_boundaries[row * self.columns + column]
    }
}

/// 形状の XY 範囲を `margin` だけ広げた領域を `pitch` 間隔で評価する。
///
/// `margin` に工具半径を与えると、形状外周で工具が接触する範囲まで包絡面を含められる。
pub fn sample_cl_grid(
    cutter: &DropCutter,
    pitch: f64,
    margin: f64,
) -> Result<ClGrid, CamSolverError> {
    if !(pitch.is_finite() && pitch > 0.0) {
        return Err(CamSolverError::InvalidInput(
            "cl grid pitch must be positive and finite".to_string(),
        ));
    }
    if !(margin.is_finite() && margin >= 0.0) {
        return Err(CamSolverError::InvalidInput(
            "cl grid margin must be non-negative and finite".to_string(),
        ));
    }

    let (xy_min, xy_max) = cutter.xy_bounds();
    let origin = [xy_min[0] - margin, xy_min[1] - margin];
    let columns = ((xy_max[0] + margin - origin[0]) / pitch).floor() as usize + 1;
    let rows = ((xy_max[1] + margin - origin[1]) / pitch).floor() as usize + 1;
    if columns.saturating_mul(rows) > MAX_CL_GRID_SAMPLES {
        return Err(CamSolverError::InvalidInput(format!(
            "cl grid has {columns}x{rows} samples, exceeding {MAX_CL_GRID_SAMPLES}"
        )));
    }

    let mut grid = ClGrid {
        origin,
        pitch,
        columns,
        rows,
        tip_heights: Vec::with_capacity(columns * rows),
        x_edge_boundaries: Vec::new(),
        y_edge_boundaries: Vec::new(),
    };
    for row in 0..rows {
        for column in 0..columns {
            let [x, y] = grid.xy(column, row);
            grid.tip_heights.push(cutter.tip_height_at(x, y));
        }
    }

    for row in 0..rows {
        for column in 0..columns.saturating_sub(1) {
            let boundary = edge_boundary(
                cutter,
                (grid.xy(column, row), grid.tip_height(column, row)),
                (grid.xy(column + 1, row), grid.tip_height(column + 1, row)),
            );
            grid.x_edge_boundaries.push(boundary);
        }
    }
    for row in 0..rows.saturating_sub(1) {
        for column in 0..columns {
            let boundary = edge_boundary(
                cutter,
                (grid.xy(column, row), grid.tip_height(column, row)),
                (grid.xy(column, row + 1), grid.tip_height(column, row + 1)),
            );
            grid.y_edge_boundaries.push(boundary);
        }
    }
    Ok(grid)
}

/// 接触あり/なしが切り替わる格子辺で、接触側に残る境界点を二分法で求める。
///
/// 区間長が既定の距離トレランス以下になるまで二分する。
fn edge_boundary(
    cutter: &DropCutter,
    (a, a_height): ([f64; 2], Option<f64>),
    (b, b_height): ([f64; 2], Option<f64>),
) -> Option<[f64; 3]> {
    let (mut inside, mut inside_height, mut outside) = match (a_height, b_height) {
        (Some(z), None) => (a, z, b),
        (None, Some(z)) => (b, z, a),
        _ => return None,
    };

    let tolerance = default_distance_tolerance::<f64>();
    while ((inside[0] - outside[0]).powi(2) + (inside[1] - outside[1]).powi(2)).sqrt() > tolerance {
        let middle = [
            0.5 * (inside[0] + outside[0]),
            0.5 * (inside[1] + outside[1]),
        ];
        match cutter.tip_height_at(middle[0], middle[1]) {
            Some(z) => {
                inside = middle;
                inside_height = z;
            }
            None => outside = middle,
        }
    }
    Some([inside[0], inside[1], inside_height])
}
