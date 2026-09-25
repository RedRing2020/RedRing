//! 逆オフセット包絡面（CL 面）の格子サンプリング
//!
//! drop-cutter を等間隔格子で評価し、工具先端高さの格子を返す。
//! 経路生成とは独立した検査・可視化用途の入口であり、ToolPath は生成しない。

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
    };
    for row in 0..rows {
        for column in 0..columns {
            let [x, y] = grid.xy(column, row);
            grid.tip_heights.push(cutter.tip_height_at(x, y));
        }
    }
    Ok(grid)
}
