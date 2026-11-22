//! 放物線・双曲線（RedRing に解析的な型がないため、稜線の区間ごとに有理でない NURBS 曲線にする）
//!
//! - 放物線 `C + F (t² x + 2 t y)` は 2 次の多項式なので、区間を 1 つの 2 次の Bézier 曲線で正確に表す
//! - 双曲線 `C + a cosh(t) x + b sinh(t) y` は、区間を等しく分け、分けた区間ごとに両端の点と接線から 3 次の
//!   Bézier 曲線を作る。分けた区間の中の点で、ずれが許容誤差の中に収まるまで、分ける数を倍にする

use geo_contracts::NurbsCurve3DConstructor;
use geo_core::{Point3D, Vector3D};
use geo_nurbs::NurbsCurve3D;

/// 分ける数の上限（それでも収まらなければ、近似できない）
const MAX_SEGMENTS: usize = 1024;
/// ずれを測る、分けた区間の中の点の数
const CHECKS_PER_SEGMENT: usize = 4;

/// 放物線か双曲線（配置の原点 `center` と、正規直交の軸 `x`・`y`）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Conic {
    /// 放物線（焦点の距離 `focal_distance`）
    Parabola {
        center: Point3D<f64>,
        x: Vector3D<f64>,
        y: Vector3D<f64>,
        focal_distance: f64,
    },
    /// 双曲線（実軸の半分 `semi_axis`、虚軸の半分 `semi_imaginary_axis`）
    Hyperbola {
        center: Point3D<f64>,
        x: Vector3D<f64>,
        y: Vector3D<f64>,
        semi_axis: f64,
        semi_imaginary_axis: f64,
    },
}

impl Conic {
    /// パラメータ `t` の点
    pub fn point_at(&self, t: f64) -> Point3D<f64> {
        match *self {
            Self::Parabola {
                center,
                x,
                y,
                focal_distance,
            } => center + x * (focal_distance * t * t) + y * (2.0 * focal_distance * t),
            Self::Hyperbola {
                center,
                x,
                y,
                semi_axis,
                semi_imaginary_axis,
            } => center + x * (semi_axis * t.cosh()) + y * (semi_imaginary_axis * t.sinh()),
        }
    }

    /// パラメータ `t` の接線（微分）
    fn derivative_at(&self, t: f64) -> Vector3D<f64> {
        match *self {
            Self::Parabola {
                x,
                y,
                focal_distance,
                ..
            } => x * (2.0 * focal_distance * t) + y * (2.0 * focal_distance),
            Self::Hyperbola {
                x,
                y,
                semi_axis,
                semi_imaginary_axis,
                ..
            } => x * (semi_axis * t.sinh()) + y * (semi_imaginary_axis * t.cosh()),
        }
    }

    /// 区間 `range` を表す有理でない NURBS 曲線と、そのずれ（区間は、元のパラメータのまま）
    ///
    /// 双曲線を `tolerance` の中で近似できなければ `None`。
    pub fn segment(&self, range: (f64, f64), tolerance: f64) -> Option<(NurbsCurve3D<f64>, f64)> {
        let (t0, t1) = range;
        if t0.partial_cmp(&t1) != Some(std::cmp::Ordering::Less) {
            return None;
        }
        match self {
            Self::Parabola { .. } => {
                let start = self.point_at(t0);
                let middle = start + self.derivative_at(t0) * ((t1 - t0) / 2.0);
                let curve = NurbsCurve3D::new(
                    2,
                    vec![t0, t0, t0, t1, t1, t1],
                    &[start, middle, self.point_at(t1)],
                    None,
                )
                .ok()?;
                Some((curve, 0.0))
            }
            Self::Hyperbola { .. } => {
                let mut segments = 1;
                while segments <= MAX_SEGMENTS {
                    let (curve, deviation) = self.cubic_segments(t0, t1, segments)?;
                    if deviation <= tolerance {
                        return Some((curve, deviation));
                    }
                    segments *= 2;
                }
                None
            }
        }
    }

    /// 区間を `segments` に分けた 3 次の Bézier 曲線をつないだ曲線と、分けた区間の中の点のずれの最大
    fn cubic_segments(
        &self,
        t0: f64,
        t1: f64,
        segments: usize,
    ) -> Option<(NurbsCurve3D<f64>, f64)> {
        let step = (t1 - t0) / segments as f64;
        let mut knots = vec![t0; 4];
        let mut points = vec![self.point_at(t0)];
        let mut deviation: f64 = 0.0;
        for segment in 0..segments {
            let a = t0 + step * segment as f64;
            let b = if segment + 1 == segments {
                t1
            } else {
                a + step
            };
            let (pa, pb) = (self.point_at(a), self.point_at(b));
            let (da, db) = (self.derivative_at(a), self.derivative_at(b));
            let control = [pa, pa + da * ((b - a) / 3.0), pb - db * ((b - a) / 3.0), pb];
            for check in 1..=CHECKS_PER_SEGMENT {
                let s = check as f64 / (CHECKS_PER_SEGMENT + 1) as f64;
                let distance = bezier(&control, s).distance_to(self.point_at(a + (b - a) * s));
                // 値があふれた（有限でない）ときは、近似できない
                if !distance.is_finite() {
                    return None;
                }
                deviation = deviation.max(distance);
            }
            points.extend(control.iter().skip(1));
            knots.extend(std::iter::repeat_n(b, 3));
        }
        knots.push(t1);
        let curve = NurbsCurve3D::new(3, knots, &points, None).ok()?;
        Some((curve, deviation))
    }
}

/// 3 次の Bézier 曲線の点（`s` は 0〜1）
fn bezier(control: &[Point3D<f64>; 4], s: f64) -> Point3D<f64> {
    let r = 1.0 - s;
    let weights = [r * r * r, 3.0 * r * r * s, 3.0 * r * s * s, s * s * s];
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let sum = control
        .iter()
        .zip(weights)
        .fold(Vector3D::new(0.0, 0.0, 0.0), |sum, (point, weight)| {
            sum + (*point - origin) * weight
        });
    origin + sum
}
