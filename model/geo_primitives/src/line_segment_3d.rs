//! 3次元線分（LineSegment3D）の Core 実装
//!
//! Core Foundation パターンに基づく LineSegment3D の必須機能のみ
//! 拡張機能は子モジュール extensions を参照

mod bounds;
mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, CrossDistance, InfiniteLine3DConstructor, InfiniteLine3DDistance,
    InfiniteLine3DEvaluation, InfiniteLine3DProperties, LineSegment3DConstructor,
    LineSegment3DContainment, LineSegment3DDerived, LineSegment3DDistance, LineSegment3DEvaluation,
    LineSegment3DProjection, LineSegment3DProperties, PrimitiveKind, PrimitiveMetadata, Scalar,
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

impl<T: Scalar> LineSegment3D<T> {
    /// 始点と終点から線分を作成
    pub fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        let line = InfiniteLine3D::from_two_points(&start, &end)?;

        Self::from_support_line_and_constraint_points(line, start, end)
    }

    /// support line と拘束端点から線分を作成
    pub fn from_support_line_and_constraint_points(
        line: InfiniteLine3D<T>,
        start_point: Point3D<T>,
        end_point: Point3D<T>,
    ) -> Option<Self> {
        let start_param = line.parameter_for_point(&start_point);
        let end_param = line.parameter_for_point(&end_point);

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

    fn ordered_params(&self) -> (T, T) {
        if self.start_param <= self.end_param {
            (self.start_param, self.end_param)
        } else {
            (self.end_param, self.start_param)
        }
    }

    /// support line 上の理想始点を取得
    pub fn ideal_start(&self) -> Point3D<T> {
        self.line.point_at_parameter(self.start_param)
    }

    /// support line 上の理想終点を取得
    pub fn ideal_end(&self) -> Point3D<T> {
        self.line.point_at_parameter(self.end_param)
    }

    /// support line 上の理想長を取得
    pub fn ideal_length(&self) -> T {
        (self.end_param - self.start_param).abs()
    }

    /// topology 連携用の拘束始点を取得
    pub fn constraint_start_point(&self) -> Point3D<T> {
        self.start_point
    }

    /// topology 連携用の拘束終点を取得
    pub fn constraint_end_point(&self) -> Point3D<T> {
        self.end_point
    }

    /// 始点を取得
    pub fn start_point(&self) -> Point3D<T> {
        self.ideal_start()
    }

    /// 終点を取得
    pub fn end_point(&self) -> Point3D<T> {
        self.ideal_end()
    }

    /// 中点を取得
    pub fn midpoint(&self) -> Point3D<T> {
        let two = T::from_f64(2.0);
        let start = self.start_point();
        let end = self.end_point();
        Point3D::new(
            (start.x() + end.x()) / two,
            (start.y() + end.y()) / two,
            (start.z() + end.z()) / two,
        )
    }

    /// 線分の長さを取得
    pub fn length(&self) -> T {
        self.ideal_length()
    }

    /// 方向ベクトルを取得
    pub fn direction(&self) -> Vector3D<T> {
        if self.end_param >= self.start_param {
            *self.line.direction()
        } else {
            (*self.line.direction()).negate()
        }
    }

    /// 基盤となる無限直線を取得
    pub fn line(&self) -> &InfiniteLine3D<T> {
        &self.line
    }

    /// 基盤となる support line を取得
    pub fn support_line(&self) -> &InfiniteLine3D<T> {
        &self.line
    }

    /// 始点のパラメータを取得
    pub fn start_param(&self) -> T {
        self.start_param
    }

    /// 終点のパラメータを取得
    pub fn end_param(&self) -> T {
        self.end_param
    }

    /// 点から線分への最短距離
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        point.distance_to(&self.closest_point(point))
    }

    /// 点が線分上にあるかを判定
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        if self.line.distance_to_point(point) > tolerance {
            return false;
        }

        let param = self.line.parameter_for_point(point);
        let (min_param, max_param) = self.ordered_params();
        param >= min_param - tolerance && param <= max_param + tolerance
    }
}

impl<T: Scalar> LineSegment3DConstructor<T> for LineSegment3D<T> {
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self> {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let end_point = Point3D::new(end.0, end.1, end.2);
        Self::new(start_point, end_point)
    }

    fn unit_x() -> Self {
        let start = Point3D::origin();
        let end = Point3D::new(T::ONE, T::ZERO, T::ZERO);
        Self::new(start, end).unwrap()
    }

    fn unit_y() -> Self {
        let start = Point3D::origin();
        let end = Point3D::new(T::ZERO, T::ONE, T::ZERO);
        Self::new(start, end).unwrap()
    }

    fn unit_z() -> Self {
        let start = Point3D::origin();
        let end = Point3D::new(T::ZERO, T::ZERO, T::ONE);
        Self::new(start, end).unwrap()
    }
}

impl<T: Scalar> LineSegment3DProperties<T> for LineSegment3D<T> {
    fn start_point(&self) -> (T, T, T) {
        let p = self.start_point();
        (p.x(), p.y(), p.z())
    }

    fn end_point(&self) -> (T, T, T) {
        let p = self.end_point();
        (p.x(), p.y(), p.z())
    }

    fn midpoint(&self) -> (T, T, T) {
        let p = self.midpoint();
        (p.x(), p.y(), p.z())
    }

    fn length(&self) -> T {
        self.length()
    }

    fn dimension(&self) -> u32 {
        3
    }
}

impl<T: Scalar> LineSegment3DDerived<T> for LineSegment3D<T> {
    fn direction_vector(&self) -> (T, T, T) {
        let dir = self.direction();
        (dir.x(), dir.y(), dir.z())
    }

    fn as_vector(&self) -> (T, T, T) {
        let start = self.start_point();
        let end = self.end_point();
        let v = Vector3D::from_points(&start, &end);
        (v.x(), v.y(), v.z())
    }
}

impl<T: Scalar> LineSegment3DDistance<T> for LineSegment3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        self.distance_to_point(&p)
    }
}

impl<T: Scalar> LineSegment3DContainment<T> for LineSegment3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        self.contains_point(&p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> LineSegment3DEvaluation<T> for LineSegment3D<T> {
    fn point_at_parameter(&self, t: T) -> Option<(T, T, T)> {
        self.point_at_parameter(t).map(|p| (p.x(), p.y(), p.z()))
    }
}

impl<T: Scalar> LineSegment3DProjection<T> for LineSegment3D<T> {
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T) {
        let p = self.closest_point(&Point3D::new(point.0, point.1, point.2));
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar> CrossDistance<T, Self> for LineSegment3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        let other_start_pt = other.start_point();
        let other_end_pt = other.end_point();
        let self_start_pt = self.start_point();
        let self_end_pt = self.end_point();

        let d1 = self.distance_to_point(&other_start_pt);
        let d2 = self.distance_to_point(&other_end_pt);
        let d3 = other.distance_to_point(&self_start_pt);
        let d4 = other.distance_to_point(&self_end_pt);

        let min1 = if d1 < d2 { d1 } else { d2 };
        let min2 = if d3 < d4 { d3 } else { d4 };
        if min1 < min2 {
            min1
        } else {
            min2
        }
    }
}

impl<T: Scalar> CrossDistance<T, ((T, T, T), (T, T, T))> for LineSegment3D<T> {
    fn distance_to(&self, other: &((T, T, T), (T, T, T))) -> T {
        use geo_commons::line_segment_to_aabb_distance;

        let start_point = self.start_point();
        let end_point = self.end_point();
        let start = (start_point.x(), start_point.y(), start_point.z());
        let end = (end_point.x(), end_point.y(), end_point.z());

        line_segment_to_aabb_distance(start, end, other.0, other.1)
    }
}
