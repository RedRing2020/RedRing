//! Ray（半無限直線）操作トレイト
//!
//! レイに関する共通操作を定義

use crate::Scalar;

/// レイの基本操作トレイト
pub trait RayOps<T: Scalar> {
    type Point;

    /// 指定されたパラメータでの点を取得（t >= 0のみ有効）
    fn point_at(&self, t: T) -> Option<Self::Point>;

    /// 指定された点のパラメータを取得（レイ上にある場合のみ）
    fn parameter_at_point(&self, point: &Self::Point) -> Option<T>;

    /// 点がレイ上にあるかを判定（許容誤差付き）
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;

    /// レイの起点からの距離を計算
    fn distance_from_origin(&self, point: &Self::Point) -> Option<T>;
}

/// レイの交差判定操作
pub trait RayIntersection<T: Scalar> {
    type Point;

    /// 他のレイとの交点を計算
    fn intersect_ray(&self, other: &Self) -> Option<Self::Point>;

    /// レイ同士が平行かを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// レイ同士が同一直線上にあるかを判定
    fn is_collinear_with(&self, other: &Self, tolerance: T) -> bool;
}

/// レイの平面交差操作（3D用）
pub trait RayPlaneIntersection<T: Scalar> {
    type Point;
    type Vector;

    /// 平面との交点を計算
    fn intersect_plane(
        &self,
        plane_point: &Self::Point,
        plane_normal: &Self::Vector,
    ) -> Option<Self::Point>;

    /// 平面と平行かを判定
    fn is_parallel_to_plane(&self, plane_normal: &Self::Vector, tolerance: T) -> bool;
}

/// レイの球面交差操作（3D用）
pub trait RaySphereIntersection<T: Scalar> {
    type Point;

    /// 球面との交点を計算（最大2点）
    fn intersect_sphere(&self, center: &Self::Point, radius: T) -> Vec<Self::Point>;

    /// 球面と接するかを判定
    fn is_tangent_to_sphere(&self, center: &Self::Point, radius: T, tolerance: T) -> bool;
}
