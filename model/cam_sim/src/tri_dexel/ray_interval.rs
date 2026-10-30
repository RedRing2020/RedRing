//! 座標軸に平行な線と、工具の掃引体の交差の区間（閉じた式）
//!
//! 掃引体は凸な形（垂直の円柱・球を線分に沿って動かした形）に分けて扱う。凸な形と線の交差は 1 本の区間で、
//! 移動のパラメータ `t ∈ [0, 1]` について「1 次式 ± √(2 次式)」の最小・最大として求まる。
//!
//! 区間の端の法線（凸な形の外向き）は、端を決めた移動の位置 `t` の工具の面の法線として解析的に求める。工具の面が
//! 滑らかでない円柱の縁では、包絡の条件（法線が移動の向きと直交する）を満たす縁の法線の錐の中の向きとし、満たす
//! 向きがない場合（移動の始まり・終わりの工具の縁）は縁の 2 つの面の法線の二等分とする。

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

    /// 線の上の、`axis` の座標が `value` の点
    fn point_at(&self, value: f64) -> [f64; 3] {
        let mut point = [self.point.x(), self.point.y(), self.point.z()];
        point[self.axis] = value;
        point
    }
}

/// 線と凸な形の交差の区間と、区間の端を決めた工具の面（法線は、使う端だけ `normal` で求める）
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ToolInterval {
    pub(super) range: (f64, f64),
    /// 区間の始まり・終わりを決めた工具の面
    faces: [ToolFace; 2],
    /// 凸な形の基準の点（円柱は先端、球は中心）の、移動の始まりの位置と向き
    origin: [f64; 3],
    direction: [f64; 3],
}

impl ToolInterval {
    /// 線 `line` の上の、区間の始まり（`end = 0`）・終わり（`end = 1`）の、凸な形の外向きの単位法線
    pub(super) fn normal(&self, end: usize, line: AxisLine) -> [f64; 3] {
        let point = line.point_at([self.range.0, self.range.1][end]);
        let at = |t: f64| -> [f64; 3] {
            std::array::from_fn(|axis| self.origin[axis] + t * self.direction[axis])
        };
        // 位置 t の円柱の軸から、端の点への水平の単位ベクトル
        let radial = |t: f64| {
            let axis = at(t);
            normalized([point[0] - axis[0], point[1] - axis[1], 0.0])
        };
        match self.faces[end] {
            ToolFace::Cap(cap) => [0.0, 0.0, cap.sign()],
            ToolFace::Side { t } => radial(t),
            ToolFace::Rim { t, cap } => rim_normal(radial(t), self.direction, cap),
            ToolFace::Sphere { t } => {
                let center = at(t);
                normalized(std::array::from_fn(|axis| point[axis] - center[axis]))
            }
        }
    }
}

#[cfg(test)]
impl ToolInterval {
    /// 端が円柱の底・上面で決まる区間 `range`（差し引きのテスト用）
    pub(super) fn cylinder_caps_for_test(range: (f64, f64)) -> Self {
        Self {
            range,
            faces: [ToolFace::Cap(Cap::Bottom), ToolFace::Cap(Cap::Top)],
            origin: [0.0; 3],
            direction: [0.0; 3],
        }
    }
}

/// 区間の端を決めた工具の面（t は端を決めた移動の位置）
#[derive(Debug, Clone, Copy, PartialEq)]
enum ToolFace {
    /// 円柱の底・上面
    Cap(Cap),
    /// 円柱の側面
    Side { t: f64 },
    /// 円柱の底・上面の縁
    Rim { t: f64, cap: Cap },
    /// 球
    Sphere { t: f64 },
}

/// 円柱の底・上面
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cap {
    Bottom,
    Top,
}

impl Cap {
    /// 面の外向きの Z 成分の符号
    fn sign(self) -> f64 {
        match self {
            Self::Bottom => -1.0,
            Self::Top => 1.0,
        }
    }
}

/// 線と掃引体の交差の区間（掃引体を分けた凸な形（円柱・球）ごと。交わらない形は `None`）
pub(super) fn swept_primitive_intervals(
    primitive: &SweptToolPrimitive,
    line: AxisLine,
) -> [Option<ToolInterval>; 2] {
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
    let cylinder = cylinder.and_then(|(radius, bottom, top)| {
        swept_cylinder_interval(start, direction, radius, bottom, top, line)
    });
    let sphere = sphere.and_then(|radius| {
        let center = [start[0], start[1], start[2] + radius];
        swept_sphere_interval(center, direction, radius, line)
    });
    [cylinder, sphere]
}

/// 先端からの高さ `[bottom, top]`・半径 `radius` の垂直の円柱を、先端 `start + t·direction` に沿って掃引した形
fn swept_cylinder_interval(
    start: [f64; 3],
    direction: [f64; 3],
    radius: f64,
    bottom: f64,
    top: f64,
    line: AxisLine,
) -> Option<ToolInterval> {
    if top < bottom {
        return None;
    }
    let interval = |range: (f64, f64), faces: [ToolFace; 2]| ToolInterval {
        range,
        faces,
        origin: start,
        direction,
    };
    if line.axis == 2 {
        // 垂直の線: XY 上の距離が半径以下となる t の区間で、先端の高さ + [bottom, top]
        let q = radial_room(start, direction, radius, line, &[0, 1]);
        let (low, high) = nonnegative_range(q, (0.0, 1.0))?;
        let height = |t: f64| start[2] + t * direction[2];
        // 高さが変わらない移動では、円の内側を通る位置（区間の中央）で底・上面に当たる
        let (lowest, highest) = match height(low).total_cmp(&height(high)) {
            std::cmp::Ordering::Less => (low, high),
            std::cmp::Ordering::Greater => (high, low),
            std::cmp::Ordering::Equal => (0.5 * (low + high), 0.5 * (low + high)),
        };
        // 円の内側を通る位置では底・上面、円に接する位置では縁に当たる
        let face = |t: f64, cap: Cap| {
            if quadratic(q, t) > 0.0 {
                ToolFace::Cap(cap)
            } else {
                ToolFace::Rim { t, cap }
            }
        };
        return Some(interval(
            (height(lowest) + bottom, height(highest) + top),
            [face(lowest, Cap::Bottom), face(highest, Cap::Top)],
        ));
    }
    // 水平の線: 高さの条件 bottom ≤ z - tip_z(t) ≤ top を満たす t の区間で、軸方向の位置 ± √(残りの半径²)
    let height = line.coordinate(2) - start[2];
    // 高さの条件の境の t（その位置では、線は円柱の底・上面の縁を通る）
    let (range, cap_times) = if direction[2] == 0.0 {
        if !(bottom..=top).contains(&height) {
            return None;
        }
        ((0.0, 1.0), None)
    } else {
        let (a, b) = (
            (height - top) / direction[2],
            (height - bottom) / direction[2],
        );
        ((a.min(b).max(0.0), a.max(b).min(1.0)), Some((b, a)))
    };
    if range.0 > range.1 {
        return None;
    }
    let other = 1 - line.axis;
    let q = radial_room(start, direction, radius, line, &[other]);
    let ((minimum, low_t), (maximum, high_t)) =
        extreme_interval((start[line.axis], direction[line.axis]), q, range)?;
    // 側面に当たる。底・上面の高さちょうどの位置では縁に当たる
    let face = |t: f64| {
        let cap = match cap_times {
            None if height == bottom => Some(Cap::Bottom),
            None if height == top => Some(Cap::Top),
            None => None,
            Some((bottom_t, _)) if t == bottom_t => Some(Cap::Bottom),
            Some((_, top_t)) if t == top_t => Some(Cap::Top),
            Some(_) => None,
        };
        match cap {
            Some(cap) => ToolFace::Rim { t, cap },
            None => ToolFace::Side { t },
        }
    };
    Some(interval((minimum, maximum), [face(low_t), face(high_t)]))
}

/// 円柱の底・上面の縁の、水平の外向き `radial` の点の、移動の向き `direction` の掃引体の外向きの単位法線
///
/// 縁の法線の錐（`radial` と面の法線の間）の中で移動の向きと直交する向き（包絡の上の点の法線）。直交する向きが
/// 錐にない場合（縁が包絡の上にない、移動の始まり・終わりの工具の縁）は、`radial` と面の法線の二等分とする。
fn rim_normal(radial: [f64; 3], direction: [f64; 3], cap: Cap) -> [f64; 3] {
    // n = a·radial + b·ẑ（a ≥ 0、b の符号は面の外向き）で n·direction = 0 となる (a, b) は (v_z, -radial·v) の定数倍
    let along = radial[0] * direction[0] + radial[1] * direction[1];
    let (a, b) = (direction[2], -along);
    let orthogonal = [1.0, -1.0].into_iter().find_map(|scale: f64| {
        let (a, b) = (scale * a, scale * b);
        (a >= 0.0 && b * cap.sign() >= 0.0 && (a != 0.0 || b != 0.0)).then_some((a, b))
    });
    let (a, b) = orthogonal.unwrap_or((1.0, cap.sign()));
    normalized([a * radial[0], a * radial[1], b])
}

/// 中心 `center + t·direction`・半径 `radius` の球を掃引した形（カプセル）
fn swept_sphere_interval(
    center: [f64; 3],
    direction: [f64; 3],
    radius: f64,
    line: AxisLine,
) -> Option<ToolInterval> {
    let others: Vec<usize> = (0..3).filter(|&axis| axis != line.axis).collect();
    let q = radial_room(center, direction, radius, line, &others);
    let ((minimum, low_t), (maximum, high_t)) =
        extreme_interval((center[line.axis], direction[line.axis]), q, (0.0, 1.0))?;
    // 端の点は位置 t の球の上にある
    Some(ToolInterval {
        range: (minimum, maximum),
        faces: [
            ToolFace::Sphere { t: low_t },
            ToolFace::Sphere { t: high_t },
        ],
        origin: center,
        direction,
    })
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

/// `t ∈ range ∩ {Q ≥ 0}` についての `[min (l(t) - √Q(t)), max (l(t) + √Q(t))]`（`l(t) = l0 + l1 t`）と、
/// 最小・最大となる t
///
/// `l ± √Q` は凹（+）・凸（-）なので、極値は区間の端か、微分が 0 となる点（`Q' = ∓2 l1 √Q` を 2 乗した
/// 2 次方程式の根）にある。区間内の点は候補に加えても結果を変えないため、2 乗で加わる根と、重解の位置
/// （丸め誤差で判別式が負になっても取りこぼさないよう、2 次式の頂点）も候補に含める。
fn extreme_interval(
    l: (f64, f64),
    q: [f64; 3],
    range: (f64, f64),
) -> Option<((f64, f64), (f64, f64))> {
    let (low, high) = nonnegative_range(q, range)?;
    let [q0, q1, q2] = q;
    let (l0, l1) = l;
    // (q1 + 2 q2 t)² = 4 l1² (q0 + q1 t + q2 t²)
    let a = 4.0 * q2 * q2 - 4.0 * l1 * l1 * q2;
    let b = 4.0 * q1 * q2 - 4.0 * l1 * l1 * q1;
    let c = q1 * q1 - 4.0 * l1 * l1 * q0;
    let vertex = (a != 0.0).then(|| -b / (2.0 * a));
    let [first, second] = quadratic_roots(a, b, c);
    let stationary = [first, second, vertex]
        .into_iter()
        .flatten()
        .filter(|t| (low..=high).contains(t));
    let room = |t: f64| quadratic(q, t).max(0.0).sqrt();
    let (minimum, maximum) = [low, high].into_iter().chain(stationary).fold(
        ((f64::INFINITY, low), (f64::NEG_INFINITY, low)),
        |(minimum, maximum), t| {
            let (lower, upper) = (l0 + l1 * t - room(t), l0 + l1 * t + room(t));
            (
                if lower < minimum.0 {
                    (lower, t)
                } else {
                    minimum
                },
                if upper > maximum.0 {
                    (upper, t)
                } else {
                    maximum
                },
            )
        },
    );
    Some((minimum, maximum))
}

/// 2 次式 `q0 + q1 t + q2 t²` の値
fn quadratic([q0, q1, q2]: [f64; 3], t: f64) -> f64 {
    q0 + q1 * t + q2 * t * t
}

/// 単位ベクトル（長さは 0 でない）
fn normalized(vector: [f64; 3]) -> [f64; 3] {
    let length = vector
        .iter()
        .map(|component| component * component)
        .sum::<f64>()
        .sqrt();
    vector.map(|component| component / length)
}

/// `a t² + b t + c = 0` の実数解（`a = b = 0` の場合は両方 `None`）
fn quadratic_roots(a: f64, b: f64, c: f64) -> [Option<f64>; 2] {
    if a == 0.0 {
        return [(b != 0.0).then(|| -c / b), None];
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return [None, None];
    }
    let sqrt = discriminant.sqrt();
    [Some((-b + sqrt) / (2.0 * a)), Some((-b - sqrt) / (2.0 * a))]
}
