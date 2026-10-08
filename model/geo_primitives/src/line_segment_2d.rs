//! 2次元線分（LineSegment2D）の Core 実装
//!
//! Core Foundation パターンに基づく LineSegment2D の必須機能のみを提供する。

use crate::{InfiniteLine2D, Point2D, Vector2D};
use geo_contracts::{
    default_distance_tolerance, CrossDistance, LineSegment2DConstructor, LineSegment2DContainment,
    LineSegment2DDerived, LineSegment2DDistance, LineSegment2DEvaluation, LineSegment2DProjection,
    LineSegment2DProperties, PrimitiveKind, PrimitiveMetadata, Scalar,
};

/// 2次元平面の線分。
///
/// support line と trim 区間を正本とし、必要に応じて拘束端点も保持できる有限線形 shape を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment2D<T: Scalar> {
    pub(crate) line: InfiniteLine2D<T>,
    pub(crate) start_param: T,
    pub(crate) end_param: T,
    pub(crate) start_point: Point2D<T>,
    pub(crate) end_point: Point2D<T>,
}

impl<T: Scalar> PrimitiveMetadata for LineSegment2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}

impl<T: Scalar> LineSegment2D<T> {
    /// 始点と終点から線分を作成
    pub fn new(start: Point2D<T>, end: Point2D<T>) -> Option<Self> {
        let line = InfiniteLine2D::from_two_points(start, end)?;

        Self::from_support_line_and_constraint_points(line, start, end)
    }

    /// support line と拘束端点から線分を作成
    pub fn from_support_line_and_constraint_points(
        line: InfiniteLine2D<T>,
        start_point: Point2D<T>,
        end_point: Point2D<T>,
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
        start: Point2D<T>,
        direction: Vector2D<T>,
        length: T,
    ) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }

        let line = InfiniteLine2D::new(start, direction)?;
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
    pub fn ideal_start(&self) -> Point2D<T> {
        self.line.point_at_parameter(self.start_param)
    }

    /// support line 上の理想終点を取得
    pub fn ideal_end(&self) -> Point2D<T> {
        self.line.point_at_parameter(self.end_param)
    }

    /// support line 上の理想長を取得
    pub fn ideal_length(&self) -> T {
        (self.end_param - self.start_param).abs()
    }

    /// topology 連携用の拘束始点を取得
    pub fn constraint_start_point(&self) -> Point2D<T> {
        self.start_point
    }

    /// topology 連携用の拘束終点を取得
    pub fn constraint_end_point(&self) -> Point2D<T> {
        self.end_point
    }

    /// 拘束端点間の距離を取得
    pub fn constraint_length(&self) -> T {
        self.start_point.distance_to(&self.end_point)
    }

    /// 始点を取得
    pub fn start_point(&self) -> Point2D<T> {
        self.ideal_start()
    }

    /// 終点を取得
    pub fn end_point(&self) -> Point2D<T> {
        self.ideal_end()
    }

    /// 中点を取得
    pub fn midpoint(&self) -> Point2D<T> {
        let two = T::from_f64(2.0);
        let start = self.start_point();
        let end = self.end_point();
        Point2D::new((start.x() + end.x()) / two, (start.y() + end.y()) / two)
    }

    /// 線分の長さを取得
    pub fn length(&self) -> T {
        self.ideal_length()
    }

    /// 方向ベクトルを取得（正規化済み）
    pub fn direction(&self) -> Vector2D<T> {
        if self.end_param >= self.start_param {
            *self.line.direction_internal()
        } else {
            (*self.line.direction_internal()).negate()
        }
    }

    /// ベクトル表現を取得（始点から終点へのベクトル）
    pub fn vector(&self) -> Vector2D<T> {
        let start = self.start_point();
        let end = self.end_point();
        Vector2D::from_points(start, end)
    }

    /// 正規化パラメータ `t`（始点 0・終点 1）での点を取得
    ///
    /// `[0, 1]` から距離トレランス分を超えて外れる `t` と、有限でない `t` は `None` を返す。
    /// 距離トレランス内のはみ出しは support line 上に外挿する。
    pub fn point_at_parameter(&self, t: T) -> Option<Point2D<T>> {
        if !self.is_parameter_in_domain(t) {
            return None;
        }

        let param = self.start_param + t * (self.end_param - self.start_param);
        Some(self.line.point_at_parameter(param))
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

    /// 点を support line に投影した位置の正規化パラメータを取得
    ///
    /// 始点 0・終点 1 とし、`[0, 1]` に制限しない。
    pub fn parameter_for_point(&self, point: &Point2D<T>) -> T {
        let line_param = self.line.parameter_for_point(point);
        (line_param - self.start_param) / (self.end_param - self.start_param)
    }

    /// 線分上で点に最も近い点の正規化パラメータを取得
    ///
    /// 始点 0・終点 1 とし、`[0, 1]` に制限する。
    pub fn closest_parameter(&self, point: &Point2D<T>) -> T {
        self.parameter_for_point(point).max(T::ZERO).min(T::ONE)
    }

    /// 点が線分上にあるかを判定
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // まず無限直線上にあるかチェック
        if !self.line.contains_point(point, tolerance) {
            return false;
        }

        let param = self.line.parameter_for_point(point);
        let (min_param, max_param) = self.ordered_params();
        param >= min_param - tolerance && param <= max_param + tolerance
    }

    /// 点を線分に投影（線分内に制限）
    pub fn project_point(&self, point: &Point2D<T>) -> Point2D<T> {
        let projected_param = self.line.parameter_for_point(point);
        let (min_param, max_param) = self.ordered_params();
        let clamped_param = projected_param.max(min_param).min(max_param);

        self.line.point_at_parameter(clamped_param)
    }

    /// 点から線分への最短距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let projected = self.project_point(point);
        point.distance_to(&projected)
    }

    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> geo_core::Aabb2D<T> {
        geo_core::Aabb2D::from_points(&[self.start_point(), self.end_point()])
            .expect("Line segment always has two endpoints")
    }

    /// 基盤となる無限直線を取得（Extension用）
    pub fn line(&self) -> &InfiniteLine2D<T> {
        &self.line
    }

    /// 基盤となる support line を取得
    pub fn support_line(&self) -> &InfiniteLine2D<T> {
        &self.line
    }

    /// 始点の support line 上のパラメータを取得
    pub fn start_param(&self) -> T {
        self.start_param
    }

    /// 終点の support line 上のパラメータを取得
    pub fn end_param(&self) -> T {
        self.end_param
    }
}

impl<T: Scalar> LineSegment2D<T> {
    /// 方向を反転
    pub fn reverse(&self) -> Self {
        Self {
            line: self.line,
            start_param: self.end_param,
            end_param: self.start_param,
            start_point: self.end_point,
            end_point: self.start_point,
        }
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    /// 接線ベクトルを取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector2D<T> {
        self.direction()
    }

    /// 境界上判定（線分では点上判定と同じ）
    pub fn on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    /// 他の線分との最短距離を計算
    pub fn distance_to_segment(&self, other: &Self) -> T {
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

impl<T: Scalar> LineSegment2DConstructor<T> for LineSegment2D<T> {
    fn new(start: (T, T), end: (T, T)) -> Option<Self> {
        let start_point = Point2D::new(start.0, start.1);
        let end_point = Point2D::new(end.0, end.1);
        Self::new(start_point, end_point)
    }

    fn from_point_direction_length(start: (T, T), direction: (T, T), length: T) -> Option<Self> {
        let start_point = Point2D::new(start.0, start.1);
        let direction_vec = Vector2D::new(direction.0, direction.1);
        Self::from_point_direction_length(start_point, direction_vec, length)
    }

    fn unit_x() -> Self {
        let start = Point2D::origin();
        let end = Point2D::new(T::ONE, T::ZERO);
        Self::new(start, end).unwrap()
    }

    fn from_midpoint_length_horizontal(midpoint: (T, T), length: T) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }
        let half_length = length / (T::ONE + T::ONE);
        let start = Point2D::new(midpoint.0 - half_length, midpoint.1);
        let end = Point2D::new(midpoint.0 + half_length, midpoint.1);
        Self::new(start, end)
    }

    fn from_midpoint_length_vertical(midpoint: (T, T), length: T) -> Option<Self> {
        if length <= T::ZERO {
            return None;
        }
        let half_length = length / (T::ONE + T::ONE);
        let start = Point2D::new(midpoint.0, midpoint.1 - half_length);
        let end = Point2D::new(midpoint.0, midpoint.1 + half_length);
        Self::new(start, end)
    }

    fn unit_y() -> Self {
        let start = Point2D::origin();
        let end = Point2D::new(T::ZERO, T::ONE);
        Self::new(start, end).unwrap()
    }
}

impl<T: Scalar> LineSegment2DProperties<T> for LineSegment2D<T> {
    fn start(&self) -> (T, T) {
        let p = self.start_point();
        (p.x(), p.y())
    }

    fn end(&self) -> (T, T) {
        let p = self.end_point();
        (p.x(), p.y())
    }

    fn midpoint(&self) -> (T, T) {
        let p = self.midpoint();
        (p.x(), p.y())
    }

    fn length(&self) -> T {
        self.length()
    }

    fn dimension(&self) -> u32 {
        2
    }

    // Phase 2: 追加プロパティ
    fn is_unit_length(&self) -> bool {
        let length = self.length();
        (length - T::ONE).abs() <= T::EPSILON
    }

    fn is_horizontal(&self) -> bool {
        let start = self.start();
        let end = self.end();
        (start.1 - end.1).abs() <= T::EPSILON
    }

    fn is_vertical(&self) -> bool {
        let start = self.start();
        let end = self.end();
        (start.0 - end.0).abs() <= T::EPSILON
    }
}

impl<T: Scalar> LineSegment2DDerived<T> for LineSegment2D<T> {
    fn direction_vector(&self) -> (T, T) {
        let dir = self.direction();
        (dir.x(), dir.y())
    }

    fn as_vector(&self) -> (T, T) {
        let v = self.vector();
        (v.x(), v.y())
    }
}

impl<T: Scalar> LineSegment2DDistance<T> for LineSegment2D<T> {
    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        self.distance_to_point(&p)
    }
}

impl<T: Scalar> LineSegment2DContainment<T> for LineSegment2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        self.contains_point(&p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> LineSegment2DEvaluation<T> for LineSegment2D<T> {
    fn point_at_parameter(&self, t: T) -> Option<(T, T)> {
        self.point_at_parameter(t).map(|p| (p.x(), p.y()))
    }
}

impl<T: Scalar> LineSegment2DProjection<T> for LineSegment2D<T> {
    fn closest_point_to(&self, point: (T, T)) -> (T, T) {
        let p = self.project_point(&Point2D::new(point.0, point.1));
        (p.x(), p.y())
    }
}

impl<T: Scalar> CrossDistance<T, Self> for LineSegment2D<T> {
    fn distance_to(&self, other: &Self) -> T {
        LineSegment2D::distance_to_segment(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_endpoints_follow_ideal_support_line() {
        let line = InfiniteLine2D::new(Point2D::origin(), Vector2D::new(1.0_f64, 0.0)).unwrap();
        let start = Point2D::new(0.0, 1.0);
        let end = Point2D::new(2.0, 1.0);

        let segment =
            LineSegment2D::from_support_line_and_constraint_points(line, start, end).unwrap();

        assert_eq!(segment.constraint_start_point(), start);
        assert_eq!(segment.constraint_end_point(), end);
        assert_eq!(segment.ideal_start(), Point2D::new(0.0, 0.0));
        assert_eq!(segment.ideal_end(), Point2D::new(2.0, 0.0));
        assert_eq!(segment.start_point(), Point2D::new(0.0, 0.0));
        assert_eq!(segment.end_point(), Point2D::new(2.0, 0.0));
        assert_eq!(
            segment.point_at_parameter(0.5),
            Some(Point2D::new(1.0, 0.0))
        );
        assert_eq!(segment.length(), 2.0);
        assert_eq!(segment.constraint_length(), 2.0);
        assert_eq!(segment.ideal_length(), 2.0);
    }

    #[test]
    fn trait_parameter_evaluation_rejects_out_of_domain_input() {
        use geo_contracts::LineSegment2DEvaluation;

        let segment = LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(2.0, 0.0))
            .expect("segment creation should succeed");

        let evaluate = |t| {
            <LineSegment2D<f64> as LineSegment2DEvaluation<f64>>::point_at_parameter(&segment, t)
        };

        assert_eq!(evaluate(0.5), Some((1.0, 0.0)));
        assert!(evaluate(1.1).is_none());
        assert!(evaluate(-0.1).is_none());
        assert!(evaluate(f64::NAN).is_none());
    }
}
