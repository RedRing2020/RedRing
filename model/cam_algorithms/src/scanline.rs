//! スキャン加工（`operation_type = scanline`）の経路化
//!
//! 走査線の向きの座標（走査線を X 軸に沿わせた座標。`ScanFrame`）で、走査の範囲を Y 方向へ `stepover` 以下の
//! 等間隔で走査ラインに分け、各ライン上を送り方（`ScanlineCutDirection`）で決めた向きに、工具の半径以下の間隔で
//! CL を求める。求めた点は加工の座標へ戻す。
//! 工具が形状に接触する連続区間を 1 パスとし、接触する点と接触しない点の間は二分して、パスの端をトレランスまで
//! 求める。パスの間のエアカットは入れない（CL の編集と工程をそろえた後に
//! 計算する）。
//!
//! パスの点と点を結ぶ直線が CL の面からトレランスを超えて上下に離れないように、区間ごとに確かめて点を足す。
//!
//! - 上側（削り残し）: 直線が CL の面より上を通る量の最大は、CL の面を与える要素が切り替わる所（谷か段差）にある。
//!   切り替わる所を求めて確かめ、最も深い谷に点を足すか、段差に段を足す
//! - 下側（削り込み）: CL の面が弦から上に出る量の最大を要素から求め、超える区間はその位置に点を足して分ける。
//!   トレランスより短い区間でも超える所は CL の面の段差とし、高い側の高さのまま区間を進んでから下ろす段にする

use cam_core::{
    ContourLevelPath, CuttingDirection, PathSegment, ScanlineCutDirection, ScanlineParams,
    SegmentType, ToolPath,
};
use geo_algorithms::Point3D;
use geo_contracts::default_kernel_numerical_zero_tolerance;

use crate::inverse_offset_shape::InverseOffsetShape;
use crate::scan_frame::{ScanFrame, ScanRegion};
use crate::solver::CamSolverError;

/// 走査線の向きの座標 `frame` の逆オフセット形状 `cutter` から、その座標の範囲 `region` のスキャン加工 ToolPath を、
/// トレランス `tolerance` で生成する（点は加工の座標）。
///
/// 各パスは `ContourLevelPath` 1 件に対応し、`z_level` にはパス内の最も高い点（段の点を含む）の高さを格納する。
/// パスの点の数の合計が `point_limit` を超えたら、走査ラインごとに確かめて止める（メモリを使い切る前に）。
///
/// # Errors
///
/// トレランスが正の有限値でない場合と、どの走査ラインでも工具が形状に接しない場合と、点の数が `point_limit` を
/// 超える場合。
pub(crate) fn generate_scanline_toolpath(
    cutter: &InverseOffsetShape,
    tool_id: &str,
    params: &ScanlineParams,
    (region, frame): (&ScanRegion, ScanFrame),
    tolerance: f64,
    point_limit: usize,
) -> Result<ToolPath<f64>, CamSolverError> {
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(CamSolverError::InvalidInput(
            "scanline toolpath requires a positive finite tolerance".to_string(),
        ));
    }
    let Some((y_min, y_max)) = region.y_range() else {
        return Err(CamSolverError::NoSolution(
            "scanline region is empty".to_string(),
        ));
    };
    // 走査ラインの数・ライン上の X の数が点の数の上限を超えるなら、並びを作る前に止める（ピックフィード・工具の半径が
    // とても小さいと、並びだけでメモリを使い切るため）
    let ys = uniform_samples(y_min, y_max, params.stepover, point_limit)?;

    let mut passes: Vec<Vec<Point3D<f64>>> = Vec::new();
    // パスの点の数の合計
    let mut points = 0usize;
    // 範囲を通る走査線の番号（ジグザグの送る向きを決める）
    let mut line = 0;
    for &y in &ys {
        let Some((x_min, x_max)) = region.x_range_at(y) else {
            continue;
        };
        if x_min == x_max {
            continue;
        }
        line += 1;
        let forward = match params.cut_direction {
            ScanlineCutDirection::DownCut => false,
            ScanlineCutDirection::UpCut => true,
            ScanlineCutDirection::ZigZag => line % 2 == 0,
        };
        let passes_before = passes.len();
        scan_line(
            cutter,
            (y, x_min, x_max),
            forward,
            tolerance,
            point_limit,
            &mut passes,
        )?;
        points = passes[passes_before..]
            .iter()
            .fold(points, |sum, pass| sum.saturating_add(pass.len()));
        if points > point_limit {
            return Err(CamSolverError::TooManyPoints);
        }
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
                    PathSegment::new_line(
                        frame.to_work(window[0]),
                        frame.to_work(window[1]),
                        SegmentType::Cutting { feed_rate },
                    )
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

/// 見積もりで抜き出す走査ラインの数
const ESTIMATE_SAMPLE_LINES: usize = 64;

/// 見積もりで、抜き出した走査ラインの上で求める X の数の合計の上限（超えるなら、パスを求めずに X の数で見積もる。
/// 見積もりに時間をかけすぎないため）
const ESTIMATE_MAX_SAMPLES: f64 = 200_000.0;

/// X の数で見積もるときに、工具が形状に接触するかを確かめる、走査ライン 1 本あたりの X の数
const ESTIMATE_CONTACT_PROBES: usize = 1024;

/// 走査線の向きの座標の逆オフセット形状 `cutter` の、範囲 `region` のスキャン加工のパスの点の数を、トレランス
/// `tolerance` で見積もる
///
/// 走査ラインを等間隔に抜き出し（全体が少なければすべて）、生成と同じ処理で各ラインのパスの点を数え、全体の走査
/// ラインの数との割合で掛け戻す。抜き出したラインの上の X の数が多すぎるときは、パスを求めずに、X の数に工具が形状
/// に接触する割合（ライン上を粗く確かめる）を掛けて見積もる（工具が形状に接触する所では、X ごとにパスの点を置く
/// ため。トレランスに収めるために足す点は数えない）。
///
/// # Errors
///
/// トレランスが正の有限値でない場合と、範囲が空の場合。
pub(crate) fn estimate_scanline_points(
    cutter: &InverseOffsetShape,
    params: &ScanlineParams,
    region: &ScanRegion,
    tolerance: f64,
) -> Result<u64, CamSolverError> {
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(CamSolverError::InvalidInput(
            "scanline toolpath requires a positive finite tolerance".to_string(),
        ));
    }
    let Some((y_min, y_max)) = region.y_range() else {
        return Err(CamSolverError::NoSolution(
            "scanline region is empty".to_string(),
        ));
    };
    // 走査ラインの数は並びを作らずに数える（生成と同じ間隔。とても多くても f64 で数える）
    let span = y_max - y_min;
    let intervals = if span > 0.0 {
        (span / params.stepover).ceil().max(1.0)
    } else {
        0.0
    };
    let lines = intervals + 1.0;
    let samples = (ESTIMATE_SAMPLE_LINES as f64).min(lines) as usize;
    let sampled: Vec<(f64, f64, f64)> = (0..samples)
        .filter_map(|i| {
            let index = if samples > 1 {
                (i as f64 * intervals / (samples - 1) as f64).round()
            } else {
                0.0
            };
            let y = if intervals > 0.0 {
                y_min + span * index / intervals
            } else {
                y_min
            };
            let (x_min, x_max) = region.x_range_at(y)?;
            (x_min != x_max).then_some((y, x_min, x_max))
        })
        .collect();
    let scale = lines / samples as f64;
    let radius = cutter.shape().radius();
    let xs: f64 = sampled
        .iter()
        .map(|&(_, x_min, x_max)| ((x_max - x_min) / radius).ceil().max(1.0) + 1.0)
        .sum();
    if xs > ESTIMATE_MAX_SAMPLES {
        let contact_xs: f64 = sampled
            .iter()
            .map(|&(y, x_min, x_max)| {
                let line_xs = ((x_max - x_min) / radius).ceil().max(1.0) + 1.0;
                let contacts = (0..ESTIMATE_CONTACT_PROBES)
                    .filter(|&i| {
                        let t = (i as f64 + 0.5) / ESTIMATE_CONTACT_PROBES as f64;
                        cutter
                            .tip_point_at(x_min + (x_max - x_min) * t, y)
                            .is_some()
                    })
                    .count();
                line_xs * contacts as f64 / ESTIMATE_CONTACT_PROBES as f64
            })
            .sum();
        return Ok((contact_xs * scale) as u64);
    }
    let mut points = 0usize;
    for &(y, x_min, x_max) in &sampled {
        let mut passes = Vec::new();
        scan_line(
            cutter,
            (y, x_min, x_max),
            true,
            tolerance,
            usize::MAX,
            &mut passes,
        )?;
        points += passes.iter().map(Vec::len).sum::<usize>();
    }
    Ok((points as f64 * scale) as u64)
}

/// 走査ライン `y` の上を、X の範囲 [`x_min`, `x_max`] で送る向き（`forward` なら X の増える向き）に進み、工具が形状
/// に接触する区間をパスにして `passes` に足す
///
/// # Errors
///
/// ライン上の X の数が `point_limit` を超える場合（並びを作る前に止める）。
fn scan_line(
    cutter: &InverseOffsetShape,
    (y, x_min, x_max): (f64, f64, f64),
    forward: bool,
    tolerance: f64,
    point_limit: usize,
    passes: &mut Vec<Vec<Point3D<f64>>>,
) -> Result<(), CamSolverError> {
    let mut run: Vec<Point3D<f64>> = Vec::new();
    // 直前の点が工具が形状に接触しない点の場合の X
    let mut outside = None;
    // 送る向きに並べた X
    let mut xs_in_feed_order = uniform_samples(x_min, x_max, cutter.shape().radius(), point_limit)?;
    if !forward {
        xs_in_feed_order.reverse();
    }
    for x in xs_in_feed_order {
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
                flush_run(&mut run, passes, cutter, y, tolerance);
                outside = Some(x);
            }
        }
    }
    flush_run(&mut run, passes, cutter, y, tolerance);
    Ok(())
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

/// 走査線 y のパスの点の間に点と段を足し、パスが CL の面から上下ともトレランスを超えて離れないようにする
///
/// 1. 弦が CL の面よりトレランスを超えて上を通る所（削り残しの側）は、CL の面を与える要素が切り替わる所（谷か段差）
///    にある。最も深い谷に点を足すか、段差に段を足す（`surface_switch`）
/// 2. CL の面が弦からトレランスを超えて上に出る所（削り込みの側）は、その位置に点を足す。トレランスより短い区間でも
///    超える所は段差とし、高い側の高さのまま区間を進んでから下ろす
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
            if (b.x() - a.x()).abs() > tolerance {
                match surface_switch(cutter, y, a, b, tolerance) {
                    Some(Switch::Fold(point)) => {
                        intervals.push((point, b));
                        intervals.push((a, point));
                        continue;
                    }
                    Some(Switch::Step(upper, lower)) => {
                        intervals.push((lower, b));
                        intervals.push((upper, lower));
                        intervals.push((a, upper));
                        continue;
                    }
                    None => {}
                }
            }
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

/// 弦が CL の面よりトレランスを超えて上を通る所に足す点か段
enum Switch {
    /// 最も深い谷（2 つの要素の面がつながる所）の点
    Fold(Point3D<f64>),
    /// 段差（2 つの要素の面の高さが跳ぶ所）の、送る向きの手前の点と先の点（同じ XY。経路は高い側の高さのまま段差の
    /// 先まで進んでから下ろすか、段差の手前で上げる）
    Step(Point3D<f64>, Point3D<f64>),
}

/// 走査線 y の区間 a→b の弦が、CL の面よりトレランスを超えて上を通る場合の、足す点か段（通らなければ `None`）
///
/// 区間を、CL の面を与える要素が切り替わる所で分けていく。切り替わる所は、両側の要素の境目を浮動小数点の精度まで
/// 二分して求め（`element_boundary`）、境目の両側の高さの差がトレランスを超えれば段差とする。要素ごとの面は上に凸か
/// 直線で、要素が効く範囲は凸のため、両端で同じ要素が CL の面を与える部分では、弦が CL の面より上を通る量は部分の
/// 端で最も大きい。そのため、その量の最大は、区間の端か切り替わる所にある。区間の中で工具が形状に接しない所は扱わない。
fn surface_switch(
    cutter: &InverseOffsetShape,
    y: f64,
    a: Point3D<f64>,
    b: Point3D<f64>,
    tolerance: f64,
) -> Option<Switch> {
    let span = b.x() - a.x();
    // 端の要素は、端から数値計算のゼロの判定の値だけ内側で求める（端がちょうど切り替わる所の場合に、内側の要素に
    // する。円の側面の近くでは、内側へずらした分の高さの変わりが大きいため、トレランスよりずっと小さくずらす）
    let inset = default_kernel_numerical_zero_tolerance::<f64>() / span.abs();
    let x_at = |t: f64| a.x() + t * span;
    let chord = |x: f64| a.z() + (b.z() - a.z()) * (x - a.x()) / span;
    // 端の点だけが高い（工具が形状の縁に接するだけの所など）段差: 端のすぐ内側で上げ下げする
    let (x_first, x_last) = (x_at(inset), x_at(1.0 - inset));
    let (z_first, first) = cutter.tip_height_and_element_at(x_first, y)?;
    let (z_last, last) = cutter.tip_height_and_element_at(x_last, y)?;
    if a.z() - z_first > tolerance {
        return Some(Switch::Step(
            Point3D::new(x_first, y, a.z()),
            Point3D::new(x_first, y, z_first),
        ));
    }
    if b.z() - z_last > tolerance {
        return Some(Switch::Step(
            Point3D::new(x_last, y, z_last),
            Point3D::new(x_last, y, b.z()),
        ));
    }
    // 弦が CL の面より上を通る量がトレランスを超える谷のうち、最も深いものの量と点
    let mut deepest: Option<(f64, Point3D<f64>)> = None;
    // 確かめる部分（両端の、区間の中の位置 t と、そこで CL の面を与える要素）
    let mut pieces = vec![((inset, first), (1.0 - inset, last))];
    while let Some((low, high)) = pieces.pop() {
        if low.1 == high.1 {
            continue;
        }
        let Some((before, after)) =
            element_boundary(cutter, y, x_at, low, high, |x, z| chord(x) - z <= tolerance)
        else {
            continue;
        };
        // 両端が隣り合う浮動小数点数で、それ以上分けられない部分
        if before == low.0 && after == high.0 {
            continue;
        }
        let (x_before, x_after) = (x_at(before), x_at(after));
        // 境目の手前で `low.1` が、先で `high.1` が効けば、要素が効く範囲は凸のため、部分の手前の端から境目の手前までは
        // `low.1` が、境目の先から部分の先の端までは `high.1` が効き、CL の面はそれより低くない。弦がその 2 つの面より
        // 上を通る量は両端で最も大きいため、境目の両側でトレランス以下なら、部分の中に深い谷も段差もない（段差で
        // CL の面が弦より上に出る所は、削り込みの側で確かめる）
        if let (Some(z_low), Some(z_high)) = (
            cutter.element_tip_height_at(low.1, x_before, y),
            cutter.element_tip_height_at(high.1, x_after, y),
        ) && chord(x_before) - z_low <= tolerance
            && chord(x_after) - z_high <= tolerance
        {
            continue;
        }
        let Some(((z_before, before_element), (z_after, after_element))) = cutter
            .tip_height_and_element_at(x_before, y)
            .zip(cutter.tip_height_and_element_at(x_after, y))
        else {
            continue;
        };
        if z_before - z_after > tolerance {
            // 下りの段差: 高い側の高さのまま段差の先まで進んでから下ろす
            return Some(Switch::Step(
                Point3D::new(x_after, y, z_before),
                Point3D::new(x_after, y, z_after),
            ));
        }
        if z_after - z_before > tolerance {
            // 上りの段差: 段差の手前で上げる
            return Some(Switch::Step(
                Point3D::new(x_before, y, z_before),
                Point3D::new(x_before, y, z_after),
            ));
        }
        let depth = (chord(x_before) - z_before).max(chord(x_after) - z_after);
        if depth > tolerance && deepest.is_none_or(|(deepest, _)| depth > deepest) {
            deepest = Some((depth, Point3D::new(x_after, y, z_after)));
        }
        pieces.push(((after, after_element), high));
        pieces.push((low, (before, before_element)));
    }
    deepest.map(|(_, point)| Switch::Fold(point))
}

/// 区間の中の位置 `low.0` と `high.0`（そこで CL の面を与える要素 `low.1` と `high.1` が違う）の間で、`low.1` が CL の
/// 面を与える側と与えない側の境目を、浮動小数点の精度まで二分して求める（境目の両側の位置）。その間で弦が CL の面より
/// トレランスを超えて上を通らないと分かった場合は `None`
///
/// ふつうは 2 つの要素だけを評価し、どちらが高いかが入れ替わる所を求める（間にほかの要素が CL の面を与える所が
/// あっても、求めた所の両側をまた確かめる）。端で 2 つの要素の高さが等しい場合など、両端で判定が分かれない場合は、
/// CL の面を与える要素で判定する。
///
/// 2 つの要素がともに効く位置 m があれば、要素が効く範囲は凸のため、`low.0`〜m では `low.1` が、m〜`high.0` では
/// `high.1` が効き、CL の面はそれより低くない。要素ごとの面は上に凸か直線のため、弦がそれぞれの面より上を通る量は、
/// 両端（`low.0` と m、m と `high.0`）で最も大きい。`low.0` と `high.0` では CL の面と同じ高さのため、m で 2 つの要素の
/// 低い方の高さ z が `shallow(x, z)`（弦が z より上を通る量がトレランス以下）なら、間に深い谷はない。
fn element_boundary(
    cutter: &InverseOffsetShape,
    y: f64,
    x_at: impl Fn(f64) -> f64,
    (low, low_element): (f64, usize),
    (high, high_element): (f64, usize),
    shallow: impl Fn(f64, f64) -> bool,
) -> Option<(f64, f64)> {
    // 位置 x での 2 つの要素の高さ。2 つの要素がともに効き、間に深い谷がないと分かれば `None`
    let heights = |x: f64| {
        let pair = (
            cutter.element_tip_height_at(low_element, x, y),
            cutter.element_tip_height_at(high_element, x, y),
        );
        match pair {
            (Some(z_low), Some(z_high)) if shallow(x, z_low.min(z_high)) => None,
            _ => Some(pair),
        }
    };
    let low_is_higher = |pair: (Option<f64>, Option<f64>)| match pair {
        (Some(z_low), Some(z_high)) => z_low > z_high,
        (Some(_), None) => true,
        (None, _) => false,
    };
    let by_two_elements =
        low_is_higher(heights(x_at(low))?) && !low_is_higher(heights(x_at(high))?);
    let (mut before, mut after) = (low, high);
    loop {
        let middle = 0.5 * (before + after);
        let x = x_at(middle);
        // 2 点が隣り合う浮動小数点数になり、それ以上分けられない場合
        if x == x_at(before) || x == x_at(after) {
            return Some((before, after));
        }
        let pair = heights(x)?;
        let on_low_side = if by_two_elements {
            low_is_higher(pair)
        } else {
            cutter
                .tip_height_and_element_at(x, y)
                .map(|(_, element)| element)
                == Some(low_element)
        };
        if on_low_side {
            before = middle;
        } else {
            after = middle;
        }
    }
}

/// [min, max] を `max_step` 以下の等間隔で両端を含めてサンプリングする。
///
/// # Errors
///
/// 点の数が `limit` を超える場合（並びを作る前に止める）。
fn uniform_samples(
    min: f64,
    max: f64,
    max_step: f64,
    limit: usize,
) -> Result<Vec<f64>, CamSolverError> {
    let span = max - min;
    if span <= 0.0 {
        return Ok(vec![min]);
    }
    // 大きすぎる値は usize の最大に丸まる
    let intervals = (span / max_step).ceil().max(1.0) as usize;
    if intervals >= limit {
        return Err(CamSolverError::TooManyPoints);
    }
    Ok((0..=intervals)
        .map(|i| min + span * i as f64 / intervals as f64)
        .collect())
}
