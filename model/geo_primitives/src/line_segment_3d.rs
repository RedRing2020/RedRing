//! 3次元線分（LineSegment3D）の Core 実装
//!
//! Core Foundation パターンに基づく LineSegment3D の必須機能のみ
//! 拡張機能は line_segment_3d_extensions.rs を参照

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, CrossDistance, LineSegment3DConstructor, LineSegment3DContainment,
    LineSegment3DDerived, LineSegment3DDistance, LineSegment3DEvaluation, LineSegment3DProjection,
    LineSegment3DProperties, PrimitiveKind, PrimitiveMetadata, Scalar,
};

/// 3次元空間の線分。
///
/// support line と trim 区間を正本とし、必要に応じて拘束端点も保持できる有限線形 shape を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment3D<T: Scalar> {
    pub(crate) line: InfiniteLine3D<T>,
    pub(crate) start_param: T,
    pub(crate) end_param: T,
    pub(crate) start_point: Point3D<T>,
    pub(crate) end_point: Point3D<T>,
}

impl<T: Scalar> PrimitiveMetadata for LineSegment3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}

impl<T: Scalar> LineSegment3D<T> {
    /// 始点と終点から線分を作成
    pub fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        let line = InfiniteLine3D::from_two_points(start, end)?;

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

    /// 点と方向ベクトル、長さから線分を作成
    pub fn from_point_direction_length(
        start: Point3D<T>,
        direction: Vector3D<T>,
        length: T,
    ) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }

        let line = InfiniteLine3D::new(start, direction)?;
        let end = line.point_at_parameter(length);

        Self::from_support_line_and_constraint_points(line, start, end)
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

    /// 拘束端点間の距離を取得
    pub fn constraint_length(&self) -> T {
        self.start_point.distance_to(&self.end_point)
    }

    /// 始点を取得
    pub fn start(&self) -> Point3D<T> {
        self.ideal_start()
    }

    /// 終点を取得
    pub fn end(&self) -> Point3D<T> {
        self.ideal_end()
    }

    /// 中点を取得
    pub fn midpoint(&self) -> Point3D<T> {
        let two = T::from_f64(2.0);
        let start = self.start();
        let end = self.end();
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
            *self.line.direction_internal()
        } else {
            (*self.line.direction_internal()).negate()
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
        let to_point = Vector3D::from_points(&self.line.point_internal(), point);
        let t = to_point.dot(&self.line.direction_internal());
        let (min_param, max_param) = self.ordered_params();

        let clamped_param = if t < min_param {
            min_param
        } else if t > max_param {
            max_param
        } else {
            t
        };

        let projected = self.line.point_at_parameter(clamped_param);
        point.distance_to(&projected)
    }

    /// 点が線分上にあるかを判定
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        if !self.line.contains_point(point, tolerance) {
            return false;
        }

        let param = self.line.parameter_for_point(point);
        let (min_param, max_param) = self.ordered_params();
        param >= min_param - tolerance && param <= max_param + tolerance
    }

    /// 線分が退化しているか（長さが0）を判定
    pub fn is_degenerate(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }
}

impl<T: Scalar> LineSegment3DConstructor<T> for LineSegment3D<T> {
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self> {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let end_point = Point3D::new(end.0, end.1, end.2);
        Self::new(start_point, end_point)
    }

    fn from_point_direction_length(
        start: (T, T, T),
        direction: (T, T, T),
        length: T,
    ) -> Option<Self> {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let direction_vec = Vector3D::new(direction.0, direction.1, direction.2);
        Self::from_point_direction_length(start_point, direction_vec, length)
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

    fn horizontal_xy(midpoint: (T, T, T), length: T) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }
        let half_length = length / (T::ONE + T::ONE);
        let start = Point3D::new(midpoint.0 - half_length, midpoint.1, midpoint.2);
        let end = Point3D::new(midpoint.0 + half_length, midpoint.1, midpoint.2);
        Self::new(start, end)
    }
}

impl<T: Scalar> LineSegment3DProperties<T> for LineSegment3D<T> {
    fn start(&self) -> (T, T, T) {
        let p = self.start();
        (p.x(), p.y(), p.z())
    }

    fn end(&self) -> (T, T, T) {
        let p = self.end();
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

    fn is_unit_length(&self) -> bool {
        let length = self.length();
        (length - T::ONE).abs() <= T::EPSILON
    }

    fn is_on_xy_plane(&self) -> bool {
        let start = self.start();
        let end = self.end();
        (start.z() - end.z()).abs() <= T::EPSILON
    }

    fn is_on_yz_plane(&self) -> bool {
        let start = self.start();
        let end = self.end();
        (start.x() - end.x()).abs() <= T::EPSILON
    }
}

impl<T: Scalar> LineSegment3DDerived<T> for LineSegment3D<T> {
    fn direction_vector(&self) -> (T, T, T) {
        let dir = self.direction();
        (dir.x(), dir.y(), dir.z())
    }

    fn as_vector(&self) -> (T, T, T) {
        let start = self.start();
        let end = self.end();
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
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let param = self.start_param + t * (self.end_param - self.start_param);
        let p = self.line.point_at_parameter(param);
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar> LineSegment3DProjection<T> for LineSegment3D<T> {
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        let line_param = self.line.parameter_for_point(&p);
        let (min_param, max_param) = self.ordered_params();
        let clamped_param = if line_param < min_param {
            min_param
        } else if line_param > max_param {
            max_param
        } else {
            line_param
        };
        let result = self.line.point_at_parameter(clamped_param);
        (result.x(), result.y(), result.z())
    }
}

impl<T: Scalar> CrossDistance<T, Self> for LineSegment3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        let other_start_pt = other.start();
        let other_end_pt = other.end();
        let self_start_pt = self.start();
        let self_end_pt = self.end();

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

        let start_point = self.start();
        let end_point = self.end();
        let start = (start_point.x(), start_point.y(), start_point.z());
        let end = (end_point.x(), end_point.y(), end_point.z());

        line_segment_to_aabb_distance(start, end, other.0, other.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_endpoints_follow_ideal_support_line() {
        let line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0_f64, 0.0, 0.0)).unwrap();
        let start = Point3D::new(0.0, 1.0, 0.0);
        let end = Point3D::new(2.0, 1.0, 0.0);

        let segment =
            LineSegment3D::from_support_line_and_constraint_points(line, start, end).unwrap();

        assert_eq!(segment.constraint_start_point(), start);
        assert_eq!(segment.constraint_end_point(), end);
        assert_eq!(segment.ideal_start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(segment.ideal_end(), Point3D::new(2.0, 0.0, 0.0));
        assert_eq!(segment.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(segment.end(), Point3D::new(2.0, 0.0, 0.0));
        assert_eq!(segment.point_at_parameter(0.5), Point3D::new(1.0, 0.0, 0.0));
        assert_eq!(segment.length(), 2.0);
        assert_eq!(segment.constraint_length(), 2.0);
        assert_eq!(segment.ideal_length(), 2.0);
    }

    #[test]
    fn checked_parameter_evaluation_rejects_out_of_range_input() {
        use geo_contracts::LineSegment3DEvaluation;

        let segment =
            LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0))
                .expect("segment creation should succeed");

        let in_range =
            <LineSegment3D<f64> as LineSegment3DEvaluation<f64>>::point_at_parameter_checked(
                &segment, 0.5,
            );
        let out_of_range =
            <LineSegment3D<f64> as LineSegment3DEvaluation<f64>>::point_at_parameter_checked(
                &segment, 1.1,
            );

        assert!(in_range.is_some());
        assert!(out_of_range.is_none());
    }

    #[test]
    fn checked_parameter_evaluation_rejects_nan_input() {
        use geo_contracts::LineSegment3DEvaluation;

        let segment =
            LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0))
                .expect("segment creation should succeed");

        let value =
            <LineSegment3D<f64> as LineSegment3DEvaluation<f64>>::point_at_parameter_checked(
                &segment,
                f64::NAN,
            );

        assert!(value.is_none());
    }
}
