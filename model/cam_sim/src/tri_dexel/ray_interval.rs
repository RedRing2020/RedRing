//! 座標軸に平行な線と、工具の掃引体の交差の区間（閉じた式）
//!
//! 掃引体は凸な形（垂直の円柱・球を線分に沿って動かした形）に分けて扱う。凸な形と線の交差は 1 本の区間で、
//! 移動のパラメータ `t ∈ [0, 1]` について「1 次式 ± √(2 次式)」の最小・最大として求まる。

use geo_algorithms::Point3D;

use crate::stock_model::{SweptToolPrimitive, SweptToolShape};

/// 座標軸に平行な線（`axis` の向き。他の 2 つの座標は `point` の値に固定）
#[derive(Debug, Clone, Copy)]
pub(super) struct AxisLine {
    pub(super) axis: usize,
    pub(super) point: Point3D<f64>,
}

impl AxisLine {
    fn coordinate(&self, axis: usize) -> f64 {
        [self.point.x(), self.point.y(), self.point.z()][axis]
    }
}

/// 線と掃引体の交差の区間（掃引体を分けた凸な形ごと。交わらない形は含まない）
pub(super) fn swept_primitive_intervals(
    primitive: &SweptToolPrimitive,
    line: AxisLine,
) -> Vec<(f64, f64)> {
    let start = [
        primitive.start.x(),
        primitive.start.y(),
        primitive.start.z(),
    ];
    let direction = primitive.direction();
    let (cylinder, sphere) = match primitive.shape {
        SweptToolShape::FlatEndMill {
            radius,
            cutting_length,
        } => (Some((radius, 0.0, cutting_length)), None),
        SweptToolShape::BallEndMill {
            radius,
            cutting_length,
        } => (Some((radius, radius, cutting_length)), Some(radius)),
    };
    let mut intervals = Vec::new();
    if let Some((radius, bottom, top)) = cylinder
        && let Some(interval) = swept_cylinder_interval(start, direction, radius, bottom, top, line)
    {
        intervals.push(interval);
    }
    if let Some(radius) = sphere {
        let center = [start[0], start[1], start[2] + radius];
        if let Some(interval) = swept_sphere_interval(center, direction, radius, line) {
            intervals.push(interval);
        }
    }
    intervals
}

/// 先端からの高さ `[bottom, top]`・半径 `radius` の垂直の円柱を、先端 `start + t·direction` に沿って掃引した形
fn swept_cylinder_interval(
    start: [f64; 3],
    direction: [f64; 3],
    radius: f64,
    bottom: f64,
    top: f64,
    line: AxisLine,
) -> Option<(f64, f64)> {
    if top < bottom {
        return None;
    }
    if line.axis == 2 {
        // 垂直の線: XY 上の距離が半径以下となる t の区間で、先端の高さ + [bottom, top]
        let q = radial_room(start, direction, radius, line, &[0, 1]);
        let (low, high) = nonnegative_range(q, (0.0, 1.0))?;
        let height = |t: f64| start[2] + t * direction[2];
        let (a, b) = (height(low), height(high));
        return Some((a.min(b) + bottom, a.max(b) + top));
    }
    // 水平の線: 高さの条件 bottom ≤ z - tip_z(t) ≤ top を満たす t の区間で、軸方向の位置 ± √(残りの半径²)
    let height = line.coordinate(2) - start[2];
    let range = if direction[2] == 0.0 {
        if !(bottom..=top).contains(&height) {
            return None;
        }
        (0.0, 1.0)
    } else {
        let (a, b) = (
            (height - top) / direction[2],
            (height - bottom) / direction[2],
        );
        (a.min(b).max(0.0), a.max(b).min(1.0))
    };
    if range.0 > range.1 {
        return None;
    }
    let other = 1 - line.axis;
    let q = radial_room(start, direction, radius, line, &[other]);
    extreme_interval((start[line.axis], direction[line.axis]), q, range)
}

/// 中心 `center + t·direction`・半径 `radius` の球を掃引した形（カプセル）
fn swept_sphere_interval(
    center: [f64; 3],
    direction: [f64; 3],
    radius: f64,
    line: AxisLine,
) -> Option<(f64, f64)> {
    let others: Vec<usize> = (0..3).filter(|&axis| axis != line.axis).collect();
    let q = radial_room(center, direction, radius, line, &others);
    extreme_interval((center[line.axis], direction[line.axis]), q, (0.0, 1.0))
}

/// `radius² - Σ (線の座標 - (origin + t·direction))²`（`axes` の成分について）を t の 2 次式 `[q0, q1, q2]` で返す
fn radial_room(
    origin: [f64; 3],
    direction: [f64; 3],
    radius: f64,
    line: AxisLine,
    axes: &[usize],
) -> [f64; 3] {
    let mut q = [radius * radius, 0.0, 0.0];
    for &axis in axes {
        let offset = line.coordinate(axis) - origin[axis];
        q[0] -= offset * offset;
        q[1] += 2.0 * offset * direction[axis];
        q[2] -= direction[axis] * direction[axis];
    }
    q
}

/// 凹な 2 次式 `q0 + q1 t + q2 t²`（`q2 ≤ 0`）が 0 以上となる t の区間と `range` の共通部分
fn nonnegative_range(q: [f64; 3], range: (f64, f64)) -> Option<(f64, f64)> {
    let [q0, q1, q2] = q;
    let (mut low, mut high) = range;
    if q2 == 0.0 {
        if q1 == 0.0 {
            return (q0 >= 0.0).then_some(range);
        }
        let root = -q0 / q1;
        if q1 > 0.0 {
            low = low.max(root);
        } else {
            high = high.min(root);
        }
    } else {
        let discriminant = q1 * q1 - 4.0 * q2 * q0;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt = discriminant.sqrt();
        let (a, b) = ((-q1 + sqrt) / (2.0 * q2), (-q1 - sqrt) / (2.0 * q2));
        low = low.max(a.min(b));
        high = high.min(a.max(b));
    }
    (low <= high).then_some((low, high))
}

/// `t ∈ range ∩ {Q ≥ 0}` についての `[min (l(t) - √Q(t)), max (l(t) + √Q(t))]`（`l(t) = l0 + l1 t`）
///
/// `l ± √Q` は凹（+）・凸（-）なので、極値は区間の端か、微分が 0 となる点（`Q' = ∓2 l1 √Q` を 2 乗した
/// 2 次方程式の根）にある。区間内の点は候補に加えても結果を変えないため、2 乗で加わる根と、重解の位置
/// （丸め誤差で判別式が負になっても取りこぼさないよう、2 次式の頂点）も候補に含める。
fn extreme_interval(l: (f64, f64), q: [f64; 3], range: (f64, f64)) -> Option<(f64, f64)> {
    let (low, high) = nonnegative_range(q, range)?;
    let [q0, q1, q2] = q;
    let (l0, l1) = l;
    // (q1 + 2 q2 t)² = 4 l1² (q0 + q1 t + q2 t²)
    let a = 4.0 * q2 * q2 - 4.0 * l1 * l1 * q2;
    let b = 4.0 * q1 * q2 - 4.0 * l1 * l1 * q1;
    let c = q1 * q1 - 4.0 * l1 * l1 * q0;
    let mut candidates = vec![low, high];
    let mut stationary = quadratic_roots(a, b, c);
    if a != 0.0 {
        stationary.push(-b / (2.0 * a));
    }
    candidates.extend(stationary.into_iter().filter(|t| (low..=high).contains(t)));
    let room = |t: f64| (q0 + q1 * t + q2 * t * t).max(0.0).sqrt();
    let minimum = candidates
        .iter()
        .map(|&t| l0 + l1 * t - room(t))
        .fold(f64::INFINITY, f64::min);
    let maximum = candidates
        .iter()
        .map(|&t| l0 + l1 * t + room(t))
        .fold(f64::NEG_INFINITY, f64::max);
    Some((minimum, maximum))
}

/// `a t² + b t + c = 0` の実数解（`a = b = 0` の場合は空）
fn quadratic_roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    if a == 0.0 {
        return if b == 0.0 { Vec::new() } else { vec![-c / b] };
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return Vec::new();
    }
    let sqrt = discriminant.sqrt();
    vec![(-b + sqrt) / (2.0 * a), (-b - sqrt) / (2.0 * a)]
}
