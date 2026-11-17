//! 曲線の区間を、許容弦誤差と角度の上限で点に分ける（設計: TRIMMED_FACE_TESSELLATION_DESIGN.md 5.1）
//!
//! 弦（隣り合う点を結ぶ線分）と曲線の離れが許容弦誤差以下になり、隣り合う弦のなす角が角度の上限以下になる
//! ように、区間の両端を含めた点を、パラメータの増える順に返す（曲線そのものが折れる点（ノットで折れる NURBS 曲線）
//! では、そこで隣り合う弦のなす角は上限を超えうる）。
//!
//! - 直線: 分けない（両端だけ）
//! - 円: 角の刻み `θ(r) = 2 acos(max(1 − ε / r, cos(θmax / 2)))` で等しく分ける
//! - 楕円: 許容弦誤差の刻みを長半径で、角度の上限の刻みを短半径の側（`θmax × 短半径 / 長半径`）で決め、小さいほうで
//!   等しく分ける（長軸の端では、隣り合う弦のなす角が刻みの 長半径 / 短半径 倍になるため）
//! - NURBS 曲線: 区間の中のノットで分けた後、弦と曲線の離れ（区間を 8 つに分けた 7 点で測る）が許容弦誤差を超えるか、
//!   区間の前半と後半の弦のなす角が上限の半分を超える区間を 2 つに分ける（接線はノットで折れることがあるため、弦で
//!   測る）。その後、隣り合う弦のなす角をつなぎ目で確かめ、超えれば両隣の弦を分ける

use geo_contracts::{
    Circle3DEvaluation, Circle3DProperties, Ellipse3DEvaluation, Ellipse3DProperties,
    InfiniteLine3DEvaluation, NurbsCurve3DEvaluation, NurbsCurve3DProperties,
};

use crate::{
    Angle, Circle3D, Ellipse3D, InfiniteLine3D, LineSegment3D, LineSegment3DConstructor,
    LineSegment3DDistance, NurbsCurve3D, Point3D, Vector3D,
};

/// 曲線を点に分ける設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurveDiscretizationSettings {
    /// 許容弦誤差（弦と曲線の離れの上限。正）
    pub chord_tolerance: f64,
    /// 隣り合う弦のなす角の上限（0 より大きく 180 度より小さい）
    pub max_angle: Angle<f64>,
    /// 1 つの区間から作る点の数の上限
    pub max_points: usize,
}

/// 曲線の上の点（曲線のパラメータと点）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvePoint {
    pub parameter: f64,
    pub point: Point3D<f64>,
}

/// 曲線を点に分けられない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveDiscretizationError {
    /// 設定が正しくない（許容弦誤差が正の有限の値でない、角度の上限が範囲の外、点の数の上限が 2 より小さい）
    InvalidSettings,
    /// 区間が正しくない（有限の値でない、空、NURBS 曲線の定義域の外）
    InvalidRange,
    /// 点の数が上限を超える
    TooManyPoints,
    /// NURBS 曲線を評価できない
    Evaluation,
}

impl std::fmt::Display for CurveDiscretizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidSettings => "invalid curve discretization settings",
            Self::InvalidRange => "invalid curve range",
            Self::TooManyPoints => "too many points",
            Self::Evaluation => "the curve cannot be evaluated",
        })
    }
}

impl std::error::Error for CurveDiscretizationError {}

/// 区間を点に分けられる曲線
pub trait DiscretizableCurve {
    /// 区間 `range`（パラメータの小さいほうから大きいほう）を、設定 `settings` で点に分ける（両端を含み、パラメータの
    /// 増える順）
    ///
    /// # Errors
    ///
    /// 設定・区間が正しくない、点の数が上限を超える、曲線を評価できないとき。
    fn discretize(
        &self,
        range: (f64, f64),
        settings: &CurveDiscretizationSettings,
    ) -> Result<Vec<CurvePoint>, CurveDiscretizationError>;
}

impl DiscretizableCurve for InfiniteLine3D<f64> {
    fn discretize(
        &self,
        range: (f64, f64),
        settings: &CurveDiscretizationSettings,
    ) -> Result<Vec<CurvePoint>, CurveDiscretizationError> {
        check(range, settings)?;
        Ok([range.0, range.1]
            .into_iter()
            .map(|parameter| CurvePoint {
                parameter,
                point: self.point_at_parameter(parameter),
            })
            .collect())
    }
}

impl DiscretizableCurve for Circle3D<f64> {
    fn discretize(
        &self,
        range: (f64, f64),
        settings: &CurveDiscretizationSettings,
    ) -> Result<Vec<CurvePoint>, CurveDiscretizationError> {
        check(range, settings)?;
        let step = angle_step(self.radius(), settings);
        uniform(range, step, settings, |t| self.point_at_parameter(t))
    }
}

impl DiscretizableCurve for Ellipse3D<f64> {
    fn discretize(
        &self,
        range: (f64, f64),
        settings: &CurveDiscretizationSettings,
    ) -> Result<Vec<CurvePoint>, CurveDiscretizationError> {
        check(range, settings)?;
        let (major, minor) = (self.semi_major_axis(), self.semi_minor_axis());
        let step = angle_step(major, settings).min(settings.max_angle.to_radians() * minor / major);
        uniform(range, step, settings, |t| self.point_at_parameter(t))
    }
}

impl DiscretizableCurve for NurbsCurve3D<f64> {
    fn discretize(
        &self,
        range: (f64, f64),
        settings: &CurveDiscretizationSettings,
    ) -> Result<Vec<CurvePoint>, CurveDiscretizationError> {
        check(range, settings)?;
        let (lower, upper) = self.parameter_domain();
        if range.0 < lower || range.1 > upper {
            return Err(CurveDiscretizationError::InvalidRange);
        }
        // 区間の中のノットで初めに分ける（ノットでは曲線が折れることがあるため）
        let mut seeds = vec![range.0];
        for &knot in <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::knot_vector(self) {
            if knot > range.0 && knot < range.1 && seeds.last().is_some_and(|&last| knot > last) {
                seeds.push(knot);
            }
        }
        seeds.push(range.1);
        let point = |t: f64| {
            self.point_at_parameter(t)
                .ok_or(CurveDiscretizationError::Evaluation)
        };
        let mut points = vec![CurvePoint {
            parameter: range.0,
            point: point(range.0)?,
        }];
        for pair in seeds.windows(2) {
            let (start, end) = (pair[0], pair[1]);
            // 分ける区間（両端のパラメータと点）を、後ろから積む（前から順に取り出す）
            let mut pending = vec![(start, point(start)?, end, point(end)?)];
            while let Some((a, pa, b, pb)) = pending.pop() {
                match split_point(a, pa, b, pb, settings, &point)? {
                    Some((middle, pm)) => {
                        if middle <= a || middle >= b {
                            return Err(CurveDiscretizationError::TooManyPoints);
                        }
                        pending.push((middle, pm, b, pb));
                        pending.push((a, pa, middle, pm));
                    }
                    None => {
                        if points.len() >= settings.max_points {
                            return Err(CurveDiscretizationError::TooManyPoints);
                        }
                        points.push(CurvePoint {
                            parameter: b,
                            point: pb,
                        });
                    }
                }
            }
        }
        // 区間ごとの確かめは、隣り合う区間の弦のなす角を見ないので、つなぎ目で確かめ、超えれば両隣を分ける（曲線
        // そのものが折れるノットのつなぎ目は除く）
        let corners: Vec<f64> = seeds
            .iter()
            .copied()
            .filter(|&knot| knot > range.0 && knot < range.1)
            .map(|knot| Ok((knot, is_corner(knot, range, settings, &point)?)))
            .collect::<Result<Vec<_>, CurveDiscretizationError>>()?
            .into_iter()
            .filter_map(|(knot, corner)| corner.then_some(knot))
            .collect();
        for _ in 0..MAX_JOINT_PASSES {
            let mut inserted = Vec::new();
            for (index, triple) in points.windows(3).enumerate() {
                let (a, b, c) = (triple[0], triple[1], triple[2]);
                if !corners.contains(&b.parameter)
                    && chord_angle(a.point, b.point, c.point) > settings.max_angle.to_radians()
                {
                    inserted.push(index);
                    inserted.push(index + 1);
                }
            }
            if inserted.is_empty() {
                return Ok(points);
            }
            inserted.dedup();
            if points.len() + inserted.len() > settings.max_points {
                return Err(CurveDiscretizationError::TooManyPoints);
            }
            // 後ろから足し、前の番号をずらさない
            for &index in inserted.iter().rev() {
                let (a, b) = (points[index].parameter, points[index + 1].parameter);
                let middle = (a + b) / 2.0;
                points.insert(
                    index + 1,
                    CurvePoint {
                        parameter: middle,
                        point: point(middle)?,
                    },
                );
            }
        }
        Err(CurveDiscretizationError::TooManyPoints)
    }
}

/// つなぎ目の角度を確かめ直す回数の上限
const MAX_JOINT_PASSES: usize = 32;

/// 弦との離れを測る、区間を分けた点の割合
const CHORD_SAMPLES: [f64; 7] = [0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875];

/// NURBS 曲線の区間 `[a, b]`（両端の点 `pa`・`pb`）を、許容弦誤差か角度の上限で分ける必要があれば、分ける点（中ほどの
/// パラメータと点）を返す
///
/// 弦との離れは、区間を 8 つに分けた 7 点で測る。角度は、前半と後半の弦のなす角を上限の半分と比べる（区間の中で曲線が
/// 向きを変える量の目安。半分と比べると、隣の区間の弦とのなす角も、ほぼ上限に収まる）。
fn split_point(
    a: f64,
    pa: Point3D<f64>,
    b: f64,
    pb: Point3D<f64>,
    settings: &CurveDiscretizationSettings,
    point: &impl Fn(f64) -> Result<Point3D<f64>, CurveDiscretizationError>,
) -> Result<Option<(f64, Point3D<f64>)>, CurveDiscretizationError> {
    let middle = (a + b) / 2.0;
    let mut pm = None;
    let mut exceeds = false;
    for fraction in CHORD_SAMPLES {
        let inner = point(a + (b - a) * fraction)?;
        if fraction == 0.5 {
            pm = Some(inner);
        }
        if distance_to_segment(inner, pa, pb) > settings.chord_tolerance {
            exceeds = true;
        }
    }
    let pm = match pm {
        Some(pm) => pm,
        None => point(middle)?,
    };
    if exceeds || chord_angle(pa, pm, pb) > settings.max_angle.to_radians() / 2.0 {
        return Ok(Some((middle, pm)));
    }
    Ok(None)
}

/// ノット `knot` で曲線そのものが折れるか（両側のごく短い弦のなす角が、角度の上限を超えるか）
fn is_corner(
    knot: f64,
    range: (f64, f64),
    settings: &CurveDiscretizationSettings,
    point: &impl Fn(f64) -> Result<Point3D<f64>, CurveDiscretizationError>,
) -> Result<bool, CurveDiscretizationError> {
    let offset = (range.1 - range.0) * CORNER_PROBE_FRACTION;
    Ok(
        chord_angle(point(knot - offset)?, point(knot)?, point(knot + offset)?)
            > settings.max_angle.to_radians(),
    )
}

/// 折れを確かめる短い弦の、区間の幅に対する割合
const CORNER_PROBE_FRACTION: f64 = 1.0e-6;

/// 点 `a` から `b`、`b` から `c` の 2 つの弦のなす角（ラジアン。弦の長さが 0 なら 0）
fn chord_angle(a: Point3D<f64>, b: Point3D<f64>, c: Point3D<f64>) -> f64 {
    let (first, second) = (Vector3D::from_points(a, b), Vector3D::from_points(b, c));
    let lengths = first.length() * second.length();
    if lengths > 0.0 {
        (first.dot(second) / lengths).clamp(-1.0, 1.0).acos()
    } else {
        0.0
    }
}

/// 点 `point` と、`start` から `end` への線分の距離（線分の長さが 0 なら、`start` との距離）
fn distance_to_segment(point: Point3D<f64>, start: Point3D<f64>, end: Point3D<f64>) -> f64 {
    match <LineSegment3D<f64> as LineSegment3DConstructor<f64>>::new(start, end) {
        Some(segment) => segment.distance_to_point(point),
        None => point.distance_to(start),
    }
}

/// 半径 `radius` の円の、許容弦誤差と角度の上限の角の刻み
fn angle_step(radius: f64, settings: &CurveDiscretizationSettings) -> f64 {
    let half_max = (settings.max_angle.to_radians() / 2.0).cos();
    2.0 * (1.0 - settings.chord_tolerance / radius)
        .max(half_max)
        .acos()
}

/// 区間 `range` を、刻み `step` 以下の等しい幅で分ける
fn uniform(
    range: (f64, f64),
    step: f64,
    settings: &CurveDiscretizationSettings,
    point: impl Fn(f64) -> Point3D<f64>,
) -> Result<Vec<CurvePoint>, CurveDiscretizationError> {
    let segments = ((range.1 - range.0) / step).ceil().max(1.0);
    if !segments.is_finite() || segments + 1.0 > settings.max_points as f64 {
        return Err(CurveDiscretizationError::TooManyPoints);
    }
    let segments = segments as usize;
    Ok((0..=segments)
        .map(|index| {
            let parameter = if index == segments {
                range.1
            } else {
                range.0 + (range.1 - range.0) * (index as f64 / segments as f64)
            };
            CurvePoint {
                parameter,
                point: point(parameter),
            }
        })
        .collect())
}

/// 設定と区間を確かめる
fn check(
    range: (f64, f64),
    settings: &CurveDiscretizationSettings,
) -> Result<(), CurveDiscretizationError> {
    let max_angle = settings.max_angle.to_radians();
    if !(settings.chord_tolerance > 0.0 && settings.chord_tolerance.is_finite())
        || !(max_angle > 0.0 && max_angle < std::f64::consts::PI)
        || settings.max_points < 2
    {
        return Err(CurveDiscretizationError::InvalidSettings);
    }
    if !(range.0.is_finite() && range.1.is_finite() && range.0 < range.1) {
        return Err(CurveDiscretizationError::InvalidRange);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
