//! Line/InfiniteLine（直線）操作トレイト
//!
//! 直線に関する共通操作を定義

use crate::Scalar;

/// 直線の基本操作トレイト
pub trait LineOps<T: Scalar> {
    type Point;
    type Vector;

    /// 指定されたパラメータでの点を取得
    fn point_at(&self, t: T) -> Self::Point;

    /// 指定された点のパラメータを取得
    fn parameter_at_point(&self, point: &Self::Point) -> T;

    /// 点から直線への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 点から直線上の最近点を取得
    fn closest_point(&self, point: &Self::Point) -> Self::Point;

    /// 点が直線上にあるかを判定（許容誤差付き）
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;
}

/// 直線の交差判定操作
pub trait LineIntersection<T: Scalar> {
    type Point;

    /// 他の直線との交点を計算
    fn intersect_line(&self, other: &Self) -> Option<Self::Point>;

    /// 他の直線と平行かを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他の直線と同一かを判定
    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool;
}

/// 3D直線の特別な操作
pub trait Line3DOps<T: Scalar>: LineOps<T> {
    type Direction;

    /// 他の直線とスキューライン（ねじれの位置）かを判定
    fn is_skew_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他の直線との最短距離を計算
    fn distance_to_line(&self, other: &Self) -> T;

    /// 他の直線との最短距離を結ぶ線分の端点を取得
    fn closest_points_to_line(&self, other: &Self) -> Option<(Self::Point, Self::Point)>;

    /// 平面との交点を計算
    fn intersect_plane(
        &self,
        plane_point: &Self::Point,
        plane_normal: &Self::Direction,
    ) -> Option<Self::Point>;
}

/// 直線の変換操作
pub trait LineTransform<T: Scalar> {
    type Point;
    type Vector;
    type Direction;

    /// 直線を平行移動
    fn translate(&self, offset: &Self::Vector) -> Self;

    /// 指定軸周りの回転
    fn rotate_around_axis(
        &self,
        axis_point: &Self::Point,
        axis_direction: &Self::Direction,
        angle: T,
    ) -> Self;
}

/// 線分の操作（有限線分用）
pub trait SegmentOps<T: Scalar> {
    type Point;

    /// 線分の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 線分の終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 線分の中点を取得
    fn midpoint(&self) -> Self::Point;

    /// パラメータでの点を取得（0.0〜1.0）
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;
}
