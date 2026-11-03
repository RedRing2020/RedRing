//! NURBS 曲線上の距離の最小値
//!
//! 曲線を Bézier の区間に分解し、分枝限定法で、相手の形状までの距離が最小になる曲線上の点の距離を求める。
//! 区間は重みを掛けた同次座標で扱い、ノット挿入（Boehm の方法）と de Casteljau 法で分割する。

use analysis::Scalar;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// 分枝限定法で区間を分割する回数の上限
///
/// 区間は分割のたびに半分になり、区間の下界と上界の差も小さくなる。上限は、許容誤差に届くまでの分割を
/// 十分に上回る回数とする（届かない場合は、それまでに得た曲線上の点の距離の最小値を返す）。
const NURBS_CURVE_MINIMUM_DISTANCE_MAX_SPLITS: usize = 100_000;

/// 同次座標の点 `(w x, w y, w z, w)`
type Homogeneous<T> = [T; 4];

/// 曲線の相手の形状までの距離の場
pub trait DistanceField<T: Scalar> {
    /// 点から相手の形状までの距離。点が `δ` 動いたときの変化が `δ` 以下であること
    fn distance(&self, point: [T; 3]) -> T;

    /// 点の集まり `points` の凸包上での距離の下界
    ///
    /// 既定は 0。探索は、制御点を囲む球による下界（`distance(c) - ρ`）と、この下界の大きい方を使う。
    fn hull_lower_bound(&self, _points: &[[T; 3]]) -> T {
        T::ZERO
    }
}

/// 凸な形状（点・直線・平面・凸な立体等）までの距離の場
///
/// 点 `c` の最近点 `q` で形状を支える平面（法線 `n = (c - q) / |c - q|`）の向こう側に形状があるので、
/// 凸包上の距離の下界は `min(n · (P - q))`（`P` は凸包の頂点）。平面に平行な曲線では正確な下界になり、
/// 曲がった配置でも区間の大きさの 2 乗で小さくなる。
pub struct ConvexSetDistance<F> {
    closest_point: F,
}

impl<F> ConvexSetDistance<F> {
    /// `closest_point` は点に最も近い形状上の点（形状の内部の点はその点）
    pub fn new(closest_point: F) -> Self {
        Self { closest_point }
    }
}

impl<T: Scalar, F: Fn([T; 3]) -> [T; 3]> DistanceField<T> for ConvexSetDistance<F> {
    fn distance(&self, point: [T; 3]) -> T {
        length(sub(point, (self.closest_point)(point)))
    }

    fn hull_lower_bound(&self, points: &[[T; 3]]) -> T {
        let center = centroid(points);
        let closest = (self.closest_point)(center);
        let offset = sub(center, closest);
        let offset_length = length(offset);
        if offset_length == T::ZERO {
            return T::ZERO;
        }
        let normal = scale(offset, T::ONE / offset_length);
        points
            .iter()
            .map(|point| dot(normal, sub(*point, closest)))
            .fold(T::INFINITY, T::min)
            .max(T::ZERO)
    }
}

/// 円（曲線）までの距離の場
///
/// 円の軸からの距離 `ρ` と円の平面からの高さ `z` で、距離は `√((ρ - radius)² + z²)`。凸包上の `z` の範囲は
/// 頂点の `z` の範囲、`ρ` の最大値は頂点の `ρ` の最大値（`ρ` は凸関数）、`ρ` の最小値は軸を支える平面による
/// 下界で求め、下界は `(ρ, z)` 平面での点 `(radius, 0)` から範囲の長方形までの距離とする。
pub struct CircleDistance<T: Scalar> {
    center: [T; 3],
    normal: [T; 3],
    radius: T,
}

impl<T: Scalar> CircleDistance<T> {
    /// `normal` は円の平面の単位法線
    pub fn new(center: [T; 3], normal: [T; 3], radius: T) -> Self {
        Self {
            center,
            normal,
            radius,
        }
    }

    /// 中心からの、円の平面に沿った成分と高さ
    fn radial_and_height(&self, point: [T; 3]) -> ([T; 3], T) {
        let offset = sub(point, self.center);
        let height = dot(offset, self.normal);
        (sub(offset, scale(self.normal, height)), height)
    }
}

impl<T: Scalar> DistanceField<T> for CircleDistance<T> {
    fn distance(&self, point: [T; 3]) -> T {
        let (radial, height) = self.radial_and_height(point);
        let gap = length(radial) - self.radius;
        (gap * gap + height * height).sqrt()
    }

    fn hull_lower_bound(&self, points: &[[T; 3]]) -> T {
        let (mut height_min, mut height_max, mut radial_max) = (T::INFINITY, -T::INFINITY, T::ZERO);
        for point in points {
            let (radial, height) = self.radial_and_height(*point);
            height_min = height_min.min(height);
            height_max = height_max.max(height);
            radial_max = radial_max.max(length(radial));
        }
        let (center_radial, _) = self.radial_and_height(centroid(points));
        let center_radial_length = length(center_radial);
        let radial_min = if center_radial_length == T::ZERO {
            T::ZERO
        } else {
            let direction = scale(center_radial, T::ONE / center_radial_length);
            points
                .iter()
                .map(|point| dot(direction, sub(*point, self.center)))
                .fold(T::INFINITY, T::min)
                .max(T::ZERO)
        };
        let radial_gap = (radial_min - self.radius)
            .max(self.radius - radial_max)
            .max(T::ZERO);
        let height_gap = height_min.max(-height_max).max(T::ZERO);
        (radial_gap * radial_gap + height_gap * height_gap).sqrt()
    }
}

/// NURBS 曲線上で、相手の形状までの距離 `field` が最小になる点の距離
///
/// - `coordinates`: 制御点の座標を `x, y, z` の順に並べた配列
/// - `weights`: 制御点の重み（`None` は重みがすべて 1）。重みは正
/// - `knots`: 両端の重複度が `degree + 1` 以上のノット
///
/// 返す距離は曲線上の点での距離で、真の最小値以上、真の最小値 + `tolerance` 以下になる（分割の回数の上限に
/// 達した場合は、曲線上の点での距離を返すが、真の最小値 + `tolerance` 以下は保証しない）。
///
/// # 計算アルゴリズム
/// 各区間の曲線は制御点の凸包の中にあり、凸包は制御点を囲む球（中心 `c`・半径 `ρ`）の中にある。区間の下界は
/// `max(0, distance(c) - ρ, hull_lower_bound(制御点))`、上界は区間の両端（曲線上の点）での距離。下界が最も
/// 小さい区間を半分に分けることを、上界の最小値と下界の最小値の差が `tolerance` 以下になるまで繰り返す。
pub fn nurbs_curve_minimum_distance<T: Scalar>(
    degree: usize,
    knots: &[T],
    coordinates: &[T],
    weights: Option<&[T]>,
    field: &(impl DistanceField<T> + ?Sized),
    tolerance: T,
) -> T {
    minimum_distance_and_splits(degree, knots, coordinates, weights, field, tolerance).0
}

/// 距離の最小値と、そこまでに区間を分割した回数
fn minimum_distance_and_splits<T: Scalar>(
    degree: usize,
    knots: &[T],
    coordinates: &[T],
    weights: Option<&[T]>,
    field: &(impl DistanceField<T> + ?Sized),
    tolerance: T,
) -> (T, usize) {
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
    let distance_at = |point: &Homogeneous<T>| field.distance(project(point));

    let mut best = T::INFINITY;
    let mut queue = BinaryHeap::new();
    for points in bezier_segments(degree, knots, control_points) {
        best = best.min(distance_at(&points[0]).min(distance_at(&points[degree])));
        queue.push(Segment::new(points, field));
    }

    for splits in 0..NURBS_CURVE_MINIMUM_DISTANCE_MAX_SPLITS {
        let Some(segment) = queue.pop() else {
            return (best, splits);
        };
        if best - segment.lower_bound <= tolerance {
            return (best, splits);
        }
        let (left, right) = split_bezier(&segment.points);
        best = best.min(distance_at(&right[0]));
        for half in [left, right] {
            let half = Segment::new(half, field);
            if best - half.lower_bound > tolerance {
                queue.push(half);
            }
        }
    }
    (best, NURBS_CURVE_MINIMUM_DISTANCE_MAX_SPLITS)
}

/// Bézier の区間の制御点と、区間の曲線上の点までの距離の下界
struct Segment<T: Scalar> {
    points: Vec<Homogeneous<T>>,
    lower_bound: T,
}

impl<T: Scalar> Segment<T> {
    fn new(points: Vec<Homogeneous<T>>, field: &(impl DistanceField<T> + ?Sized)) -> Self {
        let projected: Vec<[T; 3]> = points.iter().map(project).collect();
        let center = centroid(&projected);
        let radius = projected
            .iter()
            .map(|point| length(sub(*point, center)))
            .fold(T::ZERO, T::max);
        let lower_bound = (field.distance(center) - radius)
            .max(field.hull_lower_bound(&projected))
            .max(T::ZERO);
        Self {
            points,
            lower_bound,
        }
    }
}

fn project<T: Scalar>(point: &Homogeneous<T>) -> [T; 3] {
    [
        point[0] / point[3],
        point[1] / point[3],
        point[2] / point[3],
    ]
}

fn centroid<T: Scalar>(points: &[[T; 3]]) -> [T; 3] {
    let count = T::from_usize(points.len());
    let sum = points.iter().fold([T::ZERO; 3], |sum, point| {
        [sum[0] + point[0], sum[1] + point[1], sum[2] + point[2]]
    });
    scale(sum, T::ONE / count)
}

fn sub<T: Scalar>(a: [T; 3], b: [T; 3]) -> [T; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale<T: Scalar>(a: [T; 3], factor: T) -> [T; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn dot<T: Scalar>(a: [T; 3], b: [T; 3]) -> T {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length<T: Scalar>(a: [T; 3]) -> T {
    dot(a, a).sqrt()
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
    use super::{
        bezier_segments, minimum_distance_and_splits, CircleDistance, ConvexSetDistance,
        DistanceField,
    };

    const TOLERANCE: f64 = 1e-9;
    /// 既定の距離トレランスと同じ値
    const DEFAULT_TOLERANCE: f64 = 1e-6;
    /// 距離がほぼ一定の配置で、区間の大きさの 2 乗で下界が近づくなら収まる分割の回数（制御点を囲む球だけの
    /// 下界では、区間を許容誤差の大きさまで分割するため上限の 100,000 回に達する）
    const FEW_SPLITS: usize = 5_000;

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

    fn circle_minimum(field: &impl DistanceField<f64>) -> (f64, usize) {
        circle_minimum_with(field, TOLERANCE)
    }

    fn circle_minimum_with(
        field: &(impl DistanceField<f64> + ?Sized),
        tolerance: f64,
    ) -> (f64, usize) {
        let (knots, coordinates, weights) = unit_circle();
        minimum_distance_and_splits(2, &knots, &coordinates, Some(&weights), field, tolerance)
    }

    fn point_field(point: [f64; 3]) -> ConvexSetDistance<impl Fn([f64; 3]) -> [f64; 3]> {
        ConvexSetDistance::new(move |_| point)
    }

    /// 平面 z = height（z 方向の成分を height に置き換える）
    fn horizontal_plane(height: f64) -> ConvexSetDistance<impl Fn([f64; 3]) -> [f64; 3]> {
        ConvexSetDistance::new(move |[x, y, _]: [f64; 3]| [x, y, height])
    }

    #[test]
    fn distance_from_points_to_a_rational_circle() {
        // 円の外・内の点
        assert!((circle_minimum(&point_field([3.0, 4.0, 0.0])).0 - 4.0).abs() <= TOLERANCE);
        assert!((circle_minimum(&point_field([0.3, -0.4, 0.0])).0 - 0.5).abs() <= TOLERANCE);
    }

    #[test]
    fn nearly_constant_distances_need_few_splits() {
        let cases: [(&dyn DistanceField<f64>, f64); 3] = [
            // 円の軸上の点
            (&point_field([0.0, 0.0, 1.0]), 2.0_f64.sqrt()),
            // 円に平行な平面
            (&horizontal_plane(2.0), 2.0),
            // 同心の円
            (
                &CircleDistance::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0),
                1.0,
            ),
        ];
        for (field, expected) in cases {
            let (distance, splits) = circle_minimum_with(field, DEFAULT_TOLERANCE);
            assert!(
                (distance - expected).abs() <= DEFAULT_TOLERANCE,
                "distance: {distance}"
            );
            assert!(splits < FEW_SPLITS, "splits: {splits}");
        }
    }

    #[test]
    fn circle_lower_bound_matches_dense_evaluation() {
        // 傾いた円までの距離を、曲線上の細かい点の距離の最小値と比べる
        let length = (0.2_f64 * 0.2 + 1.0).sqrt();
        let tilted = CircleDistance::new([0.3, 0.2, 0.5], [0.2 / length, 0.0, 1.0 / length], 0.8);
        let (distance, _) = circle_minimum(&tilted);
        let sampled = (0..=100_000)
            .map(|i| {
                let angle = std::f64::consts::TAU * f64::from(i) / 100_000.0;
                tilted.distance([angle.cos(), angle.sin(), 0.0])
            })
            .fold(f64::INFINITY, f64::min);
        assert!(distance <= sampled + TOLERANCE);
        assert!(distance >= sampled - 1e-8);
    }

    #[test]
    fn distance_to_a_plane_crossing_a_cubic_curve_is_zero() {
        // 次数 3 の非有理曲線（z が -1 から 1 へ変わる）と平面 z = 0
        let knots = vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0];
        let coordinates = vec![
            0.0, 0.0, -1.0, 1.0, 2.0, -1.0, 2.0, -1.0, 0.5, 3.0, 1.0, 1.0, 4.0, 0.0, 1.0,
        ];
        let minimum = |height| {
            minimum_distance_and_splits(
                3,
                &knots,
                &coordinates,
                None,
                &horizontal_plane(height),
                TOLERANCE,
            )
            .0
        };
        assert!(minimum(0.0) <= TOLERANCE);
        // 平面 z = 2 までの最短は z = 1 の端点
        assert!((minimum(2.0) - 1.0).abs() <= TOLERANCE);
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
