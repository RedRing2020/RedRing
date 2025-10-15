//! Ray Traits - 射線（半無限線）の最小責務抽象化
//!
//! abstracts 層での Ray 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義
//!
//! # 設計方針: InfiniteLine継承
//!
//! ## 基本Rayトレイト = InfiniteLine + 開始点制約
//! ```text
//! Ray Trait = InfiniteLine + 半無限制約
//! ├── 開始点 (start_point)
//! ├── 方向ベクトル (direction) ← InfiniteLineから継承
//! ├── パラメータ t ∈ [0, ∞) （半無限）
//! └── 基本形状判定
//!
//! 除外される責務:
//! ├── 計量演算 (length = 無限) → RayMetrics
//! ├── 点判定 (contains_point, on_ray) → RayContainment
//! ├── 変換操作 (translate, rotate) → RayTransform
//! └── 交差計算 (intersect_ray) → RayIntersection
//! ```

use super::infinite_line_traits::{InfiniteLine2D, InfiniteLine3D};
use crate::Scalar;

/// 2D射線（半無限線）の最小責務抽象化
///
/// InfiniteLine + 開始点制約の概念
/// パラメータ t ∈ [0, ∞) の半無限線
pub trait Ray2D<T: Scalar>: InfiniteLine2D<T> {
    /// 射線の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// パラメータ t での射線上の点を取得（t ≥ 0）
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 射線が指定方向に向いているかを判定
    fn points_towards(&self, target: &Self::Point) -> bool;
}

/// 3D射線（半無限線）の最小責務抽象化
///
/// Ray2D + InfiniteLine3D の3D拡張
pub trait Ray3D<T: Scalar>: Ray2D<T> + InfiniteLine3D<T> {}

/// 射線の計量最小責務
///
/// 距離計算や射線特有の計量機能を提供
pub trait RayMetrics<T: Scalar>: Ray2D<T> {
    /// 点から射線への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 射線上で点に最も近い点を取得
    fn closest_point_on_ray(&self, point: &Self::Point) -> Self::Point;

    /// 射線上で点に最も近い点のパラメータを取得
    fn closest_parameter(&self, point: &Self::Point) -> T;
}

/// 射線の包含・方向判定最小責務
///
/// 点の包含判定や方向チェックを提供
pub trait RayContainment<T: Scalar>: Ray2D<T> {
    /// 点が射線上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が射線の正の方向にあるかを判定
    fn is_point_ahead(&self, point: &Self::Point) -> bool;

    /// 点が射線の開始点より後ろにあるかを判定
    fn is_point_behind(&self, point: &Self::Point) -> bool;
}

/// 射線の変換操作最小責務
///
/// 移動・回転などの変換操作を提供
pub trait RayTransform<T: Scalar>: Ray2D<T> {
    /// 射線を平行移動
    fn translate(&self, offset: &Self::Direction) -> Self;

    /// 射線を指定点基準で回転（2D）
    fn rotate_around_point(&self, center: &Self::Point, angle: T) -> Self;

    /// 射線の方向を反転（開始点は同じ）
    fn reverse_direction(&self) -> Self;
}

/// 射線の交差計算最小責務
///
/// 他の幾何要素との交差計算を提供
pub trait RayIntersection<T: Scalar>: Ray2D<T> {
    /// 他の射線との交点を計算
    fn intersect_ray(&self, other: &Self) -> Option<Self::Point>;

    /// 線分との交点を計算
    fn intersect_line_segment(&self, start: &Self::Point, end: &Self::Point)
        -> Option<Self::Point>;

    /// 円との交点を計算
    fn intersect_circle(&self, center: &Self::Point, radius: T) -> Vec<Self::Point>;
}

/// 射線のサンプリング最小責務
///
/// 射線上の点列生成機能を提供
pub trait RaySampling<T: Scalar>: Ray2D<T> {
    /// 指定距離間隔で射線上の点列を生成（有限範囲）
    fn sample_points(&self, distance_step: T, max_distance: T) -> Vec<Self::Point>;

    /// 指定パラメータ間隔で射線上の点列を生成（有限範囲）
    fn sample_by_parameter(&self, parameter_step: T, max_parameter: T) -> Vec<Self::Point>;
}
