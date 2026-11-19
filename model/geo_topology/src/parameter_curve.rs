//! 稜線の使用が持つパラメータ曲線（面の曲面のパラメータ (u, v) の上の曲線）
//!
//! パラメータ曲線は、稜線の母曲線と同じパラメータ `t` で (u, v) を返す（曲面の上で評価した点が、稜線の
//! 母曲線の点と許容誤差の中で合う）。周期的な曲面の上では、周期で折り返さず、つながった値で持つ。正確な形に
//! ならない組と NURBS 曲面の上では、NURBS 曲線を当てはめる（設計: BREP_AND_STEP_IMPORT_DESIGN.md 5.5）。

mod fitting;
#[cfg(test)]
mod tests;

use crate::{
    CurveRef, Point2D, SurfaceRef, TopoNurbsCurve2D, TopoNurbsCurve3D, TopoPlane3D, Vector2D,
};
use geo_contracts::{
    Circle3DProperties, Ellipse3DProperties, InfiniteLine3DProperties, NurbsCurve2DConstructor,
    NurbsCurve2DEvaluation, NurbsCurve2DProperties, NurbsCurve3DProperties, Plane3DProperties,
    Scalar,
};

/// 母曲線の区間を分ける数（当てはめと確かめに使う点は、区間の両端を含めて この数 + 1 個）
const SAMPLE_SEGMENTS: usize = 8;

/// パラメータ曲線
#[derive(Debug, Clone)]
pub enum ParameterCurve<T: Scalar> {
    /// 直線: (u, v) = `origin` + `t` × `direction`
    Line {
        origin: Point2D<T>,
        direction: Vector2D<T>,
    },
    /// 楕円: (u, v) = `center` + cos `t` × `major` + sin `t` × `minor`（円は、`major` と `minor` が同じ長さで直交する）
    Ellipse {
        center: Point2D<T>,
        major: Vector2D<T>,
        minor: Vector2D<T>,
    },
    /// NURBS 曲線（稜線の NURBS 曲線と同じノットベクトルの値）
    Nurbs(TopoNurbsCurve2D<T>),
}

impl<T: Scalar> ParameterCurve<T> {
    /// 母曲線のパラメータ `t` の (u, v)（NURBS で定義域の外なら `None`）
    pub fn point_at(&self, t: T) -> Option<Point2D<T>> {
        match self {
            Self::Line { origin, direction } => Some(*origin + *direction * t),
            Self::Ellipse {
                center,
                major,
                minor,
            } => Some(*center + *major * t.cos() + *minor * t.sin()),
            Self::Nurbs(curve) => curve.point_at_parameter(t),
        }
    }

    /// 曲面 `surface` の上で、母曲線 `curve` の区間 `range` を、ずれ `tolerance` の中で表すパラメータ曲線
    ///
    /// 正確な形（`exact`）が `tolerance` の中に収まればその形、収まらなければ当てはめた NURBS 曲線にする。
    /// 当てはめたずれが `tolerance` を超えれば `None`。
    pub(crate) fn new(
        surface: &SurfaceRef<T>,
        curve: &CurveRef<T>,
        range: (T, T),
        tolerance: T,
    ) -> Option<Self> {
        Self::exact(surface, curve, range, tolerance).or_else(|| {
            fitting::fit(surface, curve, range, tolerance)
                .and_then(|(fitted, deviation)| (deviation <= tolerance).then_some(fitted))
        })
    }

    /// 曲面 `surface` の上で、母曲線 `curve` の区間 `range` を表すパラメータ曲線（正確な形が `tolerance` の
    /// 中に収まればその形、収まらなければ `tolerance` を目安に当てはめた NURBS 曲線）の、曲面の上で評価した
    /// 点と母曲線の点のずれの最大（パラメータ曲線を作れなければ `None`）
    ///
    /// 当てはめたずれは `tolerance` を超えることがある。ずれの 1 倍以上に広げた許容誤差で `new` を呼べば、
    /// パラメータ曲線を作れる。NURBS 曲面の上では、曲面への最も近い点までの距離ではなく、このずれで稜線の
    /// 曲面からのずれを測る（最も近い点の探索は、曲面の離れた部分に落ちて、ないずれを返すことがあるため）。
    pub fn deviation(
        surface: &SurfaceRef<T>,
        curve: &CurveRef<T>,
        range: (T, T),
        tolerance: T,
    ) -> Option<T> {
        match Self::exact_candidate(surface, curve, range) {
            Some((_, deviation)) if deviation <= tolerance => Some(deviation),
            _ => fitting::fit(surface, curve, range, tolerance).map(|(_, deviation)| deviation),
        }
    }

    /// 曲面 `surface` の上で、母曲線 `curve` の区間 `range` を正確に表すパラメータ曲線
    ///
    /// 平面の上の直線・円・楕円・平面の中の NURBS 曲線と、円柱・円錐・球・トーラスの上で (u, v) の直線に
    /// なる曲線（母線・軸に垂直な円・経線・緯線）を作る。作った曲線を曲面の上で評価した点が、区間の中の点で
    /// 母曲線から `tolerance` より離れれば `None`（正確な形にならない組）。
    pub(crate) fn exact(
        surface: &SurfaceRef<T>,
        curve: &CurveRef<T>,
        range: (T, T),
        tolerance: T,
    ) -> Option<Self> {
        Self::exact_candidate(surface, curve, range)
            .and_then(|(candidate, deviation)| (deviation <= tolerance).then_some(candidate))
    }

    /// 正確な形の候補と、区間を分けた点で測ったずれの最大（候補を作れない・評価できなければ `None`）
    fn exact_candidate(
        surface: &SurfaceRef<T>,
        curve: &CurveRef<T>,
        range: (T, T),
    ) -> Option<(Self, T)> {
        let candidate = match surface {
            SurfaceRef::Plane(plane) => Self::on_plane(plane, curve)?,
            SurfaceRef::Nurbs(_) => return None,
            _ => Self::line_in_uv(surface, curve, range)?,
        };
        let deviation = (0..=SAMPLE_SEGMENTS).try_fold(T::ZERO, |deviation, index| {
            let t = sample(range, index);
            let uv = candidate.point_at(t)?;
            let point = surface.point_at_uv(uv.x(), uv.y())?;
            Some(deviation.max(point.distance_to(curve.point_at_parameter(t))))
        })?;
        Some((candidate, deviation))
    }

    /// 母曲線 `curve` の区間 `range` の、解析的な曲面 `surface` からのずれの最大（区間を分けた点から、曲面の
    /// 上の近い点までの距離で測る。NURBS 曲面では `None`。NURBS 曲面の上のずれは `deviation` で測る）
    ///
    /// 平面・円柱・球・トーラスでは最も近い点までの距離、円錐では軸の方向の高さが同じ点までの距離（最も
    /// 近い点までの距離を、半頂角の余弦で割った値。平たい円錐では大きくなる）になる。円錐でも、
    /// パラメータ曲線の確かめ（曲面の上で評価した点と母曲線の点の距離）と同じ測り方なので、このずれの
    /// 中に許容誤差を広げれば、正確な形のパラメータ曲線の確かめを通る。
    pub fn surface_deviation(
        surface: &SurfaceRef<T>,
        curve: &CurveRef<T>,
        range: (T, T),
    ) -> Option<T> {
        (0..=SAMPLE_SEGMENTS).try_fold(T::ZERO, |deviation, index| {
            let point = curve.point_at_parameter(sample(range, index));
            let (u, v) = surface.uv_of(point)?;
            let on_surface = surface.point_at_uv(u, v)?;
            Some(deviation.max(on_surface.distance_to(point)))
        })
    }

    /// 平面の座標に写した曲線
    fn on_plane(plane: &TopoPlane3D<T>, curve: &CurveRef<T>) -> Option<Self> {
        let u_axis = plane.u_axis().as_vector();
        let v_axis = plane.v_axis().as_vector();
        let point = |point: crate::Point3D<T>| {
            let offset = point - plane.origin();
            Point2D::new(offset.dot(u_axis), offset.dot(v_axis))
        };
        let vector =
            |vector: crate::Vector3D<T>| Vector2D::new(vector.dot(u_axis), vector.dot(v_axis));
        match curve {
            CurveRef::Line(line) => Some(Self::Line {
                origin: point(line.point()),
                direction: vector(line.direction().as_vector()),
            }),
            CurveRef::Circle(circle) => {
                let x = circle.ref_direction().as_vector();
                let y = circle.axis().as_vector().cross(x);
                Some(Self::Ellipse {
                    center: point(circle.center()),
                    major: vector(x * circle.radius()),
                    minor: vector(y * circle.radius()),
                })
            }
            CurveRef::Ellipse(ellipse) => Some(Self::Ellipse {
                center: point(ellipse.center()),
                major: vector(
                    ellipse.major_axis_direction().as_vector() * ellipse.semi_major_axis(),
                ),
                minor: vector(
                    ellipse.minor_axis_direction().as_vector() * ellipse.semi_minor_axis(),
                ),
            }),
            CurveRef::Nurbs(nurbs) => {
                let points: Vec<Point2D<T>> = (0..nurbs.control_point_count())
                    .map(|index| nurbs.control_point(index).map(point))
                    .collect::<Option<_>>()?;
                let curve = <TopoNurbsCurve2D<T> as NurbsCurve2DConstructor<T>>::new(
                    nurbs.degree(),
                    <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::knot_vector(nurbs).to_vec(),
                    &points,
                    nurbs.weights().map(<[T]>::to_vec),
                )
                .ok()?;
                Some(Self::Nurbs(curve))
            }
        }
    }

    /// 区間の中の点の (u, v) を、周期で折り返さずにつないで、通る直線（両端の点の (u, v) は使わない。球の極
    /// のように、端の点の u が決まらないことがあるため）
    fn line_in_uv(surface: &SurfaceRef<T>, curve: &CurveRef<T>, range: (T, T)) -> Option<Self> {
        let (u_period, v_period) = surface.periods();
        let mut first: Option<(T, Point2D<T>)> = None;
        let mut last: Option<(T, Point2D<T>)> = None;
        for index in 1..SAMPLE_SEGMENTS {
            let t = sample(range, index);
            let (u, v) = surface.uv_of(curve.point_at_parameter(t))?;
            let uv = match last {
                Some((_, previous)) => Point2D::new(
                    unwrap(u, previous.x(), u_period),
                    unwrap(v, previous.y(), v_period),
                ),
                None => Point2D::new(u, v),
            };
            first.get_or_insert((t, uv));
            last = Some((t, uv));
        }
        let ((t_first, uv_first), (t_last, uv_last)) = (first?, last?);
        let direction = (uv_last - uv_first) * (T::ONE / (t_last - t_first));
        Some(Self::Line {
            origin: uv_first + direction * (-t_first),
            direction,
        })
    }

    /// (u, v) を (`du`, `dv`) だけ動かした曲線（周期の分だけ動かして、ループの中でつなぐときに使う）
    pub(crate) fn shifted(&self, du: T, dv: T) -> Self {
        let offset = Vector2D::new(du, dv);
        match self {
            Self::Line { origin, direction } => Self::Line {
                origin: *origin + offset,
                direction: *direction,
            },
            Self::Ellipse {
                center,
                major,
                minor,
            } => Self::Ellipse {
                center: *center + offset,
                major: *major,
                minor: *minor,
            },
            Self::Nurbs(curve) => {
                let points: Vec<Point2D<T>> = (0..curve.control_point_count())
                    .filter_map(|index| curve.control_point(index))
                    .map(|point| point + offset)
                    .collect();
                Self::Nurbs(
                    <TopoNurbsCurve2D<T> as NurbsCurve2DConstructor<T>>::new(
                        curve.degree(),
                        <TopoNurbsCurve2D<T> as NurbsCurve2DProperties<T>>::knot_vector(curve)
                            .to_vec(),
                        &points,
                        <TopoNurbsCurve2D<T> as NurbsCurve2DProperties<T>>::weights(curve)
                            .map(<[T]>::to_vec),
                    )
                    .expect("moving the control points keeps a valid NURBS curve"),
                )
            }
        }
    }
}

/// 区間 `range` を `SAMPLE_SEGMENTS` に分けた `index` 番目の点
fn sample<T: Scalar>(range: (T, T), index: usize) -> T {
    range.0 + (range.1 - range.0) * T::from_f64(index as f64 / SAMPLE_SEGMENTS as f64)
}

/// `value` を、周期 `period` の分だけ動かして `reference` に近づけた値（周期がなければ、そのまま）
pub(crate) fn unwrap<T: Scalar>(value: T, reference: T, period: Option<T>) -> T {
    match period {
        Some(period) => value + ((reference - value) / period).round() * period,
        None => value,
    }
}
