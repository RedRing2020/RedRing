//! スキャン加工（`operation_type = scanline`）の経路化
//!
//! 加工範囲（既定は形状の XY 範囲）を Y 方向へ `stepover` 以下の等間隔で走査ラインに分け、
//! 各ライン上を送り方（`ScanlineCutDirection`）で決めた向きに、工具の半径以下の間隔で CL を求める。
//! 工具が形状に接触する連続区間を 1 パスとし、接触する点と接触しない点の間は二分して、パスの端をトレランスまで
//! 求める。パスの間のエアカットは入れない（CL の編集と工程をそろえた後に
//! 計算する）。
//!
//! パスの点と点を結ぶ直線が CL の面からトレランスを超えて下を通らないように、区間ごとに CL の面が弦から上に出る量の
//! 最大を確かめ、超える区間はその位置に点を足して分ける。トレランスより短い区間でも超える所は CL の面の段差とし、
//! 高い側の高さのまま区間を進んでから下ろす段にする。

use cam_core::{
    ContourLevelPath, CuttingDirection, PathSegment, ScanlineCutDirection, ScanlineParams,
    SegmentType, ToolPath,
};
use geo_algorithms::Point3D;

use crate::inverse_offset_shape::InverseOffsetShape;
use crate::solver::CamSolverError;

/// 逆オフセット形状から、XY 範囲 `(min, max)` のスキャン加工 ToolPath を、トレランス `tolerance` で生成する。
///
/// 各パスは `ContourLevelPath` 1 件に対応し、`z_level` にはパス内の最も高い点（段の点を含む）の高さを格納する。
///
/// # Errors
///
/// トレランスが正の有限値でない場合と、どの走査ラインでも工具が形状に接しない場合。
pub fn generate_scanline_toolpath(
    cutter: &InverseOffsetShape,
    tool_id: &str,
    params: &ScanlineParams,
    (xy_min, xy_max): ([f64; 2], [f64; 2]),
    tolerance: f64,
) -> Result<ToolPath<f64>, CamSolverError> {
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(CamSolverError::InvalidInput(
            "scanline toolpath requires a positive finite tolerance".to_string(),
        ));
    }
    let ys = uniform_samples(xy_min[1], xy_max[1], params.stepover);
    let xs = uniform_samples(xy_min[0], xy_max[0], cutter.shape().radius());
    let xs_reversed: Vec<f64> = xs.iter().rev().copied().collect();

    let mut passes: Vec<Vec<Point3D<f64>>> = Vec::new();
    for (line, &y) in ys.iter().enumerate() {
        let forward = match params.cut_direction {
            ScanlineCutDirection::DownCut => false,
            ScanlineCutDirection::UpCut => true,
            ScanlineCutDirection::ZigZag => line % 2 == 1,
        };
        let mut run: Vec<Point3D<f64>> = Vec::new();
        // 直前の点が工具が形状に接触しない点の場合の X
        let mut outside = None;
        // 送る向きに並べた X
        let xs_in_feed_order = if forward { &xs } else { &xs_reversed };
        for &x in xs_in_feed_order {
            match cutter.tip_point_at(x, y) {
                Some(point) => {
                    if let Some(start) = outside
                        .take()
                        .and_then(|outside| contact_end(cutter, y, point, outside, tolerance))
                    {
                        run.push(start);
                    }
                    run.push(point);
                }
                None => {
                    if let Some(end) = run
                        .last()
                        .and_then(|&inside| contact_end(cutter, y, inside, x, tolerance))
                    {
                        run.push(end);
                    }
                    flush_run(&mut run, &mut passes, cutter, y, tolerance);
                    outside = Some(x);
                }
            }
        }
        flush_run(&mut run, &mut passes, cutter, y, tolerance);
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

/// 工具が形状に接触する点 `inside` と、接触しない X `outside` の間を二分し、`outside` からトレランス以内の接触する
/// 点を求める。`inside` がすでにトレランス以内の場合は `None`
///
/// 2 点の間で接触する・しないが何度か入れ替わる場合は、そのうちの 1 つの境目を求める。
fn contact_end(
    cutter: &InverseOffsetShape,
    y: f64,
    inside: Point3D<f64>,
    outside: f64,
    tolerance: f64,
) -> Option<Point3D<f64>> {
    let (mut inside_x, mut outside_x) = (inside.x(), outside);
    let mut end = None;
    while (outside_x - inside_x).abs() > tolerance {
        let x = 0.5 * (inside_x + outside_x);
        // 2 点が隣り合う浮動小数点数になり、それ以上分けられない場合
        if x == inside_x || x == outside_x {
            break;
        }
        match cutter.tip_point_at(x, y) {
            Some(point) => {
                inside_x = x;
                end = Some(point);
            }
            None => outside_x = x,
        }
    }
    end
}

/// 接触区間の点の間を確かめてパスにする。1 点のみの接触区間は切削線分を持たないため破棄する。
fn flush_run(
    run: &mut Vec<Point3D<f64>>,
    passes: &mut Vec<Vec<Point3D<f64>>>,
    cutter: &InverseOffsetShape,
    y: f64,
    tolerance: f64,
) {
    if run.len() >= 2 {
        passes.push(refine_pass(cutter, y, run, tolerance));
    }
    run.clear();
}

/// 走査線 y のパスの点の間に、CL の面が弦からトレランスを超えて上に出ないように点と段を足す
fn refine_pass(
    cutter: &InverseOffsetShape,
    y: f64,
    pass: &[Point3D<f64>],
    tolerance: f64,
) -> Vec<Point3D<f64>> {
    let mut refined = vec![pass[0]];
    // 確かめる区間（送る向きの先の区間を下に積み、手前の区間から確かめる）
    let mut intervals = Vec::new();
    for window in pass.windows(2) {
        intervals.push((window[0], window[1]));
        while let Some((a, b)) = intervals.pop() {
            let chord = |x: f64| a.z() + (b.z() - a.z()) * (x - a.x()) / (b.x() - a.x());
            if let Some((x, rise)) = cutter.rise_above_chord(y, [a.x(), a.z()], [b.x(), b.z()])
                && rise > tolerance
            {
                let (low, high) = (a.x().min(b.x()), a.x().max(b.x()));
                if high - low > tolerance {
                    // 区間の端からトレランスの半分以上離して分ける（分けた区間は必ず短くなる）。工具が形状に接しない
                    // 所では弦の上に置く
                    let x = x.clamp(low + tolerance / 2.0, high - tolerance / 2.0);
                    let z = cutter.tip_height_at(x, y).unwrap_or_else(|| chord(x));
                    let point = Point3D::new(x, y, z);
                    intervals.push((point, b));
                    intervals.push((a, point));
                    continue;
                }
                // CL の面の段差: 両端と区間の中の最も高い所のうち最も高い高さで進み、下ろす（最も高い所の高さは、
                // 鉛直線と要素の交点で求め直す）
                let peak = cutter.tip_height_at(x, y).unwrap_or(f64::NEG_INFINITY);
                let height = a.z().max(b.z()).max(peak);
                if height > a.z() {
                    refined.push(Point3D::new(a.x(), y, height));
                }
                if height > b.z() {
                    refined.push(Point3D::new(b.x(), y, height));
                }
            }
            refined.push(b);
        }
    }
    refined
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
