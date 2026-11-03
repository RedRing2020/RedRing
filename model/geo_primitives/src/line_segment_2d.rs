//! 2次元線分（LineSegment2D）の実装
//!
//! support line と trim 区間を正本とする有限の線形形状。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{InfiniteLine2D, Point2D};
use geo_contracts::{
    default_distance_tolerance, InfiniteLine2DConstructor, InfiniteLine2DEvaluation,
    LineSegment2DConstructor, LineSegment2DContainment, LineSegment2DDistance,
    LineSegment2DEvaluation, LineSegment2DProjection, LineSegment2DProperties, PrimitiveKind,
    PrimitiveMetadata, Scalar,
};

/// 2次元平面の線分。
///
/// support line と trim 区間を正本とし、必要に応じて拘束端点も保持できる有限線形 shape を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment2D<T: Scalar> {
    line: InfiniteLine2D<T>,
    start_param: T,
    end_param: T,
    start_point: Point2D<T>,
    end_point: Point2D<T>,
}

impl<T: Scalar> PrimitiveMetadata for LineSegment2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}

impl<T: Scalar> LineSegment2D<T> {
    fn ordered_params(&self) -> (T, T) {
        if self.start_param <= self.end_param {
            (self.start_param, self.end_param)
        } else {
            (self.end_param, self.start_param)
        }
    }

    /// 正規化パラメータ `t` が評価できる範囲内かを判定
    ///
    /// 許容するはみ出しは距離トレランスを線分の長さで割った値とし、評価点と線分の距離が
    /// 距離トレランス以下となるようにする。
    fn is_parameter_in_domain(&self, t: T) -> bool {
        if !t.is_finite() {
            return false;
        }
        let margin = default_distance_tolerance::<T>() / self.length();
        t >= -margin && t <= T::ONE + margin
    }
}

impl<T: Scalar> LineSegment2DConstructor<T> for LineSegment2D<T> {
    fn new(start: Point2D<T>, end: Point2D<T>) -> Option<Self> {
        let line = InfiniteLine2D::from_two_points(start, end)?;
        let start_param = line.parameter_for_point(start);
        let end_param = line.parameter_for_point(end);

        if (end_param - start_param).abs() <= T::EPSILON {
            return None;
        }

        Some(Self {
            line,
            start_param,
            end_param,
            start_point: start,
            end_point: end,
        })
    }
}

impl<T: Scalar> LineSegment2DProperties<T> for LineSegment2D<T> {
    fn start_point(&self) -> Point2D<T> {
        self.line.point_at_parameter(self.start_param)
    }

    fn end_point(&self) -> Point2D<T> {
        self.line.point_at_parameter(self.end_param)
    }

    fn length(&self) -> T {
        (self.end_param - self.start_param).abs()
    }
}

impl<T: Scalar> LineSegment2DDistance<T> for LineSegment2D<T> {
    fn distance_to_point(&self, point: Point2D<T>) -> T {
        point.distance_to(self.closest_point(point))
    }
}

impl<T: Scalar> LineSegment2DContainment<T> for LineSegment2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> LineSegment2DEvaluation<T> for LineSegment2D<T> {
    /// 距離トレランス内のはみ出しは support line 上に外挿する。
    fn point_at_parameter(&self, t: T) -> Option<Point2D<T>> {
        if !self.is_parameter_in_domain(t) {
            return None;
        }

        let param = self.start_param + t * (self.end_param - self.start_param);
        Some(self.line.point_at_parameter(param))
    }
}

impl<T: Scalar> LineSegment2DProjection<T> for LineSegment2D<T> {
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T> {
        let projected_param = self.line.parameter_for_point(point);
        let (min_param, max_param) = self.ordered_params();
        let clamped_param = projected_param.max(min_param).min(max_param);

        self.line.point_at_parameter(clamped_param)
    }
}
