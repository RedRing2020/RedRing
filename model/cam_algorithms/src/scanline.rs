//! スキャン加工（`operation_type = scanline`）の経路化
//!
//! 加工範囲（既定は形状の XY 範囲）を Y 方向へ `stepover` 以下の等間隔で走査ラインに分け、
//! 各ライン上を X 正方向に `sample_pitch` 以下の間隔で CL を求める。
//! 工具が形状に接触する連続区間を 1 パスとし、パス間は早送り高さを経由して移動する。

use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, ToolPath};
use geo_algorithms::Point3D;

use crate::inverse_offset::DropCutter;
use crate::solver::{CamSolverError, ScanlineParams};

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

    let mut passes: Vec<Vec<Point3D<f64>>> = Vec::new();
    for &y in &ys {
        let mut run: Vec<Point3D<f64>> = Vec::new();
        for &x in &xs {
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

    let top = passes
        .iter()
        .flatten()
        .map(|p| p.z())
        .fold(f64::NEG_INFINITY, f64::max);
    let safe_z = top + params.clearance_height;
    let feed_rate = params.feed_rate;

    let mut levels = Vec::with_capacity(passes.len());
    let mut previous_exit: Option<Point3D<f64>> = None;
    for (level_index, pass) in passes.iter().enumerate() {
        let first = pass[0];
        let last = pass[pass.len() - 1];
        let entry = Point3D::new(first.x(), first.y(), safe_z);
        let exit = Point3D::new(last.x(), last.y(), safe_z);

        let mut segments = Vec::with_capacity(pass.len() + 2);
        if let Some(from) = previous_exit {
            segments.push(PathSegment::new_line(from, entry, SegmentType::Rapid));
        }
        segments.push(PathSegment::new_line(
            entry,
            first,
            SegmentType::Approach { feed_rate },
        ));
        for window in pass.windows(2) {
            segments.push(PathSegment::new_line(
                window[0],
                window[1],
                SegmentType::Cutting { feed_rate },
            ));
        }
        segments.push(PathSegment::new_line(
            last,
            exit,
            SegmentType::Retract { feed_rate },
        ));

        let z_level = pass.iter().map(|p| p.z()).fold(f64::NEG_INFINITY, f64::max);
        levels.push(ContourLevelPath::new(level_index, z_level, segments));
        previous_exit = Some(exit);
    }

    Ok(ToolPath::new(
        tool_id.to_string(),
        CuttingDirection::Down,
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
