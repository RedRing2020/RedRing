//! NURBS 曲線上の距離の最小値
//!
//! 曲線を Bézier の区間に分解し、分枝限定法で、相手の形状までの距離が最小になる曲線上の点の距離を求める。
//! 区間は重みを掛けた同次座標で扱い、ノット挿入（Boehm の方法）と de Casteljau 法で分割する。

use analysis::Scalar;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// 分枝限定法で区間を分割する回数の上限
///
/// 区間は分割のたびに半分になり、制御点を囲む球の半径も小さくなる。上限は、許容誤差に届くまでの分割を
/// 十分に上回る回数とする（届かない場合は、それまでに得た曲線上の点の距離の最小値を返す）。
const NURBS_CURVE_MINIMUM_DISTANCE_MAX_SPLITS: usize = 100_000;

/// 同次座標の点 `(w x, w y, w z, w)`
type Homogeneous<T> = [T; 4];

/// NURBS 曲線上で、相手の形状までの距離 `distance` が最小になる点の距離
///
/// - `coordinates`: 制御点の座標を `x, y, z` の順に並べた配列
/// - `weights`: 制御点の重み（`None` は重みがすべて 1）。重みは正
/// - `knots`: 両端の重複度が `degree + 1` のノット
/// - `distance`: 点 `(x, y, z)` から相手の形状までの距離。点が `δ` 動いたときの変化が `δ` 以下であること
///
/// 返す距離は曲線上の点での距離で、真の最小値以上、真の最小値 + `tolerance` 以下になる。
/// ただし、曲線の長い範囲で距離がほぼ一定の場合（円とその軸上の点等）は、分割の回数の上限に達して探索を
/// 終えることがある。その場合も曲線上の点での距離を返すが、真の最小値 + `tolerance` 以下は保証しない。
///
/// # 計算アルゴリズム
/// 各区間の曲線は制御点の凸包の中にあるので、制御点を囲む球（中心 `c`・半径 `ρ`）の中にある。`distance` の
/// 変化は点の移動量以下なので、区間の下界は `max(0, distance(c) - ρ)`、上界は区間の両端（曲線上の点）での距離。
/// 下界が最も小さい区間を半分に分けることを、上界の最小値と下界の最小値の差が `tolerance` 以下になるまで繰り返す。
pub fn nurbs_curve_minimum_distance<T: Scalar>(
    degree: usize,
    knots: &[T],
    coordinates: &[T],
    weights: Option<&[T]>,
    distance: impl Fn(T, T, T) -> T,
    tolerance: T,
) -> T {
    let control_points: Vec<Homogeneous<T>> = coordinates
        .as_chunks::<3>()
        .0
        .iter()
        .enumerate()
        .map(|(index, [x, y, z])| {
            let w = weights.map_or(T::ONE, |weights| weights[index]);
            [*x * w, *y * w, *z * w, w]
        })
        .collect();
    let distance_at = |point: &Homogeneous<T>| {
        distance(
            point[0] / point[3],
            point[1] / point[3],
            point[2] / point[3],
        )
    };

    let mut best = T::INFINITY;
    let mut queue = BinaryHeap::new();
    for points in bezier_segments(degree, knots, control_points) {
        best = best.min(distance_at(&points[0]).min(distance_at(&points[degree])));
        queue.push(Segment::new(points, &distance_at));
    }

    for _ in 0..NURBS_CURVE_MINIMUM_DISTANCE_MAX_SPLITS {
        let Some(segment) = queue.pop() else { break };
        if best - segment.lower_bound <= tolerance {
            break;
        }
        let (left, right) = split_bezier(&segment.points);
        best = best.min(distance_at(&right[0]));
        for half in [left, right] {
            let half = Segment::new(half, &distance_at);
            if best - half.lower_bound > tolerance {
                queue.push(half);
            }
        }
    }
    best
}

/// Bézier の区間の制御点と、区間の曲線上の点までの距離の下界
struct Segment<T: Scalar> {
    points: Vec<Homogeneous<T>>,
    lower_bound: T,
}

impl<T: Scalar> Segment<T> {
    fn new(points: Vec<Homogeneous<T>>, distance_at: &impl Fn(&Homogeneous<T>) -> T) -> Self {
        let count = T::from_usize(points.len());
        let projected: Vec<[T; 3]> = points
            .iter()
            .map(|p| [p[0] / p[3], p[1] / p[3], p[2] / p[3]])
            .collect();
        let center = projected.iter().fold([T::ZERO; 3], |sum, p| {
            [
                sum[0] + p[0] / count,
                sum[1] + p[1] / count,
                sum[2] + p[2] / count,
            ]
        });
        let radius = projected
            .iter()
            .map(|p| {
                let (dx, dy, dz) = (p[0] - center[0], p[1] - center[1], p[2] - center[2]);
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .fold(T::ZERO, T::max);
        let lower_bound =
            (distance_at(&[center[0], center[1], center[2], T::ONE]) - radius).max(T::ZERO);
        Self {
            points,
            lower_bound,
        }
    }
}

// 下界が小さい区間ほど先に取り出す
impl<T: Scalar> Ord for Segment<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .lower_bound
            .partial_cmp(&self.lower_bound)
            .unwrap_or(Ordering::Equal)
    }
}

impl<T: Scalar> PartialOrd for Segment<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Scalar> PartialEq for Segment<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T: Scalar> Eq for Segment<T> {}

/// 内部のノットの重複度を `degree` 以上に上げ、長さが 0 でない区間ごとの Bézier の制御点（各 `degree + 1` 点）に分ける
///
/// 重複度が `degree` 以上のノットで挟まれた区間 `[knots[k], knots[k + 1])` の曲線は、制御点
/// `k - degree ..= k` の Bézier 曲線になる。
fn bezier_segments<T: Scalar>(
    degree: usize,
    knots: &[T],
    mut points: Vec<Homogeneous<T>>,
) -> Vec<Vec<Homogeneous<T>>> {
    let mut knots = knots.to_vec();
    let (start, end) = (knots[degree], knots[knots.len() - degree - 1]);
    let mut interior: Vec<T> = knots
        .iter()
        .copied()
        .filter(|k| start < *k && *k < end)
        .collect();
    interior.dedup();
    for knot in interior {
        let multiplicity = knots.iter().filter(|k| **k == knot).count();
        for _ in multiplicity..degree {
            insert_knot(degree, &mut knots, &mut points, knot);
        }
    }
    (degree..knots.len() - degree - 1)
        .filter(|k| knots[*k] < knots[*k + 1])
        .map(|k| points[k - degree..=k].to_vec())
        .collect()
}

/// ノット `u`（内部のノット）を 1 回挿入する（Boehm の方法）
fn insert_knot<T: Scalar>(
    degree: usize,
    knots: &mut Vec<T>,
    points: &mut Vec<Homogeneous<T>>,
    u: T,
) {
    // knots[span] <= u < knots[span + 1]
    let span = knots.iter().rposition(|k| *k <= u).unwrap_or(degree);
    let mut inserted = Vec::with_capacity(points.len() + 1);
    inserted.extend_from_slice(&points[..=span - degree]);
    for i in span - degree + 1..=span {
        let alpha = (u - knots[i]) / (knots[i + degree] - knots[i]);
        inserted.push(lerp(&points[i - 1], &points[i], alpha));
    }
    inserted.extend_from_slice(&points[span..]);
    *points = inserted;
    knots.insert(span + 1, u);
}

/// Bézier の区間をパラメータの中央で 2 つに分ける（de Casteljau 法）
fn split_bezier<T: Scalar>(
    points: &[Homogeneous<T>],
) -> (Vec<Homogeneous<T>>, Vec<Homogeneous<T>>) {
    let half = T::ONE / (T::ONE + T::ONE);
    let mut level = points.to_vec();
    let mut left = vec![level[0]];
    let mut right = vec![level[level.len() - 1]];
    while level.len() > 1 {
        level = level
            .windows(2)
            .map(|pair| lerp(&pair[0], &pair[1], half))
            .collect();
        left.push(level[0]);
        right.push(level[level.len() - 1]);
    }
    right.reverse();
    (left, right)
}

/// `(1 - t) a + t b`
fn lerp<T: Scalar>(a: &Homogeneous<T>, b: &Homogeneous<T>, t: T) -> Homogeneous<T> {
    let s = T::ONE - t;
    [
        a[0] * s + b[0] * t,
        a[1] * s + b[1] * t,
        a[2] * s + b[2] * t,
        a[3] * s + b[3] * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::{bezier_segments, nurbs_curve_minimum_distance};

    const TOLERANCE: f64 = 1e-9;

    /// 中心が原点・半径 1 の xy 平面上の円（次数 2 の有理曲線、4 つの 4 分円）
    fn unit_circle() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let coordinates = vec![
            1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 0.0, -1.0, 0.0, 0.0, -1.0,
            -1.0, 0.0, 0.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0,
        ];
        let weights = vec![1.0, s, 1.0, s, 1.0, s, 1.0, s, 1.0];
        let knots = vec![
            0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
        ];
        (knots, coordinates, weights)
    }

    fn distance_to(px: f64, py: f64, pz: f64) -> impl Fn(f64, f64, f64) -> f64 {
        move |x, y, z| ((x - px).powi(2) + (y - py).powi(2) + (z - pz).powi(2)).sqrt()
    }

    #[test]
    fn distance_from_points_to_a_rational_circle() {
        let (knots, coordinates, weights) = unit_circle();
        let minimum = |px, py, pz| {
            nurbs_curve_minimum_distance(
                2,
                &knots,
                &coordinates,
                Some(&weights),
                distance_to(px, py, pz),
                TOLERANCE,
            )
        };
        // 円の外・内・円の軸上の点
        assert!((minimum(3.0, 4.0, 0.0) - 4.0).abs() <= TOLERANCE);
        assert!((minimum(0.3, -0.4, 0.0) - 0.5).abs() <= TOLERANCE);
        assert!((minimum(0.0, 0.0, 1.0) - 2.0_f64.sqrt()).abs() <= TOLERANCE);
    }

    #[test]
    fn distance_to_a_plane_crossing_a_cubic_curve_is_zero() {
        // 次数 3 の非有理曲線（z が -1 から 1 へ変わる）と平面 z = 0
        let knots = vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0];
        let coordinates = vec![
            0.0, 0.0, -1.0, 1.0, 2.0, -1.0, 2.0, -1.0, 0.5, 3.0, 1.0, 1.0, 4.0, 0.0, 1.0,
        ];
        let minimum = nurbs_curve_minimum_distance(
            3,
            &knots,
            &coordinates,
            None,
            |_, _, z: f64| z.abs(),
            TOLERANCE,
        );
        assert!(minimum <= TOLERANCE);
        // 平面 z = 2 までの最短は z = 1 の端点
        let minimum = nurbs_curve_minimum_distance(
            3,
            &knots,
            &coordinates,
            None,
            |_, _, z: f64| (z - 2.0).abs(),
            TOLERANCE,
        );
        assert!((minimum - 1.0).abs() <= TOLERANCE);
    }

    #[test]
    fn bezier_segments_pass_through_the_knot_points() {
        let (knots, coordinates, weights) = unit_circle();
        let points: Vec<[f64; 4]> = coordinates
            .as_chunks::<3>()
            .0
            .iter()
            .zip(&weights)
            .map(|([x, y, z], w)| [x * w, y * w, z * w, *w])
            .collect();
        let segments = bezier_segments(2, &knots, points);
        assert_eq!(segments.len(), 4);
        for segment in segments {
            for end in [segment[0], segment[2]] {
                let radius = ((end[0] / end[3]).powi(2) + (end[1] / end[3]).powi(2)).sqrt();
                assert!((radius - 1.0).abs() < 1e-12);
            }
        }
    }
}
