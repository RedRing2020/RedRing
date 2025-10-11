//! Point（点）操作トレイト
//!
//! 点に関する共通操作を定義

use crate::Scalar;

/// 点の基本操作トレイト
pub trait PointOps<T: Scalar> {
    type Vector;

    /// 他の点との距離を計算
    fn distance_to(&self, other: &Self) -> T;

    /// 他の点との距離の二乗を計算（計算効率化）
    fn distance_squared_to(&self, other: &Self) -> T;

    /// 他の点への方向ベクトルを取得
    fn direction_to(&self, other: &Self) -> Self::Vector;

    /// 他の点との中点を取得
    fn midpoint(&self, other: &Self) -> Self;
}

/// 点の変換操作トレイト
pub trait PointTransform<T: Scalar> {
    type Vector;

    /// ベクトルによる平行移動
    fn translate(&self, vector: &Self::Vector) -> Self;

    /// 原点を中心とした拡大縮小
    fn scale(&self, factor: T) -> Self;

    /// 指定点を中心とした拡大縮小
    fn scale_around(&self, center: &Self, factor: T) -> Self;
}

/// 点の幾何判定操作トレイト
pub trait PointGeometry<T: Scalar> {
    /// 3点が一直線上にあるかを判定（2D/3D対応）
    fn are_collinear(&self, point2: &Self, point3: &Self, tolerance: T) -> bool;

    /// 原点からの距離を取得
    fn distance_from_origin(&self) -> T;

    /// 指定された点と等しいかを判定（許容誤差付き）
    fn equals_with_tolerance(&self, other: &Self, tolerance: T) -> bool;
}

/// 2D点の特別な操作
pub trait Point2DOps<T: Scalar>: PointOps<T> {
    /// 時計回りの角度を取得（ラジアン）
    fn angle_from_origin(&self) -> T;

    /// 他の点との角度差を取得
    fn angle_to(&self, other: &Self) -> T;

    /// 原点を中心とした回転
    fn rotate(&self, angle: T) -> Self;

    /// 指定点を中心とした回転
    fn rotate_around(&self, center: &Self, angle: T) -> Self;
}

/// 3D点の特別な操作
pub trait Point3DOps<T: Scalar>: PointOps<T> {
    type Direction;

    /// 指定軸周りの回転
    fn rotate_around_axis(
        &self,
        axis_point: &Self,
        axis_direction: &Self::Direction,
        angle: T,
    ) -> Self;

    /// 平面への投影
    fn project_to_plane(&self, plane_point: &Self, plane_normal: &Self::Direction) -> Self;

    /// 平面からの距離
    fn distance_to_plane(&self, plane_point: &Self, plane_normal: &Self::Direction) -> T;
}
