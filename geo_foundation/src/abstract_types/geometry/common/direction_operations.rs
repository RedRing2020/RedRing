//! Direction（方向）操作トレイト
//!
//! 方向ベクトルに関する共通操作を定義

use crate::Scalar;

/// 方向ベクトルの基本操作トレイト
pub trait DirectionOps<T: Scalar> {
    type Vector;

    /// 内部ベクトルを取得
    fn to_vector(&self) -> Self::Vector;

    /// 他の方向との角度を計算
    fn angle_to(&self, other: &Self) -> T;

    /// 他の方向と平行かを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他の方向と垂直かを判定
    fn is_perpendicular_to(&self, other: &Self, tolerance: T) -> bool;

    /// 内積を計算
    fn dot(&self, other: &Self) -> T;
}

/// 2D方向ベクトルの操作
pub trait Direction2DOps<T: Scalar>: DirectionOps<T> {
    /// 90度回転した方向を取得
    fn perpendicular(&self) -> Self;

    /// 角度（ラジアン）から方向を作成
    fn from_angle(angle: T) -> Self;

    /// 角度（ラジアン）を取得
    fn to_angle(&self) -> T;

    /// 左回りに指定角度回転
    fn rotate(&self, angle: T) -> Self;
}

/// 3D方向ベクトルの操作
pub trait Direction3DOps<T: Scalar>: DirectionOps<T> {
    /// 外積を計算
    fn cross(&self, other: &Self) -> Self::Vector;

    /// 任意の垂直な方向を取得
    fn any_perpendicular(&self) -> Self;

    /// 指定軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self;

    /// 正規直交基底を構築（このベクトルをZ軸とする）
    fn build_orthonormal_basis(&self) -> (Self, Self, Self)
    where
        Self: Sized;
}

/// 方向ベクトルの定数提供
pub trait DirectionConstants<T: Scalar> {
    /// X軸正方向
    fn positive_x() -> Self;

    /// Y軸正方向
    fn positive_y() -> Self;

    /// X軸負方向
    fn negative_x() -> Self;

    /// Y軸負方向
    fn negative_y() -> Self;
}

/// 3D方向ベクトルの定数提供
pub trait Direction3DConstants<T: Scalar>: DirectionConstants<T> {
    /// Z軸正方向
    fn positive_z() -> Self;

    /// Z軸負方向
    fn negative_z() -> Self;
}
