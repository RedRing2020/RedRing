//! Ray2D Extensions - 2次元半無限直線の拡張機能
//!
//! Ray2D の高度な幾何演算、変換操作、特殊作成メソッドを提供
//! Core Foundation では提供しない拡張機能のみ

use crate::{Direction2D, InfiniteLine2D, LineSegment2D, Point2D, Ray2D, Vector2D};
use geo_foundation::{Angle, Scalar};

impl<T: Scalar> Ray2D<T> {
    // === 特殊作成メソッド ===

    /// X軸正方向の Ray を作成
    pub fn x_axis_ray(x: T, y: T) -> Self {
        let direction = Direction2D::new(T::ONE, T::ZERO).unwrap();
        Self::new(Point2D::new(x, y), direction.as_vector()).unwrap()
    }

    /// Y軸正方向の Ray を作成
    pub fn y_axis_ray(x: T, y: T) -> Self {
        let direction = Direction2D::new(T::ZERO, T::ONE).unwrap();
        Self::new(Point2D::new(x, y), direction.as_vector()).unwrap()
    }

    /// 原点から指定方向の Ray を作成
    pub fn from_origin(direction: Vector2D<T>) -> Option<Self> {
        Self::new(Point2D::origin(), direction)
    }

    /// 角度から Ray を作成（原点から）
    pub fn from_angle(angle: Angle<T>) -> Self {
        let direction = Vector2D::new(angle.cos(), angle.sin());
        Self::new(Point2D::origin(), direction).unwrap()
    }

    /// 起点と角度から Ray を作成
    pub fn from_origin_and_angle(origin: Point2D<T>, angle: Angle<T>) -> Self {
        let direction = Vector2D::new(angle.cos(), angle.sin());
        Self::new(origin, direction).unwrap()
    }

    // === 交点計算（基本機能以上の複雑な演算） ===

    /// 他の Ray との交点を計算
    pub fn intersection_with_ray(&self, other: &Self) -> Option<Point2D<T>> {
        let line1 = self.to_infinite_line();
        let line2 = other.to_infinite_line();
        let line_intersection = line1.intersection_with_line(&line2)?;

        let t1 = self.parameter_for_point(&line_intersection);
        let t2 = other.parameter_for_point(&line_intersection);

        if t1 >= T::ZERO && t2 >= T::ZERO {
            Some(line_intersection)
        } else {
            None
        }
    }

    /// LineSegment2D との交点を計算
    pub fn intersection_with_segment(&self, segment: &LineSegment2D<T>) -> Option<Point2D<T>> {
        let line_intersection = self
            .to_infinite_line()
            .intersection_with_line(segment.line())?;

        let t_ray = self.parameter_for_point(&line_intersection);
        if t_ray < T::ZERO {
            return None;
        }

        let tolerance = T::EPSILON;
        if segment.contains_point(&line_intersection, tolerance) {
            Some(line_intersection)
        } else {
            None
        }
    }

    /// InfiniteLine2D との交点を計算
    pub fn intersection_with_line(&self, line: &InfiniteLine2D<T>) -> Option<Point2D<T>> {
        let line_intersection = self.to_infinite_line().intersection_with_line(line)?;

        let t = self.parameter_for_point(&line_intersection);
        if t >= T::ZERO {
            Some(line_intersection)
        } else {
            None
        }
    }

    // === 変換操作（Extension で提供） ===

    /// Ray を回転
    pub fn rotate(&self, center: &Point2D<T>, angle: Angle<T>) -> Self {
        let rotated_origin = self.origin().rotate_around(center, angle);
        let rotated_direction = self.direction().rotate(angle);
        Self::new(rotated_origin, rotated_direction).unwrap()
    }

    /// 原点を中心に回転
    pub fn rotate_around_origin(&self, angle: Angle<T>) -> Self {
        self.rotate(&Point2D::origin(), angle)
    }

    /// Ray をスケール
    pub fn scale(&self, center: &Point2D<T>, factor: T) -> Self {
        let scaled_origin = *center + (self.origin() - *center) * factor;
        // DerefによりVector2D<T>が得られる
        Self::new(scaled_origin, self.direction()).unwrap()
    }

    /// Ray を反転（逆方向の Ray を作成）
    pub fn reverse(&self) -> Self {
        // DerefによりVector2D<T>が得られる
        Self::new(self.origin(), -self.direction()).unwrap()
    }

    // === 幾何関係判定（Extension で提供） ===

    /// 他の Ray と平行かを判定
    pub fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool {
        self.direction().is_parallel(&other.direction(), tolerance)
    }

    /// 他の Ray と垂直かを判定
    pub fn is_perpendicular_to(&self, other: &Self, tolerance: T) -> bool {
        self.direction()
            .is_perpendicular(&other.direction(), tolerance)
    }

    /// Ray が同一の無限直線上にあるかを判定
    pub fn is_collinear_with(&self, other: &Self, tolerance: T) -> bool {
        self.to_infinite_line()
            .is_coincident(&other.to_infinite_line())
            && self.is_parallel_to(other, tolerance)
    }

    // === 距離計算（Extension で提供） ===

    /// 他の Ray との最短距離
    pub fn distance_to_ray(&self, other: &Self) -> T {
        if self.intersection_with_ray(other).is_some() {
            return T::ZERO;
        }

        let dist1 = other.distance_to_point(&self.origin());
        let dist2 = self.distance_to_point(&other.origin());
        dist1.min(dist2)
    }

    /// LineSegment2D との最短距離
    pub fn distance_to_segment(&self, segment: &LineSegment2D<T>) -> T {
        if self.intersection_with_segment(segment).is_some() {
            return T::ZERO;
        }

        let dist_to_segment = segment.distance_to_point(&self.origin());
        let dist_start_to_ray = self.distance_to_point(&segment.start());
        let dist_end_to_ray = self.distance_to_point(&segment.end());

        dist_to_segment.min(dist_start_to_ray).min(dist_end_to_ray)
    }

    // === 特殊な点の取得（Extension で提供） ===

    /// Ray を指定した長さで切った時の終点を取得
    pub fn point_at_distance(&self, length: T) -> Option<Point2D<T>> {
        if length < T::ZERO {
            return None;
        }
        // DerefによりVector2D<T>が得られる
        Some(self.origin() + self.direction() * length)
    }

    /// Ray の角度を取得（X軸正方向からの角度）
    pub fn angle(&self) -> Angle<T> {
        let dir = self.direction();
        Angle::from_radians(dir.y().atan2(dir.x()))
    }
}
