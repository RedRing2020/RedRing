//! Direction trait - 方向ベクトルの最小責務抽象化
//!
//! # 設計方針: 最小責務原則
//!
//! ## 基本Directionトレイト = 必要最小限の操作のみ
//! ```text
//! Direction Trait = 基本属性・操作のみ
//! ├── ベクトル変換 (from_vector, to_vector)
//! ├── 基本計算 (dot, reverse)
//! ├── 基本判定 (is_parallel, is_perpendicular)
//! └── 方向比較 (is_same_direction, is_opposite_direction)
//!
//! 除外される責務:
//! ├── 角度操作 (from_angle, to_angle) → Direction2DExt
//! ├── 回転操作 (rotate_*) → DirectionTransform
//! ├── 座標軸生成 (x_axis, y_axis) → DirectionConstants
//! └── 複雑な幾何演算 (reflect, project) → geo_algorithms
//! ```
//!
//! ## 拡張トレイト群による機能分離
//! ```text
//! Direction2DExt: 2D特化機能
//! ├── perpendicular() - 垂直方向
//! ├── angle operations - 角度変換
//! └── axis constants - 座標軸定数
//!
//! Direction3DExt: 3D特化機能
//! ├── cross() - 外積計算
//! ├── rotate_around_axis() - 軸回転
//! └── orthonormal_basis() - 直交基底
//! ```

use crate::{Angle, Scalar};
use std::fmt::Debug;

/// 方向ベクトルの最小責務トレイト
///
/// 正規化された（長さ=1）ベクトルを表現する基本操作のみを提供。
/// 特化機能は拡張トレイトで組み合わせ可能。
pub trait Direction<T: Scalar>: Debug + Clone + PartialEq {
    /// 関連するベクトル型
    type Vector: Clone + Debug;

    /// ベクトルから方向を作成（正規化）
    ///
    /// # Returns
    /// 正規化された方向ベクトル、またはゼロベクトルの場合はNone
    fn from_vector(vector: Self::Vector) -> Option<Self>
    where
        Self: Sized;

    /// 方向ベクトルを元のベクトル型として取得
    fn to_vector(&self) -> Self::Vector;

    /// 内積計算
    fn dot(&self, other: &Self) -> T;

    /// 方向の反転（180度回転）
    fn reverse(&self) -> Self;

    /// 方向が平行かどうかを判定（許容誤差考慮）
    fn is_parallel(&self, other: &Self, tolerance: T) -> bool;

    /// 方向が垂直かどうかを判定（許容誤差考慮）
    fn is_perpendicular(&self, other: &Self, tolerance: T) -> bool;

    /// 方向が同じかどうかを判定（許容誤差考慮）
    fn is_same_direction(&self, other: &Self, tolerance: T) -> bool;

    /// 方向が反対かどうかを判定（許容誤差考慮）
    fn is_opposite_direction(&self, other: &Self, tolerance: T) -> bool;
}

/// 2D方向ベクトルの拡張機能
///
/// 基本Directionトレイトに2D特化機能を追加。
/// 角度変換や垂直方向など、2D平面特有の操作を提供。
pub trait Direction2D<T: Scalar>: Direction<T> {
    /// 90度回転した方向を取得（2D平面の垂直方向）
    fn perpendicular(&self) -> Self;
}

/// 2D方向ベクトルの角度操作拡張
///
/// 角度とDirection間の変換機能を提供。
/// 基本Directionとは分離して、角度が必要な場合のみ使用。
pub trait Direction2DAngular<T: Scalar>: Direction2D<T> {
    /// 角度（ラジアン）から方向を作成
    fn from_angle(angle: Angle<T>) -> Self
    where
        Self: Sized;

    /// 方向の角度（ラジアン）を取得
    fn to_angle(&self) -> Angle<T>;
}

/// 2D方向ベクトルの定数提供
///
/// 座標軸方向の定数を提供。独立したトレイトとして分離。
pub trait Direction2DConstants<T: Scalar>: Direction2D<T> {
    /// X軸の正方向
    fn x_axis() -> Self
    where
        Self: Sized;

    /// Y軸の正方向
    fn y_axis() -> Self
    where
        Self: Sized;
}

/// 3D方向ベクトルの基本機能
///
/// 基本Directionトレイトに3D空間特有の操作を追加。
pub trait Direction3D<T: Scalar>: Direction<T> {
    /// 外積計算（3D空間の基本操作）
    fn cross(&self, other: &Self) -> Self::Vector;
}

/// 3D方向ベクトルの回転操作拡張
///
/// 軸回転などの高度な3D変換操作を提供。
pub trait Direction3DRotation<T: Scalar>: Direction3D<T> {
    /// 指定した軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: Angle<T>) -> Self;
}

/// 3D方向ベクトルの直交基底生成
///
/// 正規直交基底の構築など、数学的操作を提供。
pub trait Direction3DBasis<T: Scalar>: Direction3D<T> {
    /// 任意の軸に対する直交ベクトルを生成
    fn any_perpendicular(&self) -> Self;

    /// 正規直交基底を構築（この方向をZ軸とする）
    fn build_orthonormal_basis(&self) -> (Self, Self, Self);
}

// Direction3DConstants は common::direction_operations モジュールから提供

/// STEP互換性マーカートレイト
///
/// STEPファイルとの相互運用性を提供。
/// 基本機能とは独立したトレイトとして分離。
pub trait StepCompatible {
    /// STEP表現の文字列を生成
    fn to_step_string(&self) -> String;

    /// STEP表現から解析（将来実装予定）
    fn from_step_string(step_str: &str) -> Result<Self, String>
    where
        Self: Sized;
}
