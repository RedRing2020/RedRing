//! 正確な形にならない組と NURBS 曲面の上の、パラメータ曲線の当てはめ
//!
//! 母曲線の区間を小区間に分け、小区間ごとに、両端と 1/3・2/3 の 4 点を曲面に写した (u, v) を通る 3 次の
//! Bezier にする。写した点と、小区間の中の 3 点（4 点を通す 3 次の補間の誤差の主な項が大きくなる、
//! 1/2 と 1/2 ± √5/6）で、曲面の上で評価した点と母曲線の点のずれを測り、中の 3 点のずれが `tolerance` を超える小区間を 2 つに分ける。ただし、母曲線が曲面から離れて
//! いて、写した点のずれ（曲面からの離れ）が `tolerance` を超えるときは、中の 3 点のずれが、写した点のずれに
//! `tolerance` を足した値を超える小区間だけ分ける（分けても、曲面からの離れより小さくならないため）。小区間を、
//! つなぎ目で C0 の、有理でない 3 次の NURBS 曲線にまとめる（設計: BREP_AND_STEP_IMPORT_DESIGN.md 5.5）。

#[cfg(test)]
mod tests;

use super::{ParameterCurve, SAMPLE_SEGMENTS, sample, unwrap};
use crate::{CurveRef, Point2D, SurfaceRef, TopoNurbsCurve2D, TopoNurbsSurface3D};
use geo_contracts::{
    NurbsCurve2DConstructor, NurbsCurve3DProperties, NurbsSurface3DEvaluation,
    NurbsSurface3DProperties, Scalar,
};

/// 小区間を 2 つに分ける深さの上限（初めの小区間 1 つが、最大で 2 の この数 乗の小区間に分かれる）
const MAX_SPLIT_DEPTH: usize = 10;
/// NURBS 曲面の上の初めの点を探す格子の、1 方向の分け方の上限
const MAX_GRID_SEGMENTS: usize = 32;
/// NURBS 曲面の上の初めの点を探す格子の、制御点 1 つあたりの分け方
const GRID_SEGMENTS_PER_CONTROL_POINT: usize = 2;
/// 当てはめる曲線の次数
const DEGREE: usize = 3;

/// 曲面 `surface` の上に当てはめた、母曲線 `curve` の区間 `range` のパラメータ曲線と、ずれの最大
///
/// ずれは、小区間ごとに測った点での、曲面の上で評価した点と母曲線の点の距離の最大（測った点の間では、
/// わずかに超えることがある）。分ける深さの上限まで分けても `tolerance` に収まらない小区間は、そのまま使う
/// （ずれは `tolerance` を超える）。曲面の点を求められない、またはずれが有限の値でなければ `None`。
pub(super) fn fit<T: Scalar>(
    surface: &SurfaceRef<T>,
    curve: &CurveRef<T>,
    range: (T, T),
    tolerance: T,
) -> Option<(ParameterCurve<T>, T)> {
    let mapper = Mapper {
        surface,
        curve,
        tolerance,
    };
    let breaks = initial_breaks(curve, range);
    // 初めの点が極なら、u は極の線の上のどこでもよいので、すぐ隣の点の u にする
    let first = mapper.uv(range.0, None)?;
    let first = if surface
        .singular_v(curve.point_at_parameter(range.0), tolerance)
        .is_some()
    {
        let next = mapper.uv(range.0 + (breaks[1] - breaks[0]) / T::from_f64(3.0), None)?;
        Point2D::new(next.x(), first.y())
    } else {
        first
    };

    let mut knots = vec![range.0; DEGREE + 1];
    let mut points = vec![first];
    let mut deviation = mapper.deviation(range.0, first);
    let mut current = first;
    // 左の小区間から順に取り出すため、右から積む
    let mut pending: Vec<(T, T, usize)> = breaks
        .windows(2)
        .rev()
        .map(|pair| (pair[0], pair[1], 0))
        .collect();
    while let Some((start, end, depth)) = pending.pop() {
        let (piece, residual, interpolation) = mapper.piece(start, end, current)?;
        if interpolation > tolerance + residual.max(T::ZERO) && depth < MAX_SPLIT_DEPTH {
            let middle = (start + end) / T::from_f64(2.0);
            pending.push((middle, end, depth + 1));
            pending.push((start, middle, depth + 1));
            continue;
        }
        deviation = deviation.max(residual).max(interpolation);
        points.extend(piece);
        knots.extend([end; DEGREE]);
        current = piece[2];
    }
    // 終わりの点は、定義域の端なので次数 + 1 回並べる
    knots.push(range.1);
    if !deviation.is_finite() {
        return None;
    }
    let curve =
        <TopoNurbsCurve2D<T> as NurbsCurve2DConstructor<T>>::new(DEGREE, knots, &points, None)
            .ok()?;
    Some((ParameterCurve::Nurbs(curve), deviation))
}

/// 初めの小区間の境（区間を `SAMPLE_SEGMENTS` に分けた点と、NURBS 曲線の区間の中のノット。小さい順）
///
/// ノットで分けるのは、NURBS 曲線がノットで折れることがあり、折れた点を小区間の中に含めないため。
fn initial_breaks<T: Scalar>(curve: &CurveRef<T>, range: (T, T)) -> Vec<T> {
    // 両端は、区間の端の値そのものにする（分けた点の計算の丸めで、定義域の端がずれないため）
    let mut breaks: Vec<T> = std::iter::once(range.0)
        .chain((1..SAMPLE_SEGMENTS).map(|index| sample(range, index)))
        .chain(std::iter::once(range.1))
        .collect();
    if let CurveRef::Nurbs(nurbs) = curve {
        breaks.extend(
            nurbs
                .knot_vector()
                .iter()
                .copied()
                .filter(|&knot| knot > range.0 && knot < range.1),
        );
    }
    breaks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    breaks.dedup();
    breaks
}

/// 母曲線の点を曲面の (u, v) に写す
struct Mapper<'a, T: Scalar> {
    surface: &'a SurfaceRef<T>,
    curve: &'a CurveRef<T>,
    tolerance: T,
}

impl<T: Scalar> Mapper<'_, T> {
    /// 母曲線のパラメータ `t` の点の (u, v)（`previous` は、母曲線に沿って前の点の (u, v)）
    ///
    /// 解析的な曲面では、点の (u, v) を、周期の分だけ動かして前の点に近づける。極の点の u は、前の点の u に
    /// する。NURBS 曲面では、前の点（初めの点は、曲面の上の格子の点で最も近いもの）から詰める。
    fn uv(&self, t: T, previous: Option<Point2D<T>>) -> Option<Point2D<T>> {
        let point = self.curve.point_at_parameter(t);
        if let SurfaceRef::Nurbs(surface) = self.surface {
            let start = match previous {
                Some(previous) => (previous.x(), previous.y()),
                None => nearest_grid_point(surface, point)?,
            };
            let (u, v) = surface.parameter_for_point_from(point, start, self.tolerance)?;
            return Some(Point2D::new(u, v));
        }
        let (u, v) = self.surface.uv_of(point)?;
        let Some(previous) = previous else {
            return Some(Point2D::new(u, v));
        };
        let (u_period, v_period) = self.surface.periods();
        let u = if self.surface.singular_v(point, self.tolerance).is_some() {
            previous.x()
        } else {
            unwrap(u, previous.x(), u_period)
        };
        Some(Point2D::new(u, unwrap(v, previous.y(), v_period)))
    }

    /// (u, v) を曲面の上で評価した点と、母曲線のパラメータ `t` の点の距離（曲面の点がなければ無限大）
    fn deviation(&self, t: T, uv: Point2D<T>) -> T {
        let on_curve = self.curve.point_at_parameter(t);
        self.surface
            .point_at_uv(uv.x(), uv.y())
            .map_or(T::INFINITY, |point| point.distance_to(on_curve))
    }

    /// 小区間 `start`〜`end`（初めの点の (u, v) は `first`）の Bezier の、初めの点の次からの 3 つの制御点と、
    /// 写した 3 点のずれの最大と、小区間の中の 3 点のずれの最大
    fn piece(&self, start: T, end: T, first: Point2D<T>) -> Option<([Point2D<T>; 3], T, T)> {
        let at = |fraction: f64| start + (end - start) * T::from_f64(fraction);
        let third = self.uv(at(1.0 / 3.0), Some(first))?;
        let two_thirds = self.uv(at(2.0 / 3.0), Some(third))?;
        let last = self.uv(end, Some(two_thirds))?;
        // 1/3・2/3 で 2 つの点を通る 3 次の Bezier の、中の 2 つの制御点
        let six = T::from_f64(6.0);
        let inner = |a: Point2D<T>, b: Point2D<T>, c: Point2D<T>, d: Point2D<T>| {
            Point2D::new(
                (T::from_f64(-5.0) * a.x() + T::from_f64(18.0) * b.x() - T::from_f64(9.0) * c.x()
                    + T::from_f64(2.0) * d.x())
                    / six,
                (T::from_f64(-5.0) * a.y() + T::from_f64(18.0) * b.y() - T::from_f64(9.0) * c.y()
                    + T::from_f64(2.0) * d.y())
                    / six,
            )
        };
        let control = [
            first,
            inner(first, third, two_thirds, last),
            inner(last, two_thirds, third, first),
            last,
        ];
        let mut residual = T::ZERO;
        for (fraction, uv) in [(1.0 / 3.0, third), (2.0 / 3.0, two_thirds), (1.0, last)] {
            residual = residual.max(self.deviation(at(fraction), uv));
        }
        let mut interpolation = T::ZERO;
        // 補間の誤差の主な項は τ(τ - 1/3)(τ - 2/3)(τ - 1) に比例し、1/2 ± √5/6 で最も大きく、1/2 で極になる
        let offset = 5.0_f64.sqrt() / 6.0;
        for fraction in [0.5 - offset, 0.5, 0.5 + offset] {
            interpolation =
                interpolation.max(self.deviation(at(fraction), bezier(&control, fraction)));
        }
        Some((
            [control[1], control[2], control[3]],
            residual,
            interpolation,
        ))
    }
}

/// 3 次の Bezier の、パラメータ `fraction`（0〜1）の点
fn bezier<T: Scalar>(control: &[Point2D<T>; 4], fraction: f64) -> Point2D<T> {
    let s = T::from_f64(fraction);
    let r = T::ONE - s;
    let three = T::from_f64(3.0);
    let weights = [r * r * r, three * r * r * s, three * r * s * s, s * s * s];
    let (x, y) = control
        .iter()
        .zip(weights)
        .fold((T::ZERO, T::ZERO), |(x, y), (point, weight)| {
            (x + point.x() * weight, y + point.y() * weight)
        });
    Point2D::new(x, y)
}

/// 曲面の上の格子の点のうち、点 `point` に最も近いものの (u, v)
fn nearest_grid_point<T: Scalar>(
    surface: &TopoNurbsSurface3D<T>,
    point: crate::Point3D<T>,
) -> Option<(T, T)> {
    let ((u_start, u_end), (v_start, v_end)) = surface.parameter_domain();
    let segments =
        |count: usize| (count * GRID_SEGMENTS_PER_CONTROL_POINT).clamp(1, MAX_GRID_SEGMENTS);
    let (u_segments, v_segments) = (segments(surface.u_count()), segments(surface.v_count()));
    let mut best: Option<(T, (T, T))> = None;
    for i in 0..=u_segments {
        let u = u_start + (u_end - u_start) * T::from_f64(i as f64 / u_segments as f64);
        for j in 0..=v_segments {
            let v = v_start + (v_end - v_start) * T::from_f64(j as f64 / v_segments as f64);
            let distance = surface.point_at_uv(u, v)?.distance_to(point);
            if best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, (u, v)));
            }
        }
    }
    best.map(|(_, uv)| uv)
}
