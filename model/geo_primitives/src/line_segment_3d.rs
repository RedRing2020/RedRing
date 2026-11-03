//! 3次元線分（LineSegment3D）の実装
//!
//! support line と trim 区間を正本とする有限の線形形状。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{InfiniteLine3D, Point3D};
use geo_contracts::{
    default_distance_tolerance, InfiniteLine3DConstructor, InfiniteLine3DEvaluation,
    LineSegment3DConstructor, LineSegment3DContainment, LineSegment3DDistance,
    LineSegment3DEvaluation, LineSegment3DProjection, LineSegment3DProperties,
    LineSegment3DSupportLine, PrimitiveKind, PrimitiveMetadata, Scalar,
};

/// 3次元空間の線分。
///
/// support line と trim 区間を正本とし、必要に応じて拘束端点も保持できる有限線形 shape を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment3D<T: Scalar> {
    line: InfiniteLine3D<T>,
    start_param: T,
    end_param: T,
    start_point: Point3D<T>,
    end_point: Point3D<T>,
}

impl<T: Scalar> PrimitiveMetadata for LineSegment3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}

impl<T: Scalar> std::fmt::Display for LineSegment3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LineSegment3D(start: {:?}, end: {:?})",
            self.start_point(),
            self.end_point()
        )
    }
}

impl<T: Scalar> LineSegment3D<T> {
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

impl<T: Scalar> LineSegment3DConstructor<T> for LineSegment3D<T> {
    fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        let line = InfiniteLine3D::from_two_points(start, end)?;
        Self::from_support_line_and_constraint_points(line, start, end)
    }
}

impl<T: Scalar> LineSegment3DSupportLine<T> for LineSegment3D<T> {
    type Line = InfiniteLine3D<T>;

    fn from_support_line_and_constraint_points(
        line: InfiniteLine3D<T>,
        start_point: Point3D<T>,
        end_point: Point3D<T>,
    ) -> Option<Self> {
        let start_param = line.parameter_for_point(start_point);
        let end_param = line.parameter_for_point(end_point);

        if (end_param - start_param).abs() <= T::EPSILON {
            return None;
        }

        Some(Self {
            line,
            start_param,
            end_param,
            start_point,
            end_point,
        })
    }

    fn support_line(&self) -> &InfiniteLine3D<T> {
        &self.line
    }

    fn start_param(&self) -> T {
        self.start_param
    }

    fn end_param(&self) -> T {
        self.end_param
    }

    fn constraint_start_point(&self) -> Point3D<T> {
        self.start_point
    }

    fn constraint_end_point(&self) -> Point3D<T> {
        self.end_point
    }
}

impl<T: Scalar> LineSegment3DProperties<T> for LineSegment3D<T> {
    fn start_point(&self) -> Point3D<T> {
        self.line.point_at_parameter(self.start_param)
    }

    fn end_point(&self) -> Point3D<T> {
        self.line.point_at_parameter(self.end_param)
    }

    fn length(&self) -> T {
        (self.end_param - self.start_param).abs()
    }
}

impl<T: Scalar> LineSegment3DDistance<T> for LineSegment3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        point.distance_to(self.closest_point(point))
    }
}

impl<T: Scalar> LineSegment3DContainment<T> for LineSegment3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> LineSegment3DEvaluation<T> for LineSegment3D<T> {
    /// 距離トレランス内のはみ出しは support line 上に外挿する。
    fn point_at_parameter(&self, t: T) -> Option<Point3D<T>> {
        if !self.is_parameter_in_domain(t) {
            return None;
        }

        let param = self.start_param + t * (self.end_param - self.start_param);
        Some(self.line.point_at_parameter(param))
    }
}

impl<T: Scalar> LineSegment3DProjection<T> for LineSegment3D<T> {
    /// reverse した線分は start_param > end_param となるため、小さい側・大きい側のパラメータで制限する
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T> {
        let projected_param = self.line.parameter_for_point(point);
        let (min_param, max_param) = self.ordered_params();
        let clamped_param = projected_param.max(min_param).min(max_param);

        self.line.point_at_parameter(clamped_param)
    }
}
