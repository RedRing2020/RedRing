//! スキャン加工（`operation_type = scanline`）の経路化
//!
//! 加工範囲（既定は形状の XY 範囲）を Y 方向へ `stepover` 以下の等間隔で走査ラインに分け、
//! 各ライン上を送り方（`ScanlineCutDirection`）で決めた向きに `sample_pitch` 以下の間隔で CL を求める。
//! 工具が形状に接触する連続区間を 1 パスとする。パスの間のエアカットは入れない（CL の編集と工程をそろえた後に
//! 計算する）。

use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, ToolPath};
use geo_algorithms::Point3D;

use crate::inverse_offset::DropCutter;
use crate::solver::{CamSolverError, ScanlineCutDirection, ScanlineParams};

/// drop-cutter から、XY 範囲 `(min, max)` のスキャン加工 ToolPath を生成する。
///
/// 各パスは `ContourLevelPath` 1 件に対応し、`z_level` にはパス内の最高 CL 高さを格納する。
pub fn generate_scanline_toolpath(
    cutter: &DropCutter,
    tool_id: &str,
    params: &ScanlineParams,
    (xy_min, xy_max): ([f64; 2], [f64; 2]),
) -> Result<ToolPath<f64>, CamSolverError> {
    let ys = uniform_samples(xy_min[1], xy_max[1], params.stepover);
    let xs = uniform_samples(xy_min[0], xy_max[0], params.sample_pitch);
    let xs_reversed: Vec<f64> = xs.iter().rev().copied().collect();

    let mut passes: Vec<Vec<Point3D<f64>>> = Vec::new();
    for (line, &y) in ys.iter().enumerate() {
        let forward = match params.cut_direction {
            ScanlineCutDirection::DownCut => false,
            ScanlineCutDirection::UpCut => true,
            ScanlineCutDirection::ZigZag => line % 2 == 1,
        };
        let mut run: Vec<Point3D<f64>> = Vec::new();
        // 送る向きに並べた X
        let xs_in_feed_order = if forward { &xs } else { &xs_reversed };
        for &x in xs_in_feed_order {
            match cutter.tip_point_at(x, y) {
                Some(point) => run.push(point),
                None => flush_run(&mut run, &mut passes),
            }
        }
        flush_run(&mut run, &mut passes);
    }

    if passes.is_empty() {
        return Err(CamSolverError::NoSolution(
            "tool does not contact geometry on any scanline".to_string(),
        ));
    }

    let feed_rate = params.feed_rate;
    let levels = passes
        .iter()
        .enumerate()
        .map(|(level_index, pass)| {
            let segments = pass
                .windows(2)
                .map(|window| {
                    PathSegment::new_line(window[0], window[1], SegmentType::Cutting { feed_rate })
                })
                .collect();
            let z_level = pass.iter().map(|p| p.z()).fold(f64::NEG_INFINITY, f64::max);
            ContourLevelPath::new(level_index, z_level, segments)
        })
        .collect();

    let cutting_direction = match params.cut_direction {
        ScanlineCutDirection::DownCut => CuttingDirection::Down,
        ScanlineCutDirection::UpCut => CuttingDirection::Up,
        ScanlineCutDirection::ZigZag => CuttingDirection::Mixed,
    };
    Ok(ToolPath::new(
        tool_id.to_string(),
        cutting_direction,
        Vec::new(),
        levels,
        Vec::new(),
    ))
}

/// 1 点のみの接触区間は切削線分を持たないため破棄する。
fn flush_run(run: &mut Vec<Point3D<f64>>, passes: &mut Vec<Vec<Point3D<f64>>>) {
    if run.len() >= 2 {
        passes.push(std::mem::take(run));
    } else {
        run.clear();
    }
}

/// [min, max] を `max_step` 以下の等間隔で両端を含めてサンプリングする。
fn uniform_samples(min: f64, max: f64, max_step: f64) -> Vec<f64> {
    let span = max - min;
    if span <= 0.0 {
        return vec![min];
    }
    let intervals = (span / max_step).ceil().max(1.0) as usize;
    (0..=intervals)
        .map(|i| min + span * i as f64 / intervals as f64)
        .collect()
}
