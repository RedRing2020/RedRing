//! 2次元円弧（Arc2D）の実装
//!
//! 基底円と、開始角から反時計回りに終了角までの角度範囲で定義する。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Circle2D, Point2D, Vector2D};
use analysis::{Angle, AngleRange};
use geo_contracts::{
    default_angle_tolerance, default_distance_tolerance, Arc2DConstructor, Arc2DContainment,
    Arc2DDerived, Arc2DDistance, Arc2DEndpoint, Arc2DEvaluation, Arc2DProperties, Arc2DTrimRange,
    Circle2DConstructor, Circle2DDistance, Circle2DProperties, PrimitiveKind, PrimitiveMetadata,
    Scalar,
};

/// 2次元円弧
///
/// 基底円と角度範囲による円弧の定義。角度はグローバル X 軸から測る。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc2D<T: Scalar> {
    /// 基底となる円
    circle: Circle2D<T>,
    /// 開始角度
    start_angle: Angle<T>,
    /// 終了角度
    end_angle: Angle<T>,
}

impl<T: Scalar> PrimitiveMetadata for Arc2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }
}

impl<T: Scalar> Arc2D<T> {
    /// 開始角から反時計回りに終了角まで進む角度範囲（開始角と終了角が等しい場合は `None`）
    fn angle_range(&self) -> Option<AngleRange<T>> {
        AngleRange::from_ccw_bounds(self.start_angle, self.end_angle)
    }

    /// 3点の外接円の中心を計算
    fn circumcenter(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Point2D<T>> {
        let (x1, y1, x2, y2, x3, y3) = (p1.x(), p1.y(), p2.x(), p2.y(), p3.x(), p3.y());

        let d = (x1 - x2) * (y2 - y3) - (x2 - x3) * (y1 - y2);
        if d.abs() < default_distance_tolerance::<T>() {
            return None; // 3点が一直線上にある
        }

        let two = T::ONE + T::ONE;
        let ux = ((x1 * x1 + y1 * y1) * (y2 - y3)
            + (x2 * x2 + y2 * y2) * (y3 - y1)
            + (x3 * x3 + y3 * y3) * (y1 - y2))
            / (two * d);
        let uy = ((x1 * x1 + y1 * y1) * (x3 - x2)
            + (x2 * x2 + y2 * y2) * (x1 - x3)
            + (x3 * x3 + y3 * y3) * (x2 - x1))
            / (two * d);

        Some(Point2D::new(ux, uy))
    }

    /// 中心からの点の角度を計算
    fn angle_from_center(center: Point2D<T>, point: Point2D<T>) -> Angle<T> {
        Angle::from_radians((point.y() - center.y()).atan2(point.x() - center.x()))
    }

    /// 角度が start から反時計回りに end までの範囲内にあるか
    fn is_angle_between(start: Angle<T>, mid: Angle<T>, end: Angle<T>) -> bool {
        let s = start.normalize().to_radians();
        let m = mid.normalize().to_radians();
        let e = end.normalize().to_radians();

        if s <= e {
            s <= m && m <= e
        } else {
            s <= m || m <= e
        }
    }
}

impl<T: Scalar> Arc2DConstructor<T> for Arc2D<T> {
    fn new(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Some(Self {
            circle: Circle2D::new(center, radius)?,
            start_angle,
            end_angle,
        })
    }

    fn from_three_points(start: Point2D<T>, mid: Point2D<T>, end: Point2D<T>) -> Option<Self> {
        let center = Self::circumcenter(start, mid, end)?;
        let radius = center.distance_to(start);

        let start_angle = Self::angle_from_center(center, start);
        let mid_angle = Self::angle_from_center(center, mid);
        let end_angle = Self::angle_from_center(center, end);

        // 中間点を通る向き（start→mid→end が反時計回り）になるようにする
        let (from, to) = if Self::is_angle_between(start_angle, mid_angle, end_angle) {
            (start_angle, end_angle)
        } else {
            (end_angle, start_angle)
        };

        <Self as Arc2DConstructor<T>>::new(center, radius, from, to)
    }
}

impl<T: Scalar> Arc2DProperties<T> for Arc2D<T> {
    fn center(&self) -> Point2D<T> {
        self.circle.center()
    }

    fn radius(&self) -> T {
        self.circle.radius()
    }

    fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    fn angle_span(&self) -> Angle<T> {
        let span = self.end_angle.to_radians() - self.start_angle.to_radians();
        if span < T::ZERO {
            Angle::from_radians(span + T::TAU)
        } else {
            Angle::from_radians(span)
        }
    }

    fn is_full_circle(&self) -> bool {
        (self.angle_span().to_radians() - T::TAU).abs() <= default_angle_tolerance::<T>()
    }
}

impl<T: Scalar> Arc2DDerived<T> for Arc2D<T> {
    fn length(&self) -> T {
        self.radius() * self.angle_span().to_radians()
    }
}

impl<T: Scalar> Arc2DEndpoint<T> for Arc2D<T> {
    fn start_point(&self) -> Point2D<T> {
        self.point_at_angle(self.start_angle)
    }

    fn end_point(&self) -> Point2D<T> {
        self.point_at_angle(self.end_angle)
    }
}

impl<T: Scalar> Arc2DEvaluation<T> for Arc2D<T> {
    fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.point_at_angle(Angle::from_radians(
            self.start_angle.to_radians() + self.angle_span().to_radians() * t,
        ))
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Point2D<T> {
        let radians = angle.to_radians();
        self.center() + Vector2D::new(self.radius() * radians.cos(), self.radius() * radians.sin())
    }
}

impl<T: Scalar> Arc2DDistance<T> for Arc2D<T> {
    /// 点の角度が角度範囲内なら母円までの距離、範囲外なら始点・終点までの距離の小さい方を返す。
    fn distance_to_point(&self, point: Point2D<T>) -> T {
        if self.contains_point_angle(point) {
            self.circle.distance_to_point(point)
        } else {
            point
                .distance_to(self.start_point())
                .min(point.distance_to(self.end_point()))
        }
    }
}

impl<T: Scalar> Arc2DContainment<T> for Arc2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> Arc2DTrimRange<T> for Arc2D<T> {
    fn contains_angle(&self, angle: Angle<T>) -> bool {
        let tolerance = default_angle_tolerance::<T>();
        match self.angle_range() {
            Some(range) => range.contains(angle, tolerance),
            None => angle.is_equivalent(&self.start_angle, tolerance),
        }
    }

    fn contains_point_angle(&self, point: Point2D<T>) -> bool {
        self.contains_angle(Angle::from_radians(self.circle.parameter_for_point(point)))
    }
}
